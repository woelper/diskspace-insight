#[cfg(test)]
mod tests;

use jwalk::WalkDir;
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

#[derive(Debug, Clone, Default)]
/// A File, representing a file on disk
pub struct Directory {
    pub size: u64,
    pub combined_size: u64,
    pub path: PathBuf,
    pub files: Vec<File>,
    pub directories: Vec<Directory>,
}

impl Directory {
    fn new(path: PathBuf) -> Directory {
        Directory {
            path,
            ..Default::default()
        }
    }
}

impl Iterator for Directory {
    type Item = Directory;
    fn next(&mut self) -> Option<Directory> {
        self.directories.pop()
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
    pub combined_size: u64
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
            combined_size: 0
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
pub fn scan_callback<P: AsRef<Path>, F: Fn(&DirInfo)>(source: P, callback: F, update_rate_ms: u128) -> DirInfo {

// pub fn scan<P: AsRef<Path>>(source: P) -> DirInfo {
    let mut dirinfo = DirInfo::new();
    let mut updatetimer = std::time::Instant::now();

    // let mut treemap = HashMap::new();

    //   WalkDir::new(&source).par_iter().for_each(|x| {});
    WalkDir::new(&source)
        .into_iter()
        .filter_map(|x| x.ok())
        .for_each(|x| {
            if x.path().is_file() {
                if let Some(ext) = x.path().extension() {
                    if let Ok(meta) = x.path().metadata() {
                        let size = meta.len();
                        dirinfo.combined_size += size;
                        let modified = meta.modified();
                        let ext_string = ext.to_string_lossy().to_string().to_lowercase();
                        let file = File {
                            size: size,
                            ext: Some(ext_string.clone()),
                            path: x.path().to_path_buf(),
                            modified: modified.unwrap_or(SystemTime::UNIX_EPOCH),
                        };
                        if let Some(parent) = x.path().parent() {
                            let p = parent.to_path_buf();
                            let d = dirinfo.tree.entry(p.clone()).or_insert(Directory {
                                path: p,
                                // files: vec![file.clone()],
                                ..Default::default()
                            });
                            d.files.push(file.clone());
                            d.size += size;
                        }
                        let ftype =
                            dirinfo
                                .filetypes
                                .entry(ext_string.clone())
                                .or_insert(FileType {
                                    ext: ext_string.clone(),
                                    size: size,
                                    files: vec![],
                                });
                        //.size += size as u64;
                        //ftype.size += size as u64;
                        ftype.files.push(file.clone());
                        dirinfo.files.push(file.clone());
                    }
                }
            }
            
            // TODO this should not include dirs outside scan root
            if x.path().is_dir() {
                if let Some(parent) = x.path().parent() {
                    //dbg!(&parent);
                    let this_dir = Directory::new(x.path().to_path_buf());

                    let entry = dirinfo
                        .tree
                        .entry(parent.to_path_buf())
                        .or_insert(Directory {
                            path: parent.to_path_buf(),
                            ..Default::default()
                        });
                    entry.directories.push(this_dir);
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

    // go through all dirs and recursively collect all sizes of subdirs
    for d in &dirinfo.tree {

    }

    dirinfo
}


pub fn scan<P: AsRef<Path>>(source: P) -> DirInfo {
    scan_callback(source, |f| {}, 100000)
}

// pub fn scan_callback<P: AsRef<Path>>(source: P, callback: &dyn Fn(&DirInfo)) -> DirInfo {
//     let mut dirinfo = DirInfo::new();
//     callback(&dirinfo);
//     dirinfo

// }