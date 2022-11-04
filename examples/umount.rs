// Copyright 2018-2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

extern crate clap;
extern crate sys_mount;

use clap::{Arg, Command};
use std::process::ExitCode;
use sys_mount::{unmount, UnmountFlags};

fn main() -> ExitCode {
    let matches = Command::new("umount")
        .arg(Arg::new("lazy").short('l').long("lazy"))
        .arg(Arg::new("source").required(true))
        .get_matches();

    let src = matches.get_one::<String>("source").unwrap();

    let flags = if matches.get_flag("lazy") {
        UnmountFlags::DETACH
    } else {
        UnmountFlags::empty()
    };

    let Err(why) = unmount(src, flags) else {
        return ExitCode::SUCCESS;
    };

    eprintln!("failed to unmount {}: {}", src, why);
    ExitCode::FAILURE
}
