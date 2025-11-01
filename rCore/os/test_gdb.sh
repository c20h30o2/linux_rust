#!/bin/bash
# 测试 GDB 符号加载

echo "Testing GDB symbol loading..."
echo ""

# 测试命令
gdb-multiarch -nh -batch \
    -ex 'file target/riscv64gc-unknown-none-elf/release/os' \
    -ex 'info functions rust_main' \
    -ex 'quit'

echo ""
echo "If you see rust_main address above, symbols are loaded correctly!"
