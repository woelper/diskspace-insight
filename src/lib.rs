#[cfg(test)]
mod tests;

use bytesize::ByteSize;
use jwalk::WalkDir;
use log::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
/// A File, representing a file on disk
pub struct File {
    pub size: u64,
    pub ext: Option<String>,
    pub path: PathBuf,
    pub modified: SystemTime,
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
            DIRECTORY
            Path: {}
            Size: {}
            Ext: {:?}
            Modified: {:?}
        ",
            self.path.display(),
            self.size,
            self.ext,
            self.modified.duration_since(std::time::UNIX_EPOCH)
        )
    }
}

#[derive(Debug, Clone, Default)]
/// A Directory, representing a directory on disk
pub struct Directory {
    pub size: u64,
    pub combined_size: u64,
    pub path: PathBuf,
    pub files: Vec<File>,
    pub directories: Vec<PathBuf>,
    pub parent: Option<PathBuf>,
}

impl Directory {
    pub fn new(path: PathBuf) -> Directory {
        Directory {
            path,
            ..Default::default()
        }
    }

    pub fn files_as_fake_dir(&self) -> Directory {
        Directory {
            files: self.files.clone(),
            size: self.size,
            combined_size: self.size,
            path: PathBuf::from("Files"),
            directories: vec![],
            parent: self.parent.clone(),
        }
    }

    pub fn sorted_subdirs(&self, info: &DirInfo) -> Vec<Directory> {
        let mut sorted_dirs: Vec<Directory> = self
            .directories
            .iter()
            .map(|d| info.tree.get(d))
            .flatten()
            .cloned()
            .collect();
        sorted_dirs.sort_by(|a, b| b.combined_size.cmp(&a.combined_size));
        sorted_dirs
    }
}

impl Iterator for Directory {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        self.directories.pop()
    }
}

impl std::fmt::Display for Directory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
            DIRECTORY
            Path: {}
            Size: {}
            Combined Size: {}
            Files: {:#?}
        ",
            self.path.display(),
            ByteSize(self.size),
            ByteSize(self.combined_size),
            self.files
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct FileType {
    pub size: u64,
    pub ext: String,
    pub files: Vec<File>,
}

#[derive(Debug, Clone)]
/// DirInfo holds all info about a diretory.
pub struct DirInfo {
    /// All file types
    pub filetypes: HashMap<String, FileType>,
    pub files: Vec<File>,
    /// Files, ordered by size, descending
    pub files_by_size: Vec<File>,
    /// Filetypes, ordered by size, descending
    pub types_by_size: Vec<FileType>,
    pub dirs_by_size: Vec<Directory>,
    pub tree: HashMap<PathBuf, Directory>,
    pub combined_size: u64,
}

impl DirInfo {
    pub fn new() -> DirInfo {
        DirInfo {
            filetypes: HashMap::new(),
            files: vec![],
            files_by_size: vec![],
            types_by_size: vec![],
            dirs_by_size: vec![],
            tree: HashMap::new(),
            combined_size: 0,
        }
    }

    pub fn types_by_size(&self) -> Vec<FileType> {
        let mut ftypes: Vec<_> = self
            .filetypes
            .par_iter()
            .map(|f| f.1)
            .map(|f| {
                let mut f = f.clone();
                f.files.par_sort_by(|a, b| b.size.cmp(&a.size));
                f
            })
            .collect();
        ftypes.par_sort_by(|a, b| b.size.cmp(&a.size));
        ftypes
    }

    pub fn files_by_size(&self) -> Vec<File> {
        let mut count = self.files.clone();
        count.par_sort_by(|a, b| b.size.cmp(&a.size));
        count
    }

    pub fn dirs_by_size(&self) -> Vec<Directory> {
        let mut dirs: Vec<Directory> = self.tree.values().cloned().collect();
        dirs.par_sort_by(|a, b| b.size.cmp(&a.size));
        dirs
    }
}

/// Scan a directory, calling callback with DirInfo periodically
pub fn scan_callback<P: AsRef<Path>, F: Fn(&DirInfo)>(
    source: P,
    callback: F,
    update_rate_ms: u128,
) -> DirInfo {
    // pub fn scan<P: AsRef<Path>>(source: P) -> DirInfo {
    let mut dirinfo = DirInfo::new();
    let mut updatetimer = std::time::Instant::now();

    WalkDir::new(&source).into_iter().for_each(|x| {
        debug!("{:?}", x);
    });

    //   WalkDir::new(&source).par_iter().for_each(|x| {});
    WalkDir::new(&source)
        .into_iter()
        .flatten()
        // .filter_map(|x| x.ok())
        .for_each(|x| {
            debug!("{:?}", &x);
            if x.path().starts_with(".") {}
            if x.path().is_file() {
                let ext_string: Option<String> = x
                    .path()
                    .extension()
                    .map(|x| x.to_string_lossy().to_string().to_lowercase());

                // Make sure metadata is available for the file
                if let Ok(meta) = x.path().metadata() {
                    let size = meta.len();
                    dirinfo.combined_size += size;
                    let file = File {
                        size: size,
                        ext: ext_string.clone(),
                        path: x.path().to_path_buf(),
                        modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    };
                    // Since we are at a file level, the parent is the enclosing folder
                    if let Some(containing_dir) = x.path().parent() {
                        // let p = parent.to_path_buf();
                        let tree_dir =
                            dirinfo
                                .tree
                                .entry(containing_dir.to_path_buf())
                                .or_insert(Directory {
                                    path: containing_dir.to_path_buf(),
                                    parent: containing_dir.parent().map(|x| x.to_path_buf()),
                                    ..Default::default()
                                });
                        tree_dir.files.push(file.clone());
                        tree_dir.size += size;
                        for a in containing_dir.ancestors() {
                            // debug!("Adding {:?} to {}", x.path().display(), a.display());

                            dirinfo
                                .tree
                                .entry(a.to_path_buf())
                                .or_insert(Directory {
                                    path: a.to_path_buf(),
                                    parent: a.parent().map(|x| x.to_path_buf()),
                                    ..Default::default()
                                })
                                .combined_size += size;
                        }
                        // tree_dir.combined_size += size;
                        // debug!(
                        //     "{}: Added {} to {}",
                        //     x.path().display(),
                        //     size,
                        //     containing_dir.display()
                        // )
                    }
                    if let Some(ext) = ext_string {
                        let ftype = dirinfo.filetypes.entry(ext.clone()).or_insert(FileType {
                            ext: ext,
                            size: size,
                            files: vec![],
                        });
                        //.size += size as u64;
                        //ftype.size += size as u64;
                        ftype.files.push(file.clone());
                    }
                    dirinfo.files.push(file.clone());
                }
            }

            // TODO this should not include dirs outside scan root
            if x.path().is_dir() {
                if let Some(parent) = x.path().parent() {
                    //debug!("{:?} parent: {:?}", x.path(), &parent);

                    let entry = dirinfo
                        .tree
                        .entry(parent.to_path_buf())
                        .or_insert(Directory {
                            path: parent.to_path_buf(),
                            ..Default::default()
                        });

                    entry.directories.push(x.path().to_path_buf());
                }
            }

            // i.filetype_sizes.insert(k, v)
            // do sth here as callback
            // too expensive
            if updatetimer.elapsed().as_millis() > update_rate_ms {
                // dirinfo.files_by_size = dirinfo.files_by_size();
                // dirinfo.types_by_size = dirinfo.types_by_size();
                // dirinfo.dirs_by_size = dirinfo.dirs_by_size();
                callback(&dirinfo);
                updatetimer = std::time::Instant::now();
            }
        });

    dirinfo.files_by_size = dirinfo.files_by_size();
    dirinfo.types_by_size = dirinfo.types_by_size();
    dirinfo.dirs_by_size = dirinfo.dirs_by_size();

    dirinfo
}

pub fn scan<P: AsRef<Path>>(source: P) -> DirInfo {
    scan_callback(source, |_| {}, 100000)
}

// pub fn scan_callback<P: AsRef<Path>>(source: P, callback: &dyn Fn(&DirInfo)) -> DirInfo {
//     let mut dirinfo = DirInfo::new();
//     callback(&dirinfo);
//     dirinfo

// }
