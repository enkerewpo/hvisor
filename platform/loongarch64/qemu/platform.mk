# hvisor for loongarch64 / QEMU virt (with LVZ extension)
# This board targets QEMU-LVZ (https://github.com/Hengyu-Yu/QEMU-LVZ) running
# the standard 'virt' machine. Memory layout is QEMU's defaults for -m 8G.
# Reference fork: https://github.com/enkerewpo/QEMU-LVZ

# Cross-compile tools used for ELF post-processing (objdump / readelf / nm).
# Override via env: e.g.
#   make BID=loongarch64/qemu LA_CROSS_COMPILE=loongarch64-linux-gnu-
LA_CROSS_COMPILE ?= loongarch64-linux-gnu-
LA_OBJDUMP       ?= $(LA_CROSS_COMPILE)objdump
LA_READELF       ?= $(LA_CROSS_COMPILE)readelf
LA_NM            ?= $(LA_CROSS_COMPILE)nm

# HVISOR ENTRY
# Matches BASE_ADDRESS in linker.ld so the bootloader does not need a trampoline.
HVISOR_ENTRY_PA := 0x9000000100000000

# QEMU invocation (for direct -kernel hvisor.bin testing, no grub).
# For the production flow you boot Arch in QEMU and chainload hvisor via
# grub2-hvisor's hvisor_loongarch command instead.
QEMU         ?= qemu-system-loongarch64
QEMU_LVZ_BIN ?= ../QEMU-LVZ/build/qemu-system-loongarch64
QEMU_EFI     ?= ../qemu-images/QEMU_EFI.fd
QEMU_ARGS    := -machine virt
QEMU_ARGS    += -cpu max
QEMU_ARGS    += -smp 4
QEMU_ARGS    += -m 8G
QEMU_ARGS    += -bios $(QEMU_EFI)
QEMU_ARGS    += -nographic
QEMU_ARGS    += -serial mon:stdio

.PHONY: qemu-run qemu-gdb
qemu-run: $(hvisor_bin)
	$(QEMU_LVZ_BIN) $(QEMU_ARGS) -kernel $(hvisor_bin)

qemu-gdb: $(hvisor_bin)
	$(QEMU_LVZ_BIN) $(QEMU_ARGS) -kernel $(hvisor_bin) -s -S

$(hvisor_bin): elf
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@
# objdump + hvisor-trap-vector.txt
	$(LA_READELF) -a $(hvisor_elf) > hvisor-elf.txt
	$(LA_OBJDUMP) --disassemble $(hvisor_elf) > hvisor.S
	cp $(hvisor_elf) hvisor.elf
	$(LA_NM) -n hvisor.elf | grep -w _hyp_trap_vector | awk '{print "0x"$$1""}' > hvisor-trap-vector.txt
