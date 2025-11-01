# 错误修复记录：panic_handler 优化

**日期：** 2025-10-31
**修改原因：** 优化 lang_items.rs 的错误处理，但遇到编译错误
**修改文件：** main.rs、lang_items.rs

---

## 错误概述

在优化 panic_handler 以显示详细错误信息时，遇到了 3 个编译错误：

```
error[E0554]: `#![feature]` may not be used on the stable release channel
error[E0599]: no method named `unwrap` found for struct `PanicMessage` (2次)
```

---

## 错误 1：feature gate 在 stable 不可用

### 错误代码

**文件：** `src/main.rs`

```rust
#![no_std]
#![no_main]
#![feature(panic_info_message)]  // ← 错误行
```

### 编译错误

```
error[E0554]: `#![feature]` may not be used on the stable release channel
  --> src/main.rs:50:1
   |
50 | #![feature(panic_info_message)]
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: the feature `panic_info_message` has been stable since 1.81.0
         and no longer requires an attribute to enable
```

### 错误原因

1. **`panic_info_message` 特性已稳定**
   - 在 Rust 1.81.0 版本中，该特性已经稳定
   - 稳定的特性不再需要 `#![feature]` 声明

2. **stable vs nightly Rust**
   ```
   Rust nightly → 可以使用 #![feature(...)]
   Rust stable  → 不能使用 #![feature(...)]，除非该特性未稳定
   ```

3. **当前环境使用的是 stable Rust**
   ```bash
   $ rustc --version
   # 如果没有 -nightly 后缀，就是 stable 版本
   ```

### 修复方案

**删除 feature gate 声明：**

```rust
#![no_std]
#![no_main]
// #![feature(panic_info_message)]  ← 删除这行
```

### 知识点：Rust 特性生命周期

```
unstable (nightly only)
  ↓
  需要 #![feature(xxx)]
  ↓
RFC 提案 → 实现 → 测试
  ↓
stabilized (某个版本稳定)
  ↓
  不再需要 #![feature(xxx)]
  ↓
成为 stable Rust 的一部分
```

**`panic_info_message` 的历史：**
```
Rust < 1.81.0  → unstable，需要 #![feature(panic_info_message)]
Rust ≥ 1.81.0  → stable，不需要 feature gate
```

---

## 错误 2 & 3：PanicMessage 没有 unwrap() 方法

### 错误代码

**文件：** `src/lang_items.rs`

```rust
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()  // ← 错误：unwrap() 不存在
        );
    } else {
        println!("Panicked: {}", info.message().unwrap());  // ← 错误：同上
    }
    shutdown(true)
}
```

### 编译错误

```
error[E0599]: no method named `unwrap` found for struct `PanicMessage`
              in the current scope
  --> src/lang_items.rs:20:28
   |
20 |             info.message().unwrap()
   |                            ^^^^^^ method not found in `PanicMessage<'_>`

error[E0599]: no method named `unwrap` found for struct `PanicMessage`
              in the current scope
  --> src/lang_items.rs:23:49
   |
23 |         println!("Panicked: {}", info.message().unwrap());
   |                                                 ^^^^^^ method not found
```

### 错误原因

#### 1. 类型误解

**错误的假设：**
```rust
info.message()  // 假设返回 Option<&PanicMessage>
    .unwrap()   // 所以需要 unwrap 提取
```

**实际类型：**
```rust
impl PanicInfo {
    pub fn message(&self) -> &PanicMessage  // 直接返回引用，不是 Option！
}
```

**类型链分析：**
```
info: &PanicInfo
  ↓ .message()
&PanicMessage  ← 这就是最终类型，不是 Option！
  ↓ .unwrap() ???
❌ 错误：PanicMessage 没有 unwrap() 方法
```

#### 2. API 演变历史

**旧 API（Rust < 1.81.0，需要 feature）：**
```rust
impl PanicInfo {
    pub fn message(&self) -> Option<&str>  // 返回 Option
}

// 使用方式
info.message()  // Option<&str>
    .unwrap()   // &str
```

**新 API（Rust ≥ 1.81.0，已稳定）：**
```rust
pub struct PanicMessage<'a> { /* ... */ }

impl Display for PanicMessage<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // 实现格式化
    }
}

impl PanicInfo {
    pub fn message(&self) -> &PanicMessage  // 直接返回引用
}

// 使用方式
info.message()  // &PanicMessage，实现了 Display
                // 可以直接用于 println!
```

#### 3. Display Trait 的作用

```rust
// PanicMessage 实现了 Display
impl Display for PanicMessage<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // 将 panic 消息格式化到 Formatter 中
    }
}

// 因此可以直接用于格式化输出
println!("{}", info.message());
//             ^^^^^^^^^^^^^^
//       自动调用 PanicMessage::fmt
```

### 修复方案

**删除 `.unwrap()` 调用：**

```rust
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()  // ✅ 直接使用，不需要 unwrap
        );
    } else {
        println!("Panicked: {}", info.message());  // ✅ 同上
    }
    shutdown(true)
}
```

### 为什么可以直接使用？

```rust
// println! 的 {} 格式化符号要求参数实现 Display trait
println!("{}", value);
//             ^^^^^
//       要求: value: impl Display

// PanicMessage 实现了 Display
impl Display for PanicMessage<'_> { ... }

// 因此可以直接使用
println!("{}", info.message());  // ✅ PanicMessage 实现了 Display
```

---

## 完整修复对比

### 修复前（错误版本）

**main.rs：**
```rust
#![no_std]
#![no_main]
#![feature(panic_info_message)]  // ❌ 错误1：stable 不允许

#[macro_use]
mod console;
mod lang_items;
mod sbi;
```

**lang_items.rs：**
```rust
use crate::sbi::shutdown;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()  // ❌ 错误2：unwrap 方法不存在
        );
    } else {
        println!("Panicked: {}", info.message().unwrap());  // ❌ 错误3：同上
    }
    shutdown(true)
}
```

**编译结果：**
```
❌ 3 个错误，无法编译
```

---

### 修复后（正确版本）

**main.rs：**
```rust
#![no_std]
#![no_main]
// ✅ 删除了 #![feature(panic_info_message)]

#[macro_use]
mod console;
mod lang_items;
mod sbi;
```

**lang_items.rs：**
```rust
use crate::sbi::shutdown;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()  // ✅ 直接使用，PanicMessage 实现了 Display
        );
    } else {
        println!("Panicked: {}", info.message());  // ✅ 同上
    }
    shutdown(true)
}
```

**编译结果：**
```
✅ 编译成功，0 个错误
```

---

## 运行测试

### 测试代码

**main.rs 中的测试：**
```rust
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    println!("this is a test");
    panic!("Shutdown machine!");  // ← 触发 panic
}
```

### 预期输出

```
[RustSBI 启动信息...]
this is a test
Panicked at src/main.rs:66 Shutdown machine!
```

### 实际运行

```bash
$ cargo build --release
   Compiling os v0.1.0 (/home/c20h30o2/rs_project/rCore/os)
    Finished `release` profile [optimized] target(s) in 0.10s

$ rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os \
    -O binary target/riscv64gc-unknown-none-elf/release/os.bin

$ qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000
```

### 实际输出

```
[rustsbi] RustSBI version 0.4.0-alpha.1, adapting to RISC-V SBI v2.0.0
[... RustSBI 启动信息 ...]
this is a test
Panicked at src/main.rs:65 Shutdown machine!
```

**结果：** ✅ 完全符合预期

---

## 执行流程分析

```
1. QEMU 启动
   ↓
2. RustSBI 初始化
   - 设置 PMP（物理内存保护）
   - 配置 SBI 服务
   ↓
3. RustSBI 跳转到 0x80200000
   ↓
4. entry.asm 执行
   - la sp, boot_stack_top  (初始化栈)
   - call rust_main         (调用 Rust)
   ↓
5. rust_main() 执行
   - clear_bss()                         → 清零 BSS 段
   - println!("this is a test")          → 输出到串口
   - panic!("Shutdown machine!")         → 触发 panic
   ↓
6. panic_handler 执行
   - 提取 location (file, line)
   - 提取 message
   - println!("Panicked at ...")         → 输出错误信息
   - shutdown(true)                      → SBI 关机请求
   ↓
7. SBI system_reset
   - system_reset(Shutdown, SystemFailure)
   - ecall 调用 RustSBI
   ↓
8. RustSBI 处理关机
   - 执行关机逻辑
   ↓
9. QEMU 退出
```

---

## 知识点总结

### 1. Rust 特性生命周期

| 阶段 | 状态 | 需要 feature gate？ | 可用版本 |
|------|------|-------------------|---------|
| **unstable** | 实验性 | ✅ 需要 `#![feature(...)]` | nightly only |
| **stabilized** | 已稳定 | ❌ 不需要 | stable + nightly |

**示例：**
```rust
// unstable 特性（需要 nightly）
#![feature(some_unstable_feature)]

// stable 特性（不需要 feature gate）
// panic_info_message 在 Rust 1.81.0+ 已稳定
```

### 2. PanicInfo API 演变

| Rust 版本 | API | 返回类型 | 使用方式 |
|-----------|-----|---------|---------|
| < 1.81.0 | `message()` | `Option<&str>` | `.unwrap()` |
| ≥ 1.81.0 | `message()` | `&PanicMessage` | 直接使用 |

**API 变化原因：**
- 旧 API：返回 `Option<&str>`，信息可能丢失
- 新 API：返回 `&PanicMessage`，保留完整信息且支持格式化

### 3. Display Trait 的便利性

```rust
pub trait Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result;
}

// 实现 Display 后，可以用于：
println!("{}", value);       // 格式化输出
format!("{}", value);        // 格式化字符串（需 alloc）
write!(f, "{}", value);      // 写入 Formatter
```

**PanicMessage 的 Display 实现：**
```rust
impl Display for PanicMessage<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // 将 panic 消息格式化输出
    }
}
```

### 4. no_std 环境的错误处理

**标准库环境：**
```rust
panic!("error");
  ↓
std::panic::panic_any
  ↓
默认 panic_handler (打印到 stderr)
  ↓
程序 abort
```

**no_std 环境（我们的情况）：**
```rust
panic!("error");
  ↓
core::panic!
  ↓
#[panic_handler] 自定义处理  ← 我们必须提供！
  ↓
自定义行为（打印 + 关机）
  ↓
shutdown(true)
  ↓
SBI system_reset
```

---

## 文件修改记录

### main.rs

**修改位置：** 第 50 行附近

**修改前：**
```rust
48 #![no_std]
49 #![no_main]
50 #![feature(panic_info_message)]  // ← 删除
51
52 #[macro_use]
```

**修改后：**
```rust
48 #![no_std]
49 #![no_main]
50
51 // 错误版本（已修复）：
52 // #![feature(panic_info_message)]
53 //
54 // 错误原因：
55 // 1. error[E0554]: `#![feature]` may not be used on the stable release channel
56 // 2. panic_info_message 特性在 Rust 1.81.0 已经稳定，不再需要 feature gate
57 // 3. 在 stable Rust 上使用 #![feature] 会导致编译错误
58 //
59 // 修改时间：2025-10-31
60 // 修改内容：删除 #![feature(panic_info_message)] 这行
61
62 #[macro_use]
```

---

### lang_items.rs

**修改位置：** panic_handler 函数

**修改前：**
```rust
13 #[panic_handler]
14 fn panic(info: &PanicInfo) -> ! {
15     if let Some(location) = info.location() {
16         println!(
17             "Panicked at {}:{} {}",
18             location.file(),
19             location.line(),
20             info.message().unwrap()  // ← 删除 .unwrap()
21         );
22     } else {
23         println!("Panicked: {}", info.message().unwrap());  // ← 删除 .unwrap()
24     }
25     shutdown(true)
26 }
```

**修改后：**
```rust
13 // 错误版本（已修复）：
14 // #[panic_handler]
15 // fn panic(info: &PanicInfo) -> ! {
16 //     if let Some(location) = info.location() {
17 //         println!(
18 //             "Panicked at {}:{} {}",
19 //             location.file(),
20 //             location.line(),
21 //             info.message().unwrap()  // ← 错误：PanicMessage 没有 unwrap() 方法
22 //         );
23 //     } else {
24 //         println!("Panicked: {}", info.message().unwrap());  // ← 错误：同上
25 //     }
26 //     shutdown(true)
27 // }
28 //
29 // 错误原因：
30 // 1. error[E0599]: no method named `unwrap` found for struct `PanicMessage`
31 // 2. 在 Rust 1.81.0+ 中，info.message() 直接返回 &PanicMessage，而不是 Option
32 // 3. PanicMessage 实现了 Display trait，可以直接用于 println! 的 {} 格式化
33 // 4. 不需要调用 .unwrap()，直接使用即可
34 //
35 // 类型分析：
36 //   info: &PanicInfo
37 //     ↓ .message()
38 //   &PanicMessage  ← 直接返回引用，不是 Option！
39 //     ↓ 实现了 Display trait
40 //   println!("{}", info.message())  ← 直接使用
41 //
42 // 修改时间：2025-10-31
43 // 修改内容：删除 .unwrap() 调用，直接使用 info.message()
44
45 #[panic_handler]
46 fn panic(info: &PanicInfo) -> ! {
47     if let Some(location) = info.location() {
48         println!(
49             "Panicked at {}:{} {}",
50             location.file(),
51             location.line(),
52             info.message()  // ✅ 正确：直接使用
53         );
54     } else {
55         println!("Panicked: {}", info.message());  // ✅ 正确：同上
56     }
57     shutdown(true)
58 }
```

---

## 经验教训

### 1. 注意 Rust 版本差异

- 特性可能在不同版本有不同行为
- 查看文档时注意版本标注
- 使用稳定的 API，避免依赖 nightly 特性

### 2. 理解类型而不是猜测

**错误做法：**
```rust
info.message().unwrap()  // ← 猜测返回 Option
```

**正确做法：**
```rust
// 查看文档或编译错误
let msg = info.message();  // 编译器会告诉你类型
```

### 3. 利用编译器错误学习

编译器错误信息非常详细：
```
error[E0599]: no method named `unwrap` found for struct `PanicMessage`
                                                         ^^^^^^^^^^^
              提示了实际类型
```

### 4. 保留错误代码作为学习记录

**好处：**
- 记录犯过的错误，避免重复
- 理解 API 演变历史
- 帮助他人学习同样的问题

---

## 参考资料

1. **Rust 1.81.0 发布说明**
   - https://blog.rust-lang.org/2024/09/05/Rust-1.81.0.html
   - 稳定了 `panic_info_message` 特性

2. **PanicInfo 文档**
   - https://doc.rust-lang.org/core/panic/struct.PanicInfo.html
   - 查看 `message()` 方法签名

3. **Display Trait 文档**
   - https://doc.rust-lang.org/core/fmt/trait.Display.html
   - 理解格式化机制

4. **Rust Unstable Book**
   - https://doc.rust-lang.org/unstable-book/
   - 查看 unstable 特性列表

---

**总结：** 修复了 3 个编译错误，优化了 panic 错误处理，现在可以显示详细的错误位置（文件名 + 行号）和错误信息，并在 panic 时自动关机。
