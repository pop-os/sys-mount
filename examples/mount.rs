// Copyright 2018-2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

use clap::{Arg, Command};
use std::process::ExitCode;
use sys_mount::{Mount, SupportedFilesystems};

fn main() -> ExitCode {
    let matches = Command::new("mount")
        .arg(Arg::new("source").required(true))
        .arg(Arg::new("directory").required(true))
        .get_matches();

    let src = matches.get_one::<String>("source").unwrap();
    let dir = matches.get_one::<String>("directory").unwrap();

    // Fetch a listed of supported file systems on this system. This will be used
    // as the fstype to `Mount::new`, as the `Auto` mount parameter.
    let supported = match SupportedFilesystems::new() {
        Ok(supported) => supported,
        Err(why) => {
            eprintln!("failed to get supported file systems: {}", why);
            return ExitCode::FAILURE;
        }
    };

    // The source block will be mounted to the target directory, and the fstype is likely
    // one of the supported file systems.
    match Mount::builder().fstype(&supported).mount(src, dir) {
        Ok(mount) => {
            println!("mounted {} ({}) to {}", src, mount.get_fstype(), dir);
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("failed to mount {} to {}: {}", src, dir, why);
            ExitCode::FAILURE
        }
    }
}
