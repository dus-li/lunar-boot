// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

/// A thin wrapper around a big endian 32-bit unsigned integer.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BEu32(u32);

impl BEu32 {
    pub const fn new(bytes: [u8; 4]) -> Self {
        BEu32(u32::from_ne_bytes(bytes))
    }

    pub const fn get(self) -> u32 {
        u32::from_be(self.0)
    }
}

impl core::fmt::Debug for BEu32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#08x}", u32::from_be(self.0))
    }
}

/// A thin wrapper around a big endian 64-bit unsigned integer.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BEu64(u64);

impl BEu64 {
    pub const fn new(bytes: [u8; 8]) -> Self {
        BEu64(u64::from_ne_bytes(bytes))
    }

    pub const fn get(self) -> u64 {
        u64::from_be(self.0)
    }
}

impl core::fmt::Debug for BEu64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#016x}", u64::from_be(self.0))
    }
}
