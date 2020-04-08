use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read, Result, Seek, SeekFrom},
    path::Path,
    mem::ManuallyDrop,
};

/// An input stream that might be seekable.
pub struct Input(BufReader<File>);

impl Input {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if path.to_str() == Some("-") {
            Self::stdin()
        } else {
            Ok(Self::from(File::open(path)?))
        }
    }

    #[cfg(unix)]
    pub fn stdin() -> Result<Self> {
        use std::os::unix::io::*;

        unsafe fn dup(fd: RawFd) -> Result<File> {
            Ok(ManuallyDrop::new(File::from_raw_fd(fd)).try_clone()?)
        }

        unsafe {
            Ok(Self::from(dup(io::stdin().as_raw_fd())?))
        }
    }

    #[cfg(windows)]
    pub fn stdin() -> Result<Self> {
        use std::os::windows::io::*;

        unsafe fn dup(handle: RawHandle) -> Result<File> {
            Ok(ManuallyDrop::new(File::from_raw_handle(handle)).try_clone()?)
        }

        unsafe {
            Ok(Self::from(dup(io::stdin().as_raw_handle())?))
        }
    }

    /// Check if this input is actually seekable.
    pub fn is_seekable(&mut self) -> bool {
        self.0.seek(SeekFrom::Current(0)).is_ok()
    }
}

impl From<File> for Input {
    fn from(file: File) -> Self {
        Self(BufReader::new(file))
    }
}

impl BufRead for Input {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        self.0.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.0.consume(amt);
    }
}

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl Seek for Input {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.0.seek(pos)
    }
}
