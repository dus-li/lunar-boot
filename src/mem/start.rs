// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

use core::cell::UnsafeCell;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use crate::align;
use crate::sections;

unsafe extern "C" {
    // See: arch/generic/sections.lds.h
    static __arena: u8;
    static __earena: u8;
}

/// Permission slip for early arena memory usage.
///
/// This is effectively a ZST whose entire purpose is to serve as a carrier of
/// a lifetime information. This allows us to invalidate all references to
/// memory acquired through [`Token::alloc_slice`] after the memory is
/// reclaimed and thus avoid use-after-free bugs.
pub struct Token<'a> {
    _marker: core::marker::PhantomData<&'a mut ()>,
}

struct Arena {
    cursor: usize,
    end: usize,
}

/// A memory manager instance for the early initialization process.
///
/// Since early initialization takes place before SMP is set up, the
/// [`UnsafeCell`] suffices as a mean of protection.
static ARENA: ArenaCell = ArenaCell(UnsafeCell::new(None));

/// A flag for double initialization prevention.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Set up early init memory allocator.
///
/// This function may only be called once. It grants the caller a token whose
/// lifetime now becomes bound with lifetime of the early init arena lifetime.
/// As soon as the token is dropped, the memory will be reclaimed. All
/// allocations after that point will fail and if someone, by some means would
/// still hold a reference to some early memory - any usage of that memory
/// would be a prime example of a UAF bug.
#[unsafe(link_section = sections::start_text!())]
pub fn init() -> Token<'static> {
    let start = core::ptr::addr_of!(__arena) as usize;
    let end = core::ptr::addr_of!(__earena) as usize;

    let arena = Arena { cursor: start, end };

    if INITIALIZED.swap(true, Ordering::SeqCst) {
        panic!("Double initialization of start arena");
    }

    unsafe {
        *ARENA.0.get() = Some(arena);
    }

    Token {
        _marker: core::marker::PhantomData,
    }
}

impl<'a> Token<'a> {
    /// Allocate a slice from the early arena.
    ///
    /// Remember that start arena memory is subject to reclaiming. Memory
    /// allocated here either needs to be temporary, or copied after more
    /// advanced memory management mechanisms are set up.
    pub fn alloc_slice<T>(&self, count: usize) -> &'a mut [T] {
        let layout = core::alloc::Layout::array::<T>(count).unwrap();

        unsafe {
            let arena = (*ARENA.0.get())
                .as_mut()
                .expect("Arena no longer accessible");

            let cursor = align::align_up!(arena.cursor, layout.align());
            let end = cursor + layout.size();

            if end > arena.end {
                panic!("OOM in start arena");
            }

            arena.cursor = end;

            core::slice::from_raw_parts_mut(cursor as *mut T, count)
        }
    }
}

/// By implementing a custom drop logic we prevent use-after-reclaim.
///
/// The [`Token::drop`] function modifies static state to ensure that if there
/// are still any holders of any start arena tokens, their allocations will
/// panic. This is a fail-fast approach, perhaps a crude one, but hopefully
/// effective.
impl<'a> Drop for Token<'a> {
    fn drop(&mut self) {
        unsafe {
            // Ensure all allocations will fail.
            *ARENA.0.get() = None;
        }
    }
}

/// See: [`START_ARENA`].
struct ArenaCell(UnsafeCell<Option<Arena>>);
unsafe impl Sync for ArenaCell {}
