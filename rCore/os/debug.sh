#!/bin/bash
# rCore OS 调试脚本
# 使用方法: ./debug.sh

set -e

echo "======================================"
echo "  rCore OS Debugger"
echo "======================================"
echo ""

# 检查是否编译
if [ ! -f "target/riscv64gc-unknown-none-elf/release/os" ]; then
    echo "[1/4] Building project..."
    cargo build --release
else
    echo "[1/4] Binary already exists, skipping build"
fi

# 生成二进制镜像
echo "[2/4] Creating binary image..."
rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os \
    -O binary target/riscv64gc-unknown-none-elf/release/os.bin

# 检查 QEMU 是否已运行
if pgrep -f "qemu-system-riscv64.*1234" > /dev/null; then
    echo "[3/4] QEMU already running on port 1234"
else
    echo "[3/4] Starting QEMU in background..."
    echo "      (Press Ctrl+C in this terminal to stop QEMU)"

    # 启动 QEMU 并保存 PID
    qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios ../bootloader/rustsbi-qemu.bin \
        -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000 \
        -s -S &

    QEMU_PID=$!
    echo "      QEMU PID: $QEMU_PID"
    sleep 1
fi

echo "[4/4] Starting GDB..."
echo ""

# 启动 GDB（跳过 ~/.gdbinit 避免冲突）
gdb-multiarch -nh -x .gdbinit

# 清理：如果 GDB 退出，询问是否停止 QEMU
if [ ! -z "$QEMU_PID" ]; then
    echo ""
    read -p "Stop QEMU? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        kill $QEMU_PID 2>/dev/null || true
        echo "QEMU stopped"
    fi
fi
