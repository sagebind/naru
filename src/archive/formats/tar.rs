//! Implementation of the [tar archive] format.
//!
//! This module only handles the tar format itself, it does not handle
//! compression wrapping.
//!
//! [TAR archive]: https://en.wikipedia.org/wiki/Tar_%28computing%29

use crate::{
    archive::{ArchiveReader, Entry, EntryType, Metadata},
    input::Input,
};
use chrono::prelude::*;
use owning_ref::OwningHandle;
use std::{
    fmt,
    io::{Read, Result},
    path::Path,
};

/// Format provider for tar.
pub struct Tar;

impl super::Format for Tar {
    fn id(&self) -> &str {
        "tar"
    }

    fn file_extensions(&self) -> &[&str] {
        &["tar"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_tar(bytes)
    }
}

impl fmt::Display for Tar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("tar")
    }
}

impl super::ArchiveFormat for Tar {
    fn open(&self, input: Input) -> Result<Box<dyn ArchiveReader>> {
        Ok(Box::new(TarReader::new(input)?))
    }
}

pub struct TarReader<'r, R: Read + 'r> {
    /// An iterator over the entries of a TAR archive.
    ///
    /// This is an owning ref handle, because the only iterator type offered by
    /// the tar library is a mutably borrowing one.
    entries: OwningHandle<Box<tar::Archive<R>>, Box<tar::Entries<'r, R>>>,
}

impl<'r, R: Read + 'r> TarReader<'r, R> {
    fn new(reader: R) -> Result<Self> {
        Ok(Self {
            entries: OwningHandle::try_new(
                Box::new(tar::Archive::new(reader)),
                |archive| unsafe {
                    let archive = &mut *(archive as *mut tar::Archive<R>);
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

    fn metadata(&self) -> Metadata {
        Metadata::builder()
            .entry_type(if self.header().entry_type().is_dir() {
                EntryType::Dir
            } else {
                EntryType::File
            })
            .size(self.header().size().unwrap_or(0))
            .modified(self.header()
                .mtime()
                .ok()
                .map(|ts| Local.timestamp(ts as i64, 0)))
            .build()
    }
}
