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
use crate::arch::trap::GLOBAL_TRAP_CONTEXT_HELPER_PER_CPU;
use crate::device::irqchip::ls7a2000::chip::get_extioi_sr;
use crate::device::irqchip::ls7a2000::consts::*;
use crate::error::{HvError, HvErrorNum::EIO, HvResult};
use crate::memory::{mmio_perform_access, MMIOAccess};
use alloc::collections::BTreeMap;
use alloc::string::String;
use core::ptr::write_volatile;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::lazy::Lazy;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MMIOAccessKey {
    offset: usize,
    size: usize,
    is_write: bool,
}

#[derive(Debug)]
struct MMIOAccessStats {
    count: AtomicU64,
    last_value: AtomicU64,
    is_compressed: AtomicU64,
}

impl MMIOAccessStats {
    const fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            last_value: AtomicU64::new(0),
            is_compressed: AtomicU64::new(0),
        }
    }
}

struct MMIOAccessTracker {
    entries: BTreeMap<MMIOAccessKey, MMIOAccessStats>,
}

impl MMIOAccessTracker {
    const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    fn find_or_insert(&mut self, key: MMIOAccessKey) -> &mut MMIOAccessStats {
        self.entries.entry(key).or_insert_with(MMIOAccessStats::new)
    }
}

static MMIO_ACCESS_STATS: Lazy<Mutex<MMIOAccessTracker>> =
    Lazy::new(|| Mutex::new(MMIOAccessTracker::new()));

const COMPRESSION_THRESHOLD: u64 = 40;
const LOG_INTERVAL: u64 = 100000;

// MMIO ranges imported from `ls7a2000::consts`.

pub fn offset(addr: usize) -> usize {
    addr - MMIO_BASE
}

macro_rules! is_in_mmio_range {
    ($addr:expr, $base:expr, $size:expr) => {
        $addr >= offset($base) && $addr < offset($base + $size)
    };
}

fn handle_uart_mmio(mmio: &mut MMIOAccess, base_addr: usize) -> HvResult {
    mmio_perform_access(base_addr, mmio);
    Ok(())
}

fn handle_extioi_status_mmio(mmio: &mut MMIOAccess, base_addr: usize, size: usize) -> HvResult {
    // first dump all 256 SR regs
    let extioi_sr = get_extioi_sr();
    debug!("extioi_sr: {}", extioi_sr);

    // write 0 to clear, so all SR regs are RW
    // since nonroot runs on cpu2(for example), but it still thinks it's on cpu0, and read SR regs from cpu0
    // we need to return the correct SR regs to nonroot according to this cpu id
    let this_cpu_id = this_cpu_id();
    let target_cpu_sr_start = EXTIOI_SR_CORE_BASE + this_cpu_id * 0x100;
    let guest_fake_cpu_id = (mmio.address - offset(EXTIOI_SR_CORE_BASE)) / 0x100;
    let guest_cpu_sr_start = EXTIOI_SR_CORE_BASE + guest_fake_cpu_id * 0x100;
    let compensation_offset = target_cpu_sr_start - guest_cpu_sr_start;
    mmio.address += compensation_offset; // since inside each cpu's SR regs region, the "inner offset" should be retained!
    mmio_perform_access(MMIO_BASE, mmio); // 1fe0_0000, do not use anything else - wheatfox
    Ok(())
}

fn handle_extioi_mapping_mmio(mmio: &mut MMIOAccess, base_addr: usize, size: usize) -> HvResult {
    if !mmio.is_write {
        warn!("should not happen, extioi core regs read makes no sense");
        return Ok(());
    }

    // if this is nonroot, we ignore the mmio
    if this_cpu_id() != 0 {
        info!("nonroot's write to extioi mapping regs, ignored");
        return Ok(());
    }

    let mut vecs = [0u8; 8];
    match mmio.size {
        1 => {
            vecs[0] = mmio.value as u8;
        }
        2 => {
            vecs[0] = (mmio.value & 0xff) as u8;
            vecs[1] = ((mmio.value >> 8) & 0xff) as u8;
        }
        4 => {
            vecs[0] = (mmio.value & 0xff) as u8;
            vecs[1] = ((mmio.value >> 8) & 0xff) as u8;
            vecs[2] = ((mmio.value >> 16) & 0xff) as u8;
            vecs[3] = ((mmio.value >> 24) & 0xff) as u8;
        }
        8 => {
            vecs[0] = (mmio.value & 0xff) as u8;
            vecs[1] = ((mmio.value >> 8) & 0xff) as u8;
            vecs[2] = ((mmio.value >> 16) & 0xff) as u8;
            vecs[3] = ((mmio.value >> 24) & 0xff) as u8;
            vecs[4] = ((mmio.value >> 32) & 0xff) as u8;
            vecs[5] = ((mmio.value >> 40) & 0xff) as u8;
            vecs[6] = ((mmio.value >> 48) & 0xff) as u8;
            vecs[7] = ((mmio.value >> 56) & 0xff) as u8;
        }
        _ => {
            error!("should not happen, wrong size: {}", mmio.size);
            return crate::hv_result_err!(EIO, "wrong size when writing extioi core regs");
        }
    }

    let base_ioi_number = mmio.address - offset(base_addr);
    for i in 0..mmio.size {
        let target_ioi_number = base_ioi_number + i; // 0 - 255
        if target_ioi_number >= size {
            error!("should not happen, wrong ioi number: {}", target_ioi_number);
            return crate::hv_result_err!(EIO, "wrong ioi number when writing extioi core regs");
        }
        let target_ioi_write_value = vecs[i];
        // caution: high 4 bits are node selection, low 4 bits are irq target cpu
        // we don't change the node selection here - wheatfox
        let target_ioi_node_selection = target_ioi_write_value & 0xf0;
        let target_ioi_irq_target = target_ioi_write_value & 0xf;
        let mut target_cpu_id = this_cpu_id();
        if target_ioi_number >= 64 {
            // msi hwirqs
            target_cpu_id = (target_ioi_number - 64) / 48;
        }
        let mut new_data = target_ioi_node_selection;
        new_data |= (1 << target_cpu_id);
        let target_write_phyaddr = base_addr + target_ioi_number as usize;
        let target_write_value = new_data as u8;
        info!(
            "loongarch64: extioi[{}], node_selection={:#x}, irq_target={:#x}, changed irq routing to cpu {}, value={:#x}",
            target_ioi_number, target_ioi_node_selection, target_ioi_irq_target, target_cpu_id, target_write_value
        );
        unsafe {
            write_volatile(target_write_phyaddr as *mut u8, target_write_value);
        }
    }
    Ok(())
}

fn handle_generic_mmio(mmio: &mut MMIOAccess, base_addr: usize) -> HvResult {
    mmio_perform_access(base_addr, mmio);
    Ok(())
}

fn handle_mmio_stats(mmio: &mut MMIOAccess) {
    let key = MMIOAccessKey {
        offset: mmio.address,
        size: mmio.size,
        is_write: mmio.is_write,
    };

    let mut tracker = MMIO_ACCESS_STATS.lock();
    let stats = tracker.find_or_insert(key);

    let count = stats.count.fetch_add(1, Ordering::SeqCst);
    let last_value = stats.last_value.load(Ordering::SeqCst);
    let is_compressed = stats.is_compressed.load(Ordering::SeqCst);

    let trap_context_helper = unsafe { &GLOBAL_TRAP_CONTEXT_HELPER_PER_CPU[this_cpu_id()] };
    let mut msg1 = format!(
        "loongarch64: generic mmio handler, zone_era={:#x}, offset={:#x}, size={}, {} {:#x}",
        trap_context_helper.era,
        mmio.address,
        mmio.size,
        if mmio.is_write { "W -> " } else { "R <- " },
        mmio.value
    );
    let mut msg2 = msg1.clone();
    msg2.push_str(" __COMPRESSED__");

    let msg = if count >= COMPRESSION_THRESHOLD {
        stats.is_compressed.store(1, Ordering::SeqCst);
        if count % LOG_INTERVAL == 0 {
            msg2.replace_range(
                msg2.find("__COMPRESSED__").unwrap()
                    ..msg2.find("__COMPRESSED__").unwrap() + "__COMPRESSED__".len(),
                &format!("(compressed, repeated {} times)", count),
            );
            msg2
        } else {
            String::new()
        }
    } else {
        msg1
    };

    if !msg.is_empty() {
        debug!("{}", msg);
    }

    stats.last_value.store(mmio.value as u64, Ordering::SeqCst);
}

pub fn loongarch_generic_mmio_handler(mmio: &mut MMIOAccess, arg: usize) -> HvResult {
    // if in uart0 region 0x1fe0_0000-0x1fe0_0008, we don't print it
    if is_in_mmio_range!(mmio.address, UART0_BASE, UART0_SIZE) {
        return handle_uart_mmio(mmio, MMIO_BASE);
    }

    let ret;

    if is_in_mmio_range!(mmio.address, EXTIOI_MAP_CORE_BASE, EXTIOI_MAP_CORE_SIZE) {
        ret = handle_extioi_mapping_mmio(mmio, EXTIOI_MAP_CORE_BASE, EXTIOI_MAP_CORE_SIZE);
    } else if is_in_mmio_range!(mmio.address, EXTIOI_SR_CORE_BASE, EXTIOI_SR_CORE_SIZE) {
        ret = handle_extioi_status_mmio(mmio, EXTIOI_SR_CORE_BASE, EXTIOI_SR_CORE_SIZE);
    } else if is_in_mmio_range!(mmio.address, EXTIOI_ENABLE_BASE, EXTIOI_ENABLE_SIZE) {
        if this_cpu_id() != 0 && mmio.is_write {
            info!("nonroot's write to extioi enable regs, ignored");
            return Ok(());
        } else {
            ret = handle_generic_mmio(mmio, MMIO_BASE);
        }
    } else if is_in_mmio_range!(mmio.address, EXTIOI_BOUNCE_BASE, EXTIOI_BOUNCE_SIZE) {
        if this_cpu_id() != 0 && mmio.is_write {
            info!("nonroot's write to extioi bounce regs, ignored");
            return Ok(());
        } else {
            ret = handle_generic_mmio(mmio, MMIO_BASE);
        }
    } else if is_in_mmio_range!(mmio.address, EXTIOI_NODE_SEL_BASE, EXTIOI_NODE_SEL_SIZE) {
        if this_cpu_id() != 0 && mmio.is_write {
            info!("nonroot's write to extioi node sel regs, ignored");
            return Ok(());
        } else {
            ret = handle_generic_mmio(mmio, MMIO_BASE);
        }
    } else {
        ret = handle_generic_mmio(mmio, MMIO_BASE);
    }

    // since our mmio can be altered inside the handler, we update the stats here
    handle_mmio_stats(mmio);

    ret
}
