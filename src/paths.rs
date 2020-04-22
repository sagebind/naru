use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::ffi::*;

/// Create a native `Path` representation compatible with the current platform
/// from the given bytes that represent a UNIX-specific path.
///
/// This is often used for archive formats with UNIX-like systems in mind, where
/// the path is simply a sequence of bytes as they are in UNIX file systems.
/// These must be converted in order to be usable on non-UNIX platforms.
pub fn path_from_unix_path_bytes(bytes: Cow<'_, [u8]>) -> Cow<'_, Path> {
    // On UNIX-like systems the bytes are already in native representation.
    #[cfg(unix)]
    match bytes {
        Cow::Borrowed(bytes) => Cow::Borrowed(Path::new(OsStr::from_bytes(bytes))),
        Cow::Owned(bytes) => Cow::Owned(PathBuf::from(OsString::from_vec(bytes))),
    }

    // On other platforms, not all paths may be representable so we just
    // approximate it so the entry can still be accessed.
    #[cfg(not(unix))]
    match bytes {
        Cow::Borrowed(bytes) => match String::from_utf8_lossy(bytes) {
            Cow::Borrowed(s) => Cow::Borrowed(Path::new(s)),
            Cow::Owned(s) => Cow::Owned(PathBuf::from(s)),
        },
        Cow::Owned(bytes) => Cow::Owned(PathBuf::from(String::from_utf8_lossy(&bytes).into_owned())),
    }
}
