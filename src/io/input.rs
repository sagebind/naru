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
pub struct Input<'r>(Inner<'r>);

enum Inner<'r> {
    File(BufReader<File>),
    Other(BufReader<DiskCacheReader<Box<dyn Read + 'r>>>),
}

impl<'r> Input<'r> {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if path.to_str() == Some("-") {
            Self::stdin()
        } else {
            Ok(Self::from_file(File::open(path)?))
        }
    }

    pub fn from_file(file: File) -> Self {
        Self(Inner::File(BufReader::new(file)))
    }

    pub fn from_reader(reader: impl Read + 'r) -> Result<Self> {
        let reader: Box<dyn Read + 'r> = Box::new(reader);

        Ok(Self(Inner::Other(BufReader::new(DiskCacheReader::new(reader)?))))
    }

    pub fn stdin() -> Result<Self> {
        let mut file = io::stdin().dup()?;

        // If stdin is actually a seekable file stream, then just use as-is.
        if file.seek(SeekFrom::Start(0)).is_ok() {
            Ok(Self::from_file(file))
        }
        // If stdin is a true unseekable stream (like a pipe) then wrap in a
        // caching reader.
        else {
            Self::from_reader(file)
        }
    }

    /// Make this input available as a file on disk. This is used when a reader
    /// implementation either requires the ability to seek or that the input is
    /// on disk.
    pub fn into_file(mut self) -> Result<File> {
        match self.0 {
            // Already a file
            Inner::File(file) => Ok(file.into_inner()),

            // Allocate a temporary file and drain the reader into it
            Inner::Other(mut reader) => {
                let mut file = tempfile::tempfile()?;
                io::copy(&mut reader, &mut file)?;
                file.seek(SeekFrom::Start(0))?;
                Ok(file)
            }
        }
    }
}

impl TryFrom<Input<'_>> for File {
    type Error = io::Error;

    fn try_from(input: Input<'_>) -> Result<File> {
        input.into_file()
    }
}

impl BufRead for Input<'_> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        match &mut self.0 {
            Inner::File(file) => file.fill_buf(),
            Inner::Other(reader) => reader.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match &mut self.0 {
            Inner::File(file) => file.consume(amt),
            Inner::Other(reader) => reader.consume(amt),
        }
    }
}

impl Read for Input<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match &mut self.0 {
            Inner::File(file) => file.read(buf),
            Inner::Other(reader) => reader.read(buf),
        }
    }
}

impl Seek for Input<'_> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match &mut self.0 {
            Inner::File(file) => file.seek(pos),
            Inner::Other(reader) => reader.seek(pos),
        }
    }
}
