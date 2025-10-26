use std::env;
use std::process;

use ioproject05::Config;
// 读取参数值
// 为了确保 minigrep 能够获取传递给它的命令行参数的值，
// 我们需要一个 Rust 标准库提供的函数，也就是 std::env::args。
// 这个函数返回一个传递给程序的命令行参数的 迭代器（iterator）。
// 我们会在 第 13 章 全面的介绍它们。
// 但是现在只需理解迭代器的两个细节：迭代器生成一系列的值，
// 可以在迭代器上调用 collect 方法将其转换为一个集合，
// 比如包含所有迭代器产生元素的 vector。
fn main() {
    // let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);
    // let query = &args[1];
    // let filename = &args[2];
    // let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        // println!("Problem parsing arguments: {}", err);
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = ioproject05::run(config) {
        // println!("Application error: {}", e);
        eprintln!("Application error: {}", e);
        // epirntln!将输出打印到标准错误流,println将输出打印到标准输出流
        // 默认情况下标准输出与标准错误拥有同一个文件描述符，他们的输出都会在终端上显示
        // 使用 cargo run to poem.txt > output.txt运行程序
        // 会将标准输出流重定向到output文件，而标准错误流仍然是输出到终端


        process::exit(1);
    }
}

// args 函数和无效的 Unicode
// 注意 std::env::args 在其任何参数包含无效 Unicode 字符时会 panic。
// 如果你需要接受包含无效 Unicode 字符的参数，使用 std::env::args_os 代替。
// 这个函数返回 OsString 值而不是 String 值。这里出于简单考虑使用了 std::env::args，
// d因为 OsString 值每个平台都不一样而且比 String 值处理起来更为复杂。
