// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT

use crate::*;

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
#[derive(smart_default::SmartDefault)]
pub struct MountBuilder<'a> {
    #[default(MountFlags::empty())]
    flags: MountFlags,
    fstype: Option<FilesystemType<'a>>,
    data: Option<&'a str>,
}

impl<'a> MountBuilder<'a> {
    /// Options to apply for the file system on mount.
    pub fn data(mut self, data: &'a str) -> Self {
        self.data = Some(data);
        self
    }

    /// The file system that is to be mounted.
    pub fn fstype(mut self, fs: impl Into<FilesystemType<'a>>) -> Self {
        self.fstype = Some(fs.into());
        self
    }

    /// Mount flags for the mount syscall.
    pub fn flags(mut self, flags: MountFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Mount the `source` to the `target`.
    pub fn mount(self, source: impl AsRef<Path>, target: impl AsRef<Path>) -> io::Result<Mount> {
        let MountBuilder {
            data,
            fstype,
            flags,
        } = self;

        let supported;

        let fstype = match fstype {
            Some(fstype) => fstype,
            None => {
                supported = SupportedFilesystems::new()?;
                FilesystemType::Auto(&supported)
            }
        };

        Mount::new(source, target, fstype, flags, data)
    }

    /// Perform a mount which auto-unmounts on drop.
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
