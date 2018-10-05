# sys-mount

![Rust Compatibility](https://img.shields.io/badge/rust-1.24.1%20tested-green.svg)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](https://gitlab.com/evertiro/maco/blob/master/LICENSE)

High level FFI bindings to the `mount` and `umount2` system calls, for Rust.

## Examples

### Mount

This is how the `mount` command could be written with this API.

```rust
extern crate clap;
extern crate sys_mount;

use clap::{App, Arg};
use sys_mount::{Mount, MountFlags, SupportedFilesystems};
use std::process::exit;

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
```

### Umount

This is how the `umount` command could be implemented.

```rust
extern crate clap;
extern crate sys_mount;

use clap::{App, Arg};
use sys_mount::{unmount, UnmountFlags};
use std::process::exit;

fn main() {
    let matches = App::new("umount")
        .arg(Arg::with_name("lazy")
            .short("l")
            .long("lazy"))
        .arg(Arg::with_name("source").required(true))
        .get_matches();

    let src = matches.value_of("source").unwrap();

    let flags = if matches.is_present("lazy") {
        UnmountFlags::DETACH
    } else {
        UnmountFlags::empty()
    };

    match unmount(src, flags) {
        Ok(()) => (),
        Err(why) => {
            eprintln!("failed to unmount {}: {}", src, why);
            exit(1);
        }
    }
}
```