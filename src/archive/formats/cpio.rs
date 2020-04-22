//! The CPIO family of file formats.
//!
//! According to the [man page][cpio(5)], there are a few different variants of
//! the CPIO format. The `cpio` crate used here only implements the "New ASCII"
//! format (newc/SVR4).
//!
//! [cpio(5)]: https://www.freebsd.org/cgi/man.cgi?query=cpio&sektion=5

use crate::{
    archive::{ArchiveReader, Entry, EntryType, Metadata},
    input::Input,
};
use chrono::prelude::*;
use std::{
    borrow::Cow,
    fmt,
    io::{Read, Result, Seek},
    path::{Path, PathBuf},
};

/// Format provider for CPIO archives.
pub struct Cpio;

impl super::Format for Cpio {
    fn id(&self) -> &str {
        "cpio"
    }

    fn file_extensions(&self) -> &[&str] {
        &["cpio"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        matches!(bytes, [0x30, 0x37, 0x30, 0x37, 0x30, 0x31, ..])
    }
}

impl fmt::Display for Cpio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("cpio (newc)")
    }
}

impl super::ArchiveFormat for Cpio {
    fn open(&self, input: Input) -> Result<Box<dyn ArchiveReader>> {
        Ok(Box::new(CpioReader {
            init: Some(input),
            entry: None,
        }))
    }
}

struct CpioReader<R: Read> {
    init: Option<R>,
    entry: Option<cpio::newc::Reader<R>>,
}

impl<R: Read> ArchiveReader for CpioReader<R> {
    fn entry(&mut self) -> Result<Option<Box<dyn Entry + '_>>> {
        // Finish previously returned entry, if any.
        let reader = if let Some(previous) = self.entry.take() {
            previous.finish()?
        } else if let Some(reader) = self.init.take() {
            reader
        } else {
            return Ok(None);
        };

        let reader = cpio::newc::Reader::new(reader)?;

        // If we reach a special entry named `TRAILER!!!`, then we've reached
        // the end of the archive.
        if reader.entry().name() == "TRAILER!!!" {
            return Ok(None);
        }

        self.entry = Some(reader);
        Ok(self.entry.as_mut().map(|r| Box::new(r) as Box<dyn Entry>))
    }
}

impl<'r, R: Read> Entry for &'r mut cpio::newc::Reader<R> {
    fn path(&self) -> Cow<'_, Path> {
        Cow::Borrowed(Path::new(self.entry().name()))
    }

    fn metadata(&self) -> Metadata {
        Metadata::builder()
            .entry_type(match self.entry().mode() & 0o0170000 {
                0o0100000 => EntryType::File,
                0o0040000 => EntryType::Directory,
                _ => todo!("account for 'unknown' entry types"),
            })
            .size(self.entry().file_size().into())
            .modified(if self.entry().mtime() > 0 {
                Some(Local.timestamp(self.entry().mtime() as i64, 0))
            } else {
                None
            })
            .unix_mode(Some(self.entry().mode() & 0o0000777))
            .build()
    }
}
