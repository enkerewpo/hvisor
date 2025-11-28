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
#![allow(unused)]

use crate::arch::{clock::*, cpu::this_cpu_id, ipi::*};
use chip::*;

pub mod chip;
pub mod consts;

pub fn primary_init_early() {
    if this_cpu_id() != 0 {
        info!("loongarch64: irqchip: primary_init_early: do nothing on secondary cpus");
        return;
    }
    info!("loongarch64: irqchip: primary_init_early: checking iochip configs");
    print_chip_info();
    csr_disable_new_codec();
    // legacy_int_enable_all();
    // extioi_mode_disable();
    info!("loongarch64: irqchip: testing percore IPI feature");
    let is_ipi_percore = get_ipi_percore();
    info!(
        "loongarch64: irqchip: percore IPI feature: {}",
        is_ipi_percore
    );
}
pub fn primary_init_late() {
    info!("loongarch64: irqchip: primary_init_late: running primary_init_late");

    // info!("loongarch64: irqchip: primary_init_late: testing UART1");
    // crate::device::uart::loongson_uart::__test_uart1();

    info!("loongarch64: irqchip: primary_init_late: probing pci");
    probe_pci();

    info!("loongarch64: irqchip: primary_init_late: clearing extioi SR regs");
    clear_extioi_sr();
    let extioi_sr = get_extioi_sr();
    info!(
        "loongarch64: irqchip: primary_init_late: extioi_sr: {}",
        extioi_sr
    );

    info!("loongarch64: irqchip: primary_init_late finished");
}

// actually these configures are from cpucfg, not irqchip, but we put all
// configuartion stuff here for convenience
pub fn clock_cpucfg_dump() {
    info!(
        "loongarch64: irqchip: clock_cpucfg_dump: cc_freq: {}",
        get_cpucfg_cc_freq()
    );
    info!(
        "loongarch64: irqchip: clock_cpucfg_dump: cc_mul: {}",
        get_cpucfg_cc_mul()
    );
    info!(
        "loongarch64: irqchip: clock_cpucfg_dump: cc_div: {}",
        get_cpucfg_cc_div()
    );
}

pub fn percpu_init() {
    info!("loongarch64: irqchip: percpu_init: running percpu_init");

    clear_all_ipi(this_cpu_id());
    enable_ipi(this_cpu_id());
    ecfg_ipi_enable();
    clock_cpucfg_dump();
    // timer_test_tick();
}
