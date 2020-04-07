use chrono::naive::NaiveDateTime;
use std::{
    path::PathBuf,
};

/// An incremental reader for some archive format.
pub trait ArchiveReader {
    /// Get the number of entries in this archive, if known.
    ///
    /// Not all formats and parsers are able to count the number of files in the
    /// archive via metadata (or by scanning).
    fn len(&self) -> Option<u64> {
        None
    }

    fn entry(&mut self) -> Option<EntryMetadata>;
}

/// An incremental writer for some archive format.
pub trait ArchiveWriter {
}

#[derive(Clone, Debug)]
pub struct EntryMetadata {
    pub name: PathBuf,
    pub file_type: EntryType,
    pub size: u64,
    pub compressed_size: Option<u64>,
    pub modified: NaiveDateTime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryType {
    File,
    Dir,
}
