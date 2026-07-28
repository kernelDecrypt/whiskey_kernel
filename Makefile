RUSTC   = rustc
SYSROOT = $(shell rustc --print sysroot)
CROSS_PREFIX ?= $(shell if command -v riscv64-elf-as >/dev/null 2>&1; then echo riscv64-elf-; elif command -v riscv64-linux-gnu-as >/dev/null 2>&1; then echo riscv64-linux-gnu-; else echo riscv64-elf-; fi)
AS = $(CROSS_PREFIX)as
LD = $(CROSS_PREFIX)ld
TARGET = riscv64gc-unknown-none-elf
RUSTFLAGS = --target $(TARGET) --sysroot $(SYSROOT) -O -C panic=abort
BIN = build/kernel.elf

VERSION := 0.0.7_alpha

export VERSION

all: $(BIN)

RUSTLIB = $(SYSROOT)/lib/rustlib/$(TARGET)/lib
CORE_LIBS = $(wildcard $(RUSTLIB)/liballoc-*.rlib) \
			$(wildcard $(RUSTLIB)/libcore-*.rlib) \
            $(wildcard $(RUSTLIB)/libcompiler_builtins-*.rlib)

$(BIN): src/boot/entry.s src/boot/trap.s src/boot/s_entry.s src/boot/strap.s src/kernel/main.rs linker.ld
	@mkdir -p build
	$(AS) src/boot/entry.s -o build/entry.o
	$(AS) src/boot/trap.s -o build/trap.o
	$(AS) src/boot/s_entry.s -o build/s_entry.o
	$(AS) src/boot/strap.s -o build/strap.o
	$(RUSTC) $(RUSTFLAGS) src/kernel/main.rs -C link-args='build/entry.o build/trap.o build/s_entry.o build/strap.o -T linker.ld' -o $(BIN)
	
run: $(BIN)
	qemu-system-riscv64 -machine virt -bios none -kernel build/kernel.elf -nographic

clean:
	rm -rf build