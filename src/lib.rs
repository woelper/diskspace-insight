#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use jwalk::{WalkDir};
use rayon::prelude::*;

pub struct File {
    size: u32,
    ext: Option<String>,
    path: PathBuf,
}

#[derive(Debug)]
/// DirInfo holds all info about a diretory.
pub struct DirInfo {
    pub filetype_sizes: HashMap<String, u64>,
}

impl DirInfo {
    pub fn new<P: AsRef<Path>>(source: P) -> DirInfo {
        DirInfo {
            filetype_sizes: HashMap::new(),
        }
    }

    pub fn largest_types(&self) -> Vec<(&String, &u64)> {
        let mut count: Vec<_> = self.filetype_sizes.iter().collect();
        count.sort_by(|a, b| b.1.cmp(a.1));

        count
        // self.filetype_sizes.iter().cloned().map(|x| x).collect()
    }
}

pub fn scan<P: AsRef<Path>>(source: P) -> DirInfo {



    let mut i = DirInfo::new(&source);

    //   WalkDir::new(&source).par_iter().for_each(|x| {});
      WalkDir::new(&source).into_iter().filter_map(|x| x.ok()).for_each(|x| {
        // println!("{:?}", x.path());
        if let Some(ext) = x.path().extension() {
            if let Ok(meta) = x.path().metadata() {

                *i.filetype_sizes.entry(ext.to_string_lossy().to_string()).or_insert(meta.len()) += meta.len() ;
            }
        }
          
        // i.filetype_sizes.insert(k, v)
      });

    i
}
