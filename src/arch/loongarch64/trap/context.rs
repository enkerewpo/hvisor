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

use crate::consts::MAX_CPU_NUM;

#[derive(Copy, Clone)]
pub struct TrapContextHelper {
    pub ecode: usize,
    pub esubcode: usize,
    pub is: usize,
    pub badv: usize,
    pub badi: usize,
    pub era: usize,
}

impl TrapContextHelper {
    pub const fn new() -> Self {
        Self {
            ecode: 0,
            esubcode: 0,
            is: 0,
            badv: 0,
            badi: 0,
            era: 0,
        }
    }

    pub fn update(
        &mut self,
        ecode: usize,
        esubcode: usize,
        is: usize,
        badv: usize,
        badi: usize,
        era: usize,
    ) {
        self.ecode = ecode;
        self.esubcode = esubcode;
        self.is = is;
        self.badv = badv;
        self.badi = badi;
        self.era = era;
    }
}

pub static mut GLOBAL_TRAP_CONTEXT_HELPER_PER_CPU: [TrapContextHelper; MAX_CPU_NUM] =
    [TrapContextHelper::new(); MAX_CPU_NUM];

pub struct TimerState {
    pub expire: usize,
    pub cfg: usize,
}

pub struct LdStInst {
    pub rd: usize,
    pub size: usize,
    pub is_write: bool,
    pub is_u: bool,
    pub value: usize,
}
