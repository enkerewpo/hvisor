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

use super::context::{LdStInst, TimerState, GLOBAL_TRAP_CONTEXT_HELPER_PER_CPU};
use super::emulate::emulate_instruction;
use super::entry::_vcpu_return;
use super::hypercall::handle_hvc;
use super::init::ktime_get;
use super::interrupt::handle_interrupt;
use super::utils::{decode_ldst, ecode2str, signed_ext};
use crate::arch::consts::*;
use crate::arch::cpu::this_cpu_id;
use crate::arch::loongarch64::zone::ZoneContext;
use crate::arch::register::*;
use crate::error::HvResult;
use crate::memory::{mmio_handle_access, MMIOAccess};
use loongArch64::register;
use loongArch64::register::ecfg;
use loongArch64::register::{badi, badv, era, estat, tlbrbadv, tlbrelo0, tlbrelo1, tlbrera};

fn handle_page_modify_fault() {
    let badv_ = badv::read();
    info!(
        "loongarch64: handling page modify exception, vaddr = 0x{:x}",
        badv_.vaddr()
    );
    info!("loongarch64: ignoring this exception, todo: set dirty bit in page table entry");
}

fn read_regs() -> super::context::TrapContextHelper {
    let estat_ = estat::read();
    let badv_ = badv::read();
    let badi_ = badi::read();
    let era_ = era::read();

    super::context::TrapContextHelper {
        ecode: estat_.ecode(),
        esubcode: estat_.esubcode(),
        is: estat_.is(),
        badv: badv_.vaddr(),
        badi: badi_.inst() as usize,
        era: era_.raw(),
    }
}

fn save_timer(ctx: &ZoneContext) -> TimerState {
    let ticks = ctx.gcsr_tval;
    let cfg = ctx.gcsr_tcfg;
    let delta = if ticks < cfg { ticks } else { 0 };
    let expire = ktime_get() + delta;

    TimerState { expire, cfg }
}

fn restore_timer(ctx: &mut ZoneContext, timer: TimerState) {
    let cfg = timer.cfg;
    ctx.gcsr_tcfg = 0;
    ctx.gcsr_estat = 0;

    debug!("loongarch64: trap_handler: restore timer, cfg={:#x}", cfg);

    if cfg & 1 == 0 {
        ctx.gcsr_tval = 0;
    } else {
        let ticks = ctx.gcsr_tval;
        let estat = ctx.gcsr_estat;

        if !((cfg & 2) != 0 && (ticks > cfg)) {
            ctx.gcsr_tval = 0;
            let cpu_timer = 1usize << 11;
            if estat & cpu_timer == 0 {
                ctx.gcsr_ticlr = 1;
            }
        } else {
            let now = ktime_get();
            let mut delta = 0;
            if now < timer.expire {
                delta = timer.expire - now;
            } else if (cfg & 2) != 0 {
                let period = cfg & (0xffff_ffff_ffff_fffc);
                delta = now - timer.expire;
                delta = period - (delta % period);
            }
            ctx.gcsr_tval = delta;
        }
    }
}

fn handle_idle(ecode: usize, badi: usize, ctx: &mut ZoneContext) -> bool {
    if ecode == ECODE_GSPR && badi == 0b0000_0110_0100_1000_1000_0000_0000_0000 {
        ctx.sepc += 4;
        unsafe {
            let _ctx_ptr = ctx as *mut ZoneContext;
            _vcpu_return(_ctx_ptr as usize);
        }
        return true;
    }
    false
}

#[no_mangle]
pub fn trap_handler(mut ctx: &mut ZoneContext) {
    trace!("loongarch64: trap_handler: ctx addr = {:p}", &ctx);

    let timer = save_timer(ctx);
    let regs = read_regs();

    let _tlbrera_ = tlbrera::read();
    let _tlbrbadv_ = tlbrbadv::read();
    let _tlbrelo0_ = tlbrelo0::read();
    let _tlbrelo1_ = tlbrelo1::read();

    unsafe {
        GLOBAL_TRAP_CONTEXT_HELPER_PER_CPU[this_cpu_id()] = regs;
    }

    if handle_idle(regs.ecode, regs.badi, ctx) {
        return;
    }

    debug!(
        "loongarch64: trap_handler: {} ecode={:#x} esubcode={:#x} is={:#x} badv={:#x} badi={:#x} era={:#x}",
        ecode2str(regs.ecode, regs.esubcode),
        regs.ecode,
        regs.esubcode,
        regs.is,
        regs.badv,
        regs.badi,
        regs.era,
    );

    handle_exception(
        regs.ecode,
        regs.esubcode,
        regs.era,
        regs.is,
        regs.badi,
        regs.badv,
        ctx,
    );

    restore_timer(ctx, timer);

    debug!("loongarch64: trap_handler: return");

    unsafe {
        let _ctx_ptr = ctx as *mut ZoneContext;
        _vcpu_return(_ctx_ptr as usize);
    }
}

fn do_mmio(inst: &LdStInst, badv: usize, ctx: &mut ZoneContext) -> HvResult {
    let mut mmio = MMIOAccess {
        address: badv,
        size: inst.size,
        is_write: inst.is_write,
        value: inst.value,
    };

    debug!(
        "!!!! {} mmio@{:#x} s={:#x} v={:#x}",
        if inst.is_write { "->write" } else { "<- read" },
        mmio.address,
        mmio.size,
        mmio.value
    );

    mmio_handle_access(&mut mmio)?;

    debug!("handle mmio success, v={:#x}", mmio.value);

    if !inst.is_write {
        let mask = match mmio.size {
            1 => 0xff,
            2 => 0xffff,
            4 => 0xffffffff,
            8 => 0xffffffffffffffff,
            _ => panic!("invalid mmio size: {}", mmio.size),
        };

        let trimmed = mmio.value & mask;
        let extended = if inst.is_u {
            trimmed
        } else {
            signed_ext(trimmed, mmio.size * 8)
        };

        debug!(
            "read mmio: raw={:#x}, trimmed={:#x}, extended={:#x}",
            mmio.value, trimmed, extended
        );
        ctx.x[inst.rd] = extended;
    }

    ctx.sepc += 4;
    Ok(())
}

fn handle_page_fault_mmio(
    ecode: usize,
    esubcode: usize,
    era: usize,
    is: usize,
    badi: usize,
    badv: usize,
    ctx: &mut ZoneContext,
) {
    debug!(
        "exception: {}: ecode={:#x}, esubcode={:#x}, era={:#x}, is={:#x}, badi={:#x}, badv={:#x}",
        ecode2str(ecode, esubcode),
        ecode,
        esubcode,
        era,
        is,
        badi,
        badv
    );

    let inst = decode_ldst(badi, ctx);

    match do_mmio(&inst, badv, ctx) {
        Ok(_) => {}
        Err(e) => {
            error!(
                "mmio access failed, error = {:?}, this is a real page fault",
                e
            );
            panic!(
                "unhandled exception: {}: ecode={:#x}, esubcode={:#x}, era={:#x}, is={:#x}, badi={:#x}, badv={:#x}",
                ecode2str(ecode, esubcode),
                ecode,
                esubcode,
                era,
                is,
                badi,
                badv
            );
        }
    }
}

fn handle_exception(
    ecode: usize,
    esubcode: usize,
    era: usize,
    is: usize,
    badi: usize,
    badv: usize,
    ctx: &mut ZoneContext,
) {
    match ecode {
        ECODE_INT => {
            debug!(
                "This is an interrupt exception, is={:#x}, ecfg.lie={:?}",
                is,
                ecfg::read().lie()
            );
            handle_interrupt(is);
        }
        ECODE_GSPR => {
            // GSPR = 0x16, Guest Sensitive Privileged Resource
            trace!(
                "This is a GSPR exception, badv={:#x}, badi={:#x}",
                badv,
                badi
            );
            emulate_instruction(era, badi, ctx);
        }
        ECODE_HVC => {
            // HVC = 0x17, Hypervisor Call
            handle_hvc(ctx);
        }
        ECODE_PIL | ECODE_PIS | ECODE_PNR => {
            handle_page_fault_mmio(ecode, esubcode, era, is, badi, badv, ctx);
        }
        _ => {
            panic!(
                "unhandled exception: {}: ecode={:#x}, esubcode={:#x}, era={:#x}, is={:#x}, badi={:#x}, badv={:#x}",
                ecode2str(ecode, esubcode),
                ecode,
                esubcode,
                era,
                is,
                badi,
                badv
            );
        }
    }
}
