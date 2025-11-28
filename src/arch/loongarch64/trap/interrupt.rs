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
use crate::arch::ipi::*;
use crate::consts::IPI_EVENT_CLEAR_INJECT_IRQ;
use crate::device::irqchip::ls7a2000::chip::get_extioi_sr;
use crate::event::{check_events, dump_cpu_events};
use crate::hypercall::SGI_IPI_ID;
use loongArch64::register::ecfg;
use loongArch64::register::ticlr;

/// handle loongarch64 interrupts here
pub fn handle_interrupt(is: usize) {
    // Handle IPI interrupts
    if is & IPI_BIT != 0 {
        let cpu_id = this_cpu_id();
        let ipi_status = get_ipi_status(cpu_id);
        debug!(
            "CPU {} received IPI interrupt, status = {:#x}",
            cpu_id, ipi_status
        );

        match ipi_status {
            status if status == SGI_IPI_ID as _ => {
                let events = dump_cpu_events(cpu_id);
                debug!("CPU {} events: {:?}", cpu_id, events);
                while check_events() {}
            }
            status if status == 0x8 => {
                debug!("CPU {} received unhandled IPI status {:#x}", cpu_id, status);
            }
            status => {
                warn!("CPU {} received unknown IPI status {:#x}", cpu_id, status);
            }
        }
        reset_ipi(cpu_id);
        return;
    }

    // Handle timer interrupts
    if is & TIMER_BIT != 0 {
        warn!("Timer interrupt received");
        ticlr::clear_timer_interrupt();
        crate::arch::loongarch64::irq::clear_hwi_injected_irq();
        return;
    }

    // Handle hardware interrupts (HWI)
    let hwi_mask = HWI0 | HWI1 | HWI2 | HWI3 | HWI4 | HWI5 | HWI6 | HWI7;
    if is & hwi_mask != 0 {
        let cpu_id = this_cpu_id();
        let sr = get_extioi_sr();
        warn!(
            "CPU {} received HWI interrupt, status = {:#x}, extioi status: {}",
            cpu_id, is, sr
        );
        return;
    }

    // Handle unknown interrupts
    error!("Received unhandled interrupt, status = {:#x}", is);
}
