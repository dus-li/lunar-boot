// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

use core::cell::UnsafeCell;

use crate::inttypes::{BEu32, BEu64};

/// FDT header magic number, as mandated by the devicetree specification.
const FDT_MAGIC: u32 = 0xD00DFEED;

unsafe extern "C" {
    // See: dtb.S
    static fdt_blob: [u8; 0];
}

/// A view into the main system FDT.
///
/// We can use [`UnsafeCell`], because sole mutable access takes place during
/// the early initialization of the system. Since we need the devicetree to
/// learn about remaining CPUs in order to wake them up, we are guaranteed that
/// there will be no competition at that time. As such, a spinlock would likely
/// be an overkill.
static SYSTEM_FDT: FdtViewCell = FdtViewCell(UnsafeCell::new(None));

/// Memory reservation entry in a devicetree.
#[repr(C)]
struct FdtReserveEntry {
    address: BEu64,
    size: BEu64,
}

/// FDT header, as defined in the devicetree specification.
#[repr(C)]
struct FdtHeader {
    magic: BEu32,
    totalsize: BEu32,
    off_dt_struct: BEu32,
    off_dt_strings: BEu32,
    off_mem_rsvmap: BEu32,
    version: BEu32,
    last_comp_version: BEu32,
    boot_cpuid_phys: BEu32,
    size_dt_strings: BEu32,
    size_dt_struct: BEu32,
}

impl FdtHeader {
    /// Given a slice with the FDT, compute its subslice containing DT struct.
    fn dt_struct<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        let off = self.off_dt_struct.get() as usize;
        let size = self.size_dt_struct.get() as usize;

        &data[off..off + size]
    }

    /// Given a slice with the FDT, compute its subslice containing DT strings.
    fn dt_strings<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        let off = self.off_dt_strings.get() as usize;
        let size = self.size_dt_strings.get() as usize;

        &data[off..off + size]
    }

    /// Given a slice with the FDT, compute its memory reservations subslice.
    fn mem_rsvmap<'a>(&self, data: &'a [u8]) -> &'a [FdtReserveEntry] {
        let off = self.off_mem_rsvmap.get() as usize;

        unsafe {
            let ptr = data[off..].as_ptr() as *const FdtReserveEntry;
            let mut count = 0;

            loop {
                let address = (*ptr.add(count)).address.get();
                let size = (*ptr.add(count)).size.get();

                if address == 0 && size == 0 {
                    break;
                }

                count += 1;
            }

            core::slice::from_raw_parts(ptr, count)
        }
    }
}

/// A ZST representing embedded devicetree blob data.
///
/// During the build process, lunar's build script is programmed to seek target
/// board's DTS file and compile it into a DTBO file. The result of this
/// procedure is then embedded directly into the binary in
/// `arch/<ARCH>/asm/dtb.S` file, where it is also assinged a symbol
/// [`fdt_blob`]. Linker is scripted to place this object in a special,
/// readonly section.
///
/// This ZST functions as a way to expose various functions related directly to
/// the underlying data to other modules. This also means, that it is, for the
/// most part, only needed for its [`Fdt::get`] method, that yields a reference
/// to a static [`FdtView`] instance, which is what modules actually need to
/// use when they need to do something productive with the information encoded
/// by the device tree.
pub struct Fdt;

impl Fdt {
    /// Initialize a devicetree internal state from an embedded FDT blob.
    pub fn init() {
        let header: FdtHeader;
        let data: &[u8];

        unsafe {
            let start = fdt_blob.as_ptr();
            header = core::ptr::read(start as *const FdtHeader);

            // Validate magic number
            if header.magic.get() != FDT_MAGIC {
                panic!("FDT magic number mismatch");
            }

            // Obtain a slice with the entire FDT
            let size = header.totalsize.get() as usize;
            data = core::slice::from_raw_parts(start, size);
        }

        let view = FdtView {
            dt_struct: header.dt_struct(data),
            dt_strings: header.dt_strings(data),
            mem_rsvmap: header.mem_rsvmap(data),
            data,
        };

        unsafe {
            *SYSTEM_FDT.0.get() = Some(view);
        }
    }

    /// Obtain a reference to a view into embedded FDT blob.
    pub fn get() -> &'static FdtView<'static> {
        unsafe { (*SYSTEM_FDT.0.get()).as_ref().expect("FDT not initialized") }
    }
}

/// A view into devicetree contents.
///
/// While [`Fdt`] represents raw memory containing the FDT blob, this structure
/// aims to expose a higher-level API over a devicetree blob, allowing users to
/// walk, search and poll data from the devicetree. It manages parsing it and
/// orchestrates reading properties from it.
pub struct FdtView<'a> {
    data: &'a [u8],
    dt_struct: &'a [u8],
    dt_strings: &'a [u8],
    mem_rsvmap: &'a [FdtReserveEntry],
}

impl<'a> FdtView<'a> {
    // TODO
}

/// See: [`SYSTEM_FDT`].
struct FdtViewCell(UnsafeCell<Option<FdtView<'static>>>);
unsafe impl Sync for FdtViewCell {}
