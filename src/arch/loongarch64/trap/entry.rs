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

use crate::arch::cpu::this_cpu_id;
use crate::arch::loongarch64::zone::ZoneContext;
use crate::arch::register::{gcfg, gintc, gstat, gtlbc};
use crate::percpu::this_cpu_data;
use core::arch::asm;
use loongArch64::register::prmd;

#[no_mangle]
pub fn _vcpu_return(ctx: usize) {
    let z = this_cpu_data().zone.as_ref();
    let vm_id;
    if z.is_none() {
        trace!("loongarch64: _vcpu_return: no zone found for cpu {}, maybe this is a kernel exception return", this_cpu_id());
        vm_id = 0;
    } else {
        // since LVZ use GID=0 for hypervisor TLB, we cannot use zone id 0 here
        // so we add it by 1 - wheatfox
        vm_id = z.unwrap().read().id + 1;
    }
    gstat::set_gid(vm_id);
    gstat::set_pgm(true);
    trace!(
        "loongarch64: _vcpu_return: set hardware Guest ID to {} for zone {}",
        vm_id,
        z.unwrap().read().id
    );
    // Configure guest TLB control
    gtlbc::set_use_tgid(true);
    gtlbc::set_tgid(vm_id);
    let gtlbc_ = gtlbc::read();
    trace!(
        "loongarch64: _vcpu_return: gtlbc.use_tgid = {}",
        gtlbc_.use_tgid()
    );
    trace!("loongarch64: _vcpu_return: gtlbc.tgid = {}", gtlbc_.tgid());
    // Configure guest control
    gcfg::set_matc(0x1);
    let gcfg_ = gcfg::read();
    // Disable GSPR guest sensitive privileged resource exception
    gcfg::set_topi(false);
    gcfg::set_toti(false);
    gcfg::set_toe(false);
    gcfg::set_top(false);
    gcfg::set_tohu(false);
    gcfg::set_toci(0x2);

    // when booting linux, linux is waiting for a HWI, but it never really comes
    // to guest vm, in JTAG it's already in host CSR: ESTATE=0000000000000004,which is HWI0(UART...)
    // so we need to relay host HWI to guest - wheatfox 2024.4.15

    gintc::set_hwip(0xff); // HWI7-HWI0 pass to guest

    // Enable interrupt
    prmd::set_pie(true);
    trace!(
        "loongarch64: _vcpu_return: calling _hyp_trap_return with ctx = {:#x}",
        ctx
    );
    unsafe {
        _hyp_trap_return(ctx);
    }
}

#[no_mangle]
#[naked]
#[link_section = ".trap_entry"]
pub extern "C" fn _hyp_trap_vector() {
    unsafe {
        asm!(
            "csrwr $r3, {LOONGARCH_CSR_DESAVE}",
            "csrrd $r3, {LOONGARCH_CSR_SAVE3}",
            //parpare VmContext for zone_trap_handler
            //save 32 GPRS except $r3
            //save gcsrs managed by guest
            "addi.d $r3, $r3, -768",
            "st.d $r0, $r3, 0",
            "st.d $r1, $r3, 8",
            "st.d $r2, $r3, 16",
            "st.d $r4, $r3, 32",
            "st.d $r5, $r3, 40",
            "st.d $r6, $r3, 48",
            "st.d $r7, $r3, 56",
            "st.d $r8, $r3, 64",
            "st.d $r9, $r3, 72",
            "st.d $r10, $r3, 80",
            "st.d $r11, $r3, 88",
            "st.d $r12, $r3, 96",
            "st.d $r13, $r3, 104",
            "st.d $r14, $r3, 112",
            "st.d $r15, $r3, 120",
            "st.d $r16, $r3, 128",
            "st.d $r17, $r3, 136",
            "st.d $r18, $r3, 144",
            "st.d $r19, $r3, 152",
            "st.d $r20, $r3, 160",
            "st.d $r21, $r3, 168",
            "st.d $r22, $r3, 176",
            "st.d $r23, $r3, 184",
            "st.d $r24, $r3, 192",
            "st.d $r25, $r3, 200",
            "st.d $r26, $r3, 208",
            "st.d $r27, $r3, 216",
            "st.d $r28, $r3, 224",
            "st.d $r29, $r3, 232",
            "st.d $r30, $r3, 240",
            "st.d $r31, $r3, 248",
            // save ERA
            "csrrd $r12, {LOONGARCH_CSR_ERA}",
            "st.d $r12, $r3, 256",

            // save GCSRS
            "gcsrrd $r12, {LOONGARCH_GCSR_CRMD}",
            "st.d $r12, $r3, 256+8*1",
            "gcsrrd $r12, {LOONGARCH_GCSR_PRMD}",
            "st.d $r12, $r3, 256+8*2",
            "gcsrrd $r12, {LOONGARCH_GCSR_EUEN}",
            "st.d $r12, $r3, 256+8*3",
            "gcsrrd $r12, {LOONGARCH_GCSR_MISC}",
            "st.d $r12, $r3, 256+8*4",
            "gcsrrd $r12, {LOONGARCH_GCSR_ECTL}",
            "st.d $r12, $r3, 256+8*5",
            "gcsrrd $r12, {LOONGARCH_GCSR_ESTAT}",
            "st.d $r12, $r3, 256+8*6",
            "gcsrrd $r12, {LOONGARCH_GCSR_ERA}",
            "st.d $r12, $r3, 256+8*7",
            "gcsrrd $r12, {LOONGARCH_GCSR_BADV}",
            "st.d $r12, $r3, 256+8*8",
            "gcsrrd $r12, {LOONGARCH_GCSR_BADI}",
            "st.d $r12, $r3, 256+8*9",
            "gcsrrd $r12, {LOONGARCH_GCSR_EENTRY}",
            "st.d $r12, $r3, 256+8*10",
            "gcsrrd $r12, {LOONGARCH_GCSR_TLBIDX}",
            "st.d $r12, $r3, 256+8*11",
            "gcsrrd $r12, {LOONGARCH_GCSR_TLBEHI}",
            "st.d $r12, $r3, 256+8*12",
            "gcsrrd $r12, {LOONGARCH_GCSR_TLBELO0}",
            "st.d $r12, $r3, 256+8*13",
            "gcsrrd $r12, {LOONGARCH_GCSR_TLBELO1}",
            "st.d $r12, $r3, 256+8*14",
            "gcsrrd $r12, {LOONGARCH_GCSR_ASID}",
            "st.d $r12, $r3, 256+8*15",
            "gcsrrd $r12, {LOONGARCH_GCSR_PGDL}",
            "st.d $r12, $r3, 256+8*16",
            "gcsrrd $r12, {LOONGARCH_GCSR_PGDH}",
            "st.d $r12, $r3, 256+8*17",
            "gcsrrd $r12, {LOONGARCH_GCSR_PGD}",
            "st.d $r12, $r3, 256+8*18",
            "gcsrrd $r12, {LOONGARCH_GCSR_PWCL}",
            "st.d $r12, $r3, 256+8*19",
            "gcsrrd $r12, {LOONGARCH_GCSR_PWCH}",
            "st.d $r12, $r3, 256+8*20",
            "gcsrrd $r12, {LOONGARCH_GCSR_STLBPS}",
            "st.d $r12, $r3, 256+8*21",
            "gcsrrd $r12, {LOONGARCH_GCSR_RAVCFG}",
            "st.d $r12, $r3, 256+8*22",
            "gcsrrd $r12, {LOONGARCH_GCSR_CPUID}",
            "st.d $r12, $r3, 256+8*23",
            "gcsrrd $r12, {LOONGARCH_GCSR_PRCFG1}",
            "st.d $r12, $r3, 256+8*24",
            "gcsrrd $r12, {LOONGARCH_GCSR_PRCFG2}",
            "st.d $r12, $r3, 256+8*25",
            "gcsrrd $r12, {LOONGARCH_GCSR_PRCFG3}",
            "st.d $r12, $r3, 256+8*26",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE0}",
            "st.d $r12, $r3, 256+8*27",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE1}",
            "st.d $r12, $r3, 256+8*28",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE2}",
            "st.d $r12, $r3, 256+8*29",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE3}",
            "st.d $r12, $r3, 256+8*30",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE4}",
            "st.d $r12, $r3, 256+8*31",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE5}",
            "st.d $r12, $r3, 256+8*32",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE6}",
            "st.d $r12, $r3, 256+8*33",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE7}",
            "st.d $r12, $r3, 256+8*34",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE8}",
            "st.d $r12, $r3, 256+8*35",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE9}",
            "st.d $r12, $r3, 256+8*36",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE10}",
            "st.d $r12, $r3, 256+8*37",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE11}",
            "st.d $r12, $r3, 256+8*38",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE12}",
            "st.d $r12, $r3, 256+8*39",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE13}",
            "st.d $r12, $r3, 256+8*40",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE14}",
            "st.d $r12, $r3, 256+8*41",
            "gcsrrd $r12, {LOONGARCH_GCSR_SAVE15}",
            "st.d $r12, $r3, 256+8*42",
            "gcsrrd $r12, {LOONGARCH_GCSR_TID}",
            "st.d $r12, $r3, 256+8*43",
            "gcsrrd $r12, {LOONGARCH_GCSR_TCFG}",
            "st.d $r12, $r3, 256+8*44",
            "gcsrrd $r12, {LOONGARCH_GCSR_TVAL}",
            "st.d $r12, $r3, 256+8*45",
            "gcsrrd $r12, {LOONGARCH_GCSR_CNTC}",
            "st.d $r12, $r3, 256+8*46",
            "gcsrrd $r12, {LOONGARCH_GCSR_TICLR}",
            "st.d $r12, $r3, 256+8*47",
            "gcsrrd $r12, {LOONGARCH_GCSR_LLBCTL}",
            "st.d $r12, $r3, 256+8*48",
            "gcsrrd $r12, {LOONGARCH_GCSR_TLBRENTRY}",
            "st.d $r12, $r3, 256+8*49",
            "gcsrrd $r12, {LOONGARCH_GCSR_TLBRBADV}",
            "st.d $r12, $r3, 256+8*50",
            "gcsrrd $r12, {LOONGARCH_GCSR_TLBRERA}",
            "st.d $r12, $r3, 256+8*51",
            "gcsrrd $r12, {LOONGARCH_GCSR_TLBRSAVE}",
            "st.d $r12, $r3, 256+8*52",
            "gcsrrd $r12, {LOONGARCH_GCSR_TLBRELO0}",
            "st.d $r12, $r3, 256+8*53",
            "gcsrrd $r12, {LOONGARCH_GCSR_TLBRELO1}",
            "st.d $r12, $r3, 256+8*54",
            "gcsrrd $r12, {LOONGARCH_GCSR_TLBREHI}",
            "st.d $r12, $r3, 256+8*55",
            "gcsrrd $r12, {LOONGARCH_GCSR_TLBRPRMD}",
            "st.d $r12, $r3, 256+8*56",
            "gcsrrd $r12, {LOONGARCH_GCSR_DMW0}",
            "st.d $r12, $r3, 256+8*57",
            "gcsrrd $r12, {LOONGARCH_GCSR_DMW1}",
            "st.d $r12, $r3, 256+8*58",
            "gcsrrd $r12, {LOONGARCH_GCSR_DMW2}",
            "st.d $r12, $r3, 256+8*59",
            "gcsrrd $r12, {LOONGARCH_GCSR_DMW3}",
            "st.d $r12, $r3, 256+8*60",
            // // now let's save the zone's pgd to ZoneContext
            // "csrrd $r12, {LOONGARCH_CSR_PGDL}",
            // "st.d $r12, $r3, 256+8*61", // PGDL
            // "csrrd $r13, {LOONGARCH_CSR_PGDH}",
            // "st.d $r13, $r3, 256+8*62", // PGDH
            // // now let's switch KSAVE5 and KSAVE6, which should already
            // // be set to kernel's pagetable base
            // "csrwr $r12, {LOONGARCH_CSR_SAVE5}",
            // "csrwr $r13, {LOONGARCH_CSR_SAVE6}",
            // "csrwr $r12, {LOONGARCH_CSR_PGDL}",
            // "csrwr $r13, {LOONGARCH_CSR_PGDH}",
            // "invtlb 0, $r0, $r0",
            // save $r3 (previously saved in DESAVE) this is guest sp
            "csrrd $r12, {LOONGARCH_CSR_DESAVE}",
            "st.d $r12, $r3, 24",
            // $r3 -> a0, now the param of zone_trap_handler is ok
            "move $r4, $r3",
            // rewind sp to PerCpu default stack top from KSAVE4
            "csrrd $r3, {LOONGARCH_CSR_SAVE4}",
            "bl trap_handler",
            LOONGARCH_CSR_SAVE3 = const 0x33,
            LOONGARCH_CSR_SAVE4 = const 0x34,
            LOONGARCH_CSR_DESAVE = const 0x502,
            LOONGARCH_CSR_ERA = const 0x6,
            LOONGARCH_GCSR_CRMD = const 0x0,
            LOONGARCH_GCSR_PRMD = const 0x1,
            LOONGARCH_GCSR_EUEN = const 0x2,
            LOONGARCH_GCSR_MISC = const 0x3,
            LOONGARCH_GCSR_ECTL = const 0x4,
            LOONGARCH_GCSR_ESTAT = const 0x5,
            LOONGARCH_GCSR_ERA = const 0x6,
            LOONGARCH_GCSR_BADV = const 0x7,
            LOONGARCH_GCSR_BADI = const 0x8,
            LOONGARCH_GCSR_EENTRY = const 0xc,
            LOONGARCH_GCSR_TLBIDX = const 0x10,
            LOONGARCH_GCSR_TLBEHI = const 0x11,
            LOONGARCH_GCSR_TLBELO0 = const 0x12,
            LOONGARCH_GCSR_TLBELO1 = const 0x13,
            LOONGARCH_GCSR_ASID = const 0x18,
            LOONGARCH_GCSR_PGDL = const 0x19,
            LOONGARCH_GCSR_PGDH = const 0x1a,
            LOONGARCH_GCSR_PGD = const 0x1b,
            LOONGARCH_GCSR_PWCL = const 0x1c,
            LOONGARCH_GCSR_PWCH = const 0x1d,
            LOONGARCH_GCSR_STLBPS = const 0x1e,
            LOONGARCH_GCSR_RAVCFG = const 0x1f,
            LOONGARCH_GCSR_CPUID = const 0x20,
            LOONGARCH_GCSR_PRCFG1 = const 0x21,
            LOONGARCH_GCSR_PRCFG2 = const 0x22,
            LOONGARCH_GCSR_PRCFG3 = const 0x23,
            LOONGARCH_GCSR_SAVE0 = const 0x30,
            LOONGARCH_GCSR_SAVE1 = const 0x31,
            LOONGARCH_GCSR_SAVE2 = const 0x32,
            LOONGARCH_GCSR_SAVE3 = const 0x33,
            LOONGARCH_GCSR_SAVE4 = const 0x34,
            LOONGARCH_GCSR_SAVE5 = const 0x35,
            LOONGARCH_GCSR_SAVE6 = const 0x36,
            LOONGARCH_GCSR_SAVE7 = const 0x37,
            LOONGARCH_GCSR_SAVE8 = const 0x38,
            LOONGARCH_GCSR_SAVE9 = const 0x39,
            LOONGARCH_GCSR_SAVE10 = const 0x3a,
            LOONGARCH_GCSR_SAVE11 = const 0x3b,
            LOONGARCH_GCSR_SAVE12 = const 0x3c,
            LOONGARCH_GCSR_SAVE13 = const 0x3d,
            LOONGARCH_GCSR_SAVE14 = const 0x3e,
            LOONGARCH_GCSR_SAVE15 = const 0x3f,
            LOONGARCH_GCSR_TID = const 0x40,
            LOONGARCH_GCSR_TCFG = const 0x41,
            LOONGARCH_GCSR_TVAL = const 0x42,
            LOONGARCH_GCSR_CNTC = const 0x43,
            LOONGARCH_GCSR_TICLR = const 0x44,
            LOONGARCH_GCSR_LLBCTL = const 0x60,
            LOONGARCH_GCSR_TLBRENTRY = const 0x88,
            LOONGARCH_GCSR_TLBRBADV = const 0x89,
            LOONGARCH_GCSR_TLBRERA = const 0x8a,
            LOONGARCH_GCSR_TLBRSAVE = const 0x8b,
            LOONGARCH_GCSR_TLBRELO0 = const 0x8c,
            LOONGARCH_GCSR_TLBRELO1 = const 0x8d,
            LOONGARCH_GCSR_TLBREHI = const 0x8e,
            LOONGARCH_GCSR_TLBRPRMD = const 0x8f,
            LOONGARCH_GCSR_DMW0 = const 0x180,
            LOONGARCH_GCSR_DMW1 = const 0x181,
            LOONGARCH_GCSR_DMW2 = const 0x182,
            LOONGARCH_GCSR_DMW3 = const 0x183,
            // LOONGARCH_CSR_PGDL = const 0x19,
            // LOONGARCH_CSR_PGDH = const 0x1a,
            // LOONGARCH_CSR_SAVE5 = const 0x35,
            // LOONGARCH_CSR_SAVE6 = const 0x36,
            options(noreturn)
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn _hyp_trap_return(ctx: usize) {
    unsafe {
        asm!(
            // a0 -> sp
            "move  $r3, $r4",
            // restore ERA
            "ld.d $r12, $r3, 256",
            "csrwr $r12, {LOONGARCH_CSR_ERA}",
            // restore GCSRS
            "ld.d $r12, $r3, 256+8*1",
            "gcsrwr $r12, {LOONGARCH_GCSR_CRMD}",
            "ld.d $r12, $r3, 256+8*2",
            "gcsrwr $r12, {LOONGARCH_GCSR_PRMD}",
            "ld.d $r12, $r3, 256+8*3",
            "gcsrwr $r12, {LOONGARCH_GCSR_EUEN}",
            "ld.d $r12, $r3, 256+8*4",
            "gcsrwr $r12, {LOONGARCH_GCSR_MISC}",
            "ld.d $r12, $r3, 256+8*5",
            "gcsrwr $r12, {LOONGARCH_GCSR_ECTL}",
            // "ld.d $r12, $r3, 256+8*6",
            // "gcsrwr $r12, {LOONGARCH_GCSR_ESTAT}",
            "ld.d $r12, $r3, 256+8*7",
            "gcsrwr $r12, {LOONGARCH_GCSR_ERA}",
            "ld.d $r12, $r3, 256+8*8",
            "gcsrwr $r12, {LOONGARCH_GCSR_BADV}",
            "ld.d $r12, $r3, 256+8*9",
            "gcsrwr $r12, {LOONGARCH_GCSR_BADI}",
            "ld.d $r12, $r3, 256+8*10",
            "gcsrwr $r12, {LOONGARCH_GCSR_EENTRY}",
            "ld.d $r12, $r3, 256+8*11",
            "gcsrwr $r12, {LOONGARCH_GCSR_TLBIDX}",
            "ld.d $r12, $r3, 256+8*12",
            "gcsrwr $r12, {LOONGARCH_GCSR_TLBEHI}",
            "ld.d $r12, $r3, 256+8*13",
            "gcsrwr $r12, {LOONGARCH_GCSR_TLBELO0}",
            "ld.d $r12, $r3, 256+8*14",
            "gcsrwr $r12, {LOONGARCH_GCSR_TLBELO1}",
            "ld.d $r12, $r3, 256+8*15",
            "gcsrwr $r12, {LOONGARCH_GCSR_ASID}",
            "ld.d $r12, $r3, 256+8*16",
            "gcsrwr $r12, {LOONGARCH_GCSR_PGDL}",
            "ld.d $r12, $r3, 256+8*17",
            "gcsrwr $r12, {LOONGARCH_GCSR_PGDH}",
            "ld.d $r12, $r3, 256+8*18",
            "gcsrwr $r12, {LOONGARCH_GCSR_PGD}",
            "ld.d $r12, $r3, 256+8*19",
            "gcsrwr $r12, {LOONGARCH_GCSR_PWCL}",
            "ld.d $r12, $r3, 256+8*20",
            "gcsrwr $r12, {LOONGARCH_GCSR_PWCH}",
            "ld.d $r12, $r3, 256+8*21",
            "gcsrwr $r12, {LOONGARCH_GCSR_STLBPS}",
            "ld.d $r12, $r3, 256+8*22",
            "gcsrwr $r12, {LOONGARCH_GCSR_RAVCFG}",
            "ld.d $r12, $r3, 256+8*23",
            "gcsrwr $r12, {LOONGARCH_GCSR_CPUID}",
            "ld.d $r12, $r3, 256+8*24",
            "gcsrwr $r12, {LOONGARCH_GCSR_PRCFG1}",
            "ld.d $r12, $r3, 256+8*25",
            "gcsrwr $r12, {LOONGARCH_GCSR_PRCFG2}",
            "ld.d $r12, $r3, 256+8*26",
            "gcsrwr $r12, {LOONGARCH_GCSR_PRCFG3}",
            "ld.d $r12, $r3, 256+8*27",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE0}",
            "ld.d $r12, $r3, 256+8*28",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE1}",
            "ld.d $r12, $r3, 256+8*29",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE2}",
            "ld.d $r12, $r3, 256+8*30",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE3}",
            "ld.d $r12, $r3, 256+8*31",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE4}",
            "ld.d $r12, $r3, 256+8*32",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE5}",
            "ld.d $r12, $r3, 256+8*33",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE6}",
            "ld.d $r12, $r3, 256+8*34",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE7}",
            "ld.d $r12, $r3, 256+8*35",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE8}",
            "ld.d $r12, $r3, 256+8*36",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE9}",
            "ld.d $r12, $r3, 256+8*37",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE10}",
            "ld.d $r12, $r3, 256+8*38",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE11}",
            "ld.d $r12, $r3, 256+8*39",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE12}",
            "ld.d $r12, $r3, 256+8*40",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE13}",
            "ld.d $r12, $r3, 256+8*41",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE14}",
            "ld.d $r12, $r3, 256+8*42",
            "gcsrwr $r12, {LOONGARCH_GCSR_SAVE15}",
            // "ld.d $r12, $r3, 256+8*43",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TID}",
            // "ld.d $r12, $r3, 256+8*44",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TCFG}",
            // "ld.d $r12, $r3, 256+8*45",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TVAL}",
            // "ld.d $r12, $r3, 256+8*46",
            // "gcsrwr $r12, {LOONGARCH_GCSR_CNTC}",
            // "ld.d $r12, $r3, 256+8*47",
            // "gcsrwr $r12, {LOONGARCH_GCSR_TICLR}",
            "ld.d $r12, $r3, 256+8*48",
            "gcsrwr $r12, {LOONGARCH_GCSR_LLBCTL}",
            "ld.d $r12, $r3, 256+8*49",
            "gcsrwr $r12, {LOONGARCH_GCSR_TLBRENTRY}",
            "ld.d $r12, $r3, 256+8*50",
            "gcsrwr $r12, {LOONGARCH_GCSR_TLBRBADV}",
            "ld.d $r12, $r3, 256+8*51",
            "gcsrwr $r12, {LOONGARCH_GCSR_TLBRERA}",
            "ld.d $r12, $r3, 256+8*52",
            "gcsrwr $r12, {LOONGARCH_GCSR_TLBRSAVE}",
            "ld.d $r12, $r3, 256+8*53",
            "gcsrwr $r12, {LOONGARCH_GCSR_TLBRELO0}",
            "ld.d $r12, $r3, 256+8*54",
            "gcsrwr $r12, {LOONGARCH_GCSR_TLBRELO1}",
            "ld.d $r12, $r3, 256+8*55",
            "gcsrwr $r12, {LOONGARCH_GCSR_TLBREHI}",
            "ld.d $r12, $r3, 256+8*56",
            "gcsrwr $r12, {LOONGARCH_GCSR_TLBRPRMD}",
            "ld.d $r12, $r3, 256+8*57",
            "gcsrwr $r12, {LOONGARCH_GCSR_DMW0}",
            "ld.d $r12, $r3, 256+8*58",
            "gcsrwr $r12, {LOONGARCH_GCSR_DMW1}",
            "ld.d $r12, $r3, 256+8*59",
            "gcsrwr $r12, {LOONGARCH_GCSR_DMW2}",
            "ld.d $r12, $r3, 256+8*60",
            "gcsrwr $r12, {LOONGARCH_GCSR_DMW3}",
            LOONGARCH_CSR_ERA = const 0x6,
            LOONGARCH_GCSR_CRMD = const 0x0,
            LOONGARCH_GCSR_PRMD = const 0x1,
            LOONGARCH_GCSR_EUEN = const 0x2,
            LOONGARCH_GCSR_MISC = const 0x3,
            LOONGARCH_GCSR_ECTL = const 0x4,
            // LOONGARCH_GCSR_ESTAT = const 0x5,
            LOONGARCH_GCSR_ERA = const 0x6,
            LOONGARCH_GCSR_BADV = const 0x7,
            LOONGARCH_GCSR_BADI = const 0x8,
            LOONGARCH_GCSR_EENTRY = const 0xc,
            LOONGARCH_GCSR_TLBIDX = const 0x10,
            LOONGARCH_GCSR_TLBEHI = const 0x11,
            LOONGARCH_GCSR_TLBELO0 = const 0x12,
            LOONGARCH_GCSR_TLBELO1 = const 0x13,
            LOONGARCH_GCSR_ASID = const 0x18,
            LOONGARCH_GCSR_PGDL = const 0x19,
            LOONGARCH_GCSR_PGDH = const 0x1a,
            LOONGARCH_GCSR_PGD = const 0x1b,
            LOONGARCH_GCSR_PWCL = const 0x1c,
            LOONGARCH_GCSR_PWCH = const 0x1d,
            LOONGARCH_GCSR_STLBPS = const 0x1e,
            LOONGARCH_GCSR_RAVCFG = const 0x1f,
            LOONGARCH_GCSR_CPUID = const 0x20,
            LOONGARCH_GCSR_PRCFG1 = const 0x21,
            LOONGARCH_GCSR_PRCFG2 = const 0x22,
            LOONGARCH_GCSR_PRCFG3 = const 0x23,
            LOONGARCH_GCSR_SAVE0 = const 0x30,
            LOONGARCH_GCSR_SAVE1 = const 0x31,
            LOONGARCH_GCSR_SAVE2 = const 0x32,
            LOONGARCH_GCSR_SAVE3 = const 0x33,
            LOONGARCH_GCSR_SAVE4 = const 0x34,
            LOONGARCH_GCSR_SAVE5 = const 0x35,
            LOONGARCH_GCSR_SAVE6 = const 0x36,
            LOONGARCH_GCSR_SAVE7 = const 0x37,
            LOONGARCH_GCSR_SAVE8 = const 0x38,
            LOONGARCH_GCSR_SAVE9 = const 0x39,
            LOONGARCH_GCSR_SAVE10 = const 0x3a,
            LOONGARCH_GCSR_SAVE11 = const 0x3b,
            LOONGARCH_GCSR_SAVE12 = const 0x3c,
            LOONGARCH_GCSR_SAVE13 = const 0x3d,
            LOONGARCH_GCSR_SAVE14 = const 0x3e,
            LOONGARCH_GCSR_SAVE15 = const 0x3f,
            // LOONGARCH_GCSR_TID = const 0x40,
            // LOONGARCH_GCSR_TCFG = const 0x41,
            // LOONGARCH_GCSR_TVAL = const 0x42,
            // LOONGARCH_GCSR_CNTC = const 0x43,
            // LOONGARCH_GCSR_TICLR = const 0x44,
            LOONGARCH_GCSR_LLBCTL = const 0x60,
            LOONGARCH_GCSR_TLBRENTRY = const 0x88,
            LOONGARCH_GCSR_TLBRBADV = const 0x89,
            LOONGARCH_GCSR_TLBRERA = const 0x8a,
            LOONGARCH_GCSR_TLBRSAVE = const 0x8b,
            LOONGARCH_GCSR_TLBRELO0 = const 0x8c,
            LOONGARCH_GCSR_TLBRELO1 = const 0x8d,
            LOONGARCH_GCSR_TLBREHI = const 0x8e,
            LOONGARCH_GCSR_TLBRPRMD = const 0x8f,
            LOONGARCH_GCSR_DMW0 = const 0x180,
            LOONGARCH_GCSR_DMW1 = const 0x181,
            LOONGARCH_GCSR_DMW2 = const 0x182,
            LOONGARCH_GCSR_DMW3 = const 0x183,
        );
        // asm!(
        //   // vm-pagetable -> save5 and save6
        //   "ld.d $r12, $r3, 256+8*61",
        //   "csrwr $r12, {LOONGARCH_CSR_SAVE5}",
        //   "ld.d $r12, $r3, 256+8*62",
        //   "csrwr $r12, {LOONGARCH_CSR_SAVE6}",
        //   // kernel-pagetable -> r12 and r13
        //   "csrrd $r12, {LOONGARCH_CSR_PGDL}",
        //   "csrrd $r13, {LOONGARCH_CSR_PGDH}",
        //   // kernel_pagetable -> save5 and save6
        //   // old save5/save6(vm_pagetable) -> r12/r13
        //   "csrwr $r12, {LOONGARCH_CSR_SAVE5}",
        //   "csrwr $r13, {LOONGARCH_CSR_SAVE6}",
        //   // change pagetable from kernel pagetable to vm page table
        //   "csrwr $r12, {LOONGARCH_CSR_PGDL}",
        //   "csrwr $r13, {LOONGARCH_CSR_PGDH}",
        //   "invtlb 0, $r0, $r0",
        //   LOONGARCH_CSR_SAVE5 = const 0x35,
        //   LOONGARCH_CSR_SAVE6 = const 0x36,
        //   LOONGARCH_CSR_PGDL = const 0x19,
        //   LOONGARCH_CSR_PGDH = const 0x1a,
        // );
        asm!(
            // restore sp
            "ld.d $r12, $r3, 24",
            "csrwr $r12, {LOONGARCH_CSR_DESAVE}",
            // restore 32 GPRS:
            "ld.d $r0, $r3, 0",
            "ld.d $r1, $r3, 8",
            "ld.d $r2, $r3, 16",
            //ld.d $r3, $r3, 24
            "ld.d $r4, $r3, 32",
            "ld.d $r5, $r3, 40",
            "ld.d $r6, $r3, 48",
            "ld.d $r7, $r3, 56",
            "ld.d $r8, $r3, 64",
            "ld.d $r9, $r3, 72",
            "ld.d $r10, $r3, 80",
            "ld.d $r11, $r3, 88",
            "ld.d $r12, $r3, 96",
            "ld.d $r13, $r3, 104",
            "ld.d $r14, $r3, 112",
            "ld.d $r15, $r3, 120",
            "ld.d $r16, $r3, 128",
            "ld.d $r17, $r3, 136",
            "ld.d $r18, $r3, 144",
            "ld.d $r19, $r3, 152",
            "ld.d $r20, $r3, 160",
            "ld.d $r21, $r3, 168",
            "ld.d $r22, $r3, 176",
            "ld.d $r23, $r3, 184",
            "ld.d $r24, $r3, 192",
            "ld.d $r25, $r3, 200",
            "ld.d $r26, $r3, 208",
            "ld.d $r27, $r3, 216",
            "ld.d $r28, $r3, 224",
            "ld.d $r29, $r3, 232",
            "ld.d $r30, $r3, 240",
            "ld.d $r31, $r3, 248",
            "csrwr $r3, {LOONGARCH_CSR_DESAVE}",
            "ertn",
            LOONGARCH_CSR_DESAVE = const 0x502
        );
    }
}
/* TLB REFILL HANDLER */
#[no_mangle]
#[naked]
#[link_section = ".tlbrefill_entry"]
pub extern "C" fn tlb_refill_handler() {
    unsafe {
        asm!(
        "csrwr      $r12, {LOONGARCH_CSR_TLBRSAVE}",
        "csrrd      $r12, {LOONGARCH_CSR_PGD}",
        "lddir      $r12, $r12, 3",
        "ori        $r12, $r12, {PAGE_WALK_MASK}",
        "xori       $r12, $r12, {PAGE_WALK_MASK}",
        "lddir      $r12, $r12, 2",
        "ori        $r12, $r12, {PAGE_WALK_MASK}",
        "xori       $r12, $r12, {PAGE_WALK_MASK}",
        "lddir      $r12, $r12, 1",
        "ori        $r12, $r12, {PAGE_WALK_MASK}",
        "xori       $r12, $r12, {PAGE_WALK_MASK}",
        "ldpte      $r12, 0",
        "ldpte      $r12, 1",
        "tlbfill",
        "csrrd      $r12, {LOONGARCH_CSR_TLBRSAVE}",
        "ertn",
        LOONGARCH_CSR_TLBRSAVE = const 0x8b,
        LOONGARCH_CSR_PGD = const 0x1b,
        PAGE_WALK_MASK = const 0xfff,
        options(noreturn)
        );
    }
}
