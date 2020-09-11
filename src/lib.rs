#[cfg(test)]
mod tests;

use jwalk::WalkDir;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[derive(Debug, Clone)]
pub struct File {
    pub size: u64,
    pub ext: Option<String>,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct FileType {
    pub size: u64,
    pub ext: String,
    pub files: Vec<File>,
}


#[derive(Debug)]
/// DirInfo holds all info about a diretory.
pub struct DirInfo {
    pub filetypes: HashMap<String, FileType>,
    pub files: Vec<File>,
    pub files_by_size: Vec<File>,
    pub types_by_size: Vec<FileType>
}

impl DirInfo {
    pub fn new<P: AsRef<Path>>(source: P) -> DirInfo {
        DirInfo {
            filetypes: HashMap::new(),
            files: vec![],
            files_by_size: vec![],
            types_by_size: vec![]
        }
    }

    fn types_by_size(&self) -> Vec<FileType> {
        let mut ftypes: Vec<_> = self.filetypes.par_iter().map(|f| f.1).map(|f| {
            let mut f = f.clone();
            f.files.par_sort_by(|a, b| b.size.cmp(&a.size));
            f
        
        }).collect();
        ftypes.par_sort_by(|a, b| b.size.cmp(&a.size));
        ftypes
    }

    fn files_by_size(&self) -> Vec<File> {
        let mut count = self.files.clone();
        count.par_sort_by(|a, b| b.size.cmp(&a.size));
        count
    }
}

pub fn scan<P: AsRef<Path>>(source: P) -> DirInfo {
    let mut dirinfo = DirInfo::new(&source);

    //   WalkDir::new(&source).par_iter().for_each(|x| {});
    WalkDir::new(&source)
        .into_iter()
        .filter_map(|x| x.ok())
        .for_each(|x| {
            // println!("{:?}", x.path());
            if let Some(ext) = x.path().extension() {
                if let Ok(meta) = x.path().metadata() {
                    let size = meta.len();
                    let ext_string = ext.to_string_lossy().to_string().to_lowercase();
                    let file = File {
                        size: size,
                        ext: Some(ext_string.clone()),
                        path: x.path().to_path_buf(),
                    };
                    let ftype = dirinfo.filetypes.entry(ext_string.clone()).or_insert(
                FileType {
                            ext: ext_string.clone(),
                            size: size,
                            files: vec![]                    
                    });
                    //.size += size as u64;
                    ftype.size += size as u64;
                    ftype.files.push(file.clone());
                    dirinfo.files.push(file.clone());
                }
            }

            // i.filetype_sizes.insert(k, v)
        });

    dirinfo.files_by_size = dirinfo.files_by_size();
    dirinfo.types_by_size = dirinfo.types_by_size();

    dirinfo
}
