use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

/// Data structure for validating if a filesystem argument is valid, and used within
/// automatic file system mounting.
#[derive(Clone, Debug)]
pub struct SupportedFilesystems {
    nodev: Vec<bool>,
    fs:    Vec<String>,
}

impl SupportedFilesystems {
    pub fn new() -> io::Result<Self> {
        let mut fss = Vec::with_capacity(64);
        let mut nodevs = Vec::with_capacity(64);

        for line in BufReader::new(File::open("/proc/filesystems")?).lines() {
            let line = line?;
            let mut line = line.split_whitespace();
            let (nodev, fs) = match (line.next(), line.next()) {
                (Some(fs), None) => (false, fs),
                (Some("nodev"), Some(fs)) => (true, fs),
                _ => continue,
            };

            nodevs.push(nodev);
            fss.push(fs.to_owned());
        }

        Ok(SupportedFilesystems { nodev: nodevs, fs: fss })
    }

    /// Check if a provided file system is valid on this system.
    ///
    /// ```rust
    /// extern crate sys_mount;
    ///
    /// use sys_mount::SupportedFilesystems;
    ///
    /// fn main() {
    ///     let supports = SupportedFilesystems::new().unwrap();
    ///     println!("btrfs is {}", if supports.is_supported("btrfs") {
    ///         "supported"
    ///     } else {
    ///         "not supported"
    ///     });
    /// }
    /// ```
    pub fn is_supported(&self, fs: &str) -> bool { self.fs.iter().any(|s| s.as_str() == fs) }

    /// Iterate through file systems which are not associated with physical devices.
    pub fn nodev_file_systems<'a>(&'a self) -> Box<Iterator<Item = &str> + 'a> {
        // TODO: When we can, switch to `impl Iterator`.
        let iter = self.nodev.iter().enumerate().flat_map(move |(id, &x)| {
            if x {
                Some(self.fs[id].as_str())
            } else {
                None
            }
        });

        Box::new(iter)
    }

    /// Iterate through file systems which are associated with physical devices.
    pub fn dev_file_systems<'a>(&'a self) -> Box<Iterator<Item = &str> + 'a> {
        // TODO: When we can, switch to `impl Iterator`.
        let iter = self.nodev.iter().enumerate().flat_map(move |(id, &x)| {
            if !x {
                Some(self.fs[id].as_str())
            } else {
                None
            }
        });

        Box::new(iter)
    }
}
