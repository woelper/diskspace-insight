use super::*;
use env_logger;
use log::*;
use std::process::Command;

#[test]
fn base_scan() {
    let i = scan(".");
    dbg!(&i.types_by_size()[..1]);
    dbg!(&i.files_by_size()[..1]);
}

#[test]
fn tree() {
    std::env::set_var("RUST_LOG", "debug");
    let _ = env_logger::try_init();

    Command::new("mkdir")
        .arg("-p")
        .arg("treetest/a/b/c")
        .output()
        .unwrap();

    Command::new("dd")
        .arg("if=/dev/urandom")
        .arg("of=treetest/a/file_5m")
        .arg("bs=5MB")
        .arg("count=1")
        .output()
        .unwrap();

    Command::new("dd")
        .arg("if=/dev/urandom")
        .arg("of=treetest/a/b/file_10m")
        .arg("bs=10MB")
        .arg("count=1")
        .output()
        .unwrap();

    Command::new("dd")
        .arg("if=/dev/urandom")
        .arg("of=treetest/a/b/c/file_15m")
        .arg("bs=15MB")
        .arg("count=1")
        .output()
        .unwrap();

    // truncate -s 5M ostechnix.txt

    let i = scan("treetest");

    info!("=== Files By Size");
    
    for d in &i.files_by_size {
        info!("{:?}", &d.path);
    }
    
    info!("=== Dirs By Size");

    for d in &i.dirs_by_size {
        info!("{:?}", &d.path);
    }

    info!("=== Dirs By Tree");
    for d in &i.tree {
        info!("{}", d.1);
    }

 
}
