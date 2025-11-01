# GDB 调试 rCore OS 指南

**日期：** 2025-10-31
**工具：** gdb-multiarch（替代 riscv64-unknown-elf-gdb）

---

## 问题：riscv64-unknown-elf-gdb 未找到

### 错误信息

```bash
$ riscv64-unknown-elf-gdb
riscv64-unknown-elf-gdb：未找到命令
```

### 原因

系统中没有安装 `riscv64-unknown-elf-gdb`，但有 **`gdb-multiarch`** 可以替代使用。

### 解决方案

使用 `gdb-multiarch`，它是一个**多架构 GDB**，完全支持 RISC-V 调试。

---

## 三种调试方法

### 方法 1：使用 debug.sh 脚本（最简单）

**一键启动调试：**

```bash
cd /home/c20h30o2/rs_project/rCore/os
./debug.sh
```

**脚本会自动：**
1. 检查并编译项目
2. 生成二进制镜像
3. 启动 QEMU（调试模式）
4. 启动 GDB 并连接

**GDB 启动后：**
```gdb
(gdb) b rust_main     # 设置断点
(gdb) c               # 继续执行
```

---

### 方法 2：使用 .gdbinit 配置文件

**终端 1（启动 QEMU）：**
```bash
cd /home/c20h30o2/rs_project/rCore/os

# 编译
cargo build --release
rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os \
    -O binary target/riscv64gc-unknown-none-elf/release/os.bin

# 启动 QEMU（调试模式）
qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000 \
    -s -S
```

**终端 2（启动 GDB）：**
```bash
cd /home/c20h30o2/rs_project/rCore/os
gdb-multiarch -x .gdbinit
```

**优点：**
- 自动加载符号文件
- 自动设置架构
- 自动连接到 QEMU
- 显示使用提示

---

### 方法 3：手动命令（完全控制）

**终端 1（启动 QEMU）：**
```bash
qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000 \
    -s -S
```

**终端 2（手动启动 GDB）：**
```bash
gdb-multiarch \
    -ex 'file target/riscv64gc-unknown-none-elf/release/os' \
    -ex 'set arch riscv:rv64' \
    -ex 'target remote localhost:1234'
```

---

## QEMU 调试参数说明

| 参数 | 作用 | 详细说明 |
|------|------|---------|
| `-s` | 启动 GDB 服务器 | 等价于 `-gdb tcp::1234`，在 localhost:1234 监听 |
| `-S` | 启动时暂停 CPU | 程序不会运行，等待 GDB 发送 `continue` 命令 |

**组合使用：**
```bash
-s -S    # 启动 GDB 服务器 + 暂停 CPU（调试用）
-s       # 启动 GDB 服务器，但程序正常运行（可随时 attach）
无参数    # 正常运行，不支持调试
```

---

## GDB 常用命令

### 基础命令

```gdb
# 连接和退出
target remote localhost:1234    # 连接到 QEMU
quit                            # 退出 GDB
Ctrl+C                          # 中断运行

# 断点管理
break rust_main                 # 在函数设置断点
break src/main.rs:72            # 在文件行号设置断点
break *0x80200000               # 在地址设置断点
info breakpoints                # 查看所有断点
delete 1                        # 删除断点 1
clear rust_main                 # 清除函数的所有断点

# 执行控制
continue (c)                    # 继续执行
step (s)                        # 单步执行（进入函数）
next (n)                        # 单步执行（跳过函数）
stepi (si)                      # 单步执行一条汇编指令
nexti (ni)                      # 执行一条指令（跳过 call）
finish                          # 执行到当前函数返回
until                           # 执行到当前循环结束
```

### 查看信息

```gdb
# 查看代码
list                            # 显示源代码
list rust_main                  # 显示函数源代码
list src/main.rs:72             # 显示指定行

# 查看寄存器
info registers                  # 显示所有寄存器
info registers pc sp            # 显示指定寄存器
print/x $pc                     # 以十六进制打印 PC
print/x $sp                     # 以十六进制打印 SP

# 查看内存
x/10i $pc                       # 查看 PC 处的 10 条指令
x/10x 0x80200000                # 查看地址的 10 个十六进制值
x/10s 0x80200000                # 查看地址的字符串
x/10gx $sp                      # 查看栈上的 10 个 64 位值

# 查看调用栈
backtrace (bt)                  # 显示调用栈
frame 0                         # 切换到栈帧 0
info frame                      # 显示当前栈帧信息

# 查看符号
info symbol 0x80200000          # 查看地址对应的符号
info address rust_main          # 查看符号的地址
info functions                  # 列出所有函数
```

### 查看变量

```gdb
# 打印变量
print variable_name             # 打印变量
print/x variable_name           # 十六进制打印
print/t variable_name           # 二进制打印
print *pointer                  # 打印指针指向的值

# 监视变量
watch variable_name             # 监视变量，变化时中断
info watchpoints                # 查看所有监视点

# 显示变量
display variable_name           # 每次停止时自动显示
undisplay 1                     # 取消显示 1
```

---

## 调试场景示例

### 场景 1：从头开始调试（_start）

```gdb
# 连接后
(gdb) break _start              # 在入口设置断点
Breakpoint 1 at 0x80200000: file src/entry.asm, line 4.

(gdb) continue                  # 继续执行
Continuing.

Breakpoint 1, _start () at src/entry.asm:4
4           la sp, boot_stack_top

(gdb) stepi                     # 单步执行汇编
0x80200004 in _start () at src/entry.asm:5
5           call rust_main

(gdb) info registers sp         # 查看 SP 是否初始化
sp             0x80210000

(gdb) stepi                     # 执行 call rust_main
rust_main () at src/main.rs:72
72      pub fn rust_main() -> ! {
```

### 场景 2：调试 rust_main

```gdb
(gdb) break rust_main
Breakpoint 1 at 0x802001a0: file src/main.rs, line 72.

(gdb) continue
Continuing.

Breakpoint 1, rust_main () at src/main.rs:72
72      pub fn rust_main() -> ! {

(gdb) next                      # 执行下一行
74          clear_bss();

(gdb) step                      # 进入 clear_bss 函数
clear_bss () at src/main.rs:87
87      fn clear_bss() {

(gdb) finish                    # 执行完 clear_bss 返回
rust_main () at src/main.rs:75
75          println!("this is a test");

(gdb) next                      # 执行 println
76          info!("this is a info");
```

### 场景 3：查看内存布局

```gdb
(gdb) break rust_main
(gdb) continue

# 查看 .text 段
(gdb) info address stext
Symbol "stext" is at 0x80200000

(gdb) info address etext
Symbol "etext" is at 0x80201234

# 查看 .bss 段
(gdb) info address sbss
Symbol "sbss" is at 0x80203000

(gdb) info address ebss
Symbol "ebss" is at 0x80203100

# 查看栈
(gdb) info address boot_stack_lower_bound
Symbol "boot_stack_lower_bound" is at 0x80210000

(gdb) info address boot_stack_top
Symbol "boot_stack_top" is at 0x80220000

# 查看栈内容
(gdb) x/16gx $sp
0x80220000: 0x0000000000000000  0x0000000000000000
0x80220010: 0x0000000000000000  0x0000000000000000
...
```

### 场景 4：调试 panic

```gdb
(gdb) break panic
Breakpoint 1 at 0x802002e0: file src/lang_items.rs, line 46.

(gdb) continue
Continuing.
[... 程序执行到 panic! ...]

Breakpoint 1, panic (info=0x80220f00) at src/lang_items.rs:46
46      fn panic(info: &PanicInfo) -> ! {

(gdb) print info
$1 = (core::panic::PanicInfo *) 0x80220f00

(gdb) print *info
$2 = {...}

(gdb) backtrace
#0  panic (info=0x80220f00) at src/lang_items.rs:46
#1  0x80200456 in rust_main () at src/main.rs:83
#2  0x80200008 in _start () at src/entry.asm:5

(gdb) frame 1                   # 切换到调用 panic 的栈帧
#1  0x80200456 in rust_main () at src/main.rs:83
83          panic!("Shutdown machine!");

(gdb) list
78          trace!("trace");
79          error!("error");
80          debug!("debug");
81
82          // info!(".text [{:#x}, {:#x})", s_text as usize, e_text as usize);
83          panic!("Shutdown machine!");
```

### 场景 5：查看汇编代码

```gdb
(gdb) break rust_main
(gdb) continue

# 反汇编当前函数
(gdb) disassemble
Dump of assembler code for function rust_main:
=> 0x802001a0 <+0>:     addi    sp,sp,-16
   0x802001a2 <+2>:     sd      ra,8(sp)
   0x802001a4 <+4>:     call    0x80200300 <clear_bss>
   ...

# 反汇编指定函数
(gdb) disassemble clear_bss
Dump of assembler code for function clear_bss:
   0x80200300 <+0>:     addi    sp,sp,-32
   0x80200302 <+2>:     sd      ra,24(sp)
   ...

# 查看指定地址的指令
(gdb) x/10i 0x80200000
   0x80200000 <_start>:         auipc   sp,0x10
   0x80200004 <_start+4>:       mv      sp,sp
   0x80200006 <_start+6>:       jal     ra,0x802001a0 <rust_main>
   ...
```

### 场景 6：条件断点

```gdb
# 在 clear_bss 中，当 a == 某个值时中断
(gdb) break clear_bss
(gdb) condition 1 a == 0x80203050

# 或者直接设置条件断点
(gdb) break src/main.rs:92 if a == 0x80203050

(gdb) continue
# 只有当条件满足时才会中断
```

---

## 高级调试技巧

### 1. 自动显示

```gdb
# 每次停止时自动显示
display/i $pc                   # 显示当前指令
display/x $sp                   # 显示栈指针
display/16gx $sp                # 显示栈内容

# 查看所有自动显示
info display

# 删除自动显示
undisplay 1
```

### 2. 保存断点

```gdb
# 保存所有断点到文件
save breakpoints breakpoints.gdb

# 加载断点
source breakpoints.gdb
```

### 3. 日志记录

```gdb
# 开启日志
set logging on
set logging file gdb.log

# 执行调试...

# 关闭日志
set logging off
```

### 4. TUI 模式（文本用户界面）

```gdb
# 启用 TUI 模式
tui enable

# 或者启动时就使用 TUI
gdb-multiarch -tui -x .gdbinit

# TUI 快捷键
Ctrl+X A        # 切换 TUI 模式
Ctrl+X 1        # 单窗口
Ctrl+X 2        # 双窗口（源码 + 汇编）
Ctrl+L          # 刷新屏幕
```

### 5. 远程调试不同端口

```gdb
# 如果 1234 端口被占用，可以指定其他端口
# QEMU 启动时：
qemu-system-riscv64 ... -gdb tcp::5678

# GDB 连接时：
target remote localhost:5678
```

---

## 调试 RustSBI 启动过程

### 查看 RustSBI 入口

```gdb
# 不设置断点，直接查看初始状态
(gdb) target remote localhost:1234
(gdb) info registers pc
pc             0x1000

(gdb) x/10i $pc
   0x1000:      auipc   t0,0x0
   0x1004:      addi    a2,t0,40
   0x1008:      csrr    a0,mhartid
   ...

# 查看 RustSBI 代码
(gdb) x/100i 0x80000000
```

### 追踪跳转到 0x80200000

```gdb
# 在 0x80200000 设置断点
(gdb) break *0x80200000
Breakpoint 1 at 0x80200000

(gdb) continue
# 会在 RustSBI 跳转到你的内核时中断

# 查看是如何跳转的
(gdb) backtrace
(gdb) x/10i $pc-20    # 查看跳转前的指令
```

---

## 常见问题

### 1. GDB 连接失败

**错误：**
```
Connection refused
```

**解决：**
```bash
# 确保 QEMU 已启动且使用了 -s 参数
ps aux | grep qemu

# 确保端口 1234 未被占用
netstat -tlnp | grep 1234
```

### 2. 无法查看源代码

**错误：**
```
(gdb) list
No symbol table is loaded.
```

**解决：**
```gdb
# 加载符号文件
file target/riscv64gc-unknown-none-elf/release/os

# 或者使用 debug 版本（包含更多调试信息）
file target/riscv64gc-unknown-none-elf/debug/os
```

### 3. 断点设置失败

**错误：**
```
(gdb) break rust_main
No symbol "rust_main" in current context.
```

**解决：**
```gdb
# 确保已加载符号文件
file target/riscv64gc-unknown-none-elf/release/os

# 查看可用函数
info functions

# 使用地址设置断点
break *0x802001a0
```

### 4. 单步执行进入 SBI 代码

**现象：**
```gdb
(gdb) stepi
0x80000000 in ?? ()
```

**解决：**
```gdb
# 使用 finish 返回
finish

# 或者使用 next/nexti 跳过 call
nexti
```

---

## 调试流程总结

### 典型调试会话

```bash
# 1. 编译项目
cargo build --release

# 2. 生成二进制
rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os \
    -O binary target/riscv64gc-unknown-none-elf/release/os.bin

# 3. 启动 QEMU（终端 1）
qemu-system-riscv64 \
    -machine virt -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000 \
    -s -S

# 4. 启动 GDB（终端 2）
gdb-multiarch -x .gdbinit
```

**GDB 中的操作：**
```gdb
(gdb) b rust_main      # 设置断点
(gdb) c                # 继续执行
(gdb) n                # 单步执行
(gdb) p variable       # 查看变量
(gdb) bt               # 查看调用栈
(gdb) c                # 继续执行
(gdb) quit             # 退出
```

---

## 创建别名简化命令

**添加到 `~/.bashrc`：**

```bash
# RISC-V 调试别名
alias riscv-gdb='gdb-multiarch'
alias rcore-debug='cd ~/rs_project/rCore/os && ./debug.sh'

# QEMU 运行别名
alias rcore-run='cd ~/rs_project/rCore/os && \
    cargo build --release && \
    rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os -O binary target/riscv64gc-unknown-none-elf/release/os.bin && \
    qemu-system-riscv64 -machine virt -nographic -bios ../bootloader/rustsbi-qemu.bin -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000'
```

**使用：**
```bash
source ~/.bashrc

# 直接运行
rcore-run

# 调试运行
rcore-debug
```

---

## 参考资料

1. **GDB 官方文档**
   - https://sourceware.org/gdb/documentation/

2. **RISC-V GDB 调试**
   - https://github.com/riscv/riscv-tools/blob/master/README.md

3. **QEMU GDB 调试**
   - https://qemu.readthedocs.io/en/latest/system/gdb.html

4. **rCore Tutorial Book**
   - https://rcore-os.cn/rCore-Tutorial-Book-v3/

---

**总结：** 使用 `gdb-multiarch` 替代 `riscv64-unknown-elf-gdb`，功能完全相同。建议使用 `./debug.sh` 脚本或 `.gdbinit` 配置文件简化调试流程。
