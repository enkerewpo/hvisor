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

use crate::arch::loongarch64::trap::entry::_hyp_trap_vector;
use crate::arch::register::*;
use core::arch::asm;
use loongArch64::register::ecfg::LineBasedInterrupt;
use loongArch64::register::*;
use loongArch64::time;

pub fn install_trap_vector() {
    // force disable INT here
    crmd::set_ie(false);
    // clear UEFI firmware's previous timer configs
    ticlr::clear_timer_interrupt();

    timer_init();
    tcfg::set_en(false); // we may need to use timer irq to trap for our virtio clear injection
                         // only enable timer irq trap for debugging, because it may cause overheads for realtime nonroots

    // set CSR.EENTRY to _hyp_trap_vector and int vector offset to 0
    ecfg::set_vs(0);
    eentry::set_eentry(_hyp_trap_vector as usize);

    // enable floating point
    euen::set_fpe(true); // basic floating point
    euen::set_sxe(true); // 128-bit SIMD
    euen::set_asxe(true); // 256-bit SIMD

    enable_global_interrupt();

    // we also init uart here because we need it at very early stage
    crate::device::uart::loongson_uart::init_uart();
}

/// enable CRMD.IE
#[inline(always)]
pub fn enable_global_interrupt() {
    crmd::set_ie(true);
}

/// disable CRMD.IE
#[inline(always)]
pub fn disable_global_interrupt() {
    crmd::set_ie(false);
}

#[inline(always)]
pub fn get_ms_counter(ms: usize) -> usize {
    ms * (time::get_timer_freq() / 1000)
}

#[inline(always)]
pub fn get_us_counter(us: usize) -> usize {
    us * (time::get_timer_freq() / 1000_000)
}

/// read the current stable counter value, not ns!
#[inline(always)]
pub fn ktime_get() -> usize {
    let mut current_counter_time;
    unsafe {
        asm!(
            "rdtime.d {}, {}",
            out(reg) current_counter_time,
            in(reg) 0,
        );
    }
    current_counter_time
}

pub fn timer_init() {
    // uefi firmware leaves timer interrupt pending, we need to clear it manually
    ticlr::clear_timer_interrupt();
    let timer_freq = time::get_timer_freq();
    tcfg::set_periodic(true);
    let init_val = get_ms_counter(200);
    tcfg::set_init_val(init_val);

    tcfg::set_en(true);

    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ | LineBasedInterrupt::TIMER;
    ecfg::set_lie(lie_);
}

pub fn ipi_init() {
    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ | LineBasedInterrupt::IPI;
    ecfg::set_lie(lie_);
}
