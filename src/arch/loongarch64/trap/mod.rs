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

mod context;
mod emulate;
mod entry;
mod handler;
mod hypercall;
mod init;
mod interrupt;
mod utils;

pub use context::{TrapContextHelper, GLOBAL_TRAP_CONTEXT_HELPER_PER_CPU};
pub use entry::{_hyp_trap_return, _hyp_trap_vector, _vcpu_return, tlb_refill_handler};
pub use handler::trap_handler;
pub use init::{
    disable_global_interrupt, enable_global_interrupt, get_ms_counter, get_us_counter,
    install_trap_vector, ipi_init, ktime_get, timer_init,
};
pub use utils::dump_reset_gcsrs;
