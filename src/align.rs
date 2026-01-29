// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

/// Align a value upwards to a byte boundary.
///
/// # Arguments
///
/// - `what`: Value that is to be aligned.
/// - `how`: Alignment constant, must be a power of two.
macro_rules! align_up {
    ($what:expr, $how:expr) => {{
        let mask = $how - 1;
        assert!(($how & mask) == 0);

        ($what + mask) & !mask
    }};
}
pub(crate) use align_up;
