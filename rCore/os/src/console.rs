use crate::sbi::console_putchar;
use core::fmt::{self, Write};
// 我们在 console 子模块中编写 println! 宏。结构体 Stdout 不包含任何字段，因此它被称为类单元结构体（Unit-like structs，请参考 1 ）。 core::fmt::Write trait 包含一个用来实现 println! 宏很好用的 write_fmt 方法，为此我们准备为结构体 Stdout 实现 Write trait 。在 Write trait 中， write_str 方法必须实现，因此我们需要为 Stdout 实现这一方法，它并不难实现，只需遍历传入的 &str 中的每个字符并调用 console_putchar 就能将传入的整个字符串打印到屏幕上。

// 在此之后 Stdout 便可调用 Write trait 提供的 write_fmt 方法并进而实现 print 函数。在声明宏（Declarative macros，参考 2 ） print! 和 println! 中会调用 print 函数完成输出。
struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

// TODO： 1.实现带颜色的输出 2.log等级控制，不同log等级按照不同颜色输出 3.利用彩色输出宏输出os内存空间布局 输出 .text、.data、.rodata、.bss 各段位置，输出等级为 INFO。 支持如下调用：
// info!(".text [{:#x}, {:#x})", s_text as usize, e_text as usize);
// debug!(".rodata [{:#x}, {:#x})", s_rodata as usize, e_rodata as usize);
// error!(".data [{:#x}, {:#x})", s_data as usize, e_data as usize);

// gdb调试追踪qemu从机器加电到跳转到 0x80200000 的简单过程  阅读rustsbi的起始代码
// TODO: 4.在log信息中增加线程cpu等信息
// TODO:5.支持以如下方式直接运行：
// cd os
// git checkout ch1
// make run LOG=INFO

#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[34m","[INFO]",$fmt, "\n","\x1b[0m") $(, $($arg)+)?));
    }
}
#[macro_export]
macro_rules! warn {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[93m","[WARN]",$fmt, "\n","\x1b[0m") $(, $($arg)+)?));
    }
}
#[macro_export]
macro_rules! error {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[31m","[ERROR]",$fmt, "\n","\x1b[0m") $(, $($arg)+)?));
    }
}
#[macro_export]
macro_rules! debug {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[32m","[DEBUG]",$fmt, "\n","\x1b[0m") $(, $($arg)+)?));
    }
}
#[macro_export]
macro_rules! trace {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[90m","[TRACE]",$fmt, "\n","\x1b[0m") $(, $($arg)+)?));
    }
}