use std::{
    fs::File,
    io::Result,
    mem::ManuallyDrop,
};

pub mod buffers;
pub mod input;
pub mod output;

/// Helper trait for file-like handles that can be duplicated.
trait Dup {
    /// Try to duplicate this file descriptor.
    fn dup(&self) -> Result<File>;
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::os::unix::io::*;

    impl<T: AsRawFd> Dup for T {
        fn dup(&self) -> Result<File> {
            ManuallyDrop::new(unsafe {
                File::from_raw_fd(self.as_raw_fd())
            }).try_clone()
        }
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::os::windows::io::*;

    impl<T: AsRawHandle> Dup for T {
        fn dup(&self) -> Result<File> {
            ManuallyDrop::new(unsafe {
                File::from_raw_handle(self.as_raw_handle())
            }).try_clone()
        }
    }
}

