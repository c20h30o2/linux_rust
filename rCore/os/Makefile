# rCore OS Makefile
# 支持通过 LOG 参数控制日志等级
# 使用方式：make run LOG=INFO

# ============================================================================
# 配置变量
# ============================================================================

# 目标架构
TARGET := riscv64gc-unknown-none-elf

# 编译模式（debug 或 release）
MODE := release

# 日志等级（默认 ERROR）
# 可选值：ERROR, WARN, INFO, DEBUG, TRACE
LOG ?= ERROR

# 转换为小写的 feature 名称
LOG_FEATURE := log-$(shell echo $(LOG) | tr A-Z a-z)

# ============================================================================
# 路径配置
# ============================================================================

# 内核 ELF 文件
KERNEL_ELF := target/$(TARGET)/$(MODE)/os

# 内核二进制文件（去除元数据）
KERNEL_BIN := $(KERNEL_ELF).bin

# Bootloader（RustSBI）
BOOTLOADER := ../bootloader/rustsbi-qemu.bin

# ============================================================================
# 工具链
# ============================================================================

# Rust 工具
OBJDUMP := rust-objdump
OBJCOPY := rust-objcopy
NM := rust-nm

# QEMU
QEMU := qemu-system-riscv64
QEMU_ARGS := -machine virt \
             -nographic \
             -bios $(BOOTLOADER) \
             -device loader,file=$(KERNEL_BIN),addr=0x80200000

# GDB
GDB := gdb-multiarch
GDB_ARGS := -nh -x .gdbinit

# ============================================================================
# 颜色输出
# ============================================================================

# 终端颜色代码
COLOR_RESET := \033[0m
COLOR_GREEN := \033[32m
COLOR_BLUE := \033[34m
COLOR_YELLOW := \033[33m

# ============================================================================
# 主要目标
# ============================================================================

.PHONY: all
all: $(KERNEL_BIN)

# ============================================================================
# 编译目标
# ============================================================================

.PHONY: build
build:
	@echo "$(COLOR_GREEN)Building kernel with LOG=$(LOG)...$(COLOR_RESET)"
	@echo "$(COLOR_BLUE)Features: $(LOG_FEATURE)$(COLOR_RESET)"
	@cargo build --release --features $(LOG_FEATURE)
	@echo "$(COLOR_GREEN)Build complete!$(COLOR_RESET)"

# 生成二进制镜像
$(KERNEL_BIN): build
	@echo "$(COLOR_GREEN)Generating binary image...$(COLOR_RESET)"
	@$(OBJCOPY) --strip-all $(KERNEL_ELF) -O binary $@
	@ls -lh $@ | awk '{print "$(COLOR_BLUE)Binary size: " $$5 "$(COLOR_RESET)"}'

# ============================================================================
# 运行目标
# ============================================================================

.PHONY: run
run: $(KERNEL_BIN)
	@echo "$(COLOR_GREEN)Running OS with LOG=$(LOG)...$(COLOR_RESET)"
	@echo "$(COLOR_YELLOW)Press Ctrl-A then X to exit QEMU$(COLOR_RESET)"
	@$(QEMU) $(QEMU_ARGS)

# 调试模式运行（启动 GDB 服务器）
.PHONY: debug
debug: $(KERNEL_BIN)
	@echo "$(COLOR_GREEN)Starting QEMU in debug mode...$(COLOR_RESET)"
	@echo "$(COLOR_YELLOW)GDB server listening on localhost:1234$(COLOR_RESET)"
	@echo "$(COLOR_YELLOW)Run 'make gdb' in another terminal to connect$(COLOR_RESET)"
	@$(QEMU) $(QEMU_ARGS) -s -S

# 连接 GDB
.PHONY: gdb
gdb:
	@echo "$(COLOR_GREEN)Connecting GDB...$(COLOR_RESET)"
	@$(GDB) $(GDB_ARGS)

# ============================================================================
# 分析目标
# ============================================================================

# 反汇编
.PHONY: disasm
disasm: build
	@echo "$(COLOR_GREEN)Disassembling kernel...$(COLOR_RESET)"
	@$(OBJDUMP) -d $(KERNEL_ELF) | less

# 查看符号表
.PHONY: nm
nm: build
	@echo "$(COLOR_GREEN)Symbol table:$(COLOR_RESET)"
	@$(NM) $(KERNEL_ELF) | sort

# 查看段信息
.PHONY: sections
sections: build
	@echo "$(COLOR_GREEN)Section information:$(COLOR_RESET)"
	@$(OBJDUMP) -h $(KERNEL_ELF)

# 查看文件头
.PHONY: header
header: build
	@echo "$(COLOR_GREEN)ELF header:$(COLOR_RESET)"
	@$(OBJDUMP) -f $(KERNEL_ELF)

# ============================================================================
# 清理目标
# ============================================================================

.PHONY: clean
clean:
	@echo "$(COLOR_GREEN)Cleaning build artifacts...$(COLOR_RESET)"
	@cargo clean
	@rm -f $(KERNEL_BIN)
	@echo "$(COLOR_GREEN)Clean complete!$(COLOR_RESET)"

# ============================================================================
# 帮助信息
# ============================================================================

.PHONY: help
help:
	@echo "$(COLOR_GREEN)rCore OS Build System$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_BLUE)Basic Usage:$(COLOR_RESET)"
	@echo "  make run              - Build and run (default LOG=ERROR)"
	@echo "  make run LOG=INFO     - Build and run with INFO level"
	@echo "  make run LOG=DEBUG    - Build and run with DEBUG level"
	@echo "  make run LOG=TRACE    - Build and run with TRACE level"
	@echo ""
	@echo "$(COLOR_BLUE)Build Targets:$(COLOR_RESET)"
	@echo "  make build            - Build kernel"
	@echo "  make clean            - Clean build artifacts"
	@echo ""
	@echo "$(COLOR_BLUE)Debug Targets:$(COLOR_RESET)"
	@echo "  make debug            - Start QEMU with GDB server (-s -S)"
	@echo "  make gdb              - Connect GDB to QEMU"
	@echo ""
	@echo "$(COLOR_BLUE)Analysis Targets:$(COLOR_RESET)"
	@echo "  make disasm           - Disassemble kernel"
	@echo "  make nm               - Show symbol table"
	@echo "  make sections         - Show section information"
	@echo "  make header           - Show ELF header"
	@echo ""
	@echo "$(COLOR_BLUE)Available LOG levels:$(COLOR_RESET)"
	@echo "  ERROR  - Only error messages (default)"
	@echo "  WARN   - Error + Warning"
	@echo "  INFO   - Error + Warning + Info"
	@echo "  DEBUG  - Error + Warning + Info + Debug"
	@echo "  TRACE  - All messages (including trace)"
	@echo ""
	@echo "$(COLOR_BLUE)Examples:$(COLOR_RESET)"
	@echo "  make run LOG=INFO     - Run with info level logging"
	@echo "  make clean build      - Clean and rebuild"
	@echo "  make disasm | less    - View disassembly"
