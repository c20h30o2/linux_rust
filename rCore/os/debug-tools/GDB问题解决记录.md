# GDB 调试问题解决记录

**日期：** 2025-11-01
**问题：** `disassemble rust_main` 显示 "No symbol table is loaded"

---

## 问题现象

使用 `debug.sh` 或手动启动 GDB 后，尝试反汇编函数时报错：

```gdb
>>> disassemble rust_main
No symbol table is loaded.  Use the "file" command.
```

---

## 根本原因

**GDB 配置文件冲突：**

1. **全局配置：** `~/.gdbinit` 包含 GDB Dashboard 配置
2. **项目配置：** `./.gdbinit` 包含 rCore 项目配置
3. **加载顺序：** GDB 先加载 `~/.gdbinit`，再处理命令行参数

**问题：**
- GDB Dashboard 的配置可能干扰符号加载
- 即使使用 `-x .gdbinit`，全局配置仍会先执行
- 导致符号表未正确加载

---

## 解决方案

### 方案 1：使用 `-nh` 参数跳过全局配置（已修复）

**修改 `debug.sh`：**

```bash
# 旧版本（有问题）
gdb-multiarch -x .gdbinit

# 新版本（修复后）
gdb-multiarch -nh -x .gdbinit
```

**`-nh` 参数说明：**
- `-n` = `--nx` = 不执行任何初始化文件
- `-nh` = 不读取 `~/.gdbinit`（但仍读取系统级配置）

**效果：**
- 跳过 `~/.gdbinit`（GDB Dashboard）
- 只加载项目的 `.gdbinit`
- 避免配置冲突

---

### 方案 2：手动命令（临时方案）

如果不想修改脚本，可以手动启动：

**终端 1（QEMU）：**
```bash
cd /home/c20h30o2/rs_project/rCore/os

qemu-system-riscv64 \
    -machine virt -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000 \
    -s -S
```

**终端 2（GDB，跳过全局配置）：**
```bash
cd /home/c20h30o2/rs_project/rCore/os

gdb-multiarch -nh -x .gdbinit
```

**或者完全手动：**
```bash
gdb-multiarch -nh \
    -ex 'file target/riscv64gc-unknown-none-elf/release/os' \
    -ex 'set arch riscv:rv64' \
    -ex 'target remote localhost:1234'
```

---

### 方案 3：配置 `~/.gdbinit` 允许本地配置

在 `~/.gdbinit` 开头添加（不推荐，可能破坏 Dashboard）：

```python
# 允许加载项目本地的 .gdbinit
set auto-load safe-path /
```

**不推荐原因：**
- 安全风险（允许任何目录的 .gdbinit）
- 可能与 GDB Dashboard 冲突

---

## GDB 参数对比

| 参数 | 效果 | 适用场景 |
|------|------|---------|
| **无参数** | 加载所有配置文件 | 日常使用（非 rCore 调试）|
| **`-n` / `--nx`** | 不加载任何初始化文件 | 完全干净的环境 |
| **`-nh`** | 不加载 `~/.gdbinit` | **rCore 调试（推荐）** |
| **`-x file`** | 显式加载指定配置 | 配合 `-nh` 使用 |

---

## 验证修复

### 测试 1：检查符号是否加载

```bash
cd /home/c20h30o2/rs_project/rCore/os

gdb-multiarch -nh -batch \
    -ex 'file target/riscv64gc-unknown-none-elf/release/os' \
    -ex 'info functions rust_main'
```

**期望输出：**
```
All functions matching regular expression "rust_main":

Non-debugging symbols:
0x0000000080200384  rust_main
```

**✅ 成功：** 显示 `rust_main` 的地址
**❌ 失败：** 显示 "No symbol table is loaded"

---

### 测试 2：完整调试流程

**终端 1：**
```bash
cd /home/c20h30o2/rs_project/rCore/os
./debug.sh
```

**GDB 启动后：**
```gdb
(gdb) info files
Symbols from "/home/c20h30o2/.../os".
...                                    ← 应该显示符号文件路径

(gdb) info functions
All defined functions:
...
File src/main.rs:
0x0000000080200384  rust_main          ← 应该看到函数列表
...

(gdb) disassemble rust_main            ← 现在应该工作了！
Dump of assembler code for function rust_main:
   0x0000000080200384 <+0>:     addi    sp,sp,-32
   0x0000000080200386 <+2>:     sd      ra,24(sp)
   ...
```

---

## 其他修复

### 修复 1：移除 RISC-V 不支持的配置

**.gdbinit 第 20 行（已修复）：**

```gdb
# 旧版本（错误）
set disassembly-flavor intel

# 新版本（修复）
# set disassembly-flavor intel  # 仅 x86 架构有效，RISC-V 不需要
```

**原因：**
- `disassembly-flavor` 只在 x86/x86_64 有效
- RISC-V 架构不支持此配置
- 可能导致警告或错误

---

## 文件对比

### ELF vs .bin

| 文件 | 路径 | 用途 | 符号信息 |
|------|------|------|---------|
| **os** | `target/riscv64gc-unknown-none-elf/release/os` | **GDB 调试** | ✅ 有 |
| **os.bin** | `target/riscv64gc-unknown-none-elf/release/os.bin` | **QEMU 加载** | ❌ 无 |

**关键点：**
```
编译 → os (ELF)
         ↓
      ┌──┴────┐
      ↓       ↓
    GDB      rust-objcopy
    用这个    ↓
            os.bin
              ↓
            QEMU
            用这个
```

**调试时：**
- **QEMU 加载：** `os.bin`（无符号，纯二进制）
- **GDB 加载：** `os`（有符号，ELF 格式）

---

## 常见错误

### 错误 1：加载了 .bin 文件

```gdb
(gdb) file target/riscv64gc-unknown-none-elf/release/os.bin
(gdb) info functions
No symbol table is loaded.
```

**原因：** `.bin` 是纯二进制，没有符号信息

**修复：**
```gdb
(gdb) file target/riscv64gc-unknown-none-elf/release/os
Reading symbols from ...
```

---

### 错误 2：没有加载文件

```gdb
(gdb) target remote localhost:1234
(gdb) disassemble rust_main
No symbol table is loaded.  Use the "file" command.
```

**原因：** 连接了 QEMU 但没有加载符号文件

**修复：**
```gdb
(gdb) file target/riscv64gc-unknown-none-elf/release/os
(gdb) set arch riscv:rv64
(gdb) target remote localhost:1234
(gdb) disassemble rust_main  # 现在可以了
```

---

### 错误 3：全局配置冲突

```bash
gdb-multiarch -x .gdbinit
# 仍然报错：No symbol table is loaded
```

**原因：** `~/.gdbinit`（GDB Dashboard）先加载，干扰配置

**修复：**
```bash
gdb-multiarch -nh -x .gdbinit  # 跳过 ~/.gdbinit
```

---

## 快速使用

### 方法 1：使用脚本（最简单）

```bash
cd /home/c20h30o2/rs_project/rCore/os
./debug.sh
```

**自动完成：**
- ✅ 编译项目
- ✅ 生成 .bin
- ✅ 启动 QEMU
- ✅ 启动 GDB（正确配置）

---

### 方法 2：手动两步

**终端 1：**
```bash
cd /home/c20h30o2/rs_project/rCore/os
qemu-system-riscv64 \
    -machine virt -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000 \
    -s -S
```

**终端 2：**
```bash
cd /home/c20h30o2/rs_project/rCore/os
gdb-multiarch -nh -x .gdbinit
```

---

## 总结

**问题：** GDB Dashboard（`~/.gdbinit`）干扰项目配置

**解决：** 使用 `-nh` 跳过全局配置

**修改文件：**
1. ✅ `debug.sh` - 添加 `-nh` 参数
2. ✅ `.gdbinit` - 移除 RISC-V 不支持的配置

**验证：**
```bash
./debug.sh
```

**GDB 中：**
```gdb
(gdb) disassemble rust_main
Dump of assembler code for function rust_main:
   0x0000000080200384 <+0>:     addi    sp,sp,-32
   ...
```

✅ **成功！**

---

## 参考

- GDB 文档：https://sourceware.org/gdb/documentation/
- GDB Dashboard：https://github.com/cyrus-and/gdb-dashboard
- `-nh` 参数：`man gdb` 或 `gdb --help`
