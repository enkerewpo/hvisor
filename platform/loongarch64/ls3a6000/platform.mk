# hvisor for loongarch64 makefile
# wheatfox(wheatfox17@icloud.com) 2024.6
# boneinscri(boneinscri@163.com) 2026.4

# HVISOR ENTRY
HVISOR_ENTRY_PA := 0x9000000080000000

# Cross-compile tools used for ELF post-processing (objdump / readelf / nm).
# Override via env: e.g.
#   make BID=loongarch64/ls3a6000 LA_CROSS_COMPILE=loongarch64-linux-gnu-
# Default matches the more common upstream binutils target triple.
LA_CROSS_COMPILE ?= loongarch64-linux-gnu-
LA_OBJDUMP       ?= $(LA_CROSS_COMPILE)objdump
LA_READELF       ?= $(LA_CROSS_COMPILE)readelf
LA_NM            ?= $(LA_CROSS_COMPILE)nm

$(hvisor_bin): elf
	$(OBJCOPY) $(hvisor_elf) --strip-all -O binary $@
# objdump + hvisor-trap-vector.txt
	$(LA_READELF) -a $(hvisor_elf) > hvisor-elf.txt
	$(LA_OBJDUMP) --disassemble $(hvisor_elf) > hvisor.S
	cp $(hvisor_elf) hvisor.elf
	$(LA_NM) -n hvisor.elf | grep -w _hyp_trap_vector | awk '{print "0x"$$1""}' > hvisor-trap-vector.txt
