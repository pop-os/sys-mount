extern crate clap;
extern crate sys_mount;

use clap::{App, Arg};
use std::process::exit;
use sys_mount::{Mount, MountFlags, SupportedFilesystems};

fn main() {
    let matches = App::new("mount")
        .arg(Arg::with_name("source").required(true))
        .arg(Arg::with_name("directory").required(true))
        .get_matches();

    let src = matches.value_of("source").unwrap();
    let dir = matches.value_of("directory").unwrap();

    // Fetch a listed of supported file systems on this system. This will be used
    // as the fstype to `Mount::new`, as the `Auto` mount parameter.
    let supported = match SupportedFilesystems::new() {
        Ok(supported) => supported,
        Err(why) => {
            eprintln!("failed to get supported file systems: {}", why);
            exit(1);
        }
    };

    // The source block will be mounted to the target directory, and the fstype is likely
    // one of the supported file systems.
    match Mount::new(src, dir, &supported, MountFlags::empty(), None) {
        Ok(mount) => {
            println!("mounted {} ({}) to {}", src, mount.get_fstype(), dir);
        }
        Err(why) => {
            eprintln!("failed to mount {} to {}: {}", src, dir, why);
            exit(1);
        }
    }
}
