use supported::SupportedFilesystems;

/// Defines how the file system type should be derived for a mount -- auto or manual
#[derive(Debug)]
pub enum FilesystemType<'a> {
    /// The automatic variant will iterate through a list of pre-generated supported
    /// file systems, and attempt to mount each one before giving up.
    Auto(&'a SupportedFilesystems),
    /// The caller can avoid costly trial-and-error iteration with this variant.
    Manual(&'a str),
    /// A specific set of file systems to attempt to mount with.
    Set(&'a [&'a str]),
}

impl<'a> From<&'a SupportedFilesystems> for FilesystemType<'a> {
    fn from(s: &'a SupportedFilesystems) -> Self { FilesystemType::Auto(s) }
}

impl<'a> From<&'a str> for FilesystemType<'a> {
    fn from(s: &'a str) -> Self { FilesystemType::Manual(s) }
}

impl<'a> From<&'a [&'a str]> for FilesystemType<'a> {
    fn from(s: &'a [&'a str]) -> Self { FilesystemType::Set(s) }
}
