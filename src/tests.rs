use super::*;

#[test]
fn base_scan() {
    let i = scan(".");
    dbg!(&i.types_by_size()[..1]);
    dbg!(&i.files_by_size()[..1]);
}
