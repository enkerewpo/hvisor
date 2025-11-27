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
use crate::{
    arch::loongarch64::consts::{
        LOONGARCH64_CACHED_DMW_PREFIX, LOONGARCH64_PHY_ADDR_MASK, LOONGARCH64_UNCACHED_DMW_PREFIX,
    },
    arch::s2pt::Stage2PageTable,
    error::HvResult,
    memory::MemorySet,
};

#[macro_export]
macro_rules! DMW_TO_PHY {
    ($addr:expr) => {
        ($addr as u64 & crate::arch::consts::LOONGARCH64_PHY_ADDR_MASK as u64) as usize
    };
}

#[macro_export]
macro_rules! PHY_TO_DMW_CACHED {
    ($addr:expr) => {
        ($addr as u64 | crate::arch::consts::LOONGARCH64_CACHED_DMW_PREFIX) as usize
    };
}

#[macro_export]
macro_rules! PHY_TO_DMW_UNCACHED {
    ($addr:expr) => {
        ($addr as u64 | crate::arch::consts::LOONGARCH64_UNCACHED_DMW_PREFIX) as usize
    };
}

pub fn init_hv_page_table() -> HvResult {
    todo!();
}

pub fn new_s2_memory_set() -> MemorySet<Stage2PageTable> {
    MemorySet::new(4)
}

pub fn arch_setup_parange() {
    // LoongArch64 does not have a parange setup like AArch64.
    // This function can be used to set up any architecture-specific parameters if needed.
    // Currently, it does nothing.
}

pub fn arch_post_heap_init(host_dtb: usize) {
    // LoongArch64 does not need to do some setup work after heap init like x86_64.
    // This function can be used to set up any architecture-specific parameters if needed.
    // Currently, it does nothing.
}
