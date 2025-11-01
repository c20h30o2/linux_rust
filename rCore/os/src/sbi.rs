// os/src/sbi.rs
pub fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{NoReason, Shutdown, SystemFailure, system_reset};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}
// sbi_rt 是如何调用 SBI 服务的
// SBI spec 的 Chapter 3 介绍了服务的调用方法：只需将要调用功能的拓展 ID 和功能 ID 分别放在 a7 和 a6 寄存器中，并按照 RISC-V 调用规范将参数放置在其他寄存器中，随后执行 ecall 指令即可。这会将控制权转交给 RustSBI 并由 RustSBI 来处理请求，处理完成后会将控制权交还给内核。返回值会被保存在 a0 和 a1 寄存器中。在本书的第二章中，我们会手动编写汇编代码来实现类似的过程。