// 第一版
// use core::panic::PanicInfo;

// #[panic_handler]
// fn panic(_info:&PanicInfo)->! {
//     loop{}
// }   

// 第二版 配合sbi.rs 优化错误处理
use crate::sbi::shutdown;
use core::panic::PanicInfo;

// 错误版本（已修复）：
// #[panic_handler]
// fn panic(info: &PanicInfo) -> ! {
//     if let Some(location) = info.location() {
//         println!(
//             "Panicked at {}:{} {}",
//             location.file(),
//             location.line(),
//             info.message().unwrap()  // ← 错误：PanicMessage 没有 unwrap() 方法
//         );
//     } else {
//         println!("Panicked: {}", info.message().unwrap());  // ← 错误：同上
//     }
//     shutdown(true)
// }
//
// 错误原因：
// 1. error[E0599]: no method named `unwrap` found for struct `PanicMessage`
// 2. 在 Rust 1.81.0+ 中，info.message() 直接返回 &PanicMessage，而不是 Option
// 3. PanicMessage 实现了 Display trait，可以直接用于 println! 的 {} 格式化
// 4. 不需要调用 .unwrap()，直接使用即可
//
// 类型分析：
//   info: &PanicInfo
//     ↓ .message()
//   &PanicMessage  ← 直接返回引用，不是 Option！
//     ↓ 实现了 Display trait
//   println!("{}", info.message())  ← 直接使用
//
// 修改时间：2025-10-31
// 修改内容：删除 .unwrap() 调用，直接使用 info.message()

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()  // ✅ 正确：直接使用，PanicMessage 实现了 Display
        );
    } else {
        println!("Panicked: {}", info.message());  // ✅ 正确：同上
    }
    shutdown(true)
}