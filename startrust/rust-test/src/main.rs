// #![allow(unused)]
// fn main() {
//     let i1:i32=1;
//     // let _i2:&i32=&i1;
//     let i2:&i32=&i1;
//     let _i3=&i1;
// }
// #[warn(non_camel_case_types)]// 为使用大驼峰命名法的报错
#[allow(dead_code)] // 存在未使用的字段的报错
enum List {
    Cons(i32, Box<List>),
    Nil,
}
use crate::List::{Nil, Cons};
fn main() {
    // let li=cons(1,cons(2,cons,))
    // let li = Box::new(cons(1, Box::new(2, Box::new(3, Nil))));
    let li=Box::new(Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil)))))));
    match *li {
        Cons(first,_)=>println!("the first{}",first),
        Nil=>println!("Nil"),
    }
}
