// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

mod commands;
mod sources;

use std::env;
use std::path::PathBuf;
use std::process::Command;

use cargo_build;
use cc;

fn archdir(arch: &str) -> PathBuf {
    PathBuf::from("arch").join(arch)
}

fn arch_generic_dir() -> PathBuf {
    PathBuf::from("arch").join("generic")
}

fn boarddir(board: &str) -> PathBuf {
    PathBuf::from("boards").join(board)
}

fn build_asm(arch: &str) -> Result<(), ()> {
    let asmdir = archdir(arch).join("asm");
    let mut cc = cc::Build::new();

    for file in sources::Sources::new(&asmdir, &["S", "s"]) {
        let path = file.path();
        cargo_build::rerun_if_changed(&path);
        cc.file(&path);
    }

    // FIXME: make this smarter and dont hide it in code
    if arch == "riscv64" {
        cc.flag("-mabi=lp64d");
    }

    cc.flags(["-x", "assembler-with-cpp"])
        .include(&archdir(arch))
        .include(&arch_generic_dir())
        .compile("archasm");

    Ok(())
}

fn build_dtb(out: &PathBuf, arch: &str, board: &str) -> Result<(), ()> {
    let raw = boarddir(board).join("board.dts");
    let dts = out.join("board.dts");
    let dtb = out.join("board.dtb");

    if !raw.exists() {
        cargo_build::error!("DTS for board '{}' missing", board);
        return Err(());
    }

    cargo_build::rerun_if_changed(&raw);
    for file in sources::Sources::new(&archdir(arch), &["dts", "dtsi"]) {
        cargo_build::rerun_if_changed(&file.path());
    }

    commands::Cpp::new(&dts, &raw)
        .define("__DTS__", None)
        .includes(&[&archdir(arch)])
        .includes(&[&arch_generic_dir()])
        .run()?;

    let status = Command::new("dtc")
        .args(["-I", "dts"])
        .args(["-O", "dtb"])
        .args(["-o", &dtb.display().to_string()])
        .arg(&dts)
        .status();

    if !status.is_ok_and(|result| result.success()) {
        cargo_build::error("Failed to compile DTS");
        return Err(());
    }

    Ok(())
}

fn cons_lds(out: &PathBuf, arch: &str, board: &str) -> Result<PathBuf, ()> {
    let input = archdir(arch).join("lunar.lds");
    let lds = out.join("lunar.lds");

    for file in sources::Sources::new(&archdir(arch), &["lds", "ld"]) {
        cargo_build::rerun_if_changed(&file.path());
    }

    commands::Cpp::new(&out.join("lunar.lds"), &input)
        .define("__LINKER_SCRIPT__", None)
        .includes(&[&archdir(arch)])
        .includes(&[&arch_generic_dir()])
        .includes(&[&boarddir(board)])
        .run()?;

    Ok(lds)
}

fn do_main(out: &PathBuf, arch: &str, board: &str) -> Result<(), ()> {
    build_asm(arch)?;
    build_dtb(out, arch, board)?;

    let lds = cons_lds(out, arch, board)?;
    cargo_build::rustc_link_arg!("-T{}", { lds.display() });

    Ok(())
}

fn main() {
    println!("cargo::rerun-if-env-changed=BOARD");
    cargo_build::rerun_if_changed(&PathBuf::from("build.rs"));

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let Ok(board) = env::var("BOARD") else {
        cargo_build::error("BOARD value not set");
        return;
    };

    cargo_build::rustc_env("BOARD", &board);

    let _ = do_main(&out, &arch, &board);
}
