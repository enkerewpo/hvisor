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

use super::context::GLOBAL_TRAP_CONTEXT_HELPER_PER_CPU;
use super::emulate::emulate_instruction;
use super::entry::_vcpu_return;
use super::hypercall::handle_hvc;
use super::init::ktime_get;
use super::interrupt::handle_interrupt;
use super::utils::{ecode2str, extract_field, signed_ext};
use crate::arch::consts::*;
use crate::arch::cpu::this_cpu_id;
use crate::arch::loongarch64::zone::ZoneContext;
use crate::arch::register::*;
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

#[no_mangle]
pub fn trap_handler(mut ctx: &mut ZoneContext) {
    trace!("loongarch64: trap_handler: ctx addr = {:p}", &ctx);

    // save timer
    let delta;
    let ticks = ctx.gcsr_tval;
    let cfg = ctx.gcsr_tcfg;
    if ticks < cfg {
        delta = ticks;
    } else {
        delta = 0;
    }
    let expire = ktime_get() + delta;

    // dump trap csr regs
    let estat_ = estat::read();
    let ecode = estat_.ecode();
    let esubcode = estat_.esubcode();
    let is = estat_.is();
    let badv_ = badv::read();
    let badi_ = badi::read();
    let era_ = era::read();
    // TLB dump
    let tlbrera_ = tlbrera::read();
    let tlbrbadv_ = tlbrbadv::read();
    let tlbrelo0_ = tlbrelo0::read();
    let tlbrelo1_ = tlbrelo1::read();

    // update global trap context helper
    unsafe {
        GLOBAL_TRAP_CONTEXT_HELPER_PER_CPU[this_cpu_id()].update(
            ecode,
            esubcode,
            is,
            badv_.vaddr(),
            badi_.inst() as usize,
            era_.raw(),
        );
    }

    let mut is_idle = false;
    if ecode == ECODE_GSPR && badi_.inst() == 0b0000_0110_0100_1000_1000_0000_0000_0000 {
        is_idle = true;
        ctx.sepc += 4;
        // just return to guest
        unsafe {
            let _ctx_ptr = ctx as *mut ZoneContext;
            _vcpu_return(_ctx_ptr as usize);
        }
    }

    debug!(
            "loongarch64: trap_handler: {} ecode={:#x} esubcode={:#x} is={:#x} badv={:#x} badi={:#x} era={:#x}", 
            ecode2str(ecode, esubcode),
            ecode,
            esubcode,
            is,
            badv_.vaddr(),
            badi_.inst(),
            era_.raw(),
        );

    handle_exception(
        ecode,
        esubcode,
        era_.raw(),
        is,
        badi_.inst() as usize,
        badv_.vaddr(),
        ctx,
    );

    // restore timer
    let cfg = ctx.gcsr_tcfg;

    ctx.gcsr_tcfg = 0;

    // restore GCSR_ESTAT and GCSR_TCFG
    ctx.gcsr_estat = 0;
    ctx.gcsr_tcfg = 0;

    debug!("loongarch64: trap_handler: restore timer, cfg={:#x}", cfg);

    if cfg & 1 == 0 {
        // guest has disabled timer, we just restore the tval
        ctx.gcsr_tval = 0;
    } else {
        let ticks = ctx.gcsr_tval;
        let estat = ctx.gcsr_estat;

        if !((cfg & 2) != 0 && (ticks > cfg)) {
            ctx.gcsr_tval = 0; // inject irq
            let cpu_timer = 1usize << 11;
            if estat & cpu_timer == 0 {
                ctx.gcsr_ticlr = 1; // clear timer interrupt
            }
        } else {
            let now = ktime_get();
            let mut __delta = 0;
            if now < expire {
                __delta = expire - now;
            } else if (cfg & 2) != 0 {
                // tcfg[63:2] || 00 is tval
                let period = cfg & (0xffff_ffff_ffff_fffc);
                __delta = now - expire;
                __delta = period - (__delta % period);
                // kvm queued guest timer irq injection here but we do nothing here
            }

            ctx.gcsr_tval = __delta;
        }
    }

    debug!("loongarch64: trap_handler: return");

    unsafe {
        let _ctx_ptr = ctx as *mut ZoneContext;
        _vcpu_return(_ctx_ptr as usize);
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
            // INT = 0x0,   Interrupt
            handle_interrupt(is);
        }
        ECODE_GSPR => {
            // according to kvm's code, we should emulate the instruction that cause the GSPR exception - wheatfox 2024.4.12
            // GSPR = 0x16, Guest Sensitive Privileged Resource
            trace!(
                "This is a GSPR exception, badv={:#x}, badi={:#x}",
                badv,
                badi
            );
            // arch_send_event(1, 0x7);
            emulate_instruction(era, badi, ctx);
        }
        ECODE_HVC => {
            // HVC = 0x17,  Hypervisor Call
            // code = a0(r4), arg0 = a1(r5), arg1 = a2(r6)
            handle_hvc(ctx);
        }
        ECODE_PIL | ECODE_PIS | ECODE_PNR => {
            debug!("exception: {}: ecode={:#x}, esubcode={:#x}, era={:#x}, is={:#x}, badi={:#x}, badv={:#x}",
                    ecode2str(ecode,esubcode), ecode, esubcode, era, is, badi, badv);
            // we first assume this lies in virtio region
            // since we didn't add these regions into VMM Pages
            /*
                LD.B    rd, rj, si12    0010100000  si12    rj5   rd5
                LD.H    rd, rj, si12    0010100001  si12    rj5   rd5
                LD.W    rd, rj, si12    0010100010  si12    rj5   rd5
                LD.D    rd, rj, si12    0010100011  si12    rj5   rd5
                ST.B    rd, rj, si12    0010100100  si12    rj5   rd5
                ST.H    rd, rj, si12    0010100101  si12    rj5   rd5
                ST.W    rd, rj, si12    0010100110  si12    rj5   rd5
                ST.D    rd, rj, si12    0010100111  si12    rj5   rd5
                LD.BU   rd, rj, si12    0010101000  si12    rj5   rd5
                LD.HU   rd, rj, si12    0010101001  si12    rj5   rd5
                LD.WU   rd, rj, si12    0010101010  si12    rj5   rd5
                LDPTR.W rd, rj, si14    00100100    si14    rj5   rd5
                STPTR.W rd, rj, si14    00100101    si14    rj5   rd5
                LDPTR.D rd, rj, si14    00100110    si14    rj5   rd5
                STPTR.D rd, rj, si14    00100111    si14    rj5   rd5
                LDX.B   rd, rj, rk      00111000000000 000 rk rj  rd5
                LDX.H   rd, rj, rk      00111000000001 000 rk rj  rd5
                LDX.W   rd, rj, rk      00111000000010 000 rk rj  rd5
                LDX.D   rd, rj, rk      00111000000011 000 rk rj  rd5
                STX.B   rd, rj, rk      00111000000100 000 rk rj  rd5
                STX.H   rd, rj, rk      00111000000101 000 rk rj  rd5
                STX.W   rd, rj, rk      00111000000110 000 rk rj  rd5
                STX.D   rd, rj, rk      00111000000111 000 rk rj  rd5
                LDX.BU  rd, rj, rk      00111000001000 000 rk rj  rd5
                LDX.HU  rd, rj, rk      00111000001001 000 rk rj  rd5
                LDX.WU  rd, rj, rk      00111000001010 000 rk rj  rd5
            */
            let ins = badi;
            let mut is_write = false;
            let mut is_u = false;
            let mut value = 0;
            let mut size = 0;
            let mut addr = 0;
            let mut target_rd_idx = 0;
            let prefix6 = extract_field(ins, 26, 6);
            if prefix6 == 0b001010 {
                // load/store
                let rd = extract_field(ins, 0, 5);
                target_rd_idx = rd;
                let rj = extract_field(ins, 5, 5);
                let si12 = extract_field(ins, 10, 12);
                let ty = extract_field(ins, 24, 2); // ld/st/ldu - 0b00/0b01/0b10
                let sz = extract_field(ins, 22, 2); // 0b00=byte, 0b01=half, 0b10=word, 0b11=double
                match ty {
                    0b00 => {
                        // LD
                        is_write = false;
                    }
                    0b01 => {
                        // ST
                        is_write = true;
                        value = ctx.x[rd];
                    }
                    0b10 => {
                        // LDU
                        is_write = false;
                        is_u = true;
                    }
                    _ => panic!("unhandled type"),
                }
                size = match sz {
                    0b00 => 1,
                    0b01 => 2,
                    0b10 => 4,
                    0b11 => 8,
                    _ => panic!("unhandled size"),
                };
            } else if prefix6 == 0b001001 {
                // load/store pointer
                let rd = extract_field(ins, 0, 5);
                target_rd_idx = rd;
                let rj = extract_field(ins, 5, 5);
                let si14 = extract_field(ins, 10, 14);
                let mem_addr = ctx.x[rj] as usize + si14 as usize;
                let ty = extract_field(ins, 24, 2);
                match ty {
                    0b00 => {
                        // LDPTR.W
                        is_write = false;
                        size = 4;
                    }
                    0b01 => {
                        // STPTR.W
                        is_write = true;
                        size = 4;
                        value = ctx.x[rd];
                    }
                    0b10 => {
                        // LDPTR.D
                        is_write = false;
                        size = 8;
                    }
                    0b11 => {
                        // STPTR.D
                        is_write = true;
                        size = 8;
                        value = ctx.x[rd];
                    }
                    _ => panic!("unhandled size"),
                }
            } else if prefix6 == 0b001110 {
                // load/store extended
                let rd = extract_field(ins, 0, 5);
                target_rd_idx = rd;
                let rj = extract_field(ins, 5, 5);
                let rk = extract_field(ins, 10, 5);
                let sz = extract_field(ins, 18, 2);
                let ty = extract_field(ins, 20, 2);
                match ty {
                    0b00 => {
                        // LDX
                        is_write = false;
                    }
                    0b01 => {
                        // STX
                        is_write = true;
                        value = ctx.x[rd];
                    }
                    0b10 => {
                        // LDXU
                        is_write = false;
                        is_u = true;
                    }
                    _ => panic!("unhandled type"),
                }
                size = match sz {
                    0b00 => 1,
                    0b01 => 2,
                    0b10 => 4,
                    0b11 => 8,
                    _ => panic!("unhandled size"),
                };
            } else {
                panic!("unhandled instruction: {:#b}/{:#x}", ins, ins);
            }

            let mut mmio_access = MMIOAccess {
                address: badv,
                size,
                is_write,
                value,
            };
            debug!(
                "!!!! {} mmio_access@{:#x} s={:#x} v={:#x}",
                if is_write { "->write" } else { "<- read" },
                mmio_access.address,
                mmio_access.size,
                mmio_access.value
            );
            let res = mmio_handle_access(&mut mmio_access);
            match res {
                Ok(_) => {
                    debug!("handle mmio success, v={:#x}", mmio_access.value);
                    if !is_write {
                        // we read an usize from our zone0 virtio-daemon
                        // need to trim and extend it to 64-bit reg according to is_u and size
                        let mask = match mmio_access.size {
                            1 => 0xff,
                            2 => 0xffff,
                            4 => 0xffffffff,
                            8 => 0xffffffffffffffff,
                            _ => panic!("invalid mmio access size: {}", mmio_access.size),
                        };
                        let trimmed_by_size = mmio_access.value & mask;
                        let extended = if !is_u {
                            // normal instruction with no .u use sign extension
                            signed_ext(trimmed_by_size, mmio_access.size * 8)
                        } else {
                            // .u instruction zero extend
                            trimmed_by_size
                        };
                        debug!(
                            "read from mmio, raw={:#x}, trimmed={:#x}, extended={:#x}",
                            mmio_access.value, trimmed_by_size, extended
                        );
                        ctx.x[target_rd_idx] = extended;
                    }
                    // we should jump to next instruction because we 'emulated' the instruction
                    ctx.sepc += 4;
                }
                Err(e) => {
                    error!(
                        "mmio access failed, error = {:?}, this is a real page fault",
                        e
                    );
                    panic!("unhandled exception: {}: ecode={:#x}, esubcode={:#x}, era={:#x}, is={:#x}, badi={:#x}, badv={:#x}",
                    ecode2str(ecode,esubcode), ecode, esubcode, era, is, badi, badv)
                }
            }
        }
        _ => {
            panic!("unhandled exception: {}: ecode={:#x}, esubcode={:#x}, era={:#x}, is={:#x}, badi={:#x}, badv={:#x}",  
            ecode2str(ecode,esubcode), ecode, esubcode, era, is, badi, badv)
        }
    }
}
