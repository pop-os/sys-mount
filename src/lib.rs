//! High level abstraction over the `mount` and `umount2` system calls.

extern crate libc;
#[macro_use]
extern crate bitflags;

mod fstype;
mod mount;
mod supported;
mod temp_mount;
mod umount;

pub use self::fstype::*;
pub use self::mount::*;
pub use self::supported::*;
pub use self::umount::*;
pub use self::temp_mount::*;

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