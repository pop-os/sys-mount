extern crate clap;
extern crate sys_mount;

use clap::{App, Arg};
use std::process::exit;
use sys_mount::{unmount, UnmountFlags};

fn main() {
    let matches = App::new("umount")
        .arg(Arg::with_name("lazy").short("l").long("lazy"))
        .arg(Arg::with_name("source").required(true))
        .get_matches();

    let src = matches.value_of("source").unwrap();

    let flags =
        if matches.is_present("lazy") { UnmountFlags::DETACH } else { UnmountFlags::empty() };

    match unmount(src, flags) {
        Ok(()) => (),
        Err(why) => {
            eprintln!("failed to unmount {}: {}", src, why);
            exit(1);
        }
    }
}
