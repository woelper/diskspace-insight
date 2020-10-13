use super::*;
use env_logger;
use log::*;
use std::process::Command;

// #[test]
// fn base_scan() {
//     let i = scan(".");
//     dbg!(&i.types_by_size()[..1]);
//     dbg!(&i.files_by_size()[..1]);
// }

#[test]
fn tree() {
    std::env::set_var("RUST_LOG", "debug");
    let _ = env_logger::try_init();

    Command::new("mkdir")
        .arg("-p")
        .arg("treetest/a/b/c")
        .output()
        .unwrap();
    Command::new("mkdir")
        .arg("-p")
        .arg("treetest/a/.conf/c")
        .output()
        .unwrap();
    Command::new("mkdir")
        .arg("-p")
        .arg("treetest/a/b/d")
        .output()
        .unwrap();

    Command::new("dd")
        .arg("if=/dev/urandom")
        .arg("of=treetest/a/file_5m.5")
        .arg("bs=5MB")
        .arg("count=1")
        .output()
        .unwrap();

    Command::new("dd")
        .arg("if=/dev/urandom")
        .arg("of=treetest/a/b/file_10m.10")
        .arg("bs=10MB")
        .arg("count=1")
        .output()
        .unwrap();

    Command::new("dd")
        .arg("if=/dev/urandom")
        .arg("of=treetest/a/b/c/file_15m.15")
        .arg("bs=15MB")
        .arg("count=1")
        .output()
        .unwrap();

    Command::new("dd")
        .arg("if=/dev/urandom")
        .arg("of=treetest/a/b/d/file_20m.20")
        .arg("bs=20MB")
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
        info!("{:?}: {}", d.path, ByteSize(d.size));
    }

    info!("=== Dirs By Combined Size");
    for (p, d) in &i.tree {
        info!("{} {}", p.display(), ByteSize(d.combined_size));
    }

    Command::new("rm")
        .arg("-rf")
        .arg("treetest")
        .output()
        .unwrap();
}

#[test]
fn walk() {
    std::env::set_var("RUST_LOG", "debug");
    let _ = env_logger::try_init();

    Command::new("mkdir")
        .arg("-p")
        .arg("test/a/.b/c")
        .output()
        .unwrap();

    Command::new("dd")
        .arg("if=/dev/urandom")
        .arg("of=test/a/file_20m.20")
        .arg("bs=20MB")
        .arg("count=1")
        .output()
        .unwrap();

    Command::new("dd")
        .arg("if=/dev/urandom")
        .arg("of=test/a/.b/file_20m.20")
        .arg("bs=20MB")
        .arg("count=1")
        .output()
        .unwrap();

    let ls = Command::new("ls").arg("-lRa").arg("test").output().unwrap().stdout;
    // let ls = Command::new("tree").arg("-a").arg("test").output().unwrap().stdout;
    for line in std::str::from_utf8(&ls).unwrap().split("\n") {

        info!("{:?}", line);
    }

    for entry in WalkDir::new("test").into_iter()
    // .filter_map(|e| e.ok())
    {
        info!("{:?}", entry);
    }

    Command::new("rm").arg("-rf").arg("test").output().unwrap();
}
