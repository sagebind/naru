use super::{
    Dup,
    buffers::DiskCacheWriter,
};
use std::{
    fs::File,
    io::{self, BufWriter, Result, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

pub struct Output(Inner);

enum Inner {
    Direct(BufWriter<File>, Option<PathBuf>),
    Cached(BufWriter<DiskCacheWriter<BufWriter<File>>>),
}

impl Output {
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if path.to_str() == Some("-") {
            Self::stdout()
        } else {
            Ok(Self(Inner::Direct(
                BufWriter::new(File::create(path)?),
                Some(path.to_owned()),
            )))
        }
    }

    pub fn stdout() -> Result<Self> {
        let mut file = io::stdout().dup()?;

        // If stdout is actually a seekable file stream, then just use as-is.
        if file.seek(SeekFrom::Start(0)).is_ok() {
            Ok(Self(Inner::Direct(
                BufWriter::new(file),
                None,
            )))
        }
        // If stdout is a true unseekable stream (like a pipe) then wrap in a
        // caching reader.
        else {
            Ok(Self(Inner::Cached(BufWriter::new(DiskCacheWriter::new(BufWriter::new(file))?))))
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match &self.0 {
            Inner::Direct(_, path) => path.as_deref(),
            _ => None,
        }
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match &mut self.0 {
            Inner::Direct(writer, _) => writer.write(buf),
            Inner::Cached(writer) => writer.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match &mut self.0 {
            Inner::Direct(writer, _) => writer.flush(),
            Inner::Cached(writer) => writer.flush(),
        }
    }
}

impl Seek for Output {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match &mut self.0 {
            Inner::Direct(writer, _) => writer.seek(pos),
            Inner::Cached(writer) => writer.seek(pos),
        }
    }
}
