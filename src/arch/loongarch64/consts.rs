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
