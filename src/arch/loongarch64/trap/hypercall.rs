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

use crate::arch::loongarch64::zone::ZoneContext;
use crate::hypercall::*;
use crate::percpu::this_cpu_data;

/// hypercall handler
pub fn handle_hvc(ctx: &mut ZoneContext) {
    // HVC
    let code = ctx.get_a0();
    let arg0 = ctx.get_a1();
    let arg1 = ctx.get_a2();

    debug!(
        "HVC exception, HVC call code: {:#x}, arg0: {:#x}, arg1: {:#x}",
        code, arg0, arg1
    );
    let cpu_data = this_cpu_data();
    let res = match HyperCall::new(cpu_data).hypercall(code as _, arg0 as _, arg1 as _) {
        Ok(ret) => ret as _,
        Err(e) => {
            error!("HVC exception failed: {:?}", e);
            e.code()
        }
    };
    debug!("HVC result: {:#x}", res);
    ctx.set_a0(res as _);
    ctx.sepc += 4;
}
