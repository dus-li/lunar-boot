// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

use core::cell::UnsafeCell;
use core::iter::Iterator;
use core::str;

use crate::inttypes::{BEu32, BEu64};
use crate::{align, sections};

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

/// A trait representing an object which can be turned to an [`FdtStream`].
///
/// Upon implementing [`FdtStreamable::stream`], it automatically becomes
/// possible to search the implementing type by name and phandles to locate
/// nodes and properties.
pub trait FdtStreamable<'a> {
    /// Obtain an instance of an FDT stream over the type.
    fn stream(&self) -> FdtStream<'a>;

    /// Search for a node with a given name, ignoring unit address.
    fn node_by_name(&self, target: &str) -> Option<FdtNode<'a>> {
        self.stream().find(|node| name_matches(&node.name, target))
    }

    /// Search for a node with a given phandle.
    fn node_by_phandle(&self, id: Phandle) -> Option<FdtNode<'a>> {
        self.stream()
            .find(|node| {
                node.prop_u32("phandle").is_some_and(|ph| id.get() == ph)
            })
            .or_else(|| {
                self.stream()
                    .fold(None, |acc, node| acc.or(node.node_by_phandle(id)))
            })
    }

    /// Search for a given property and return its value as raw bytes.
    fn prop_raw(&self, target: &str) -> Option<&'a [u8]> {
        use FdtToken::*;

        let mut stream = self.stream();

        while let Some(token) = stream.next_u32() {
            match token {
                _ if token == Prop as u32 => {
                    let len = stream.next_u32()? as usize;
                    let off = stream.next_u32()? as usize;
                    let name = stream.string_at_off(off)?;

                    if name == target {
                        return stream.tape.get(stream.off..stream.off + len);
                    }

                    stream.off = align::align_up!(stream.off + len, 4);
                }
                _ if token == BeginNode as u32 => break,
                _ if token == EndNode as u32 => break,
                _ => {}
            }
        }

        None
    }

    /// Search for a given property and return its value as a [`u32`].
    fn prop_u32(&self, target: &str) -> Option<u32> {
        self.prop_raw(target)
            .map(|bytes| bytes[0..4].try_into().ok().map(u32::from_be_bytes))
            .flatten()
    }

    /// Search for a given property and return its value as a string slice.
    fn prop_str(&self, target: &str) -> Option<&'a str> {
        self.prop_raw(target)
            .map(|bytes| str::from_utf8(bytes).ok())
            .flatten()
    }

    /// Search for a given property and return its value as a phandle.
    fn prop_phandle(&self, target: &str) -> Option<Phandle> {
        self.prop_raw(target)
            .map(|bytes| bytes.try_into().ok().map(BEu32::new))
            .flatten()
    }
}

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
    #[unsafe(link_section = sections::start_text!())]
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

/// A streamed reader of an FDT blob.
pub struct FdtStream<'a> {
    tape: &'a [u8],
    strings: &'a [u8],
    off: usize,
}

impl<'a> FdtStream<'a> {
    /// Construct a new instance of an FDT streaming reader.
    ///
    /// # Arguments
    ///
    /// - `tape`: A DT struct slice starting with an FDT_BEGIN_NODE token.
    /// - `strings`: Entire DT strings slice.
    fn new(tape: &'a [u8], strings: &'a [u8]) -> Self {
        FdtStream {
            tape,
            strings,
            off: 0,
        }
    }

    /// Read next [`u32`] from the FDT and move cursor 4 bytes forward.
    fn next_u32(&mut self) -> Option<u32> {
        let bytes = self.tape.get(self.off..self.off + 4)?;
        let ret = Some(u32::from_be_bytes(bytes.try_into().ok()?));

        self.off += 4;

        ret
    }

    /// Read next NUL-terminated string and move cursor with proper alignment.
    fn next_str(&mut self) -> Option<&'a str> {
        let start = self.off;

        let mut end = start;
        while self.tape.get(end)? != &0 {
            end += 1;
        }

        let ret = str::from_utf8(&self.tape[start..end]).ok()?;

        self.off = align::align_up!(end + 1, 4);

        Some(ret)
    }

    /// Locate offset of the end of the node that stream cursor points to.
    fn node_end_off(&mut self) -> Option<usize> {
        use FdtToken::*;

        let mut depth = 1;
        let mut prev = self.off;

        while depth > 0 {
            let token = self.next_u32()?;
            match token {
                _ if token == BeginNode as u32 => {
                    self.next_str()?;
                    depth += 1;
                }
                _ if token == EndNode as u32 => depth -= 1,
                _ if token == Prop as u32 => self.skip_prop()?,
                _ if token == End as u32 => break,
                _ => {}
            }

            prev = self.off;
        }

        Some(prev)
    }

    /// Set stream cursor to after current property.
    ///
    /// This method assumes that it was called immediately after FDT_PROP token
    /// was read. If cursor does not point to its direct successor when this
    /// method is called, the result is guaranteed to be off.
    fn skip_prop(&mut self) -> Option<()> {
        let len = self.next_u32()? as usize;
        self.next_u32()?;

        self.off = align::align_up!(self.off + len, 4);
        Some(())
    }

    fn string_at_off(&self, off: usize) -> Option<&'a str> {
        let mut end = off;

        while self.strings.get(end)? != &0 {
            end += 1;
        }

        str::from_utf8(&self.strings[off..end]).ok()
    }
}

/// Iterator over nodes in a devicetree stream.
impl<'a> Iterator for FdtStream<'a> {
    type Item = FdtNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use FdtToken::*;

        while let Some(token) = self.next_u32() {
            match token {
                _ if token == BeginNode as u32 => {
                    let name = self.next_str()?;

                    let mut end_lookup = FdtStream {
                        tape: self.tape,
                        strings: self.strings,
                        off: self.off,
                    };

                    let start = self.off;
                    let end = end_lookup.node_end_off()?;

                    return Some(FdtNode {
                        name,
                        body: &self.tape[start..end],
                        strings: self.strings,
                    });
                }
                _ if token == Prop as u32 => self.skip_prop()?,
                _ if token == End as u32 => break,
                _ => {}
            }
        }

        None
    }
}

/// Tokens delimiting pieces in the FDT structure block.
#[repr(u32)]
enum FdtToken {
    BeginNode = 0x1,
    EndNode = 0x2,
    Prop = 0x3,
    _Nop = 0x4,
    End = 0x9,
}

/// A zero-copy handle into a devicetree node.
pub struct FdtNode<'a> {
    name: &'a str,
    body: &'a [u8],
    strings: &'a [u8],
}

/// Devicetree phandle.
pub type Phandle = BEu32;

impl<'a> FdtStreamable<'a> for FdtNode<'a> {
    fn stream(&self) -> FdtStream<'a> {
        FdtStream::new(&self.body, &self.strings)
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

/// Check if a name matches some target, ignoring unit address in the process.
fn name_matches(name: &str, target: &str) -> bool {
    name == target
        || (name.starts_with(target)
            && name.as_bytes().get(target.len()) == Some(&b'@'))
}

impl<'a> FdtStreamable<'a> for FdtView<'a> {
    /// Start streaming contents of the devicetree structure.
    fn stream(&self) -> FdtStream<'a> {
        FdtStream::new(&self.dt_struct, &self.dt_strings)
    }
}

/// See: [`SYSTEM_FDT`].
struct FdtViewCell(UnsafeCell<Option<FdtView<'static>>>);
unsafe impl Sync for FdtViewCell {}
