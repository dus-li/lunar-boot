// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

pub struct Cpp<'a> {
    output: &'a PathBuf,
    input: &'a PathBuf,
    build: cc::Build,
}

impl<'a> Cpp<'a> {
    pub fn new(output: &'a PathBuf, input: &'a PathBuf) -> Self {
        let mut build = cc::Build::new();

        build.flags(["-x", "assembler-with-cpp"]);
        build.flag("-nostdinc");
        build.flag("-undef");
        build.flag("-E");

        Cpp {
            output,
            input,
            build,
        }
    }

    pub fn define(mut self, key: &str, value: Option<&str>) -> Self {
        self.build.define(key, value);
        self
    }

    pub fn includes(mut self, includes: &[&PathBuf]) -> Self {
        self.build.includes(includes);
        self
    }

    pub fn run(self) -> Result<(), ()> {
        let output = self.output.display().to_string();
        let input = self.input.display().to_string();

        let status = self
            .build
            .get_compiler()
            .to_command()
            .args(["-o", &output])
            .arg(&input)
            .status();

        if status.is_ok_and(|result| result.success()) {
            return Ok(());
        }

        cargo_build::error!("Failed to preprocess {}", input);
        Err(())
    }
}
