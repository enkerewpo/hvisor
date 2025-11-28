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
//  ForeverYolo <2572131118@qq.com>
//  Yulong Han <wheatfox17@icloud.com>

use crate::PHY_TO_DMW_UNCACHED;

pub const DMW_DA_BITS: usize = 48;
pub const CSR_DMW0_PLV0: usize = 1 << 0;
pub const CSR_DMW0_VSEG: usize = 0x8000;
pub const CSR_DMW0_BASE: usize = CSR_DMW0_VSEG << DMW_DA_BITS;
pub const CSR_DMW0_INIT: usize = CSR_DMW0_BASE | CSR_DMW0_PLV0;

pub const CSR_DMW1_PLV0: usize = 1 << 0;
pub const CSR_DMW1_MAT: usize = 1 << 4;
pub const CSR_DMW1_VSEG: usize = 0x9000;
pub const CSR_DMW1_BASE: usize = CSR_DMW1_VSEG << DMW_DA_BITS;
pub const CSR_DMW1_INIT: usize = CSR_DMW1_BASE | CSR_DMW1_PLV0 | CSR_DMW1_MAT;

// LoongArch64 DMW prefixes and physical address mask.
pub const LOONGARCH64_CACHED_DMW_PREFIX: u64 = 0x9000_0000_0000_0000;
pub const LOONGARCH64_UNCACHED_DMW_PREFIX: u64 = 0x8000_0000_0000_0000;
pub const LOONGARCH64_PHY_ADDR_MASK: usize = 0x0000_ffff_ffff_ffff;

// Trap exception codes (ECODE values).
pub const ECODE_INT: usize = 0x0;
pub const ECODE_PIL: usize = 0x1;
pub const ECODE_PIS: usize = 0x2;
pub const ECODE_PNR: usize = 0x5;
pub const ECODE_GSPR: usize = 0x16;
pub const ECODE_HVC: usize = 0x17;

// LoongArch64 interrupt indices (CRMD.IS layout).
pub const INT_SWI0: usize = 0;
pub const INT_SWI1: usize = 1;
pub const INT_HWI0_IDX: usize = 2;
pub const INT_HWI7_IDX: usize = 9;
pub const INT_TIMER: usize = 11;
pub const INT_IPI: usize = 12;

/// Convenience aliases for first/last hardware interrupt numbers.
pub const INT_HWI0: usize = INT_HWI0_IDX;
pub const INT_HWI7: usize = INT_HWI7_IDX;

// Convenience bitmasks for `estat.is()`.
pub const IPI_BIT: usize = 1 << INT_IPI;
pub const TIMER_BIT: usize = 1 << INT_TIMER;
pub const HWI0: usize = 1 << INT_HWI0_IDX;
pub const HWI1: usize = 1 << (INT_HWI0_IDX + 1);
pub const HWI2: usize = 1 << (INT_HWI0_IDX + 2);
pub const HWI3: usize = 1 << (INT_HWI0_IDX + 3);
pub const HWI4: usize = 1 << (INT_HWI0_IDX + 4);
pub const HWI5: usize = 1 << (INT_HWI0_IDX + 5);
pub const HWI6: usize = 1 << (INT_HWI0_IDX + 6);
pub const HWI7: usize = 1 << (INT_HWI0_IDX + 7);

// Instruction opcodes for trap emulation (LoongArch64).
pub const OPCODE_CPUCFG: usize = 0b0000000000000000011011;
pub const OPCODE_CPUCFG_LENGTH: usize = 22;
pub const OPCODE_CACOP: usize = 0b0000011000;
pub const OPCODE_CACOP_LENGTH: usize = 10;
pub const OPCODE_IDLE: usize = 0b00000_11001_0010001;
pub const OPCODE_IDLE_LENGTH: usize = 17;
pub const OPCODE_CSRX: usize = 0b00000100;
pub const OPCODE_CSRX_LENGTH: usize = 8;
pub const OPCODE_IOCSR: usize = 0b00000_11001_001000000;
pub const OPCODE_IOCSR_LENGTH: usize = 19;
pub const OPCODE_LD_B: usize = 0b0010100000;
pub const OPCODE_LD_B_LENGTH: usize = 10;
pub const OPCODE_ST_B: usize = 0b0010100100;
pub const OPCODE_ST_B_LENGTH: usize = 10;
pub const OPCODE_LD_BU: usize = 0b0010101000;
pub const OPCODE_LD_BU_LENGTH: usize = 10;

// PCI / HT constants
/// Turn to virtual address for HT accessing.
pub const HV_ADDR_PREFIX: u64 = LOONGARCH64_CACHED_DMW_PREFIX;
pub const LOONG_HT_PREFIX: u64 = 0xe00_0000_0000;
pub const BDF_SHIFT: usize = 8;

// offset of all GCSR available registers
pub const GCSR_CRMD: usize = 0x0;
pub const GCSR_PRMD: usize = 0x1;
pub const GCSR_EUEN: usize = 0x2;
pub const GCSR_MISC: usize = 0x3;
pub const GCSR_ECTL: usize = 0x4;
pub const GCSR_ESTAT: usize = 0x5;
pub const GCSR_ERA: usize = 0x6;
pub const GCSR_BADV: usize = 0x7;
pub const GCSR_BADI: usize = 0x8;
pub const GCSR_EENTRY: usize = 0xc;
pub const GCSR_TLBIDX: usize = 0x10;
pub const GCSR_TLBEHI: usize = 0x11;
pub const GCSR_TLBELO0: usize = 0x12;
pub const GCSR_TLBELO1: usize = 0x13;
pub const GCSR_ASID: usize = 0x18;
pub const GCSR_PGDL: usize = 0x19;
pub const GCSR_PGDH: usize = 0x1a;
pub const GCSR_PGD: usize = 0x1b;
pub const GCSR_PWCL: usize = 0x1c;
pub const GCSR_PWCH: usize = 0x1d;
pub const GCSR_STLBPS: usize = 0x1e;
pub const GCSR_RAVCFG: usize = 0x1f;
pub const GCSR_CPUID: usize = 0x20;
pub const GCSR_PRCFG1: usize = 0x21;
pub const GCSR_PRCFG2: usize = 0x22;
pub const GCSR_PRCFG3: usize = 0x23;
pub const GCSR_SAVE0: usize = 0x30;
pub const GCSR_SAVE1: usize = 0x31;
pub const GCSR_SAVE2: usize = 0x32;
pub const GCSR_SAVE3: usize = 0x33;
pub const GCSR_SAVE4: usize = 0x34;
pub const GCSR_SAVE5: usize = 0x35;
pub const GCSR_SAVE6: usize = 0x36;
pub const GCSR_SAVE7: usize = 0x37;
pub const GCSR_SAVE8: usize = 0x38;
pub const GCSR_SAVE9: usize = 0x39;
pub const GCSR_SAVE10: usize = 0x3a;
pub const GCSR_SAVE11: usize = 0x3b;
pub const GCSR_SAVE12: usize = 0x3c;
pub const GCSR_SAVE13: usize = 0x3d;
pub const GCSR_SAVE14: usize = 0x3e;
pub const GCSR_SAVE15: usize = 0x3f;
pub const GCSR_TID: usize = 0x40;
pub const GCSR_TCFG: usize = 0x41;
pub const GCSR_TVAL: usize = 0x42;
pub const GCSR_CNTC: usize = 0x43;
pub const GCSR_TICLR: usize = 0x44;
pub const GCSR_LLBCTL: usize = 0x60;
pub const GCSR_TLBRENTRY: usize = 0x88;
pub const GCSR_TLBRBADV: usize = 0x89;
pub const GCSR_TLBRERA: usize = 0x8a;
pub const GCSR_TLBRSAVE: usize = 0x8b;
pub const GCSR_TLBRELO0: usize = 0x8c;
pub const GCSR_TLBRELO1: usize = 0x8d;
pub const GCSR_TLBREHI: usize = 0x8e;
pub const GCSR_TLBRPRMD: usize = 0x8f;
pub const GCSR_DMW0: usize = 0x180;
pub const GCSR_DMW1: usize = 0x181;
pub const GCSR_DMW2: usize = 0x182;
pub const GCSR_DMW3: usize = 0x183;

// IPI MMIO base addresses and offsets.
pub const IPI_MMIO_BASE: usize = PHY_TO_DMW_UNCACHED!(0x1fe0_0000);
pub const IPI_ANY_SEND_BASE: usize = IPI_MMIO_BASE + 0x1158;
pub const IPI_MMIO_IPI_SEND: usize = IPI_MMIO_BASE + 0x1040; // 32 bits Write Only
pub const IPI_MMIO_MAIL_SEND: usize = IPI_MMIO_BASE + 0x1048; // 64 bits Write Only

// IPI action codes.
pub const SMP_BOOT_CPU: usize = 0x1;
pub const SMP_RESCHEDULE: usize = 0x2;
pub const SMP_CALL_FUNCTION: usize = 0x4;
pub const HVISOR_START_VCPU: usize = 0x8;
