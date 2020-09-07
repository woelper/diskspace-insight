use super::*;

#[test]
fn it_works() {
    let i = scan("/home/woelper/Downloads");
    dbg!(&i.largest_types()[..10]);
}