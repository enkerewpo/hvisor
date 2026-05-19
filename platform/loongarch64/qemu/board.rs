// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//      Yulong Han <wheatfox17@icloud.com>
//
// QEMU LoongArch64 'virt' machine board for hvisor.
// Targets QEMU-LVZ (https://github.com/Hengyu-Yu/QEMU-LVZ) — the QEMU fork that
// implements the LoongArch virtualization extension (LVZ) in TCG.
//
// Memory map (per QEMU 'virt': hw/loongarch/virt.c + include/hw/pci-host/ls7a.h):
//   PCH-PIC      0x10000000..0x10000400      32 IRQ
//   RTC          0x100d0100..0x100d0200      ls7a-rtc
//   GED (ACPI)   0x100e0000..0x100e1000
//   PCI IO       0x18004000..0x18010000      ~48 KB
//   PCI CFG      0x20000000..0x28000000      128 MB
//   PCH-MSI      0x2ff00000..0x2ff00008      8 B
//   PCI MEM      0x40000000..0x80000000      1 GB
//   pflash0/BIOS 0x1c000000..0x1d000000      16 MB
//   pflash1      0x1d000000..0x1e000000      16 MB
//   FW_CFG       0x1e020000..0x1e020100
//   UART 16550   0x1fe001e0..0x1fe002e0      console
//   IOCSR/EXTIOI 0x1fe00000..0x1fe02000      CPU side
// DRAM:
//   lowmem  0x00000000..0x10000000           256 MB
//   highmem 0x80000000..(0x80000000 + (-m N) - 0x10000000)
//
// We assume QEMU is launched with '-m 8G' so highmem ends at 0x270000000.
// hvisor reserves itself the region 0x100000000..0x270000000 (5.75 GB).
// Root zone gets lowmem + the bottom 2 GB of highmem (0x80000000..0x100000000).

use crate::pci_dev;
use crate::{arch::zone::HvArchZoneConfig, config::*, pci::vpci_dev::VpciDevType};

pub const BOARD_NAME: &str = "qemu";

pub const BOARD_NCPUS: usize = 4;

pub const CPU_BOOT_CONTEXT_ADDRESS: usize = 0x90000001e0000000;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0xfff00000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x90200000;
pub const ROOT_ZONE_ENTRY: u64 = 0x90200000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3);

pub const ROOT_ZONE_NAME: &str = "root-linux-la64";

pub const ROOT_ZONE_MEMORY_REGIONS: &[HvConfigMemoryRegion] = &[
    // ===== Root zone DRAM (QEMU virt layout, assuming -m 8G) =====
    // EFI fix attempt 2: split lowmem. Keep 0..2MB as a tiny separate region
    // (starting at 0x1000 to avoid PA 0 which trips hvisor s2pt setup).
    // 0x1000..0x200000 covers EFI TLB refill handler at 0x1a000 etc.
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1000,
        virtual_start:  0x1000,
        size: 0x1ff000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x200000,
        virtual_start:  0x200000,
        size: 0xfe00000, // normal RAM
    },
    // hvisor occupies PA 0x200000000..0x230000000 (768 MB hardcoded reserve in
    // loongstub's hvisor reservation fix). Stage-2 must NOT map this hole —
    // root Linux doesn't see it because loongstub marked it EFI_RESERVED_MEMORY.
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x80000000,
        virtual_start:  0x80000000,
        size: 0x180000000, // 0x80000000..0x200000000 (6 GB, below hvisor)
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x230000000,
        virtual_start:  0x230000000,
        size: 0x240000000, // 0x230000000..0x470000000 (9 GB, above hvisor)
    },

    // ===== MMIO regions exposed to root =====
    // CPU-side IOCSR (UART + EXTIOI + IPI)
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x1fe00000,
        virtual_start:  0x1fe00000,
        size: 0x2000,
    },
    // PCH-PIC (IOAPIC equivalent)
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10000000,
        virtual_start:  0x10000000,
        size: 0x1000,
    },
    // ls7a-rtc
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x100d0000,
        virtual_start:  0x100d0000,
        size: 0x1000,
    },
    // ACPI GED
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x100e0000,
        virtual_start:  0x100e0000,
        size: 0x1000,
    },
    // PCI IO window
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x18000000,
        virtual_start:  0x18000000,
        size: 0x10000,
    },
    // pflash0 (BIOS) + pflash1
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x1c000000,
        virtual_start:  0x1c000000,
        size: 0x2000000, // 32 MB covers both flashes
    },
    // FW_CFG
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x1e020000,
        virtual_start:  0x1e020000,
        size: 0x1000,
    },
    // PCI CFG ECAM
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x20000000,
        virtual_start:  0x20000000,
        size: 0x8000000, // 128 MB
    },
    // PCH-MSI
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x2ff00000,
        virtual_start:  0x2ff00000,
        size: 0x1000,
    },
    // PCI MEM
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x40000000,
        virtual_start:  0x40000000,
        size: 0x40000000, // 1 GB
    },
];

pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 32 + 0x20;
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[]);
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig { dummy: 0 };
pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];

// QEMU virt PCI bus: ECAM at 0x20000000, MEM at 0x40000000, IO at 0x18004000.
// hvisor's vPCI on QEMU is currently bypassed for the root zone (per PR #304).
pub const ROOT_PCI_CONFIG: [HvPciConfig; 1] = [HvPciConfig {
    ecam_base: 0x20000000,
    ecam_size: 0x8000000,
    io_base: 0x18004000,
    io_size: 0xc000,
    pci_io_base: 0x4000,
    mem32_base: 0x40000000,
    mem32_size: 0x40000000,
    pci_mem32_base: 0x40000000,
    mem64_base: 0x0,
    mem64_size: 0x0,
    pci_mem64_base: 0x0,
    bus_range_begin: 0,
    bus_range_end: 0xff,
    domain: 0x0,
}];

// On QEMU virt the device topology is dynamic — depends on which -device flags
// are passed. Keep this minimal; PR #304 bypasses vPCI for root anyway.
pub const ROOT_PCI_DEVS: &[HvPciDevConfig] = &[
    pci_dev!(0x0, 0x0, 0x0, 0x0, VpciDevType::Physical), // host bridge
];
