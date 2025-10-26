use std::env;
use std::process;

use ioproject06::Config;

fn main() {
  
    let args: Vec<String> = env::args().collect();

    let config = Config::new(args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = ioproject06::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}



// 为什么 let args: Vec<String> = env::args().collect(); 的args能被接受为 &[String] 与 &Vec<String> 类型的参数

// 简短回答 — 因为借用与自动类型协变/解引用（deref coercion）。
// 要点：
// env::args().collect() 返回一个 Vec<String>（拥有所有权的容器）。
// 函数如果接受 &[String] 或 &Vec<String>，调用时你传入的是对 Vec 的引用（借用），例如 &args 或 &args[..]。
// &Vec<String> 可以自动“解引用”为 &[String]（Vec<T> 实现了 Deref<Target=[T]>，编译器会做 deref coercion），所以这两种形参都能接受 &args。
// 示例：
// fn take_slice(s: &[String]) { /* ... */ }
// fn take_vec_ref(v: &Vec<String>) { /* ... */ }

// let args: Vec<String> = std::env::args().collect();

// // 两种调用都可以
// take_slice(&args);      // &Vec<String> -> &[String] 自动协变
// take_slice(&args[..]);  // 显式切片

// take_vec_ref(&args);    // 传入 &Vec<String>

// 补充：
// 如果函数签名是 fn foo(args: Vec<String>)，那就必须把所有权转移进去：foo(args) 会移动（consume）args，之后不能再使用。
// 如果想避免拷贝但又不想移动所有权，使用借用（&args）并在函数内部按需 clone 或者只借用字符串引用。