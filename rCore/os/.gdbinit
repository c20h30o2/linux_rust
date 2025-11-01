# GDB 配置文件 for rCore-OS 调试
# 使用方法：
#   终端1: make run-gdb (或手动启动 QEMU with -s -S)
#   终端2: gdb-multiarch -x .gdbinit

# 加载 ELF 符号
file target/riscv64gc-unknown-none-elf/release/os

# 设置架构为 RISC-V 64 位
set arch riscv:rv64

# 连接到 QEMU GDB 服务器
target remote localhost:1234

# 设置断点
# break _start
# break rust_main

# 显示配置
# set disassembly-flavor intel  # 仅 x86 架构有效，RISC-V 不需要
set print pretty on
set print array on
set print array-indexes on

# 自动显示当前位置的汇编代码
# display/i $pc

# 欢迎信息
echo \n
echo ===================================\n
echo  rCore OS Debugger Connected!\n
echo ===================================\n
echo Useful commands:\n
echo   b rust_main      - Break at rust_main\n
echo   b _start         - Break at _start\n
echo   c                - Continue execution\n
echo   si               - Step instruction\n
echo   s                - Step into\n
echo   n                - Step over\n
echo   info registers   - Show all registers\n
echo   bt               - Backtrace\n
echo   x/10i $pc        - Show 10 instructions at PC\n
echo ===================================\n
echo \n

# 提示设置断点
echo Type 'b rust_main' then 'c' to start debugging\n
echo \n
