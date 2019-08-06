use super::to_cstring;
use fstype::FilesystemType;
use libc::*;
use loopdev::{LoopControl, LoopDevice};
use std::{
    ffi::CString,
    io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    ptr,
};
use umount::{unmount_, Unmount, UnmountDrop, UnmountFlags};

bitflags! {
    /// Flags which may be specified when mounting a file system.
    pub struct MountFlags: c_ulong {
        /// Perform a bind mount, making a file or a directory subtree visible at another
        /// point within a file system. Bind mounts may cross file system boundaries and
        /// span chroot(2) jails. The filesystemtype and data arguments are ignored. Up
        /// until Linux 2.6.26, mountflags was also ignored (the bind mount has the same
        /// mount options as the underlying mount point).
        const BIND = MS_BIND;

        /// Make directory changes on this file system synchronous.(This property can be
        /// obtained for individual directories or subtrees using chattr(1).)
        const DIRSYNC = MS_DIRSYNC;

        /// Permit mandatory locking on files in this file system. (Mandatory locking must
        /// still be enabled on a per-file basis, as described in fcntl(2).)
        const MANDLOCK = MS_MANDLOCK;

        /// Move a subtree. source specifies an existing mount point and target specifies
        /// the new location. The move is atomic: at no point is the subtree unmounted.
        /// The filesystemtype, mountflags, and data arguments are ignored.
        const MOVE = MS_MOVE;

        /// Do not update access times for (all types of) files on this file system.
        const NOATIME = MS_NOATIME;

        /// Do not allow access to devices (special files) on this file system.
        const NODEV = MS_NODEV;

        /// Do not update access times for directories on this file system. This flag provides
        /// a subset of the functionality provided by MS_NOATIME; that is, MS_NOATIME implies
        /// MS_NODIRATIME.
        const NODIRATIME = MS_NODIRATIME;

        /// Do not allow programs to be executed from this file system.
        const NOEXEC = MS_NOEXEC;

        /// Do not honor set-user-ID and set-group-ID bits when executing programs from this
        /// file system.
        const NOSUID = MS_NOSUID;

        /// Mount file system read-only.
        const RDONLY = MS_RDONLY;

        /// When a file on this file system is accessed, only update the file's last access
        /// time (atime) if the current value of atime is less than or equal to the file's
        /// last modification time (mtime) or last status change time (ctime). This option is
        /// useful for programs, such as mutt(1), that need to know when a file has been read
        /// since it was last modified. Since Linux 2.6.30, the kernel defaults to the behavior
        /// provided by this flag (unless MS_NOATIME was specified), and the MS_STRICTATIME
        /// flag is required to obtain traditional semantics. In addition, since Linux 2.6.30,
        /// the file's last access time is always updated if it is more than 1 day old.
        const RELATIME = MS_RELATIME;

        /// Remount an existing mount. This allows you to change the mountflags and data of an
        /// existing mount without having to unmount and remount the file system. target should
        /// be the same value specified in the initial mount() call; source and filesystemtype
        /// are ignored.
        ///
        /// The following mountflags can be changed: MS_RDONLY, MS_SYNCHRONOUS, MS_MANDLOCK;
        /// before kernel 2.6.16, the following could also be changed: MS_NOATIME and
        /// MS_NODIRATIME; and, additionally, before kernel 2.4.10, the following could also
        /// be changed: MS_NOSUID, MS_NODEV, MS_NOEXEC.
        const REMOUNT = MS_REMOUNT;

        /// Suppress the display of certain (printk()) warning messages in the kernel log.
        /// This flag supersedes the misnamed and obsolete MS_VERBOSE flag (available
        /// since Linux 2.4.12), which has the same meaning.
        const SILENT = MS_SILENT;

        /// Always update the last access time (atime) when files on this file system are
        /// accessed. (This was the default behavior before Linux 2.6.30.) Specifying this
        /// flag overrides the effect of setting the MS_NOATIME and MS_RELATIME flags.
        const STRICTATIME = MS_STRICTATIME;

        /// Make writes on this file system synchronous (as though the O_SYNC flag to
        /// open(2) was specified for all file opens to this file system).
        const SYNCHRONOUS = MS_SYNCHRONOUS;
    }
}

/// Handle for managing a mounted file system.
#[derive(Debug)]
pub struct Mount {
    pub(crate) target: CString,
    pub(crate) fstype: String,
    loopback:          Option<LoopDevice>,
    loop_path:         Option<PathBuf>,
}

impl Unmount for Mount {
    fn unmount(&self, flags: UnmountFlags) -> io::Result<()> {
        unsafe {
            unmount_(self.target.as_ptr(), flags)?;
            if let Some(ref loopback) = self.loopback {
                loopback.detach()?;
            }
        }

        Ok(())
    }
}

impl Mount {
    /// Mounts a file system at `source` to a `target` path in the system.
    ///
    /// ```rust,no_run
    /// extern crate sys_mount;
    ///
    /// use sys_mount::{
    ///     Mount,
    ///     MountFlags,
    ///     SupportedFilesystems
    /// };
    ///
    /// fn main() {
    ///     // Fetch a list of supported file systems.
    ///     // When mounting, a file system will be selected from this.
    ///     let supported = SupportedFilesystems::new().unwrap();
    ///
    ///     // Attempt to mount the src device to the dest directory.
    ///     let mount_result = Mount::new(
    ///         "/imaginary/block/device",
    ///         "/tmp/location",
    ///         &supported,
    ///         MountFlags::empty(),
    ///         None
    ///     );
    /// }
    /// ```
    /// # Notes
    ///
    /// The provided `source` device and `target` destinations must exist within the file system.
    ///
    /// If the `source` is a file with an `iso` or `squashfs` extension, a loopback device will
    /// be created, and the file will be associated with the loopback device. The `MountFlags`
    /// will also be modified to ensure that the `MountFlags::RDONLY` flag is set before mounting.
    ///
    /// The `fstype` parameter accepts either a `&str` or `&SupportedFilesystem` as input. If the
    /// input is a `&str`, then a particular file system will be used to mount the `source` with.
    /// If the input is a `&SupportedFilesystems`, then the file system will be selected
    /// automatically from the list.
    ///
    /// The automatic variant of `fstype` works by attempting to mount the `source` with all
    /// supported device-based file systems until it succeeds, or fails after trying all
    /// possible options.
    pub fn new<'a, S, T, F>(
        source: S,
        target: T,
        fstype: F,
        mut flags: MountFlags,
        data: Option<&str>,
    ) -> io::Result<Self>
    where
        S: AsRef<Path>,
        T: AsRef<Path>,
        F: Into<FilesystemType<'a>>,
    {
        let mut fstype = fstype.into();
        let mut loopback = None;
        let mut loop_path = None;

        let source = source.as_ref();
        let c_source = if !source.as_os_str().is_empty() {
            // Create a loopback device if an iso or squashfs is being mounted.
            if let Some(ext) = source.extension() {
                let extf = if ext == "iso" { 1 } else { 0 } | if ext == "squashfs" { 2 } else { 0 };

                if extf != 0 {
                    fstype = if extf == 1 {
                        flags |= MountFlags::RDONLY;
                        FilesystemType::Manual("iso9660")
                    } else {
                        flags |= MountFlags::RDONLY;
                        FilesystemType::Manual("squashfs")
                    };
                }
                loopback = Some(mount_loopback(source)?);
            }

            let source = match loopback {
                Some(ref loopback) => {
                    let path = loopback.path().expect("loopback does not have path");
                    let cstr = to_cstring(path.as_os_str().as_bytes())?;
                    loop_path = Some(path);
                    cstr
                }
                None => to_cstring(source.as_os_str().as_bytes())?,
            };

            Some(source)
        } else {
            None
        };

        let c_target = to_cstring(target.as_ref().as_os_str().as_bytes())?;
        let c_data = match data.map(|o| to_cstring(o.as_bytes())) {
            Some(Ok(string)) => Some(string),
            Some(Err(why)) => return Err(why),
            None => None,
        };

        let data = c_data.as_ref().map_or(ptr::null(), |d| d.as_ptr()) as *const c_void;
        let mut mount_data = MountData { c_source, c_target, flags, data };

        let mut res = match fstype {
            FilesystemType::Auto(supported) => mount_data.automount(supported.dev_file_systems()),
            FilesystemType::Set(set) => mount_data.automount(set.iter().cloned()),
            FilesystemType::Manual(fstype) => mount_data.mount(fstype),
        };

        match res {
            Ok(ref mut mount) => {
                mount.loopback = loopback;
                mount.loop_path = loop_path;
            }
            Err(_) => {
                if let Some(loopback) = loopback {
                    let _ = loopback.detach();
                }
            }
        }

        res
    }

    /// If the device was associated with a loopback device, that device's path
    /// can be retrieved here.
    pub fn backing_loop_device(&self) -> Option<&Path> {
        self.loop_path.as_ref().map(|p| p.as_path())
    }

    /// Describes the file system which this mount was mounted with.
    ///
    /// This is useful in the event that the mounted device was mounted automatically.
    pub fn get_fstype(&self) -> &str { &self.fstype }
}

struct MountData {
    c_source: Option<CString>,
    c_target: CString,
    flags:    MountFlags,
    data:     *const c_void,
}

impl MountData {
    fn mount(&mut self, fstype: &str) -> io::Result<Mount> {
        let c_fstype = to_cstring(fstype.as_bytes())?;
        match mount_(self.c_source.as_ref(), &self.c_target, &c_fstype, self.flags, self.data) {
            Ok(()) => Ok(Mount {
                target:    self.c_target.clone(),
                fstype:    fstype.to_owned(),
                loopback:  None,
                loop_path: None,
            }),
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
            Ok(()) => {
                Err(io::Error::new(io::ErrorKind::NotFound, "no supported file systems found"))
            }
            Err(why) => Err(why),
        }
    }
}

fn mount_(
    c_source: Option<&CString>,
    c_target: &CString,
    c_fstype: &CString,
    flags: MountFlags,
    c_data: *const c_void,
) -> io::Result<()> {
    let result = unsafe {
        mount(
            c_source.map_or_else(ptr::null, |s| s.as_ptr()),
            c_target.as_ptr(),
            c_fstype.as_ptr(),
            flags.bits(),
            c_data,
        )
    };

    match result {
        0 => Ok(()),
        _err => Err(io::Error::last_os_error()),
    }
}

fn mount_loopback(source: &Path) -> io::Result<LoopDevice> {
    let loopback = LoopControl::open().and_then(|ctrl| ctrl.next_free())?;
    loopback.attach_with_offset(source, 0)?;
    Ok(loopback)
}

/// An abstraction that will ensure that temporary mounts are dropped in reverse.
pub struct Mounts(pub Vec<UnmountDrop<Mount>>);

impl Mounts {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn unmount(&mut self, lazy: bool) -> io::Result<()> {
        let flags = if lazy { UnmountFlags::DETACH } else { UnmountFlags::empty() };
        self.0.iter_mut().rev().map(|mount| mount.unmount(flags)).collect()
    }
}

impl Drop for Mounts {
    fn drop(&mut self) {
        for mount in self.0.drain(..).rev() {
            drop(mount);
        }
    }
}
