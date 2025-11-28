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

use crate::arch::loongarch64::consts::*;
use crate::arch::loongarch64::trap::context::LdStInst;
use crate::arch::loongarch64::zone::ZoneContext;
use crate::arch::register::*;

/// Translate exception code to string
pub fn ecode2str(ecode: usize, esubcode: usize) -> &'static str {
    match ecode {
        0x0 => "INT(Interrupt)",
        0x1 => "PIL(Page Illegal Load)",
        0x2 => "PIS(Page Illegal Store)",
        0x3 => "PIF(Page Illegal Fetch)",
        0x4 => "PME(Page Modify Exception)",
        0x5 => "PNR(Page Not Readable)",
        0x6 => "PNX(Page Not Executable)",
        0x7 => "PPI(Page Privilege Illegal)",
        0x8 => match esubcode {
            0x0 => "ADEF(Instruction Fetch Address Exception)",
            0x1 => "ADEM(Memory Access Address Exception)",
            _ => "error_esubcode",
        },
        0x9 => "ALE(Address Misaligned Exception)",
        0xa => "BCE(Edge Check Exception)",
        0xb => "SYS(System Call Exception)",
        0xc => "BRK(Breakpoint Exception)",
        0xd => "INE(Instruction Not Exist)",
        0xe => "IPE(Instruction Privilege Exception)",
        0xf => "FPD(Floating Point Disabled)",
        0x10 => "SXD(128-bit SIMD Disabled)",
        0x11 => "ASXD(256-bit SIMD Disabled)",
        0x12 => match esubcode {
            0x0 => "FPE(Floating Point Exception)",
            0x1 => "VFPE(Vector Floating Point Exception)",
            _ => "error_esubcode",
        },
        0x13 => match esubcode {
            0x0 => "WPEF(Watchpoint Exception Fetch)",
            0x1 => "WPEM(Watchpoint Exception Memory)",
            _ => "error_esubcode",
        },
        0x14 => "BTD(Binary Translation Disabled)",
        0x15 => "BTE(Binary Translation Exception)",
        0x16 => "GSPR(Guest Sensitive Privileged Resource)",
        0x17 => "HVC(Hypervisor Call)",
        0x18 => match esubcode {
            0x0 => "GCSC(Guest CSR Software Change)",
            0x1 => "GCHC(Guest CSR Hardware Change)",
            _ => "error_esubcode",
        },
        _ => "reserved_ecode",
    }
}

pub fn extract_field(inst: usize, offset: usize, length: usize) -> usize {
    let mask = (1 << length) - 1;
    (inst >> offset) & mask
}

/// get the sign-extended imm12 to i64
pub fn imm12toi64(imm12: usize) -> isize {
    let imm12 = imm12 as isize;
    let imm12 = imm12 << 52;
    imm12 >> 52
}

pub fn signed_ext(value: usize, size: usize) -> usize {
    let sign_bit = 1 << (size - 1);
    if value & sign_bit != 0 {
        value | !((1 << size) - 1)
    } else {
        value
    }
}

pub fn check_op_type(inst: usize, opcode: usize, opcode_length: usize) -> bool {
    let mask = (1 << opcode_length) - 1;
    let shifted = inst >> (32 - opcode_length);
    (shifted & mask) == opcode
}

/// call this before running any guests to acquire the default GCSR values
pub fn dump_reset_gcsrs() -> ZoneContext {
    let mut ctx = ZoneContext::new();
    ctx.gcsr_crmd = read_gcsr_crmd();
    ctx.gcsr_prmd = read_gcsr_prmd();
    ctx.gcsr_euen = read_gcsr_euen();
    ctx.gcsr_misc = read_gcsr_misc();
    ctx.gcsr_ectl = read_gcsr_ectl();
    ctx.gcsr_estat = read_gcsr_estat();
    ctx.gcsr_era = read_gcsr_era();
    ctx.gcsr_badv = read_gcsr_badv();
    ctx.gcsr_badi = read_gcsr_badi();
    ctx.gcsr_eentry = read_gcsr_eentry();
    ctx.gcsr_tlbidx = read_gcsr_tlbidx();
    ctx.gcsr_tlbehi = read_gcsr_tlbehi();
    ctx.gcsr_tlbelo0 = read_gcsr_tlbelo0();
    ctx.gcsr_tlbelo1 = read_gcsr_tlbelo1();
    ctx.gcsr_asid = read_gcsr_asid();
    ctx.gcsr_pgdl = read_gcsr_pgdl();
    ctx.gcsr_pgdh = read_gcsr_pgdh();
    ctx.gcsr_pgd = read_gcsr_pgd();
    ctx.gcsr_pwcl = read_gcsr_pwcl();
    ctx.gcsr_pwch = read_gcsr_pwch();
    ctx.gcsr_stlbps = read_gcsr_stlbps();
    ctx.gcsr_ravcfg = read_gcsr_ravcfg();
    ctx.gcsr_cpuid = read_gcsr_cpuid();
    ctx.gcsr_prcfg1 = read_gcsr_prcfg1();
    ctx.gcsr_prcfg2 = read_gcsr_prcfg2();
    ctx.gcsr_prcfg3 = read_gcsr_prcfg3();
    ctx.gcsr_save0 = read_gcsr_save0();
    ctx.gcsr_save1 = read_gcsr_save1();
    ctx.gcsr_save2 = read_gcsr_save2();
    ctx.gcsr_save3 = read_gcsr_save3();
    ctx.gcsr_save4 = read_gcsr_save4();
    ctx.gcsr_save5 = read_gcsr_save5();
    ctx.gcsr_save6 = read_gcsr_save6();
    ctx.gcsr_save7 = read_gcsr_save7();
    ctx.gcsr_save8 = read_gcsr_save8();
    ctx.gcsr_save9 = read_gcsr_save9();
    ctx.gcsr_save10 = read_gcsr_save10();
    ctx.gcsr_save11 = read_gcsr_save11();
    ctx.gcsr_save12 = read_gcsr_save12();
    ctx.gcsr_save13 = read_gcsr_save13();
    ctx.gcsr_save14 = read_gcsr_save14();
    ctx.gcsr_save15 = read_gcsr_save15();
    ctx.gcsr_tid = read_gcsr_tid();
    ctx.gcsr_tcfg = read_gcsr_tcfg();
    ctx.gcsr_tval = read_gcsr_tval();
    ctx.gcsr_cntc = read_gcsr_cntc();
    ctx.gcsr_ticlr = read_gcsr_ticlr();
    ctx.gcsr_llbctl = read_gcsr_llbctl();
    ctx.gcsr_tlbrentry = read_gcsr_tlbrentry();
    ctx.gcsr_tlbrbadv = read_gcsr_tlbrbadv();
    ctx.gcsr_tlbrera = read_gcsr_tlbrera();
    ctx.gcsr_tlbrsave = read_gcsr_tlbrsave();
    ctx.gcsr_tlbrelo0 = read_gcsr_tlbrrelo0();
    ctx.gcsr_tlbrelo1 = read_gcsr_tlbrrelo1();
    ctx.gcsr_tlbrehi = read_gcsr_tlbrrehi();
    ctx.gcsr_tlbrprmd = read_gcsr_tlbrprmd();
    ctx.gcsr_dmw0 = read_gcsr_dmw0();
    ctx.gcsr_dmw1 = read_gcsr_dmw1();
    ctx.gcsr_dmw2 = read_gcsr_dmw2();
    ctx.gcsr_dmw3 = read_gcsr_dmw3();

    ctx
}

pub fn decode_ldst(ins: usize, ctx: &ZoneContext) -> LdStInst {
    let prefix6 = extract_field(ins, 26, 6);
    match prefix6 {
        0b001010 => decode_ldst_std(ins, ctx),
        0b001001 => decode_ldst_ptr(ins, ctx),
        0b001110 => decode_ldst_ext(ins, ctx),
        _ => panic!("unhandled instruction: {:#b}/{:#x}", ins, ins),
    }
}

fn decode_ldst_std(ins: usize, ctx: &ZoneContext) -> LdStInst {
    let rd = extract_field(ins, 0, 5);
    let ty = extract_field(ins, 24, 2);
    let sz = extract_field(ins, 22, 2);

    let (is_write, is_u, value) = match ty {
        0b00 => (false, false, 0),
        0b01 => (true, false, ctx.x[rd]),
        0b10 => (false, true, 0),
        _ => panic!("unhandled type"),
    };

    let size = match sz {
        0b00 => 1,
        0b01 => 2,
        0b10 => 4,
        0b11 => 8,
        _ => panic!("unhandled size"),
    };

    LdStInst {
        rd,
        size,
        is_write,
        is_u,
        value,
    }
}

fn decode_ldst_ptr(ins: usize, ctx: &ZoneContext) -> LdStInst {
    let rd = extract_field(ins, 0, 5);
    let ty = extract_field(ins, 24, 2);

    let (is_write, size, value) = match ty {
        0b00 => (false, 4, 0),
        0b01 => (true, 4, ctx.x[rd]),
        0b10 => (false, 8, 0),
        0b11 => (true, 8, ctx.x[rd]),
        _ => panic!("unhandled size"),
    };

    LdStInst {
        rd,
        size,
        is_write,
        is_u: false,
        value,
    }
}

fn decode_ldst_ext(ins: usize, ctx: &ZoneContext) -> LdStInst {
    let rd = extract_field(ins, 0, 5);
    let sz = extract_field(ins, 18, 2);
    let ty = extract_field(ins, 20, 2);

    let (is_write, is_u, value) = match ty {
        0b00 => (false, false, 0),
        0b01 => (true, false, ctx.x[rd]),
        0b10 => (false, true, 0),
        _ => panic!("unhandled type"),
    };

    let size = match sz {
        0b00 => 1,
        0b01 => 2,
        0b10 => 4,
        0b11 => 8,
        _ => panic!("unhandled size"),
    };

    LdStInst {
        rd,
        size,
        is_write,
        is_u,
        value,
    }
}
