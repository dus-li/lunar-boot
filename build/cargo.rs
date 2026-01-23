// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

macro_rules! rerun_if_env_changed {
    ($($arg:tt)+) => {
        println!("cargo::rerun-if-env-changed={}", format!($($arg)+))
    };
}
pub(crate) use rerun_if_env_changed;

macro_rules! rerun_if_changed {
    ($arg:expr) => {
        println!("cargo::rerun-if-changed={:?}", $arg)
    };
    ($fmt:literal $(,$arg:tt)+) => {
        println!("cargo::rerun-if-changed={}", format!($($arg)+))
    };
}
pub(crate) use rerun_if_changed;

macro_rules! rustc_env {
    ($key:expr, $val:expr) => {
        println!("cargo::rustc-env={}={}", $key, $val)
    };
}
pub(crate) use rustc_env;

macro_rules! rustc_link_arg {
    ($($arg:tt)+) => {
        println!("cargo::rustc-link-arg={}", format!($($arg)+))
    };
}
pub(crate) use rustc_link_arg;

macro_rules! error {
    ($($arg:tt)+) => {
        println!("cargo::error={}", format!($($arg)*))
    };
}
pub(crate) use error;

#[macro_export]
macro_rules! bprintln {
    () => {
        println!("cargo::warning=\x1B[2K\r")
    };
    ($($arg:tt)+) => {
        println!("cargo::warning=\x1B[2K\r   {}", format!($($arg)+))
    };
}

macro_rules! info {
    ($($arg:tt)+) => {
        bprintln!("\x1B[1;37minfo:\x1B[0m {}", format!($($arg)+))
    };
}
pub(crate) use info;
