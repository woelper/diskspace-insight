#[cfg(test)]
mod tests;

use bytesize::ByteSize;
use log::{info, error, debug};
use walkdir::WalkDir;
use rayon::prelude::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{Read, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone)]
/// A File, representing a file on disk
pub struct File {
    pub size: u64,
    pub ext: Option<String>,
    pub path: PathBuf,
    pub modified: SystemTime,
    pub hash: u64
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

    /// Return a list of directories by size
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

    /// Return a list of files by size
    pub fn sorted_files(&self) -> Vec<File> {
        let mut sorted_files = self.files.clone();
        sorted_files.sort_by(|a, b| b.size.cmp(&a.size));
        sorted_files
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
/// A type of files, such as all txt files.
pub struct FileType {
    /// Combined size of this FileType
    pub size: u64,
    /// The extension of this FileType, such as `txt`
    pub ext: String,
    /// The files belonging to this type
    pub files: Vec<File>,
}

#[derive(Debug, Clone, Default)]
/// A DirInfo holds all info about a directory.
pub struct DirInfo {
    /// All file types
    pub filetypes: HashMap<String, FileType>,
    pub files: Vec<File>,
    /// Files, ordered by size, descending
    pub files_by_size: Vec<File>,
    /// Filetypes, ordered by size, descending
    pub types_by_size: Vec<FileType>,
    /// Directories, ordered by size
    pub dirs_by_size: Vec<Directory>,
    pub tree: HashMap<PathBuf, Directory>,
    /// Cumulated size
    pub combined_size: u64,
    /// All duplicates
    pub duplicates: HashMap<u64, Vec<File>>,
}

impl DirInfo {
    /// Construct a new DirInfo
    pub fn new() -> DirInfo {
        DirInfo::default()
    }

    /// Return file types, ordered by size
    pub fn types_by_size(&self) -> Vec<FileType> {
        let mut ftypes: Vec<_> = self
            .filetypes
            .par_iter()
            .map(|(_ext, filetype)| filetype)
            .map(|f| {
                let mut f = f.clone();
                f.files.par_sort_by(|a, b| b.size.cmp(&a.size));
                f
            })
            .collect();
        ftypes.par_sort_by(|a, b| b.size.cmp(&a.size));
        ftypes
    }

    /// Return all files, ordered by size
    pub fn files_by_size(&self) -> Vec<File> {
        let mut count = self.files.clone();
        count.par_sort_by(|a, b| b.size.cmp(&a.size));
        count
    }

    /// Return all directories, ordered by size
    pub fn dirs_by_size(&self) -> Vec<Directory> {
        let mut dirs: Vec<Directory> = self.tree.values().cloned().collect();
        dirs.par_sort_by(|a, b| b.size.cmp(&a.size));
        dirs
    }

    /// Return all duplicates
    pub fn duplicates_from_files(&self) -> HashMap<u64, Vec<File>> {
        let mut dupemap: HashMap<u64, Vec<File>> = HashMap::new();
        for file in &self.files {
            dupemap
                .entry(file.hash)
                .and_modify(|e| e.push(file.clone()))
                .or_insert(vec![file.clone()]);
        }
        // leave only duplicates in
        dupemap = Self::build_duplicates(&dupemap);
        // TODO: Calculate a proper hash and check if these are actually identical
        dupemap
    }

    /// Return all duplicates
    fn build_duplicates(map: &HashMap<u64, Vec<File>>) -> HashMap<u64, Vec<File>> {
        map.par_iter()
            .filter(|e| e.1.len() > 1)
            .map(|(h, f)| (*h, f.clone()))
            .collect()
    }

    /// Return all duplicates
    fn build_duplicates_mut(&mut self) {
        self.duplicates = Self::build_duplicates(&self.duplicates);
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

    WalkDir::new(&source)
        // .skip_hidden(false)
        .into_iter()
        .flatten()
        .for_each(|x| {
            // TODO this should not include dirs outside scan root
            // TODO: if/else to avoid both is_dir and is_file
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
            // if x.path().is_file() {
            // Assume it's a file
            else {
                let ext_string: Option<String> = x
                    .path()
                    .extension()
                    .map(|x| x.to_string_lossy().to_string().to_lowercase());

                // Make sure metadata is available for the file
                if let Ok(meta) = x.metadata() {
                    let size = meta.len();
                    dirinfo.combined_size += size;
                    let hash = hash_file(&x.path()).unwrap_or_default();
                    debug!("{:?} is {}", x.path(), hash);
                    let file = File {
                        size,
                        ext: ext_string.clone(),
                        path: x.path().to_path_buf(),
                        modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                        hash
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

                            if let Some(p) = source.as_ref().parent() {
                                if a == p {
                                    break;
                                }
                            }
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
                    }
                    if let Some(ext) = ext_string {
                        let ftype = dirinfo.filetypes.entry(ext.clone()).or_insert(FileType {
                            ext,
                            size: 0,
                            files: vec![],
                        });
                        ftype.files.push(file.clone());
                        ftype.size += file.size;
                    }
                    dirinfo.files.push(file);
                }
            }

            // do sth here as callback
            if updatetimer.elapsed().as_millis() > update_rate_ms {
                callback(&dirinfo);
                updatetimer = std::time::Instant::now();
            }
        });

    dirinfo.files_by_size = dirinfo.files_by_size();
    dirinfo.types_by_size = dirinfo.types_by_size();
    dirinfo.dirs_by_size = dirinfo.dirs_by_size();
    dirinfo.duplicates = dirinfo.duplicates_from_files();

    dirinfo
}

/// Scan a root path and produce a DirInfo
pub fn scan<P: AsRef<Path>>(source: P) -> DirInfo {
    scan_callback(source, |_| {}, std::u128::MAX)
}

pub fn scan_archive<P: AsRef<Path>>(source: P) -> DirInfo {
    let mut dirinfo = DirInfo::new();

    let zipfile = fs::File::open(&source.as_ref()).unwrap();

    let mut archive = zip::ZipArchive::new(zipfile).unwrap();

    for i in 0..archive.len() {
        let mut zip_entry = archive.by_index(i).unwrap();

        if zip_entry.is_dir() {
        } else {
            // it's a file
            let ext_string = Path::new(zip_entry.name())
                .extension()
                .map(|x| x.to_string_lossy().to_string().to_lowercase());

            let size = zip_entry.compressed_size();
            let mut buf: Vec<u8> = vec![];
      
            if let Err(e)  = zip_entry.read_to_end(&mut buf) {
                error!("{e}");
            }
            
            let hash = hash_bytes(&buf);
            info!("{} {}", zip_entry.name(), hash);
            dirinfo.combined_size += size;
            let file = File {
                size,
                ext: ext_string.clone(),
                path: Path::new(zip_entry.name()).to_path_buf(),
                modified: SystemTime::now(),
                hash
            };

            dirinfo
                .duplicates
                .entry(zip_entry.compressed_size() as u64)
                .and_modify(|e| e.push(file.clone()))
                .or_insert(vec![file.clone()]);

            if let Some(containing_dir) = Path::new(zip_entry.name()).parent() {
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

                    if let Some(p) = source.as_ref().parent() {
                        if a == p {
                            break;
                        }
                    }
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
            }

            if let Some(ext) = ext_string {
                let ftype = dirinfo.filetypes.entry(ext.clone()).or_insert(FileType {
                    ext,
                    size: 0,
                    files: vec![],
                });
                ftype.files.push(file.clone());
                ftype.size += file.size;
            }
            dirinfo.files.push(file.clone());
        }
    }

    dirinfo.files_by_size = dirinfo.files_by_size();
    dirinfo.types_by_size = dirinfo.types_by_size();
    dirinfo.dirs_by_size = dirinfo.dirs_by_size();
    dirinfo.duplicates = dirinfo.duplicates_from_files();

    // dirinfo.build_duplicates_mut();

    dirinfo
}


fn hash_file(file: &Path) -> Result<u64> {
    let f = std::fs::File::open(file)?;
    let mut r = BufReader::new(f);
    let mut buf: Vec<u8> = vec![];
    r.read_to_end(&mut buf)?;
    // r.(&mut buf)?;
    // Ok(hash_bytes(&buf))
    Ok(hash_bytes(&buf))
}

fn hash_bytes(b: &[u8]) -> u64 {
    let mut s :twox_hash::Xxh3Hash64 = Default::default();
    b.hash(&mut s);
    s.finish()
}


// pub fn scan_callback<P: AsRef<Path>>(source: P, callback: &dyn Fn(&DirInfo)) -> DirInfo {
//     let mut dirinfo = DirInfo::new();
//     callback(&dirinfo);
//     dirinfo

// }
