// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

#![no_std]
#![no_main]

pub mod align;
pub mod fdt;
pub mod inttypes;
pub mod mem;

/// A module exporting build-generated section constants.
pub mod sections {
    include!(env!("BUILD_SECTIONS"));
}

use core::panic::PanicInfo;

use crate::fdt::FdtStreamable;

#[unsafe(no_mangle)]
#[unsafe(link_section = sections::start_text!())]
pub extern "C" fn kentry() -> ! {
    fdt::init();

    // XXX temporary, for GDB testing
    #[allow(unused_variables)]
    let arena = mem::start::init();

    // XXX temporary, for GDB testing
    let stdout_path = fdt::get()
        .node_by_name("chosen")
        .unwrap()
        .prop_str("stdout-path")
        .unwrap();

    // XXX temporary, for GDB testing
    let stdout = fdt::get().node_by_path(stdout_path).unwrap();

    // XXX temporary, for GDB testing
    #[allow(unused_variables)]
    let range = stdout.reg_u64();

    kmain();
}

#[inline(never)]
fn kmain() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
