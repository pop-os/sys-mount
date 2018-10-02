use libc::*;
use std::ffi::CString;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

bitflags! {
    /// Flags which may be specified when unmounting a file system.
    pub struct UnmountFlags: i32 {
        /// Force unmount even if busy. This can cause data loss. (Only for NFS mounts.) 
        const FORCE = MNT_FORCE;

        /// Perform a lazy unmount: make the mount point unavailable for new accesses,
        /// and actually perform the unmount when the mount point ceases to be busy. 
        const DETACH = MNT_DETACH;

        /// Mark the mount point as expired. If a mount point is not currently in use,
        /// then an initial call to umount2() with this flag fails with the error EAGAIN,
        /// but marks the mount point as expired. The mount point remains expired as
        /// long as it isn't accessed by any process. A second umount2() call specifying
        /// MNT_EXPIRE unmounts an expired mount point. This flag cannot be specified with
        /// either MNT_FORCE or MNT_DETACH. 
        const EXPIRE = MNT_EXPIRE;

        /// Don't dereference target if it is a symbolic link. This flag allows security
        /// problems to be avoided in set-user-ID-root programs that allow unprivileged
        /// users to unmount file systems.
        const NOFOLLOW = O_NOFOLLOW;
    }
}

/// Unmounts the device at `path` using the provided `UnmountFlags`.
/// 
/// ```rust,no_run
/// extern crate sys_mount;
/// 
/// use sys_mount::{unmount, UnmountFlags};
/// 
/// fn main() {
///     // Unmount device at `/target/path` lazily.
///     let result = unmount("/target/path", UnmountFlags::DETACH);
/// }
/// ```
pub fn unmount<P: AsRef<Path>>(path: P, flags: UnmountFlags) -> io::Result<()> {
    let mount = CString::new(path.as_ref().as_os_str().as_bytes().to_owned());
    let mount_ptr = mount.as_ref()
        .ok()
        .map_or(ptr::null(), |cstr| cstr.as_ptr());

    unsafe { unmount_(mount_ptr, flags) }
}

pub(crate) unsafe fn unmount_(mount_ptr: *const c_char, flags: UnmountFlags) -> io::Result<()> {
    match umount2(mount_ptr, flags.bits()) {
        0 => Ok(()),
        _err => Err(io::Error::last_os_error())
    }
}