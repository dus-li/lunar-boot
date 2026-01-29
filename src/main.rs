// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

#![no_std]
#![no_main]

pub mod align;
pub mod fdt;
pub mod inttypes;

use core::panic::PanicInfo;

use crate::fdt::FdtStreamable;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
pub extern "C" fn kmain() -> ! {
    fdt::Fdt::init();

    // XXX temporary, for GDB testing
    let stdout_handle = fdt::Fdt::get()
        .node_by_name("chosen")
        .unwrap()
        .prop_phandle("stdout")
        .unwrap();

    // XXX temporary, for GDB testing
    #[allow(unused_variables)]
    let stdout = fdt::Fdt::get().node_by_phandle(stdout_handle).unwrap();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
