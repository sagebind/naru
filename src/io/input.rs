use super::{
    Dup,
    buffers::DiskCacheReader,
};
use std::{
    convert::TryFrom,
    fs::File,
    io::{self, BufRead, BufReader, Read, Result, Seek, SeekFrom},
    path::{Path, PathBuf},
};

/// An input stream that might be seekable and might have a file path.
///
/// This type is used to abstract over multiple kinds of file sources.
pub struct Input {
    path: Option<PathBuf>,
    file: Option<BufReader<File>>,
    dynamic: Option<BufReader<DiskCacheReader<Box<dyn Read>>>>,
}

impl Input {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if path.to_str() == Some("-") {
            Self::stdin()
        } else {
            Ok(Self {
                path: Some(path.to_owned()),
                file: Some(BufReader::new(File::open(path)?)),
                dynamic: None,
            })
        }
    }

    pub fn stdin() -> Result<Self> {
        let mut file = io::stdin().dup()?;

        // If stdin is actually a seekable file stream, then just use as-is.
        if file.seek(SeekFrom::Start(0)).is_ok() {
            Ok(Self {
                path: None,
                file: Some(BufReader::new(file)),
                dynamic: None,
            })
        }
        // If stdin is a true unseekable stream (like a pipe) then wrap in a
        // caching reader.
        else {
            Ok(Self {
                path: None,
                file: None,
                dynamic: Some(BufReader::new(DiskCacheReader::new(
                    Box::new(
                        BufReader::new(file)
                    ) as Box<dyn Read>
                )?)),
            })
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn try_wrap<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(Box<dyn Read>) -> Result<Box<dyn Read>>,
    {
        let inner: Box<dyn Read> = if let Some(file) = self.file.take() {
            Box::new(file)
        } else {
            Box::new(self.dynamic.take().unwrap())
        };

        self.dynamic = Some(BufReader::new(DiskCacheReader::new(f(inner)?)?));

        Ok(())
    }
}

impl TryFrom<Input> for File {
    type Error = Input;

    fn try_from(input: Input) -> std::result::Result<File, Input> {
        if let Some(file) = input.file {
            Ok(file.into_inner())
        } else {
            Err(input)
        }
    }
}

impl BufRead for Input {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if let Some(file) = self.file.as_mut() {
            file.fill_buf()
        } else {
            self.dynamic.as_mut().unwrap().fill_buf()
        }
    }

    fn consume(&mut self, amt: usize) {
        if let Some(file) = self.file.as_mut() {
            file.consume(amt)
        } else {
            self.dynamic.as_mut().unwrap().consume(amt)
        }
    }
}

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if let Some(file) = self.file.as_mut() {
            file.read(buf)
        } else {
            self.dynamic.as_mut().unwrap().read(buf)
        }
    }
}

impl Seek for Input {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        if let Some(file) = self.file.as_mut() {
            file.seek(pos)
        } else {
            self.dynamic.as_mut().unwrap().seek(pos)
        }
    }
}
