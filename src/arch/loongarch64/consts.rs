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

// PCI / HT constants
/// Turn to virtual address for HT accessing.
pub const HV_ADDR_PREFIX: u64 = LOONGARCH64_CACHED_DMW_PREFIX;
pub const LOONG_HT_PREFIX: u64 = 0xe00_0000_0000;
pub const BDF_SHIFT: usize = 8;
