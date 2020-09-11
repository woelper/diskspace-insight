use super::*;

#[test]
fn it_works() {
    let i = scan(".");
    dbg!(&i.types_by_size()[..1]);
    dbg!(&i.files_by_size()[..1]);
}
