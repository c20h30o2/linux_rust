# 四个文件如何协作构建 ELF 文件

**日期：** 2025-10-31
**分析文件：** config.toml, main.rs, entry.asm, linker.ld

---

## 目录

1. [构建流程总览](#构建流程总览)
2. [文件 1: config.toml - 构建配置](#文件1-configtoml---构建配置)
3. [文件 2: entry.asm - 汇编入口](#文件2-entryasm---汇编入口)
4. [文件 3: main.rs - Rust 主程序](#文件3-mainrs---rust-主程序)
5. [文件 4: linker.ld - 链接脚本](#文件4-linkerld---链接脚本)
6. [完整构建过程](#完整构建过程)
7. [最终 ELF 文件结构](#最终-elf-文件结构)
8. [符号依赖关系](#符号依赖关系)
9. [调试与验证](#调试与验证)

---

## 构建流程总览

```
┌─────────────────────────────────────────────────────┐
│ 1. Cargo 读取配置                                     │
│    - Cargo.toml (项目元数据)                         │
│    - .cargo/config.toml (构建配置)                   │
└────────────────┬────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────────────────┐
│ 2. Rustc 编译源代码                                   │
│    - main.rs → 包含 entry.asm                       │
│    - lang_items.rs → panic_handler                  │
│    生成：os.o (目标文件)                              │
└────────────────┬────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────────────────┐
│ 3. 链接器 (ld.lld) 使用 linker.ld                     │
│    - 设置内存布局                                     │
│    - 确定入口点: _start                               │
│    - 定义符号: sbss, ebss, stext, ...                │
│    - 合并所有段                                       │
└────────────────┬────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────────────────┐
│ 4. 生成最终 ELF 文件                                  │
│    target/riscv64gc-unknown-none-elf/release/os    │
│    - Entry: 0x80200000                              │
│    - Sections: .text, .rodata, .data, .bss          │
└─────────────────────────────────────────────────────┘
```

---

## 文件1: config.toml - 构建配置

### 位置
`.cargo/config.toml`

### 完整内容

```toml
[build]
target = "riscv64gc-unknown-none-elf"

[target.riscv64gc-unknown-none-elf]
rustflags = [
    "-Clink-arg=-Tsrc/linker.ld",
    "-Cforce-frame-pointers=yes"
]
```

### 逐行分析

#### 1. `target = "riscv64gc-unknown-none-elf"`

**作用：** 设置默认构建目标架构

**解释：**
- `riscv64` - 64 位 RISC-V 架构
- `gc` - 扩展集：G (IMAFD) + C (压缩指令)
  - I: 基础整数指令
  - M: 乘除法
  - A: 原子操作
  - F: 单精度浮点
  - D: 双精度浮点
  - C: 压缩指令（16位）
- `unknown` - 未知的操作系统供应商
- `none` - 无操作系统（裸机环境）
- `elf` - 使用 ELF 文件格式

#### 2. `rustflags = [...]`

**作用：** 传递编译器标志

**标志详解：**

```toml
"-Clink-arg=-Tsrc/linker.ld"
```
- `-C` - 传递 codegen 选项
- `link-arg` - 传递参数给链接器
- `-T` - 链接器选项：使用自定义链接脚本
- `src/linker.ld` - 链接脚本路径

```toml
"-Cforce-frame-pointers=yes"
```
- 强制使用帧指针
- 便于调试和栈回溯
- 性能略有下降，但调试友好

### 效果

运行 `cargo build` 时，实际执行的命令：

```bash
rustc \
    --crate-name os \
    --edition=2024 \
    src/main.rs \
    --crate-type bin \
    --target riscv64gc-unknown-none-elf \
    -Clink-arg=-Tsrc/linker.ld \
    -Cforce-frame-pointers=yes \
    -o target/riscv64gc-unknown-none-elf/release/os
```

---

## 文件2: entry.asm - 汇编入口

### 位置
`src/entry.asm`

### 完整内容

```asm
# os/src/entry.asm
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

### 逐行分析

#### 代码段部分

```asm
.section .text.entry
```
- 定义一个特殊的段：`.text.entry`
- 与普通 `.text` 段分开
- linker.ld 会将它放在代码段最前面

```asm
.globl _start
```
- 声明全局符号 `_start`
- 链接器和其他模块可见
- linker.ld 将它设置为入口点

```asm
_start:
```
- 程序的**真正入口点**
- CPU 开始执行的第一条指令
- 地址：0x80200000（由 linker.ld 确定）

```asm
la sp, boot_stack_top
```
- `la` - Load Address（伪指令）
- 将栈顶地址加载到 `sp` 寄存器
- `boot_stack_top` - 在下面的 .bss.stack 段定义
- **关键作用：** 初始化栈指针，Rust 代码需要栈

```asm
call rust_main
```
- 调用 Rust 函数 `rust_main`
- `rust_main` 在 main.rs 中定义
- 使用 `#[no_mangle]` 保证函数名不变

#### BSS 栈段部分

```asm
.section .bss.stack
```
- 定义 BSS 段的子段
- BSS (Block Started by Symbol) - 未初始化数据段
- 运行时会被清零

```asm
.globl boot_stack_lower_bound
boot_stack_lower_bound:
```
- 栈的低地址边界
- 用于栈溢出检测

```asm
.space 4096 * 16
```
- 分配 **64KB** (65536 字节) 的栈空间
- 4096 = 4KB（一个页）
- 16 * 4KB = 64KB

```asm
.globl boot_stack_top
boot_stack_top:
```
- 栈的高地址边界（栈顶）
- `sp` 初始指向这里
- RISC-V 栈向下增长：sp 减小

### 栈布局

```
低地址
    ↓
┌─────────────────┐ ← boot_stack_lower_bound
│                 │
│   64KB 栈空间   │
│                 │
│  (向下增长 ↓)   │
│                 │
└─────────────────┘ ← boot_stack_top (sp 初始值)
    ↑
高地址
```

---

## 文件3: main.rs - Rust 主程序

### 位置
`src/main.rs`

### 完整版本（应该包含的内容）

```rust
// os/src/main.rs
#![no_std]                    // 不使用标准库
#![no_main]                   // 不使用 Rust 默认 main 入口

mod lang_items;               // panic_handler 等语言项

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));  // 包含汇编代码

#[no_mangle]                  // 不修改函数名
pub fn rust_main() -> ! {     // 永不返回
    clear_bss();              // 清零 BSS 段
    loop {}                   // 主循环
}

fn clear_bss() {
    extern "C" {
        fn sbss();            // 链接脚本定义
        fn ebss();            // 链接脚本定义
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}
```

### 逐行分析

#### 属性声明

```rust
#![no_std]
```
- 不链接标准库 `std`
- 只能使用核心库 `core`
- 必须自己实现：
  - 内存分配
  - I/O 操作
  - 线程管理
  - ...

```rust
#![no_main]
```
- 不使用 Rust 默认的 `main` 入口
- 不需要 `fn main() {}`
- 入口点由 entry.asm 的 `_start` 提供

#### 模块和汇编

```rust
mod lang_items;
```
- 引入 `lang_items.rs`
- 提供 `#[panic_handler]` 等必需的语言项

```rust
use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
```
- `global_asm!` - 内联汇编宏
- `include_str!("entry.asm")` - 编译时读取文件内容
- 将 entry.asm 的内容嵌入到编译结果中
- **关键作用：** 连接汇编入口和 Rust 代码

#### Rust 主函数

```rust
#[no_mangle]
```
- 禁止名称修饰 (name mangling)
- 确保函数名就是 `rust_main`
- entry.asm 的 `call rust_main` 才能找到它

```rust
pub fn rust_main() -> ! {
```
- `-> !` - 发散函数（永不返回）
- 必须以 `loop {}`、`panic!()` 或 `exit()` 结束
- 不能 return

```rust
clear_bss();
```
- **关键操作：** 清零 BSS 段
- BSS 段包含未初始化的全局变量
- C/Rust 语义要求全局变量初始化为 0

```rust
loop {}
```
- 主循环
- CPU 在这里空转
- 真实 OS 会在这里调度任务

#### clear_bss 函数

```rust
extern "C" {
    fn sbss();
    fn ebss();
}
```
- 声明外部符号（由链接脚本提供）
- `sbss` - BSS 段起始地址
- `ebss` - BSS 段结束地址
- 没有实现，只是占位符

```rust
(sbss as usize..ebss as usize).for_each(|a| {
    unsafe { (a as *mut u8).write_volatile(0) }
});
```
- 遍历 BSS 段的每个字节
- `write_volatile(0)` - 写入 0
- `volatile` - 防止编译器优化掉

### 为什么需要 clear_bss？

**C/Rust 语义要求：**
```rust
static mut GLOBAL_VAR: u32;  // 期望初始值为 0
```

**但 ELF 文件中：**
- BSS 段不占用文件空间（节省空间）
- 加载到内存后，内容是随机的
- **必须手动清零**

---

## 文件4: linker.ld - 链接脚本

### 位置
`src/linker.ld`

### 完整内容

```ld
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}
```

### 逐行分析

#### 头部配置

```ld
OUTPUT_ARCH(riscv)
```
- 指定输出架构：RISC-V
- 确保生成正确的 ELF 头

```ld
ENTRY(_start)
```
- **设置入口点**：`_start`
- ELF Header 的 Entry 字段会被设置为 `_start` 的地址
- CPU 加载程序后从这里开始执行

```ld
BASE_ADDRESS = 0x80200000;
```
- **定义基地址常量**
- `0x80200000` - QEMU virt 机器的内核加载地址
- OpenSBI 会将内核加载到这个地址

**为什么是 0x80200000？**

| 地址范围 | 用途 |
|---------|------|
| 0x00000000 - 0x80000000 | 设备 MMIO 区域 |
| 0x80000000 - 0x80200000 | OpenSBI 固件 |
| **0x80200000 - ...**     | **内核代码（我们的程序）** |

#### SECTIONS 定义

```ld
. = BASE_ADDRESS;
```
- `.` - 位置计数器（当前地址）
- 设置为 0x80200000
- 后续所有内容从这里开始布局

```ld
skernel = .;
```
- **定义符号** `skernel`
- 值 = 当前地址 (0x80200000)
- Rust/C 代码可以引用这个符号

#### .text 段（代码段）

```ld
stext = .;
```
- 定义符号：代码段起始

```ld
.text : {
    *(.text.entry)        # ← 首先放入 .text.entry
    *(.text .text.*)      # ← 然后是所有其他代码
}
```
- `*(.text.entry)` - **所有文件的 .text.entry 段**
  - entry.asm 的 `_start` 在这里
  - **确保 _start 在最前面**
- `*(.text .text.*)` - 所有其他代码
  - main.rs 编译的 `rust_main`
  - lang_items.rs 编译的 `panic_handler`

**段顺序的重要性：**
```
0x80200000: _start         ← 入口点必须在最前面
0x80200010: rust_main
0x80200100: clear_bss
...
```

```ld
. = ALIGN(4K);
etext = .;
```
- `ALIGN(4K)` - 对齐到 4KB 边界
- 原因：页表管理需要页对齐
- `etext` - 代码段结束标记

#### .rodata 段（只读数据）

```ld
srodata = .;
.rodata : {
    *(.rodata .rodata.*)
    *(.srodata .srodata.*)
}
```
- 存放只读数据：
  - 字符串字面量
  - `const` 常量
  - 虚函数表
- `srodata`/`.srodata` - RISC-V 小数据优化

#### .data 段（已初始化数据）

```ld
sdata = .;
.data : {
    *(.data .data.*)
    *(.sdata .sdata.*)
}
```
- 存放已初始化的全局变量
- 例如：`static mut VAR: u32 = 42;`

#### .bss 段（未初始化数据）

```ld
.bss : {
    *(.bss.stack)         # ← 栈（entry.asm）
    sbss = .;             # ← BSS 起始（Rust 使用）
    *(.bss .bss.*)
    *(.sbss .sbss.*)
}
```

**关键点：**

1. `*(.bss.stack)` - entry.asm 的 64KB 栈
   - 放在 BSS 段最前面
   - 不计入 sbss-ebss 范围

2. `sbss = .;` - **Rust 代码使用的符号**
   - main.rs 的 `clear_bss()` 引用它
   - 表示"需要清零的 BSS 起始地址"

3. `*(.bss .bss.*)` - Rust 编译的全局变量

**为什么栈不在 sbss-ebss 之间？**
```
.bss.stack (64KB)   ← 栈，不需要清零
sbss ───┐
        │           ← 这部分需要清零
        │  .bss.*
ebss ───┘
```
- 栈空间不需要清零（会被覆盖）
- 全局变量需要清零（语义要求）

```ld
ebss = .;
ekernel = .;
```
- `ebss` - BSS 结束（Rust 使用）
- `ekernel` - 整个内核结束

#### 丢弃段

```ld
/DISCARD/ : {
    *(.eh_frame)
}
```
- 丢弃异常帧信息
- 裸机环境不需要 C++ 异常处理
- 减小二进制大小

---

## 完整构建过程

### 步骤 1: Cargo 解析配置

```bash
$ cargo build --release
```

**Cargo 读取：**
1. `Cargo.toml` - 项目名称、版本、依赖
2. `.cargo/config.toml` - 目标架构、编译选项

**确定构建参数：**
- Target: `riscv64gc-unknown-none-elf`
- Rustflags: `-Clink-arg=-Tsrc/linker.ld -Cforce-frame-pointers=yes`

### 步骤 2: Rustc 编译

**编译 main.rs：**

```bash
rustc \
    --crate-name os \
    --edition=2024 \
    src/main.rs \
    --crate-type bin \
    --target riscv64gc-unknown-none-elf \
    -C opt-level=3 \
    -C embed-bitcode=no \
    -C metadata=... \
    --out-dir target/riscv64gc-unknown-none-elf/release/deps \
    -Clink-arg=-Tsrc/linker.ld \
    -Cforce-frame-pointers=yes
```

**处理过程：**

1. **解析 Rust 源码**
   - `#![no_std]` - 不链接 std
   - `#![no_main]` - 不生成默认 main
   - `mod lang_items` - 编译 lang_items.rs

2. **处理内联汇编**
   ```rust
   global_asm!(include_str!("entry.asm"));
   ```
   - 读取 entry.asm 文件
   - 将汇编代码嵌入到目标文件

3. **编译 Rust 代码**
   - `rust_main()` → RISC-V 机器码
   - `clear_bss()` → RISC-V 机器码
   - `#[no_mangle]` 保证函数名不变

4. **生成目标文件**
   ```
   target/.../deps/os-<hash>.o (ELF relocatable)
   ```
   - 包含 .text.entry 段（entry.asm）
   - 包含 .text 段（Rust 代码）
   - 包含 .bss.stack 段（栈空间）
   - 包含未解析的符号引用（sbss, ebss）

### 步骤 3: 链接

**链接器命令（简化）：**

```bash
ld.lld \
    -Tsrc/linker.ld \
    target/.../deps/os-<hash>.o \
    -o target/riscv64gc-unknown-none-elf/release/os
```

**链接器处理流程：**

1. **读取 linker.ld**
   - 设置基地址：0x80200000
   - 设置入口点：_start
   - 读取段布局规则

2. **解析目标文件**
   - 读取 os-<hash>.o
   - 提取所有段：.text.entry, .text, .bss.stack, .bss

3. **布局段**
   ```
   0x80200000: .text
      - 首先放 *(.text.entry)  ← _start
      - 然后放 *(.text)         ← rust_main

   0x80201000: .rodata (4KB 对齐)

   0x80202000: .data (4KB 对齐)

   0x80203000: .bss (4KB 对齐)
      - 首先放 *(.bss.stack)    ← 64KB 栈
      - 设置 sbss = .
      - 然后放 *(.bss)
      - 设置 ebss = .
   ```

4. **解析符号**
   - `_start` → 0x80200000
   - `rust_main` → 0x802000XX
   - `boot_stack_top` → 0x80213000 (例如)
   - `sbss` → 0x80213000
   - `ebss` → 0x80213YYY

5. **重定位**
   - entry.asm 中的 `la sp, boot_stack_top`
     - 替换为实际地址
   - entry.asm 中的 `call rust_main`
     - 替换为实际地址
   - main.rs 中的 `sbss as usize`
     - 替换为实际地址

6. **生成 ELF 文件**
   ```
   target/riscv64gc-unknown-none-elf/release/os
   ```
   - ELF Header: Entry = 0x80200000
   - Program Headers: 加载信息
   - Section Headers: 段信息
   - 实际代码和数据

### 步骤 4: 验证

```bash
# 查看文件类型
$ file target/riscv64gc-unknown-none-elf/release/os
ELF 64-bit LSB executable, UCB RISC-V, version 1 (SYSV),
statically linked, not stripped

# 查看 ELF 头
$ rust-readobj -h target/riscv64gc-unknown-none-elf/release/os
Entry: 0x80200000  ← 入口点正确

# 反汇编
$ rust-objdump -d target/riscv64gc-unknown-none-elf/release/os
0000000080200000 <_start>:  ← _start 在最前面
80200000: auipc sp, ...     ← la sp, boot_stack_top
80200004: addi sp, sp, ...
80200008: auipc ra, ...     ← call rust_main
8020000c: jalr ra, ...
```

---

## 最终 ELF 文件结构

### 内存布局

```
╔══════════════════════════════════════════════════════════════╗
║              最终 ELF 文件的内存布局                          ║
║              (加载到 0x80200000 后)                           ║
╚══════════════════════════════════════════════════════════════╝

地址          段          内容                      来源
─────────────────────────────────────────────────────────────

0x80200000   ┌──────┐
  skernel → │        │
  stext →   │ .text  │   _start:              entry.asm
            │        │     la sp, ...
            │        │     call rust_main
            │        │
            │        │   rust_main:           main.rs
            │        │     call clear_bss
            │        │     loop
            │        │
            │        │   clear_bss:           main.rs
            │        │     ...
  etext →   └──────┘
            ↓ ALIGN(4K)

0x80201000   ┌──────┐
  srodata → │        │   字符串字面量          Rust 编译器
            │.rodata │   常量数据
  erodata → └──────┘
            ↓ ALIGN(4K)

0x80202000   ┌──────┐
  sdata →   │ .data  │   已初始化全局变量      Rust 编译器
  edata →   └──────┘
            ↓ ALIGN(4K)

0x80203000   ┌──────┐
            │        │   boot_stack_lower_  entry.asm
            │        │   bound:
            │ .bss   │   [64KB 栈空间]
            │        │   boot_stack_top: ←─ sp 初始指向
            │        │
  sbss →    │        │   未初始化全局变量      Rust 编译器
            │        │   (clear_bss 清零)
  ebss →    └──────┘
            ↓ ALIGN(4K)

0x8021XXXX
  ekernel →
```

### ELF 文件头

```
ELF Header:
  Magic:   7f 45 4c 46 02 01 01 00 00 00 00 00 00 00 00 00
  Class:                             ELF64
  Data:                              2's complement, little endian
  Version:                           1 (current)
  OS/ABI:                            UNIX - System V
  ABI Version:                       0
  Type:                              EXEC (Executable file)
  Machine:                           RISC-V
  Version:                           0x1
  Entry point address:               0x80200000  ← _start 地址
  Start of program headers:          64 (bytes into file)
  Start of section headers:          XXXX (bytes into file)
  Flags:                             0x5, RVC, double-float ABI
  Size of this header:               64 (bytes)
  Size of program headers:           56 (bytes)
  Number of program headers:         3
  Size of section headers:           64 (bytes)
  Number of section headers:         8
  Section header string table index: 6
```

### 程序头（Program Headers）

```
Program Headers:
  Type           Offset   VirtAddr           PhysAddr           FileSiz  MemSiz   Flg Align
  LOAD           0x001000 0x0000000080200000 0x0000000080200000 0x000004 0x000004 R E 0x1000
  LOAD           0x002000 0x0000000080201000 0x0000000080201000 0x000000 0x000000 R   0x1000
  LOAD           0x003000 0x0000000080203000 0x0000000080203000 0x000000 0x010000 RW  0x1000
  GNU_STACK      0x000000 0x0000000000000000 0x0000000000000000 0x000000 0x000000 RW  0x0
```

**解释：**
- LOAD 段 1: .text (可执行)
- LOAD 段 2: .rodata (只读)
- LOAD 段 3: .bss (读写，64KB)

### 段头（Section Headers）

```
Section Headers:
  [Nr] Name              Type            Address          Off    Size   ES Flg Lk Inf Al
  [ 0]                   NULL            0000000000000000 000000 000000 00      0   0  0
  [ 1] .text             PROGBITS        0000000080200000 001000 000004 00  AX  0   0  1
  [ 2] .bss              NOBITS          0000000080203000 003000 010000 00  WA  0   0  1
  [ 3] .comment          PROGBITS        0000000000000000 003000 000013 01  MS  0   0  1
  [ 4] .symtab           SYMTAB          0000000000000000 003018 000120 18      5   3  8
  [ 5] .strtab           STRTAB          0000000000000000 003138 000050 00      0   0  1
  [ 6] .shstrtab         STRTAB          0000000000000000 003188 000030 00      0   0  1
```

---

## 符号依赖关系

### 符号定义和使用关系图

```
┌──────────────────────────────────────────────────────────┐
│                     符号流动图                            │
└──────────────────────────────────────────────────────────┘

linker.ld 定义:
├─ _start         → entry.asm 引用 (ENTRY)
├─ boot_stack_top → entry.asm 定义 (实际)
├─ sbss           → main.rs 引用
├─ ebss           → main.rs 引用
├─ stext          → (可被 Rust 引用)
├─ etext          → (可被 Rust 引用)
├─ srodata        → (可被 Rust 引用)
└─ ekernel        → (可被 Rust 引用)

entry.asm 定义:
├─ _start              → linker.ld ENTRY 使用
├─ boot_stack_top      → entry.asm 使用 (la sp)
└─ boot_stack_lower_  → (可选，用于检查)
    bound

entry.asm 引用:
├─ boot_stack_top  ← entry.asm 定义
└─ rust_main       ← main.rs 定义

main.rs 定义:
├─ rust_main       → entry.asm 调用
└─ clear_bss       → rust_main 调用

main.rs 引用:
├─ sbss            ← linker.ld 定义
└─ ebss            ← linker.ld 定义
```

### 符号解析过程

#### 阶段 1: 编译时

**entry.asm → 目标文件：**
```
符号定义:
  _start             (global, defined)
  boot_stack_top     (global, defined)

符号引用:
  rust_main          (undefined, 外部符号)
```

**main.rs → 目标文件：**
```
符号定义:
  rust_main          (global, defined)
  clear_bss          (local)

符号引用:
  sbss               (undefined, 外部符号)
  ebss               (undefined, 外部符号)
```

#### 阶段 2: 链接时

**链接器处理：**

1. **读取 linker.ld，定义符号：**
   ```
   sbss  = 0x80213000  (示例地址)
   ebss  = 0x80213100
   ```

2. **合并目标文件，解析符号：**
   ```
   _start       = 0x80200000  (来自 entry.asm)
   rust_main    = 0x80200010  (来自 main.rs)
   boot_stack_top = 0x80213000 (来自 entry.asm)
   sbss         = 0x80213000  (来自 linker.ld)
   ebss         = 0x80213100  (来自 linker.ld)
   ```

3. **重定位：**
   ```
   entry.asm:
     la sp, boot_stack_top
     → lui sp, %hi(0x80213000)
     → addi sp, sp, %lo(0x80213000)

     call rust_main
     → auipc ra, %pcrel_hi(rust_main)
     → jalr ra, %pcrel_lo(rust_main)

   main.rs:
     sbss as usize
     → 替换为常量 0x80213000
   ```

---

## 调试与验证

### 查看符号表

```bash
$ rust-readobj -s target/riscv64gc-unknown-none-elf/release/os
```

**输出示例：**
```
Symbol table '.symtab' contains 18 entries:
   Num:    Value          Size Type    Bind   Vis      Ndx Name
     0: 0000000000000000     0 NOTYPE  LOCAL  DEFAULT  UND
     1: 0000000080200000     0 NOTYPE  LOCAL  DEFAULT    1 skernel
     2: 0000000080200000     0 NOTYPE  LOCAL  DEFAULT    1 stext
     3: 0000000080200004     0 NOTYPE  LOCAL  DEFAULT    1 etext
     4: 0000000080203000     0 NOTYPE  LOCAL  DEFAULT    2 sbss
     5: 0000000080213000     0 NOTYPE  LOCAL  DEFAULT    2 ebss
     6: 0000000080200000     0 NOTYPE  GLOBAL DEFAULT    1 _start
     7: 0000000080200010    16 FUNC    GLOBAL DEFAULT    1 rust_main
     8: 0000000080213000     0 NOTYPE  GLOBAL DEFAULT    2 boot_stack_top
```

### 反汇编验证

```bash
$ rust-objdump -d target/riscv64gc-unknown-none-elf/release/os
```

**输出示例：**
```
0000000080200000 <_start>:
80200000: 37 31 21 08   lui     sp, 0x8213       # 加载 boot_stack_top 高位
80200004: 13 01 01 00   addi    sp, sp, 0        # 加载低位
80200008: 97 00 00 00   auipc   ra, 0            # 计算 rust_main 地址
8020000c: e7 80 80 00   jalr    ra, 8(ra)        # 跳转到 rust_main

0000000080200010 <rust_main>:
80200010: 13 01 01 ff   addi    sp, sp, -16
80200014: 23 30 11 00   sd      ra, 0(sp)
80200018: 97 00 00 00   auipc   ra, 0
8020001c: e7 80 c0 01   jalr    ra, 28(ra)       # 调用 clear_bss
80200020: ...
```

### 使用 GDB 调试

```bash
# 启动 QEMU（带调试选项）
$ qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000 \
    -s -S

# 另一个终端启动 GDB
$ riscv64-unknown-elf-gdb \
    -ex 'file target/riscv64gc-unknown-none-elf/release/os' \
    -ex 'set arch riscv:rv64' \
    -ex 'target remote localhost:1234'

# GDB 命令
(gdb) b *0x80200000       # 在 _start 设置断点
(gdb) c                   # 继续执行
(gdb) info registers sp   # 查看 sp 寄存器
(gdb) x/16i $pc           # 查看当前指令
(gdb) si                  # 单步执行
```

### 检查内存布局

```bash
# 查看段大小
$ rust-size target/riscv64gc-unknown-none-elf/release/os
   text    data     bss     dec     hex filename
      4       0   65536   65540   10004 os

# 查看详细段信息
$ rust-readobj -S target/riscv64gc-unknown-none-elf/release/os
```

---

## 总结

### 四个文件的职责

| 文件 | 职责 | 关键内容 |
|-----|------|---------|
| **config.toml** | 构建配置 | 目标架构、链接脚本路径 |
| **entry.asm** | 汇编入口 | _start 入口点、栈空间、调用 rust_main |
| **main.rs** | Rust 主程序 | rust_main 函数、clear_bss、包含汇编 |
| **linker.ld** | 链接脚本 | 内存布局、段排列、符号定义 |

### 协作流程

```
config.toml
    ↓ 告诉 Cargo
Cargo 调用 rustc
    ↓ 编译
main.rs (包含 entry.asm)
    ↓ 生成目标文件
rustc 调用链接器
    ↓ 使用
linker.ld
    ↓ 链接
最终 ELF 文件
```

### 关键依赖

1. **config.toml → linker.ld**
   - `-Clink-arg=-Tsrc/linker.ld`

2. **main.rs → entry.asm**
   - `global_asm!(include_str!("entry.asm"))`

3. **entry.asm → main.rs**
   - `call rust_main`

4. **main.rs → linker.ld**
   - `extern "C" { fn sbss(); fn ebss(); }`

5. **linker.ld → entry.asm**
   - `.text.entry` 放在最前面
   - `ENTRY(_start)`

### 最终效果

运行 `cargo build --release` 后，生成的 ELF 文件：
- ✅ 入口点：0x80200000 (_start)
- ✅ 第一条指令：初始化栈指针
- ✅ 第二条指令：跳转到 rust_main
- ✅ BSS 段：包含 64KB 栈 + 全局变量
- ✅ 符号：sbss 和 ebss 正确定义
- ✅ 可以被 QEMU 加载并执行

---

## 补充：生成二进制镜像（重要！）

### 为什么需要这一步？

**教材要求：** 在使用 QEMU 加载内核之前，需要将 ELF 文件转换为纯二进制镜像。

**原因：**

1. **QEMU `-device loader` 的限制**
   - 直接逐字节加载文件到指定地址
   - **不解析 ELF 格式**
   - 如果直接加载 ELF，会把元数据也加载到内存

2. **文件大小对比**
   ```bash
   ELF 文件:  5.4KB  (包含大量元数据)
   .bin 文件: 4 字节 (只有纯机器码)
   ```

3. **内存布局问题**
   ```
   ELF 直接加载 (错误):
   0x80200000: 7f 45 4c 46  ← ELF Magic (不可执行)
   0x80201000: 93 00 40 06  ← 实际代码在这里

   .bin 加载 (正确):
   0x80200000: 93 00 40 06  ← 直接是代码
   ```

### 生成二进制镜像

```bash
# 去除所有元数据，生成纯二进制文件
rust-objcopy --strip-all \
    target/riscv64gc-unknown-none-elf/release/os \
    -O binary \
    target/riscv64gc-unknown-none-elf/release/os.bin
```

**命令解释：**
- `--strip-all` - 移除所有符号表和调试信息
- `-O binary` - 输出格式为纯二进制（无 ELF 头）
- 输入：ELF 文件
- 输出：.bin 文件（纯机器码）

### 文件对比

```bash
# 查看文件大小
$ ls -lh target/riscv64gc-unknown-none-elf/release/os*
-rwxrwxr-x 1 user user 5.4K  os       # ELF 文件
-rwxrwxr-x 1 user user   4   os.bin   # 二进制镜像

# 查看 .bin 内容
$ xxd target/riscv64gc-unknown-none-elf/release/os.bin
00000000: 9300 4006                                ..@.

# 对比反汇编
$ rust-objdump -d target/riscv64gc-unknown-none-elf/release/os
0000000080200000 <stext>:
80200000: 06400093     	li	ra, 0x64
          ^^^^^^^^
          机器码: 06400093 = 93 00 40 06 (小端序)
```

**验证：** .bin 文件就是 ELF 中 .text 段的原始字节！

### 使用 QEMU 加载

```bash
qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000
```

**关键参数：**
- `-device loader,file=os.bin,addr=0x80200000`
  - 将 `os.bin` 的内容加载到物理地址 0x80200000
  - 逐字节复制，不解析格式

### 完整构建和运行流程

```bash
# 1. 编译（生成 ELF）
cargo build --release

# 2. 转换（生成 .bin）
rust-objcopy --strip-all \
    target/riscv64gc-unknown-none-elf/release/os \
    -O binary \
    target/riscv64gc-unknown-none-elf/release/os.bin

# 3. 运行（使用 .bin）
qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000
```

### 为什么之前的分析遗漏了这一步？

**原因：** 之前主要关注"如何构建 ELF 文件"，而忽略了"如何部署到裸机环境"。

**重要性：**
- ⚠️ **必需步骤**：不能直接用 ELF 文件
- 📚 **教学重点**：理解裸机加载机制
- 🔧 **实践要求**：真实硬件也需要这个步骤

**详细说明：** 参见 [ELF与二进制镜像的区别.md](./ELF与二进制镜像的区别.md)

---

**参考资料：**
- [rCore Tutorial Book](https://rcore-os.cn/rCore-Tutorial-Book-v3/)
- [RISC-V Assembly Programmer's Manual](https://github.com/riscv-non-isa/riscv-asm-manual/blob/master/riscv-asm.md)
- [GNU LD Linker Script Manual](https://sourceware.org/binutils/docs/ld/Scripts.html)
- [ELF Format Specification](https://refspecs.linuxfoundation.org/elf/elf.pdf)
