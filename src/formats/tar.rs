//! Implementation of the [tar archive] format.
//!
//! This module only handles the tar format itself, it does not handle
//! compression wrapping.
//!
//! [TAR archive]: https://en.wikipedia.org/wiki/Tar_%28computing%29

use crate::{
    archive::{ArchiveReader, Entry, EntryType},
    io::Input,
};
use chrono::naive::NaiveDateTime;
use std::{
    io::{Read, Result},
    path::Path,
};
use tar::{Archive, Entries};

/// Format provider for tar.
pub struct Tar;

impl super::Format for Tar {
    fn file_extensions(&self) -> &[&str] {
        &["tar"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_tar(bytes)
    }

    fn open(&self, input: Input) -> Result<Box<dyn ArchiveReader>> {
        Ok(Box::new(TarReader::new(input)?))
    }
}

pub struct TarReader<'r, R: Read + 'r> {
    /// An iterator over the entries of a TAR archive.
    ///
    /// This is an owning ref handle, because the only iterator type offered by
    /// the tar library is a mutably borrowing one.
    entries: owning_ref::OwningHandle<Box<Archive<R>>, Box<Entries<'r, R>>>,
}

impl<'r, R: Read + 'r> TarReader<'r, R> {
    fn new(reader: R) -> Result<Self> {
        Ok(Self {
            entries: owning_ref::OwningHandle::try_new(
                Box::new(Archive::new(reader)),
                |archive| unsafe {
                    let archive = &mut *(archive as *mut Archive<R>);
                    archive.entries().map(Box::new)
                },
            )?,
        })
    }
}

impl<'r, R: Read + 'r> ArchiveReader for TarReader<'r, R> {
    fn entry(&mut self) -> Result<Option<Box<dyn Entry + '_>>> {
        Ok(self.entries.next().transpose()?.map(|e| Box::new(e) as Box<dyn Entry + '_>))
    }
}

impl<'r, R: Read + 'r> Entry for tar::Entry<'r, R> {
    fn path(&self) -> std::borrow::Cow<'_, Path> {
        self.path().unwrap()
    }

    fn entry_type(&self) -> EntryType {
        if self.header().entry_type().is_dir() {
            EntryType::Dir
        } else {
            EntryType::File
        }
    }

    fn size(&self) -> u64 {
        self.header().size().unwrap_or(0)
    }

    fn modified(&self) -> Option<NaiveDateTime> {
        self.header()
            .mtime()
            .ok()
            .map(|ts| NaiveDateTime::from_timestamp(ts as i64, 0))
    }
}
