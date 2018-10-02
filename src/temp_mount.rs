use mount::Mount;
use std::ops::Deref;
use umount::{unmount_, UnmountFlags};

/// Unmounts the underlying mounted device upon drop.
pub struct TempMount {
    pub(crate) mount: Mount,
    pub(crate) unmount_flags: UnmountFlags,
}

impl Deref for TempMount {
    type Target = Mount;
    fn deref(&self) -> &Mount {
        &self.mount
    }
}

impl Drop for TempMount {
    fn drop(&mut self) {
        let _ = unsafe { unmount_(self.mount.target.as_ptr(), self.unmount_flags) };
    }
}