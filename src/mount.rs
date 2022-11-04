// Copyright 2018-2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::umount::{unmount_, Unmount, UnmountDrop};
use crate::{MountBuilder, UnmountFlags};
use std::{
    ffi::{CString, OsStr},
    io,
    os::unix::ffi::OsStrExt,
    path::Path,
};

/// Handle for managing a mounted file system.
#[derive(Debug)]
pub struct Mount {
    pub(crate) target: CString,
    pub(crate) fstype: String,
    pub(crate) loopback: Option<loopdev::LoopDevice>,
    pub(crate) loop_path: Option<std::path::PathBuf>,
}

impl Unmount for Mount {
    fn unmount(&self, flags: UnmountFlags) -> io::Result<()> {
        unsafe {
            unmount_(self.target.as_ptr(), flags)?;
        }

        if let Some(ref loopback) = self.loopback {
            loopback.detach()?;
        }

        Ok(())
    }
}

impl Mount {
    /// Creates a [`MountBuilder`] for configuring a new mount.
    ///
    /// ```no_run
    /// use sys_mount::*;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let _mount = Mount::builder()
    ///         .fstype("btrfs")
    ///         .data("subvol=@home")
    ///         .mount("/dev/sda1", "/home")?;
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn builder<'a>() -> MountBuilder<'a> {
        MountBuilder::default()
    }

    /// Mounts the source device to the target path.
    ///
    /// Attempts to automatically detect the filesystem of the source device.
    ///
    /// For more flexibility, use [`Mount::builder`] instead.
    ///
    /// # Errors
    ///
    /// Errors if supported filesystems cannot be detected, or the mount fails.
    #[inline]
    pub fn new(source: impl AsRef<Path>, target: impl AsRef<Path>) -> io::Result<Mount> {
        let supported = crate::SupportedFilesystems::new()?;
        MountBuilder::default()
            .fstype(&supported)
            .mount(source, target)
    }

    /// If the device was associated with a loopback device, that device's path
    /// can be retrieved here.
    #[inline]
    #[must_use]
    pub fn backing_loop_device(&self) -> Option<&Path> {
        self.loop_path.as_deref()
    }

    /// Describes the file system which this mount was mounted with.
    ///
    /// This is useful in the event that the mounted device was mounted automatically.
    #[inline]
    #[must_use]
    pub fn get_fstype(&self) -> &str {
        &self.fstype
    }

    /// Return the path this mount was mounted on.
    #[inline]
    #[must_use]
    pub fn target_path(&self) -> &Path {
        Path::new(OsStr::from_bytes(self.target.as_bytes()))
    }

    #[inline]
    pub(crate) fn from_target_and_fstype(target: CString, fstype: String) -> Self {
        Mount {
            target,
            fstype,
            loopback: None,
            loop_path: None,
        }
    }
}

/// An abstraction that will ensure that temporary mounts are dropped in reverse.
pub struct Mounts(pub Vec<UnmountDrop<Mount>>);

impl Mounts {
    /// Unmounts all mounts, with the option to do so lazily.
    ///
    /// # Errors
    ///
    /// Returns on the first error when unmounting.
    pub fn unmount(&mut self, lazy: bool) -> io::Result<()> {
        let flags = if lazy {
            UnmountFlags::DETACH
        } else {
            UnmountFlags::empty()
        };
        self.0
            .iter_mut()
            .rev()
            .try_for_each(|mount| mount.unmount(flags))
    }
}

impl Drop for Mounts {
    fn drop(&mut self) {
        for mount in self.0.drain(..).rev() {
            drop(mount);
        }
    }
}
