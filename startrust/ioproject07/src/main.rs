use std::env;
use std::process;

use ioproject07::Config;

fn main() {
    // let args: Vec<String> = env::args().collect();

    // env::args() 返回的是可变所有权迭代器 ，所以可以直接在new函数中转移所有权
    // env::args 函数返回一个迭代器！不同于将迭代器的值收集到一个 vector 中接着传递一个 slice 给 Config::new，现在我们直接将 env::args 返回的迭代器的所有权传递给 Config::new。
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = ioproject07::run(config) {
        eprintln!("Application error: {}", e);


        process::exit(1);
    }
}

