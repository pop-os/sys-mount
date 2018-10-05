//! High level abstraction over the `mount` and `umount2` system calls.
//! 
//! Additionally creates loopback devices automatically when mounting an iso or
//! squashfs file.
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
//!     let mount_result = Mount::new(
//!         "/imaginary/block/device",
//!         "/tmp/location",
//!         &supported,
//!         MountFlags::empty(),
//!         None
//!     );
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

mod fstype;
mod mount;
mod supported;
mod umount;

pub use self::fstype::*;
pub use self::mount::*;
pub use self::supported::*;
pub use self::umount::*;

use std::io;
use std::ffi::CString;

fn to_cstring(data: &[u8]) -> io::Result<CString> {
    CString::new(data).map_err(|why| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to create `CString`: {}", why)
        )
    })
}