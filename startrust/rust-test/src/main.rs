// #![allow(unused)]
// fn main() {
//     let i1:i32=1;
//     // let _i2:&i32=&i1;
//     let i2:&i32=&i1;
//     let _i3=&i1;
// }

enum list {
    cons(i32,BOX<list>);
    Nil;
}
use crate::list::{cons,Nil};
fn main() {
    // let li=cons(1,cons(2,cons,))
    let li=BOX::new
}