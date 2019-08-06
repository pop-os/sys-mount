use libc::*;
use std::{ffi::CString, io, ops::Deref, os::unix::ffi::OsStrExt, path::Path, ptr};

/// Unmount trait which enables any type that implements it to be upgraded into an `UnmountDrop`.
pub trait Unmount {
    /// Unmount this mount with the given `flags`.
    ///
    /// This will also detach the loopback device that the mount is assigned to, if
    /// it was associated with a loopback device.
    fn unmount(&self, flags: UnmountFlags) -> io::Result<()>;

    /// Upgrades `Self` into an `UnmountDrop`, which will unmount the mount when it is dropped.
    fn into_unmount_drop(self, flags: UnmountFlags) -> UnmountDrop<Self>
    where
        Self: Sized,
    {
        UnmountDrop { mount: self, flags }
    }
}

/// Unmounts the underlying mounted device upon drop.
pub struct UnmountDrop<T: Unmount> {
    pub(crate) mount: T,
    pub(crate) flags: UnmountFlags,
}

impl<T: Unmount> UnmountDrop<T> {
    /// Modify the previously-set unmount flags.
    pub fn set_unmount_flags(&mut self, flags: UnmountFlags) { self.flags = flags; }
}

impl<T: Unmount> Deref for UnmountDrop<T> {
    type Target = T;

    fn deref(&self) -> &T { &self.mount }
}

impl<T: Unmount> Drop for UnmountDrop<T> {
    fn drop(&mut self) { let _ = self.mount.unmount(self.flags); }
}

bitflags! {
    /// Flags which may be specified when unmounting a file system.
    pub struct UnmountFlags: c_int {
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
/// This will not detach a loopback device if the mount was attached to one. This behavior may
/// change in the future, once the [loopdev](https://crates.io/crates/loopdev) crate supports
/// querying loopback device info.
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
    let mount_ptr = mount.as_ref().ok().map_or(ptr::null(), |cstr| cstr.as_ptr());

    unsafe { unmount_(mount_ptr, flags) }
}

pub(crate) unsafe fn unmount_(mount_ptr: *const c_char, flags: UnmountFlags) -> io::Result<()> {
    match umount2(mount_ptr, flags.bits()) {
        0 => Ok(()),
        _err => Err(io::Error::last_os_error()),
    }
}
