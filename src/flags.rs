// Copyright 2018-2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

use libc::{
    c_int, c_ulong, MNT_DETACH, MNT_EXPIRE, MNT_FORCE, MS_BIND, MS_DIRSYNC, MS_MANDLOCK, MS_MOVE,
    MS_NOATIME, MS_NODEV, MS_NODIRATIME, MS_NOEXEC, MS_NOSUID, MS_PRIVATE, MS_RDONLY, MS_REC,
    MS_RELATIME, MS_REMOUNT, MS_SHARED, MS_SILENT, MS_SLAVE, MS_STRICTATIME, MS_SYNCHRONOUS,
    MS_UNBINDABLE, O_NOFOLLOW,
};

bitflags! {
    /// Flags which may be specified when mounting a file system.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

        /// Used in conjunction with MS_BIND to create a recursive bind mount, and in
        /// conjunction with the propagation type flags to recursively change the propagation
        /// type of all of the mounts in a subtree.
        const REC = MS_REC;

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

bitflags! {
    /// Propagation type flags which may be specified after mounting a file system to specify how mount
    /// events are propagated.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PropagationType: c_ulong {
        /// The mount is in a peer group, it can be replicated to as many mountpoints, and all replicas are identical
        /// (events are propagated to other peer mounts).
        const SHARED = MS_SHARED;

        /// The mount can receive propagated mount events from its parent (peer group), but cannot propagate mount
        /// events to the peer group.
        const SLAVE = MS_SLAVE;

        /// The mount is private, it neither receives or sends any mount events
        const PRIVATE = MS_PRIVATE;

        /// The mount is private and cannot be used as a bind mount source.
        const UNBINDABLE = MS_UNBINDABLE;
    }
}

bitflags! {
    /// Flags which may be specified when unmounting a file system.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
