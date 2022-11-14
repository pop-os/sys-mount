// Copyright 2018-2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::to_cstring;
use crate::{
    io, libc, CString, FilesystemType, Mount, MountFlags, OsStrExt, Path, SupportedFilesystems,
    Unmount, UnmountDrop, UnmountFlags,
};
use libc::mount;
use std::ptr;

/// Builder API for mounting devices
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
#[derive(Clone, Copy, smart_default::SmartDefault)]
#[allow(clippy::module_name_repetitions)]
pub struct MountBuilder<'a> {
    #[default(MountFlags::empty())]
    flags: MountFlags,
    fstype: Option<FilesystemType<'a>>,
    #[cfg(feature = "loop")]
    loopback_offset: u64,
    data: Option<&'a str>,
}

impl<'a> MountBuilder<'a> {
    /// Options to apply for the file system on mount.
    #[must_use]
    pub fn data(mut self, data: &'a str) -> Self {
        self.data = Some(data);
        self
    }

    /// The file system that is to be mounted.
    #[must_use]
    pub fn fstype(mut self, fs: impl Into<FilesystemType<'a>>) -> Self {
        self.fstype = Some(fs.into());
        self
    }

    /// Mount flags for the mount syscall.
    #[must_use]
    pub fn flags(mut self, flags: MountFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Offset for the loopback device
    #[cfg(feature = "loop")]
    #[must_use]
    pub fn loopback_offset(mut self, offset: u64) -> Self {
        self.loopback_offset = offset;
        self
    }

    /// Mounts a file system at `source` to a `target` path in the system.
    ///
    /// ```rust,no_run
    /// use sys_mount::{
    ///     Mount,
    ///     MountFlags,
    ///     SupportedFilesystems
    /// };
    ///
    /// // Fetch a list of supported file systems.
    /// // When mounting, a file system will be selected from this.
    /// let supported = SupportedFilesystems::new().unwrap();
    ///
    /// // Attempt to mount the src device to the dest directory.
    /// let mount_result = Mount::builder()
    ///     .fstype(&supported)
    ///     .mount("/imaginary/block/device", "/tmp/location");
    /// ```
    /// # Notes
    ///
    /// The provided `source` device and `target` destinations must exist within the file system.
    ///
    /// If the `source` is a file with an extension, a loopback device will be created, and the
    /// file will be associated with the loopback device. If the extension is `iso` or `squashfs`,
    /// the filesystem type will be set accordingly, and the `MountFlags` will also be modified to
    /// ensure that the `MountFlags::RDONLY` flag is set before mounting.
    ///
    /// The `fstype` parameter accepts either a `&str` or `&SupportedFilesystem` as input. If the
    /// input is a `&str`, then a particular file system will be used to mount the `source` with.
    /// If the input is a `&SupportedFilesystems`, then the file system will be selected
    /// automatically from the list.
    ///
    /// The automatic variant of `fstype` works by attempting to mount the `source` with all
    /// supported device-based file systems until it succeeds, or fails after trying all
    /// possible options.
    ///
    /// # Errors
    ///
    /// - If a fstype is not defined and supported filesystems cannot be detected
    /// - If a loopback device cannot be created
    /// - If the source or target are not valid C strings
    /// - If mounting fails
    pub fn mount(self, source: impl AsRef<Path>, target: impl AsRef<Path>) -> io::Result<Mount> {
        let MountBuilder {
            data,
            fstype,
            flags,
            #[cfg(feature = "loop")]
            loopback_offset,
        } = self;

        let supported;

        let fstype = if let Some(fstype) = fstype {
            fstype
        } else {
            supported = SupportedFilesystems::new()?;
            FilesystemType::Auto(&supported)
        };

        let source = source.as_ref();
        let mut c_source = None;

        #[cfg(feature = "loop")]
        let (mut flags, mut fstype, mut loopback, mut loop_path) = (flags, fstype, None, None);

        if !source.as_os_str().is_empty() {
            // Create a loopback device if an iso or squashfs is being mounted.
            #[cfg(feature = "loop")]
            if let Some(ext) = source.extension() {
                let extf = i32::from(ext == "iso") | if ext == "squashfs" { 2 } else { 0 };

                if extf != 0 {
                    fstype = if extf == 1 {
                        flags |= MountFlags::RDONLY;
                        FilesystemType::Manual("iso9660")
                    } else {
                        flags |= MountFlags::RDONLY;
                        FilesystemType::Manual("squashfs")
                    };
                }

                let new_loopback = loopdev::LoopControl::open()?.next_free()?;
                new_loopback
                    .with()
                    .read_only(flags.contains(MountFlags::RDONLY))
                    .offset(loopback_offset)
                    .attach(source)?;
                let path = new_loopback.path().expect("loopback does not have path");
                c_source = Some(to_cstring(path.as_os_str().as_bytes())?);
                loop_path = Some(path);
                loopback = Some(new_loopback);
            }

            if c_source.is_none() {
                c_source = Some(to_cstring(source.as_os_str().as_bytes())?);
            }
        };

        let c_target = to_cstring(target.as_ref().as_os_str().as_bytes())?;
        let data = match data.map(|o| to_cstring(o.as_bytes())) {
            Some(Ok(string)) => Some(string),
            Some(Err(why)) => return Err(why),
            None => None,
        };

        let mut mount_data = MountData {
            c_source,
            c_target,
            flags,
            data,
        };

        let mut res = match fstype {
            FilesystemType::Auto(supported) => mount_data.automount(supported.dev_file_systems()),
            FilesystemType::Set(set) => mount_data.automount(set.iter().copied()),
            FilesystemType::Manual(fstype) => mount_data.mount(fstype),
        };

        match res {
            Ok(ref mut _mount) => {
                #[cfg(feature = "loop")]
                {
                    _mount.loopback = loopback;
                    _mount.loop_path = loop_path;
                }
            }
            Err(_) =>
            {
                #[cfg(feature = "loop")]
                if let Some(loopback) = loopback {
                    let _res = loopback.detach();
                }
            }
        }

        res
    }

    /// Perform a mount which auto-unmounts on drop.
    ///
    /// # Errors
    ///
    /// On failure to mount
    pub fn mount_autodrop(
        self,
        source: impl AsRef<Path>,
        target: impl AsRef<Path>,
        unmount_flags: UnmountFlags,
    ) -> io::Result<UnmountDrop<Mount>> {
        self.mount(source, target)
            .map(|m| m.into_unmount_drop(unmount_flags))
    }
}

struct MountData {
    c_source: Option<CString>,
    c_target: CString,
    flags: MountFlags,
    data: Option<CString>,
}

impl MountData {
    fn mount(&mut self, fstype: &str) -> io::Result<Mount> {
        let c_fstype = to_cstring(fstype.as_bytes())?;
        match mount_(
            self.c_source.as_ref(),
            &self.c_target,
            &c_fstype,
            self.flags,
            self.data.as_ref(),
        ) {
            Ok(()) => Ok(Mount::from_target_and_fstype(
                self.c_target.clone(),
                fstype.to_owned(),
            )),
            Err(why) => Err(why),
        }
    }

    fn automount<'a, I: Iterator<Item = &'a str> + 'a>(mut self, iter: I) -> io::Result<Mount> {
        let mut res = Ok(());

        for fstype in iter {
            match self.mount(fstype) {
                mount @ Ok(_) => return mount,
                Err(why) => res = Err(why),
            }
        }

        match res {
            Ok(()) => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "no supported file systems found",
            )),
            Err(why) => Err(why),
        }
    }
}

fn mount_(
    c_source: Option<&CString>,
    c_target: &CString,
    c_fstype: &CString,
    flags: MountFlags,
    c_data: Option<&CString>,
) -> io::Result<()> {
    let result = unsafe {
        mount(
            c_source.map_or_else(ptr::null, |s| s.as_ptr()),
            c_target.as_ptr(),
            c_fstype.as_ptr(),
            flags.bits(),
            c_data
                .map_or_else(ptr::null, |s| s.as_ptr())
                .cast::<libc::c_void>(),
        )
    };

    match result {
        0 => Ok(()),
        _err => Err(io::Error::last_os_error()),
    }
}
