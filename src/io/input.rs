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
pub struct Input(Inner);

enum Inner {
    Direct(BufReader<File>, Option<PathBuf>),
    Cached(BufReader<DiskCacheReader<BufReader<File>>>),
}

impl Input {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if path.to_str() == Some("-") {
            Self::stdin()
        } else {
            Ok(Self(Inner::Direct(
                BufReader::new(File::open(path)?),
                Some(path.to_owned()),
            )))
        }
    }

    pub fn stdin() -> Result<Self> {
        let mut file = io::stdin().dup()?;

        // If stdin is actually a seekable file stream, then just use as-is.
        if file.seek(SeekFrom::Start(0)).is_ok() {
            Ok(Self(Inner::Direct(
                BufReader::new(file),
                None,
            )))
        }
        // If stdin is a true unseekable stream (like a pipe) then wrap in a
        // caching reader.
        else {
            Ok(Self(Inner::Cached(BufReader::new(DiskCacheReader::new(BufReader::new(file))?))))
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match &self.0 {
            Inner::Direct(_, path) => path.as_deref(),
            _ => None,
        }
    }
}

impl TryFrom<Input> for File {
    type Error = Input;

    fn try_from(input: Input) -> std::result::Result<File, Input> {
        match input.0 {
            Inner::Direct(reader, _) => Ok(reader.into_inner()),
            _ => Err(input),
        }
    }
}

impl BufRead for Input {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        match &mut self.0 {
            Inner::Direct(reader, _) => reader.fill_buf(),
            Inner::Cached(reader) => reader.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match &mut self.0 {
            Inner::Direct(reader, _) => reader.consume(amt),
            Inner::Cached(reader) => reader.consume(amt),
        }
    }
}

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match &mut self.0 {
            Inner::Direct(reader, _) => reader.read(buf),
            Inner::Cached(reader) => reader.read(buf),
        }
    }
}

impl Seek for Input {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match &mut self.0 {
            Inner::Direct(reader, _) => reader.seek(pos),
            Inner::Cached(reader) => reader.seek(pos),
        }
    }
}
