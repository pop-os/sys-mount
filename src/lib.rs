// Copyright 2018-2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! High level abstraction over the `mount` and `umount2` system calls.
//!
//! If the `loop` feature is enabled (default), additionally supports creating loopback devices
//! automatically when mounting an iso or squashfs file.
//!
//! # Example
//!
//! ```rust,no_run
//! extern crate sys_mount;
//!
//! use std::process::exit;
//! use sys_mount::{
//!     Mount,
//!     MountFlags,
//!     SupportedFilesystems,
//!     Unmount,
//!     UnmountFlags
//! };
//!
//! fn main() {
//!     // Fetch a list of supported file systems.
//!     // When mounting, a file system will be selected from this.
//!     let supported = SupportedFilesystems::new().unwrap();
//!
//!     // Attempt to mount the src device to the dest directory.
//!     let mount_result = Mount::builder()
//!         .fstype("btrfs")
//!         .data("subvol=@home")
//!         .mount("/dev/sda1", "/home");
//!
//!     match mount_result {
//!         Ok(mount) => {
//!             // Make the mount temporary, so that it will be unmounted on drop.
//!             let mount = mount.into_unmount_drop(UnmountFlags::DETACH);
//!             // Do thing with the mounted device.
//!         }
//!         Err(why) => {
//!             eprintln!("failed to mount device: {}", why);
//!             exit(1);
//!         }
//!     }
//! }

extern crate libc;
extern crate loopdev;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate thiserror;

mod builder;
mod flags;
mod fstype;
mod mount;
mod supported;
mod umount;

pub use self::{builder::*, flags::*, fstype::*, mount::*, supported::*, umount::*};

use libc::swapoff as c_swapoff;
use std::{
    ffi::CString,
    io::{self, Error, ErrorKind},
    os::unix::ffi::OsStrExt,
    path::Path,
};

#[derive(Debug, Error)]
pub enum ScopedMountError {
    #[error("cannot get list of supported file systems")]
    Supported(#[source] io::Error),
    #[error("could not mount partition")]
    Mount(#[source] io::Error),
}

/// Mount a partition temporarily for the duration of the scoped block within.
///
/// # Errors
///
/// - Fails if the supported file systems cannot be found.
/// - Or if it fails to unmount
pub fn scoped_mount<T, S: FnOnce() -> T>(
    source: &Path,
    mount_at: &Path,
    scope: S,
) -> Result<T, ScopedMountError> {
    let supported = SupportedFilesystems::new().map_err(ScopedMountError::Supported)?;

    Mount::builder()
        .fstype(&supported)
        .mount(source, mount_at)
        .map_err(ScopedMountError::Mount)?;

    let result = scope();

    if let Err(why) = unmount(mount_at, UnmountFlags::empty()) {
        tracing::warn!("{}: failed to unmount: {}", mount_at.display(), why);
    }

    Ok(result)
}

/// Unmounts a swap partition using `libc::swapoff`
///
/// # Errors
///
/// - If the destination path is not a valid C String
/// - Or the swapoff function fails
pub fn swapoff<P: AsRef<Path>>(dest: P) -> io::Result<()> {
    let Ok(swap) = CString::new(dest.as_ref().as_os_str().as_bytes().to_owned()) else {
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "swap path is not a valid c string: '{}'",
                dest.as_ref().display()
            )
        ))
    };

    match unsafe { c_swapoff(swap.as_ptr()) } {
        0 => Ok(()),

        _err => Err(Error::new(
            ErrorKind::Other,
            format!(
                "failed to swapoff {}: {}",
                dest.as_ref().display(),
                Error::last_os_error()
            ),
        )),
    }
}

#[inline]
fn to_cstring(data: &[u8]) -> io::Result<CString> {
    CString::new(data).map_err(|why| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to create `CString`: {}", why),
        )
    })
}
