use super::*;

#[test]
fn it_works() {
    let i = scan("/home/woelper/Downloads");
    dbg!(&i.types_by_size()[..10]);
    dbg!(&i.files_by_size()[..10]);
}
