# before start
## 环境配置
### rust配置
新建 ~/.cargo/config.toml添加以下内容：

[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"

检查：
cargo --version
### 按照步骤配置qemu  
之前已经安装了  
检查：  
qemu-system-riscv64 --version  
qemu-riscv64 --version  
which qemu-system-riscv64
   
c20h30o2@c20h30o2:~/files/rCore-Tutorial-Code-2025S$  
qemu-system-riscv64 --version
QEMU emulator version 8.2.2 (Debian 1:8.2.2+ds-0ubuntu1.10)
Copyright (c) 2003-2023 Fabrice Bellard and the QEMU Project developers
注意：
如果使用 Qemu8，你需要：
1.替换 bootloader/rustsbi-qemu.bin 为最新版 在这里下载 后更名为 bootloader/rustsbi-qemu.bin 并替换同名文件即可
2.将 os/src/sbi.rs 中的常量 SBI_SHUTDOWN 的值替换为 const SBI_SHUTDOWN: usize = 0x53525354;，SBI_SET_TIMER 的值替换为 const SBI_SET_TIMER: usize = 0x54494D45;
经过测试，发现只需要做第一步即可，并且第二步提到的文件中并不存在那两个常量

https://github.com/rustsbi/rustsbi-qemu/releases/tag/Unreleased
到上述网址获取rustsbi-qumu-debug.zip,解压后得到rustsbi-qemu.bin ，替换即可




## 测试运行
git clone https://github.com/LearningOS/rCore-Tutorial-Code-2025S  
cd rCore-Tutorial-Code-2025S
  
git checkout ch1  
cd os  
LOG=DEBUG make run  
使用qemu8若不做上面的修改，则这一步无法看到正常输出，且qemu不自动退出，需要ctrl a + x手动退出



# 第一章
main.rs01 -> main.rs02 -> main.rs03 -> main.rs

## 注意1：
官方提供的代码：git clone https://github.com/LearningOS/rCore-Tutorial-Code-2025S  其中有一个与os同级的目录bootloader，用于将os文件载入qemu,该目录下有一个文件 rustsbi-qemu.bin ,  
如果是跟随教程文档复现官方源码，记得要将这个文件粘贴到自己的项目目录下  
否则在执行：  
    qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000  
时，会报错：  
qemu-system-riscv64: Unable to find the RISC-V BIOS "../bootloader/rustsbi-qemu.bin"

## 注意2：  
$ riscv64-unknown-elf-gdb命令报：

riscv64-unknown-elf-gdb: command not found
解决办法：安装 RISC-V 工具集

$ git clone https://github.com/riscv/riscv-gnu-toolchain

$ cd riscv-gnu-toolchain

/home/kay/tools/riscv-gnu-toolchain 为安装目录  
$ ./configure --prefix=/home/kay/tools/riscv-gnu-toolchain

riscv-gnu-toolchain 目录下  
$ make  
make这一步可能需要花费很长时间
  
$ make install clean

添加环境变量  
编辑 .bashrc 文件，添加：  
export PATH=$HOME/tools/riscv-gnu-toolchain/bin:$PATH

刷新  
$ source ./.bashrc

