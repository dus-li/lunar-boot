// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

use std::fs::{DirEntry, ReadDir};
use std::iter::Iterator;
use std::path::PathBuf;

pub struct Sources<'a> {
    root: &'a PathBuf,
    exts: Vec<&'a str>,
}

pub struct SourcesIter<'a> {
    sources: Sources<'a>,
    iter: ReadDir,
}

impl<'a> Sources<'a> {
    pub fn new(root: &'a PathBuf, exts: &[&'a str]) -> Self {
        Sources {
            root,
            exts: exts.to_vec(),
        }
    }
}

impl<'a> IntoIterator for Sources<'a> {
    type IntoIter = SourcesIter<'a>;
    type Item = DirEntry;

    fn into_iter(self) -> Self::IntoIter {
        let iter = match std::fs::read_dir(self.root) {
            Ok(iter) => iter.into_iter(),
            Err(err) => {
                let path = self.root.display();
                cargo_build::error!("Failed to read {path}: {err}");
                std::process::exit(1);
            }
        };

        SourcesIter {
            sources: self,
            iter,
        }
    }
}

impl<'a> Iterator for SourcesIter<'a> {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(result) = self.iter.next() {
            let dentry = match result {
                Ok(dentry) => dentry,
                Err(err) => {
                    cargo_build::error!("Directory access failure: {err}");
                    std::process::exit(1);
                }
            };

            if dentry
                .path()
                .extension()
                .map(std::ffi::OsStr::to_str)
                .flatten()
                .is_some_and(|val| self.sources.exts.contains(&val))
            {
                return Some(dentry);
            }
        }

        return None;
    }
}
