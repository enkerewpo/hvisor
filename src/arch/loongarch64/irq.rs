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

use crate::arch::consts::*;
use crate::arch::cpu::this_cpu_id;
use crate::arch::register::{read_gcsr_estat, write_gcsr_estat};
use crate::consts::MAX_CPU_NUM;
use crate::zone::Zone;
use loongArch64::register::tcfg;
use spin::Mutex;

/// inject irq to THIS cpu
pub fn inject_irq(_irq: usize, is_hardware: bool) {
    debug!(
        "loongarch64: inject_irq: _irq: {}, is_hardware: {}",
        _irq, is_hardware
    );
    if _irq > INT_IPI {
        error!("loongarch64: inject_irq: _irq > {}, not valid", INT_IPI);
        return;
    }
    let bit = 1 << _irq;
    if _irq >= INT_HWI0 && _irq <= INT_HWI7 {
        // use gintc to inject
        use crate::arch::register::gintc;
        gintc::set_hwis(bit >> INT_HWI0);
    } else {
        // use gcsr to inject, just set the bit
        let mut gcsr_estat = read_gcsr_estat();
        gcsr_estat |= bit;
        write_gcsr_estat(gcsr_estat);
    }
    let mut status = GLOBAL_IRQ_INJECT_STATUS.lock();
    status.cpu_status[this_cpu_id()].status = InjectionStatus::Injecting;

    tcfg::set_en(true); // start timer to avoid endless timer injection
                        // please only enable this for debugging because it may cause overheads for realtime nonroots
}

/// clear the injecting irq ctrl bit on THIS cpu
pub fn clear_hwi_injected_irq() {
    use crate::arch::register::gintc;
    gintc::set_hwis(0);
    // gintc::set_hwip(0);
    // gintc::set_hwic(0xff);
    let mut gintc_raw = 0usize;
    use core::arch::asm;
    unsafe {
        asm!("csrrd {0}, 0x52", out(reg) gintc_raw);
    }
    debug!(
        "loongarch64: clear_hwi_injected_irq: current gintc: {:#x}",
        gintc_raw
    );
    let mut status = GLOBAL_IRQ_INJECT_STATUS.lock();
    status.cpu_status[this_cpu_id()].status = InjectionStatus::Idle;

    tcfg::set_en(false); // stop timer
}

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        // clear all SR regs
        // Note: This calls ls7a2000-specific functions, but the interface is architecture-level
        use crate::device::irqchip::ls7a2000::chip::{clear_extioi_sr, get_extioi_sr};
        clear_extioi_sr();
        let extioi_sr = get_extioi_sr();
        info!(
            "loongarch64: irqchip: arch_irqchip_reset: extioi_sr: {}",
            extioi_sr
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InjectionStatus {
    Injecting,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PercpuInjectionStatus {
    pub status: InjectionStatus,
    pub irqs: [u32; 32],
}

#[derive(Debug)]
pub struct GlobalInjectionStatus {
    pub cpu_status: [PercpuInjectionStatus; MAX_CPU_NUM],
}

pub static GLOBAL_IRQ_INJECT_STATUS: Mutex<GlobalInjectionStatus> =
    Mutex::new(GlobalInjectionStatus {
        cpu_status: [PercpuInjectionStatus {
            status: InjectionStatus::Idle,
            irqs: [0; 32],
        }; MAX_CPU_NUM],
    });
