// 第一版实现
// // os/src/main.rs
// #[no_mangle]
// pub fn rust_main() -> ! {
//     clear_bss();
//     loop {}
// }
// fn clear_bss() {
//     extern "C" {
//         fn sbss();
//         fn ebss();
//     }
//     (sbss as usize..ebss as usize).for_each(|a| {
//         unsafe { (a as *mut u8).write_volatile(0) }
//     });
// }

// 第二版实现 注意与第一版的区别，基本上是第一版存在的错误
// os/src/main.rs
// #![no_std] // 不使用标准库
// #![no_main] // 不使用 Rust 默认 main 入口

// mod lang_items; // panic_handler 等语言项

// use core::arch::global_asm;
// global_asm!(include_str!("entry.asm")); // 包含汇编代码

// #[unsafe(no_mangle)] // 不修改函数名
// pub fn rust_main() -> ! {
//     // 永不返回
//     clear_bss(); // 清零 BSS 段
//     loop {} // 主循环
// }

// fn clear_bss() {
//     unsafe extern "C" {
//         unsafe fn sbss(); // 链接脚本定义
//         unsafe fn ebss(); // 链接脚本定义
//     }
//     (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
// }

// 第三版实现，引入了sbi-st封装的调用sbi服务的接口，并基于sbi服务完成输出和关机
// 在Cargo.toml中添加对sbi-rt的依赖
// 在sbi.rs中为sbi_rt封装了函数
// 在console.rs中利用sbi.rs中的输出函数做了输出宏的封装
// os/src/main.rs
#![no_std] // 不使用标准库
#![no_main] // 不使用 Rust 默认 main 入口

// 错误版本（已修复）：
// #![feature(panic_info_message)]
//
// 错误原因：
// 1. error[E0554]: `#![feature]` may not be used on the stable release channel
// 2. panic_info_message 特性在 Rust 1.81.0 已经稳定，不再需要 feature gate
// 3. 在 stable Rust 上使用 #![feature] 会导致编译错误
//
// 修改时间：2025-10-31
// 修改内容：删除 #![feature(panic_info_message)] 这行

#[macro_use]
mod console;
mod lang_items; // panic_handler 等语言项
mod sbi; // SBI 调用封装,虽然main.rs中没有直接用到sbi.rs但是console.rs需要依赖sbi.rs,
// 如果不mod sbi引入sbi,编译器将不会知道sbi的存在，所以仍然需要引入

use core::arch::global_asm;
global_asm!(include_str!("entry.asm")); // 包含汇编代码

unsafe extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
}

#[unsafe(no_mangle)] // 不修改函数名
pub fn rust_main() -> ! {
    // 永不返回
    clear_bss(); // 清零 BSS 段
    println!("this is a test");
    info!("this is a info");
    warn!("warn");
    trace!("trace");
    error!("error");
    debug!("debug");

    info!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    debug!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    error!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    info!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);

    panic!("Shutdown machine!");
    // loop {} // 主循环
}

fn clear_bss() {
    unsafe extern "C" {
        unsafe fn sbss(); // 链接脚本定义
        unsafe fn ebss(); // 链接脚本定义
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

// 测试流程：
// cargo build --release
// rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os -O binary target/riscv64gc-unknown-none-elf/release/os.bin
// 然后运行：
// qemu-system-riscv64     -machine virt     -nographic     -bios ../bootloader/rustsbi-qemu.bin     -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000

// 注意，要运行这里的测试，需要以日常开发的方式运行，如果以调试方法运行，则this is a test不会直接输出，需要用gdb连接后调试才能输出
//   日常开发（看输出）：
//   qemu-system-riscv64 \
//       -machine virt \
//       -nographic \
//       -bios ../bootloader/rustsbi-qemu.bin \
//       -device
//   loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000

//   需要调试时（设置断点）：
// break *0x80200000: 在 0x80200000 处设置断点。
//   终端 1：
//   qemu-system-riscv64 ... -s -S

//   终端 2：
//   riscv64-unknown-elf-gdb target/riscv64gc-unknown-none-elf/release/os
// 或者使用下面指令启动：
//  riscv64-unknown-elf-gdb \
//      -ex 'file target/riscv64gc-unknown-none-elf/release/os' \
//      -ex 'set arch riscv:rv64' \
//      -ex 'target remote localhost:1234'

//   (gdb) target remote :1234
//   (gdb) break rust_main
//   (gdb) continue

// 现在将lang_item.rs改版调用shutdown,当有错误产生时打印错误并关机


// 配合makefile可以这样运行
// cd os
// git checkout ch1
// make run LOG=INFO