use crate::archive::{ArchiveReader, Entry, EntryType};
use chrono::naive::NaiveDateTime;
use std::{
    io::{Read, Result},
    path::Path,
};
use tar::{Archive, Entries};

pub struct TarReader<'r, R: Read + 'r> {
    archive: Archive<R>,
    entries: Entries<'r, R>,
}

impl<'r, R: Read + 'r> TarReader<'r, R> {
    fn new(reader: R) -> Result<Self> {
        let mut archive = Archive::new(reader);
        let entries = archive.entries()?;

        Ok(Self {
            archive,
            entries,
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
