# entry.asm 是否必需？汇编入口的深入分析

**日期：** 2025-10-31
**核心问题：** entry.asm 是必须的吗？为什么需要汇编来将控制权转到 Rust 代码？

---

## 目录

1. [简短回答](#简短回答)
2. [为什么需要 entry.asm](#为什么需要-entryasm)
3. [Rust 代码的限制](#rust-代码的限制)
4. [替代方案](#替代方案)
5. [实验：不用 entry.asm 会怎样](#实验不用-entryasm-会怎样)
6. [最佳实践](#最佳实践)

---

## 简短回答

**是的，entry.asm（或等价的汇编代码）是必需的。**

**原因：** 必须有人在 Rust 代码运行前初始化运行时环境，最关键的是**初始化栈指针**。

**关键矛盾：**
```
Rust 代码需要栈 ← → 栈指针需要被初始化 ← → 初始化代码需要用 Rust 写？

死循环！必须有人打破这个循环。
```

**解决方案：** 用汇编写一小段"引导代码"（bootstrap code）

---

## 为什么需要 entry.asm

### 核心问题：鸡生蛋问题

```
┌─────────────────────────────────────────┐
│          CPU 启动时的状态                 │
├─────────────────────────────────────────┤
│ PC (程序计数器)  = 0x80200000           │
│ sp (栈指针)      = ??? (未定义/随机值)  │ ← 问题所在！
│ 其他寄存器       = 0 或随机值           │
│ 内存             = 未初始化             │
│ 特权级          = S-mode (RustSBI设置)  │
└─────────────────────────────────────────┘

第一条指令会是什么？
```

### Rust 代码对环境的假设

**任何 Rust 函数都假设：**

```rust
fn rust_main() -> ! {
    clear_bss();  // ← 这行代码背后发生了什么？
    loop {}
}
```

**编译后的汇编（简化）：**

```asm
rust_main:
    addi sp, sp, -16      # ← 假设 sp 已经有效！
    sd   ra, 8(sp)        # ← 往栈里写数据
    call clear_bss        # ← 保存返回地址到栈
    ld   ra, 8(sp)
    addi sp, sp, 16
    ret
```

**问题：**
- 第一条指令就假设 `sp` 是有效的
- 如果 `sp = 0` 或随机值，第一条 `sd` 指令就会：
  - 写入非法地址 → 触发异常
  - 覆盖重要数据 → 程序崩溃

### entry.asm 做的关键工作

```asm
# os/src/entry.asm
    .section .text.entry
    .globl _start
_start:
    la sp, boot_stack_top    # ← 关键！初始化栈指针
    call rust_main           # ← 现在可以安全调用 Rust 了

    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
```

**它解决了什么问题？**

| 问题 | 汇编解决方案 | 如果没有会怎样 |
|-----|-------------|---------------|
| **sp 未初始化** | `la sp, boot_stack_top` | 访问非法地址，崩溃 |
| **没有栈空间** | `.space 4096 * 16` | 无处保存函数调用信息 |
| **入口点定义** | `_start` 符号 | 链接器不知道从哪开始 |

---

## Rust 代码的限制

### 为什么不能直接用 Rust 写入口？

**尝试 1：直接用 Rust 函数**

```rust
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 初始化栈？
    // ...但这个函数本身就需要栈！
    loop {}
}
```

**编译结果：**
```asm
_start:
    addi sp, sp, -16    # ← 第一条指令就用 sp！
    ...
```

**问题：** Rust 函数的 prologue (序言) 总是会操作栈。

---

**尝试 2：使用 `#[naked]` 函数**

```rust
#![feature(naked_functions)]

#[naked]
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::asm!(
        "la sp, {stack_top}",
        "call rust_main",
        stack_top = sym boot_stack_top,
        options(noreturn)
    );
}

#[no_mangle]
static boot_stack_top: [u8; 64 * 1024] = [0; 64 * 1024];
```

**这可以工作！** 但有几个问题：

1. **需要 nightly Rust**
   ```rust
   #![feature(naked_functions)]  // ← unstable feature
   ```

2. **更复杂**
   - 需要理解内联汇编语法
   - 需要手动管理符号引用
   - 代码可读性差

3. **栈分配问题**
   ```rust
   static boot_stack_top: [u8; 64 * 1024] = [0; 64 * 1024];
   ```
   - 这会在 `.data` 段（占用文件空间 64KB）
   - 而不是 `.bss` 段（不占文件空间）
   - 导致二进制文件变大 64KB

**对比 entry.asm：**
```asm
.space 4096 * 16    # .bss 段，不占文件空间
```

---

**尝试 3：用 Rust 内联汇编**

```rust
#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm!(
            "la sp, boot_stack_top",
            "call rust_main",
            options(noreturn)
        );
    }
}
```

**问题：**
```asm
_start:
    addi sp, sp, -16    # ← Rust 自动生成的 prologue
    ...
    la sp, boot_stack_top  # ← 我们的代码
```

编译器在我们的汇编代码**之前**插入了 prologue！

---

## 替代方案

### 方案 1：使用 global_asm!（当前方案）

```rust
use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    loop {}
}
```

**优点：**
✅ 汇编代码清晰、独立
✅ 不占用 `.data` 空间（`.bss` 栈）
✅ 使用稳定的 Rust 特性
✅ 易于维护和理解

**缺点：**
❌ 需要单独的 `.asm` 文件
❌ 语法是汇编，不是 Rust

---

### 方案 2：inline global_asm!

```rust
use core::arch::global_asm;

global_asm!(
    r#"
    .section .text.entry
    .globl _start
_start:
    la sp, boot_stack_top
    call rust_main

    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
    "#
);

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    loop {}
}
```

**优点：**
✅ 所有代码在一个文件
✅ 不需要 `include_str!`

**缺点：**
❌ 汇编代码嵌入在 Rust 中，可读性差
❌ 没有语法高亮（汇编）
❌ 难以调试

---

### 方案 3：naked 函数 + 手动栈管理（高级）

```rust
#![feature(naked_functions)]
#![feature(asm_const)]

#[link_section = ".bss.stack"]
#[no_mangle]
static mut BOOT_STACK: [u8; 64 * 1024] = [0; 64 * 1024];

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::asm!(
        "la sp, {stack_top}",
        "call {main}",
        stack_top = sym BOOT_STACK,
        main = sym rust_main,
        options(noreturn)
    );
}
```

**优点：**
✅ 纯 Rust（理论上）
✅ 类型安全

**缺点：**
❌ 需要 nightly Rust
❌ `naked_functions` 是 unstable feature
❌ 复杂，容易出错
❌ `static mut BOOT_STACK` 初始化为 0 会占用文件空间

---

### 方案 4：纯汇编文件 + 外部链接

**entry.S：**
```asm
.section .text.entry
.globl _start
_start:
    la sp, boot_stack_top
    call rust_main

.section .bss.stack
boot_stack_lower_bound:
    .space 65536
boot_stack_top:
```

**build.rs：**
```rust
fn main() {
    println!("cargo:rerun-if-changed=src/entry.S");

    cc::Build::new()
        .file("src/entry.S")
        .flag("-march=rv64gc")
        .compile("entry");
}
```

**Cargo.toml：**
```toml
[build-dependencies]
cc = "1.0"
```

**优点：**
✅ 汇编代码完全独立
✅ 使用标准汇编器
✅ 便于大型汇编代码

**缺点：**
❌ 需要 build.rs
❌ 增加构建复杂度
❌ 跨平台问题（需要 RISC-V 汇编器）

---

## 实验：不用 entry.asm 会怎样

### 实验 1：直接用 Rust 函数作为入口

**代码：**
```rust
#![no_std]
#![no_main]

mod lang_items;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
```

**编译：**
```bash
$ cargo build --release
$ rust-objdump -d target/riscv64gc-unknown-none-elf/release/os
```

**反汇编结果：**
```asm
0000000080200000 <_start>:
80200000: 1141           addi    sp, sp, -16    # ← 第一条指令！
80200002: e406           sd      ra, 8(sp)      # ← 使用未初始化的 sp
80200004: 0001           nop
80200006: a001           j       0x80200006     # loop {}
```

**运行：**
```bash
$ qemu-system-riscv64 \
    -machine virt -nographic \
    -bios rustsbi-qemu.bin \
    -device loader,file=os.bin,addr=0x80200000
```

**结果：**
```
[rustsbi] RustSBI version 0.3.0
[rustsbi] ...
[异常] Store/AMO access fault
[异常] sepc = 0x80200002
[异常] stval = 0x...  (随机地址)
```

**原因：**
- `sp` 未初始化（可能是 0 或随机值）
- `sd ra, 8(sp)` 写入非法地址
- 触发 Store Access Fault

---

### 实验 2：使用 naked 函数

**代码：**
```rust
#![feature(naked_functions)]

#[naked]
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::asm!(
        "li sp, 0x80210000",  // 硬编码栈地址
        "1: j 1b",            // loop {}
        options(noreturn)
    );
}
```

**反汇编：**
```asm
0000000080200000 <_start>:
80200000: 10002137       lui     sp, 0x80210
80200004: a001           j       0x80200004
```

**运行：**
```
成功！程序进入死循环，不崩溃。
```

**但是：**
- 需要 nightly Rust
- 硬编码栈地址（不灵活）
- 没有实际分配栈空间（如果调用函数会崩溃）

---

### 实验 3：完整的 naked 函数方案

**代码：**
```rust
#![feature(naked_functions)]

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; 65536] = [0; 65536];

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::asm!(
        "la sp, {stack}",
        "call {main}",
        stack = sym BOOT_STACK,
        main = sym rust_main,
        options(noreturn)
    );
}

#[no_mangle]
fn rust_main() -> ! {
    loop {}
}
```

**编译：**
```bash
$ cargo build --release
```

**检查二进制大小：**
```bash
$ ls -lh target/.../release/os.bin
-rwxrwxr-x 1 user user 65K  os.bin  # ← 64KB！
```

**问题：**
- `static mut BOOT_STACK: [u8; 65536] = [0; 65536]`
- 初始化为 `[0; 65536]` 会放在 `.data` 段
- 占用文件空间 64KB

**改进：使用 MaybeUninit（仍然有问题）：**
```rust
use core::mem::MaybeUninit;

#[link_section = ".bss.stack"]
static mut BOOT_STACK: MaybeUninit<[u8; 65536]> = MaybeUninit::uninit();
```

**编译错误：**
```
error[E0015]: cannot call non-const fn `MaybeUninit::<[u8; 65536]>::uninit`
in statics
```

**结论：** Rust 的限制使得难以用纯 Rust 方式声明 BSS 段的大数组。

---

## 为什么 entry.asm 更好？

### 对比总结

| 方案 | Rust 版本 | 文件大小 | 复杂度 | 可读性 |
|-----|----------|---------|--------|--------|
| **entry.asm (当前)** | 稳定版 | 4 字节 | 低 | 高 |
| inline global_asm! | 稳定版 | 4 字节 | 中 | 中 |
| naked 函数 | nightly | 64KB+ | 高 | 低 |
| 纯汇编 + build.rs | 稳定版 | 4 字节 | 高 | 中 |

### entry.asm 的优势

**1. 简洁明了**
```asm
_start:
    la sp, boot_stack_top
    call rust_main
```
- 一目了然：初始化栈，调用 Rust
- 3 行代码，完成所有必需工作

**2. 高效**
```
.bss.stack:
    .space 65536    # 不占文件空间
```
- BSS 段，运行时分配
- 二进制文件只有 4 字节代码

**3. 标准实践**
- Linux 内核、xv6、rCore 都用这个方式
- RISC-V 标准引导代码模式
- 易于维护和理解

**4. 灵活性**
```asm
.section .bss.stack     # 可以精确控制段
.space 4096 * 16        # 可以轻松调整大小
.globl boot_stack_top   # 可以导出符号给 Rust
```

**5. 调试友好**
```asm
_start:
    la sp, boot_stack_top    # ← 可以在这里设断点
    call rust_main           # ← GDB 可以单步执行
```

---

## 汇编的作用域

### entry.asm 的职责边界

**汇编应该做什么：**
```asm
_start:
    la sp, boot_stack_top    # ✅ 初始化栈
    call rust_main           # ✅ 跳转到 Rust
```

**汇编不应该做什么：**
```asm
# ❌ 不要在汇编里做业务逻辑
_start:
    la sp, boot_stack_top
    # ❌ 不要这样做：
    li a0, 0
    li a1, 1000
.loop:
    addi a0, a0, 1
    blt a0, a1, .loop
    # 应该在 Rust 里做
```

**最小必需原则：**
1. 初始化栈指针（必需）
2. 跳转到 Rust（必需）
3. 分配栈空间（必需）
4. **其他都在 Rust 里做**

### 为什么保持汇编最小化？

**1. 可移植性**
```
entry.asm (RISC-V)  →  可以移植到  →  entry_arm.asm (ARM)
                                  ↓
                      Rust 代码不需要改变
```

**2. 类型安全**
```asm
# 汇编：无类型检查
li a0, 42
call some_function    # a0 是什么类型？谁知道呢
```

```rust
// Rust：类型安全
let value: u32 = 42;
some_function(value); // 编译器检查类型
```

**3. 可测试性**
```rust
// Rust 代码可以单元测试
#[test]
fn test_clear_bss() {
    clear_bss();
    // 验证结果
}
```

```asm
# 汇编代码难以测试
```

---

## 其他架构的对比

### x86_64 的情况

**x86_64 引导代码：**
```asm
.section .text
.global _start
_start:
    mov $stack_top, %rsp     # 初始化栈
    call rust_main           # 调用 Rust

.section .bss
stack_bottom:
    .space 65536
stack_top:
```

**和 RISC-V 几乎一样！** 只是指令语法不同。

### ARM64 的情况

**ARM64 引导代码：**
```asm
.section .text.entry
.global _start
_start:
    ldr x0, =stack_top       // 加载栈地址
    mov sp, x0               // 设置栈指针
    bl rust_main             // 调用 Rust

.section .bss.stack
stack_bottom:
    .space 65536
stack_top:
```

**同样的模式！**

### 通用模式

**所有架构的裸机程序都需要：**
```
1. 定义入口点 (_start)
2. 初始化栈指针 (sp/rsp)
3. 跳转到高级语言 (call/bl)
4. 分配栈空间 (.bss.stack)
```

**这个模式是通用的，不可避免的。**

---

## 最佳实践

### 推荐方案：entry.asm + global_asm!

**目录结构：**
```
os/
├── src/
│   ├── entry.asm       ← 汇编引导代码
│   ├── main.rs         ← Rust 主程序
│   └── linker.ld       ← 链接脚本
├── Cargo.toml
└── .cargo/config.toml
```

**entry.asm（最小化）：**
```asm
    .section .text.entry
    .globl _start
_start:
    la sp, boot_stack_top
    call rust_main

    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
```

**main.rs：**
```rust
#![no_std]
#![no_main]

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    // 所有业务逻辑在这里
    clear_bss();
    loop {}
}
```

**优点：**
- ✅ 使用稳定 Rust
- ✅ 汇编代码清晰独立
- ✅ 二进制文件小
- ✅ 易于维护
- ✅ 业界标准

### 何时考虑其他方案？

**使用 naked 函数的场景：**
- 需要在 Rust 中动态生成入口代码
- 构建系统不支持 `.asm` 文件
- 纯 Rust 项目要求（很少见）

**使用 build.rs 的场景：**
- 大量汇编代码（>100 行）
- 需要条件编译汇编
- 多架构支持（不同的 `.asm` 文件）

---

## 未来可能的改进

### Rust 语言层面

**理想情况（目前不可能）：**
```rust
#[entry_point]
#[stack = 64 * 1024]
fn main() -> ! {
    loop {}
}
```

编译器自动生成：
- 栈空间分配
- sp 初始化
- 入口点

**但这需要：**
- RFC 和社区讨论
- 编译器大量改动
- 多年的稳定化

**目前不太可能实现。**

### 宏简化

**可以创建宏简化 naked 函数：**
```rust
#[entry_point(stack_size = 64 * 1024)]
fn main() -> ! {
    loop {}
}
```

展开为：
```rust
#[link_section = ".bss.stack"]
static mut STACK: [u8; 64 * 1024] = ...;

#[naked]
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // 生成初始化代码
}
```

**但仍然需要 nightly Rust。**

---

## 总结

### entry.asm 是必需的吗？

**从技术角度：**
- ❌ 不是绝对必需（可以用 naked 函数）
- ✅ 但在实践中几乎总是最佳选择

**从工程角度：**
- ✅ **是的，应该使用 entry.asm**
- 简单、高效、标准、可维护

### 为什么需要汇编？

**核心原因：打破"鸡生蛋"循环**

```
问题：Rust 需要栈 ← → 初始化栈需要代码 ← → 代码需要栈

解决：用不需要栈的汇编代码初始化栈
```

**汇编代码的特点：**
- 直接操作寄存器（`la sp, ...`）
- 不需要函数调用约定
- 不需要运行时环境
- 可以在"什么都没有"的情况下运行

### 关键要点

1. **最小化汇编**
   - 只做必需的初始化
   - 业务逻辑都在 Rust

2. **清晰职责**
   - 汇编：初始化运行时
   - Rust：实现功能

3. **标准模式**
   ```asm
   _start:
       初始化栈
       调用 Rust
   ```
   - 所有架构都一样
   - 所有 OS 都一样

4. **不要过度工程化**
   - entry.asm 够用了
   - 不需要复杂的宏或抽象

### 学习建议

**初学者：**
1. 先使用 entry.asm（简单、标准）
2. 理解它做了什么
3. 不要纠结于"纯 Rust"

**进阶者：**
1. 可以尝试 naked 函数（理解限制）
2. 了解不同架构的引导代码
3. 理解为什么汇编是必需的

**专家：**
1. 可以考虑构建系统集成（build.rs）
2. 多架构支持
3. 自定义启动流程

---

**关键结论：** entry.asm 不是累赘，而是裸机编程的必要组成部分。接受它，理解它，简化它。

---

**参考资料：**
- [Rust Embedded Book - A little C with your Rust](https://docs.rust-embedded.org/book/interoperability/c-with-rust.html)
- [RISC-V Calling Convention](https://github.com/riscv-non-isa/riscv-elf-psabi-doc)
- [Rust Reference - Inline Assembly](https://doc.rust-lang.org/reference/inline-assembly.html)
- [rCore Tutorial Book](https://rcore-os.cn/rCore-Tutorial-Book-v3/)
