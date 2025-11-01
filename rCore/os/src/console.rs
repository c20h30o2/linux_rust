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

// ============================================================================
// 日志宏 - 使用 Cargo Features 实现条件编译
// ============================================================================
//
// 实现方式：通过 #[cfg(feature = "log-xxx")] 在编译期控制日志输出
//
// 优点：
//   - 零运行时开销（未启用的日志代码完全不存在）
//   - 更小的二进制体积（只包含启用的日志代码）
//   - 编译期确定，性能最优
//
// 使用方式：
//   make run LOG=ERROR  - 只显示 ERROR
//   make run LOG=WARN   - 显示 WARN + ERROR
//   make run LOG=INFO   - 显示 INFO + WARN + ERROR
//   make run LOG=DEBUG  - 显示 DEBUG + INFO + WARN + ERROR
//   make run LOG=TRACE  - 显示所有日志
//
// 实现原理：
//   #[cfg(feature = "log-info")] 会在编译期检查 feature 是否启用
//   - 启用：保留代码，编译进二进制
//   - 未启用：完全移除代码，不占用任何空间和性能
//
// ============================================================================

/// ERROR 级别日志 - 红色
/// 用于严重错误，总是应该显示
#[macro_export]
macro_rules! error {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        #[cfg(feature = "log-error")]
        $crate::console::print(format_args!(
            concat!("\x1b[31m[ERROR] ", $fmt, "\x1b[0m\n")
            $(, $($arg)+)?
        ));
    }
}

/// WARN 级别日志 - 亮黄色
/// 用于警告信息
#[macro_export]
macro_rules! warn {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        #[cfg(feature = "log-warn")]
        $crate::console::print(format_args!(
            concat!("\x1b[93m[WARN ] ", $fmt, "\x1b[0m\n")
            $(, $($arg)+)?
        ));
    }
}

/// INFO 级别日志 - 蓝色
/// 用于一般信息
#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        #[cfg(feature = "log-info")]
        $crate::console::print(format_args!(
            concat!("\x1b[34m[INFO ] ", $fmt, "\x1b[0m\n")
            $(, $($arg)+)?
        ));
    }
}

/// DEBUG 级别日志 - 绿色
/// 用于调试信息
#[macro_export]
macro_rules! debug {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        #[cfg(feature = "log-debug")]
        $crate::console::print(format_args!(
            concat!("\x1b[32m[DEBUG] ", $fmt, "\x1b[0m\n")
            $(, $($arg)+)?
        ));
    }
}

/// TRACE 级别日志 - 灰色
/// 用于详细跟踪信息
#[macro_export]
macro_rules! trace {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        #[cfg(feature = "log-trace")]
        $crate::console::print(format_args!(
            concat!("\x1b[90m[TRACE] ", $fmt, "\x1b[0m\n")
            $(, $($arg)+)?
        ));
    }
}