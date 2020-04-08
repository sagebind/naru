//! Implementation of the Unix [ar] file format.
//!
//! [ar]: https://en.wikipedia.org/wiki/Ar_(Unix)

use crate::{
    archive::{ArchiveReader, Entry, EntryType},
    input::Input,
};
use chrono::naive::NaiveDateTime;
use std::{
    borrow::Cow,
    io::{Read, Result, Seek},
    path::{Path, PathBuf},
};

/// Format provider for AR.
pub struct Ar;

impl super::Format for Ar {
    fn file_extensions(&self) -> &[&str] {
        &["a", "ar", "deb", "lib"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_ar(bytes)
    }

    fn open(&self, input: Input) -> Result<Box<dyn ArchiveReader>> {
        Ok(Box::new(ArReader::new(input)))
    }
}

pub struct ArReader<R: Read + Seek> {
    archive: ar::Archive<R>,
}

impl<R: Read + Seek> ArReader<R> {
    fn new(reader: R) -> Self {
        Self {
            archive: ar::Archive::new(reader),
        }
    }
}

impl<R: Read + Seek> ArchiveReader for ArReader<R> {
    fn len(&mut self) -> Option<u64> {
        self.archive.count_entries().ok().map(|l| l as u64)
    }

    fn entry(&mut self) -> Result<Option<Box<dyn Entry + '_>>> {
        Ok(self.archive.next_entry().transpose()?.map(|e| Box::new(e) as Box<dyn Entry + '_>))
    }
}

impl<'r, R: Read + Seek> Entry for ar::Entry<'r, R> {
    fn path(&self) -> Cow<'_, Path> {
        match String::from_utf8_lossy(self.header().identifier()) {
            Cow::Borrowed(s) => Cow::Borrowed(Path::new(s)),
            Cow::Owned(s) => Cow::Owned(PathBuf::from(s)),
        }
    }

    fn entry_type(&self) -> EntryType {
        EntryType::File
    }

    fn size(&self) -> u64 {
        self.header().size()
    }

    fn modified(&self) -> Option<NaiveDateTime> {
        Some(NaiveDateTime::from_timestamp(self.header().mtime() as i64, 0))
    }
}
