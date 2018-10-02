use supported::SupportedFilesystems;

/// Defines how the file system type should be derived for a mount -- auto or manual
pub enum FilesystemType<'a> {
    /// The automatic variant will iterate through a list of pre-generated supported
    /// file systems, and attempt to mount each one before giving up.
    Auto(&'a SupportedFilesystems),
    /// The caller can avoid costly trial-and-error iteration with this variant.
    Manual(&'a str),
}

impl<'a> From<&'a SupportedFilesystems> for FilesystemType<'a> {
    fn from(s: &'a SupportedFilesystems) -> Self {
        FilesystemType::Auto(s)
    }
}

impl<'a> From<&'a str> for FilesystemType<'a> {
    fn from(s: &'a str) -> Self {
        FilesystemType::Manual(s)
    }
}