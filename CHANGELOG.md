# 2.0.0 (2022-11-04)

- Improvements to loopback device mounting support
- `Mount::builder` is now the preferred way to configure mount options
- `Mount::new` has been simplified to accept only a source and dest target
    - Which will attempt to automatically guess the filesystem type
- Applied all suggestions from clippy's pedantic lints
- Updated to Rust 2021 edition, with Rust 1.65.0 features
- Updated all dependencies

# 1.2.0

Add ability to mount any file that contains a filesystem

# 1.1.0

Add `swapoff` and `Mounts` to API

# 1.0.3

Fix ISO and squashfs mounting

# 1.0.2

Fix the crate examples

# 1.0.1

Fix source type parameter in `Mount`

# 1.0.0

Intiail release
