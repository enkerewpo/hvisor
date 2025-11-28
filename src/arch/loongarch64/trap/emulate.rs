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

use super::utils::{check_op_type, extract_field, imm12toi64, signed_ext};
use crate::arch::consts::*;
use crate::arch::cpu::this_cpu_id;
use crate::arch::loongarch64::irq::inject_irq;
use crate::arch::loongarch64::zone::ZoneContext;
use crate::device::irqchip::ls7a2000::consts::UART0_BASE;
use crate::memory::{mmio_handle_access, MMIOAccess};
use core::arch::asm;

pub(super) fn emulate_cpucfg(ins: usize, ctx: &mut ZoneContext) {
    // cpucfg
    // now let get rd and rj, cpucfg rd[4:0], rj[9:5]
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    // let cpucfg_target_idx = ctx.x[rj];
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    let cpucfg_target_idx = ctx.x[rj];

    const MAX_CPUCFG_REGS: usize = 21;

    info!(
        "cpucfg emulation, target cpucfg index is {:#x}",
        cpucfg_target_idx
    );

    if cpucfg_target_idx >= MAX_CPUCFG_REGS {
        // invalid cpucfg target
        warn!("invalid cpucfg target");
        ctx.x[rd] = 0;
        // according to manual, we should set result to 0 if index is invalid
    } else {
        // just run cpucfg here
        let result: usize;
        unsafe {
            asm!("cpucfg {}, {}", out(reg) result, in(reg) cpucfg_target_idx);
        }
        ctx.x[rd] = result;
        // finish the emulation by tweaking the ZoneContext's registers
        // as ctx.sepc is already added by 4 which means we will jump to next instruction - wheatfox
    }
}

pub(super) fn emulate_csrx(ins: usize, ctx: &mut ZoneContext) {
    // csrrd csrwr csrxchg

    // let ty = (ins >> 5) & 0x1f;
    // let rd = ins & 0x1f;
    // let csr = (ins >> 10) & 0x3fff;
    let ty = extract_field(ins, 5, 5);
    let rd = extract_field(ins, 0, 5);
    let csr = extract_field(ins, 10, 14);
    // ty: [9:5], 0 - csrrd, 1 - csrwr, else - csrxchg
    // rd [4:0]
    // csr [23:10] 14 bits
    match ty {
        0 => {
            // csrrd
            info!("csrrd emulation for CSR {:#x}", csr);
            ctx.x[rd] = 0;
            // just set it to 0
        }
        1 => {
            // csrwr
            info!("csrwr emulation for CSR {:#x}", csr);
            ctx.x[rd] = 0;
            // do nothing to GCSR, but we also need to set rd to 0
        }
        _ => {
            // csrxchg
            info!("csrxchg emulation for CSR {:#x}", csr);
            ctx.x[rd] = 0;
            // do nothing to GCSR, but we also need to set rd to 0
        }
    }
}

pub(super) fn emulate_cacop(ins: usize, ctx: &mut ZoneContext) {
    // cacop code,rj,si12   0000011000 si12 rj[9:5] code[4:0]
    warn!("cacop emulation not implemented, skipped this instruction");
}

pub(super) fn emulate_idle(ins: usize, ctx: &mut ZoneContext) {
    // idle level           0000011001 0010001 level[14:0]
    let level = extract_field(ins, 0, 15);
    trace!("guest request an idle at level {:#x}", level);
}

fn ty2str(ty: usize) -> &'static str {
    match ty {
        0 => "iocsrrd.b",
        1 => "iocsrrd.h",
        2 => "iocsrrd.w",
        3 => "iocsrrd.d",
        4 => "iocsrwr.b",
        5 => "iocsrwr.h",
        6 => "iocsrwr.w",
        7 => "iocsrwr.d",
        _ => "unknown",
    }
}

pub(super) fn emulate_iocsr(ins: usize, ctx: &mut ZoneContext) {
    // iocsrrd.b rd, rj     0000011001 001000000000 rj[9:5] rd[4:0]
    // iocsrrd.h rd, rj     0000011001 001000000001 rj[9:5] rd[4:0]
    // iocsrrd.w rd, rj     0000011001 001000000010 rj[9:5] rd[4:0]
    // iocsrrd.d rd, rj     0000011001 001000000011 rj[9:5] rd[4:0]
    // iocsrwr.b rd, rj     0000011001 001000000100 rj[9:5] rd[4:0]
    // iocsrwr.h rd, rj     0000011001 001000000101 rj[9:5] rd[4:0]
    // iocsrwr.w rd, rj     0000011001 001000000110 rj[9:5] rd[4:0]
    // iocsrwr.d rd, rj     0000011001 001000000111 rj[9:5] rd[4:0]
    let ty = extract_field(ins, 10, 3);
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    debug!("iocsr emulation, ty = {}, rd = {}, rj = {}", ty, rd, rj);
    debug!("GPR[rd] = {:#x}, GPR[rj] = {:#x}", ctx.x[rd], ctx.x[rj]);

    const IOCSR_BASE_ADDR_PHY: usize = 0x1fe0_0000;
    let mut mmio_access = MMIOAccess {
        address: IOCSR_BASE_ADDR_PHY + ctx.x[rj], // iocsr only issues an offset from IOCSR_BASE_ADDR_PHY, so we need the calculate the real phy addr
        size: 0,
        is_write: false,
        value: ctx.x[rd],
    };

    match ty {
        0 => {
            // iocsrrd.b
            mmio_access.size = 1;
            mmio_access.is_write = false;
        }
        1 => {
            // iocsrrd.h
            mmio_access.size = 2;
            mmio_access.is_write = false;
        }
        2 => {
            // iocsrrd.w
            mmio_access.size = 4;
            mmio_access.is_write = false;
        }
        3 => {
            // iocsrrd.d
            mmio_access.size = 8;
            mmio_access.is_write = false;
        }
        4 => {
            // iocsrwr.b
            mmio_access.size = 1;
            mmio_access.is_write = true;
        }
        5 => {
            // iocsrwr.h
            mmio_access.size = 2;
            mmio_access.is_write = true;
        }
        6 => {
            // iocsrwr.w
            mmio_access.size = 4;
            mmio_access.is_write = true;
        }
        7 => {
            // iocsrwr.d
            mmio_access.size = 8;
            mmio_access.is_write = true;
        }
        _ => {
            // should not reach here
            panic!("invalid iocsr type, this is impossible");
        }
    }

    debug!(
        "iocsr issues a mmio access: {}, target address: {:#x}, size: {}, {}",
        ty2str(ty),
        mmio_access.address,
        mmio_access.size,
        if mmio_access.is_write { "W" } else { "R" }
    );

    let res = mmio_handle_access(&mut mmio_access);
    match res {
        Ok(_) => {
            debug!("handle mmio success, v={:#x}", mmio_access.value);
            if !mmio_access.is_write {
                let mask = match mmio_access.size {
                    1 => 0xff,
                    2 => 0xffff,
                    4 => 0xffffffff,
                    8 => 0xffffffffffffffff,
                    _ => panic!("invalid mmio access size: {}", mmio_access.size),
                };
                let trimmed_by_size = mmio_access.value & mask;
                let extended = if ty < 4 {
                    signed_ext(trimmed_by_size, mmio_access.size * 8)
                } else {
                    trimmed_by_size
                };
                ctx.x[rd] = extended;
            }
        }
        Err(e) => {
            panic!(
                "mmio access failed, error = {:?}, this is a real page fault",
                e
            );
        }
    }
}

pub(super) fn emulate_iocsr_legacy(ins: usize, ctx: &mut ZoneContext) {
    // iocsrrd.b rd, rj     0000011001 001000000000 rj[9:5] rd[4:0]
    // iocsrrd.h rd, rj     0000011001 001000000001 rj[9:5] rd[4:0]
    // iocsrrd.w rd, rj     0000011001 001000000010 rj[9:5] rd[4:0]
    // iocsrrd.d rd, rj     0000011001 001000000011 rj[9:5] rd[4:0]
    // iocsrwr.b rd, rj     0000011001 001000000100 rj[9:5] rd[4:0]
    // iocsrwr.h rd, rj     0000011001 001000000101 rj[9:5] rd[4:0]
    // iocsrwr.w rd, rj     0000011001 001000000110 rj[9:5] rd[4:0]
    // iocsrwr.d rd, rj     0000011001 001000000111 rj[9:5] rd[4:0]
    // let ty = (ins >> 10) & 0x7;
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    let ty = extract_field(ins, 10, 3);
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    debug!("iocsr emulation, ty = {}, rd = {}, rj = {}", ty, rd, rj);
    debug!("GPR[rd] = {:#x}, GPR[rj] = {:#x}", ctx.x[rd], ctx.x[rj]);
    match ty {
        0 => {
            // iocsrrd.b
            // GPR[rd] = iocsrrd.b(GPR[rj])
            let mut val = 0;
            unsafe {
                asm!("iocsrrd.b {}, {}", out(reg) val, in(reg) ctx.x[rj]);
            }
            ctx.x[rd] = val & 0xff;
        }
        1 => {
            // iocsrrd.h
            // GPR[rd] = iocsrrd.h(GPR[rj])
            let mut val = 0;
            unsafe {
                asm!("iocsrrd.h {}, {}", out(reg) val, in(reg) ctx.x[rj]);
            }
            ctx.x[rd] = val & 0xffff;
        }
        2 => {
            // iocsrrd.w
            // GPR[rd] = iocsrrd.w(GPR[rj])
            let mut val = 0;
            unsafe {
                asm!("iocsrrd.w {}, {}", out(reg) val, in(reg) ctx.x[rj]);
            }
            ctx.x[rd] = val & 0xffffffff;
        }
        3 => {
            // iocsrrd.d
            // GPR[rd] = iocsrrd.d(GPR[rj])
            let mut val = 0;
            unsafe {
                asm!("iocsrrd.d {}, {}", out(reg) val, in(reg) ctx.x[rj]);
            }
            ctx.x[rd] = val;
        }
        4 => {
            // iocsrwr.b
            // iocsrwr.b(GPR[rd], GPR[rj])
            unsafe {
                asm!("iocsrwr.b {}, {}", in(reg) ctx.x[rd], in(reg) ctx.x[rj]);
            }
        }
        5 => {
            // iocsrwr.h
            // iocsrwr.h(GPR[rd], GPR[rj])
            unsafe {
                asm!("iocsrwr.h {}, {}", in(reg) ctx.x[rd], in(reg) ctx.x[rj]);
            }
        }
        6 => {
            // iocsrwr.w
            // iocsrwr.w(GPR[rd], GPR[rj])

            // hack: since guest linux will use iocsrwr.w [xxx] 0x1014 to send IPI to itself (ACTION_IRQ_WORK)
            // we need to check if ctx.x[rj] is 0x1014, if so, we should parse ctx.x[rd][31:0]
            // then inject IPI bit to GCSR_ESTAT, and prepare the 7A2000's IPI register to have the exact target status
            // using the debug ANY_SEND register (TODO)

            let target_io = ctx.x[rj];
            let target_write_data = ctx.x[rd];

            match target_io {
                0x1014 => {
                    // IPI send issued from guest is tricky ...
                    // IPI_send is 32 bit, we ignore the upper 32 bits
                    // bit [31]: wait for completion
                    // bit [25:16] target cpu id
                    // bit [4:0] ipi id (IPI_status, 32 bit) indicates the IPI type (0-31)
                    let ipi_send = target_write_data as u32;
                    let ipi_id = ipi_send & 0x1f;
                    let target_cpu_id = (ipi_send >> 16) & 0x3ff;
                    let wait_for_completion = (ipi_send >> 31) & 0x1;
                    warn!("IPI send issued from guest, ipi_id = {:#x}, target_cpu_id = {:#x}, wait_for_completion = {:#x}", ipi_id, target_cpu_id, wait_for_completion);
                    if target_cpu_id == this_cpu_id() as u32 {
                        warn!("send IPI to itself, injecting IPI to GCSR_ESTAT");
                        inject_irq(INT_IPI, false);
                    } else {
                        // TODO
                        panic!("send IPI from guest to other cpu is not supported yet!");
                    }
                }
                _ => unsafe {
                    asm!("iocsrwr.w {}, {}", in(reg) ctx.x[rd], in(reg) ctx.x[rj]);
                },
            }
        }
        7 => {
            // iocsrwr.d
            // iocsrwr.d(GPR[rd], GPR[rj])
            unsafe {
                asm!("iocsrwr.d {}, {}", in(reg) ctx.x[rd], in(reg) ctx.x[rj]);
            }
        }
        _ => {
            // should not reach here
            panic!("invalid iocsr type, this is impossible");
        }
    }
}

pub(super) fn emulate_ld_b(ins: usize, ctx: &mut ZoneContext) {
    // ld.b   rd, rj, si12  opcode[31:22]=0010100000 si12[21:10] rj[9:5] rd[4:0]
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    // let si12 = (ins >> 10) & 0x3ff; ??? should be 0xfff
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    let si12 = extract_field(ins, 10, 12);

    info!("ld.b emulation, rd = {}, rj = {}, si12 = {}", rd, rj, si12);
    // vaddr = GR[rj] + SignExt(si12, GRLEN(64))
    // paddr = translate(vaddr)
    // byte = load (paddr, BYTE)
    // GR[rd] = byte
    let vaddr = ctx.x[rj] as isize + imm12toi64(si12);
    info!("vaddr = 0x{:x}", vaddr as usize);
    let offset = (vaddr - UART0_BASE as isize) as usize; // minus the UART0 base address
}

pub(super) fn emulate_st_b(ins: usize, ctx: &mut ZoneContext) {
    // st.b   rd, rj, si12  opcode[31:22]=0010100100 si12[21:10] rj[9:5] rd[4:0]
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    // let si12 = (ins >> 10) & 0x3ff;
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    let si12 = extract_field(ins, 10, 12);
    // info!("st.b emulation, rd = {}, rj = {}, si12 = {}", rd, rj, si12);
    // vaddr = GR[rj] + SignExt(si12, GRLEN(64))
    // paddr = translate(vaddr)
    // store (paddr, BYTE, GR[rd])
    let vaddr = ctx.x[rj] as isize + imm12toi64(si12);
    // info!("vaddr = 0x{:x}", vaddr as usize);
    let offset = (vaddr - UART0_BASE as isize) as usize; // minus the UART0 base address
}

pub(super) fn emulate_ld_bu(ins: usize, ctx: &mut ZoneContext) {
    // ld.bu  rd, rj, si12  opcode[31:22]=0010101000 si12[21:10] rj[9:5] rd[4:0]
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    // let si12 = (ins >> 10) & 0x3ff;
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    let si12 = extract_field(ins, 10, 12);
    // info!("ld.bu emulation, rd = {}, rj = {}, si12 = {}", rd, rj, si12);
    // vaddr = GR[rj] + SignExt(si12, GRLEN(64))
    // paddr = translate(vaddr)
    // byte = load (paddr, BYTE)
    // GR[rd] = byte
    let vaddr = ctx.x[rj] as isize + imm12toi64(si12);
    let offset = (vaddr - UART0_BASE as isize) as usize; // minus the UART0 base address
}

type OpcodeHandler = fn(usize, &mut ZoneContext);

pub(super) fn emulate_instruction(era: usize, ins: usize, ctx: &mut ZoneContext) {
    let pc = era;
    // after we emulate the instruction, we should jump to next instruction
    ctx.sepc = pc + 4;

    let opcodes = vec![
        (
            OPCODE_CPUCFG,
            OPCODE_CPUCFG_LENGTH,
            emulate_cpucfg as OpcodeHandler,
        ),
        (
            OPCODE_CACOP,
            OPCODE_CACOP_LENGTH,
            emulate_cacop as OpcodeHandler,
        ),
        (
            OPCODE_IDLE,
            OPCODE_IDLE_LENGTH,
            emulate_idle as OpcodeHandler,
        ),
        (
            OPCODE_CSRX,
            OPCODE_CSRX_LENGTH,
            emulate_csrx as OpcodeHandler,
        ),
        (
            OPCODE_IOCSR,
            OPCODE_IOCSR_LENGTH,
            emulate_iocsr as OpcodeHandler,
        ),
        (
            OPCODE_LD_B,
            OPCODE_LD_B_LENGTH,
            emulate_ld_b as OpcodeHandler,
        ),
        (
            OPCODE_ST_B,
            OPCODE_ST_B_LENGTH,
            emulate_st_b as OpcodeHandler,
        ),
        (
            OPCODE_LD_BU,
            OPCODE_LD_BU_LENGTH,
            emulate_ld_bu as OpcodeHandler,
        ),
    ];
    for &(code, length, handler) in &opcodes {
        if check_op_type(ins, code, length) {
            handler(ins, ctx);
            return;
        }
    }

    panic!("unexpected opcode encountered, ins = {:#x}", ins);
}
