//! Archive format APIs for reading and writing.

use crate::{
    input::Input,
    output::Output,
};
use chrono::prelude::*;
use std::{
    fs,
    io::{BufRead, Result},
};

pub mod formats;
mod read;
mod write;

pub use self::{
    read::*,
    write::*,
};

#[derive(Clone, Debug, Default, Eq, PartialEq, TypedBuilder)]
pub struct Metadata {
    /// The type of entry this metadata represents.
    pub entry_type: EntryType,

    /// The size of the file in bytes.
    ///
    /// For other entry types like directories, will likely be zero and isn't
    /// used.
    #[builder(default = 0)]
    pub size: u64,

    /// If the enclosing archive supports inherent compression, the compressed
    /// size taken up by this file in bytes.
    ///
    /// For other entry types like directories, will likely be zero and isn't
    /// used.
    #[builder(default)]
    pub compressed_size: Option<u64>,

    /// Timestamp of when the entry was last modified.
    #[builder(default)]
    pub modified: Option<DateTime<Local>>,
}

impl Metadata {
    pub fn is_dir(&self) -> bool {
        self.entry_type == EntryType::Dir
    }
}

impl From<fs::Metadata> for Metadata {
    fn from(metadata: fs::Metadata) -> Self {
        Self::builder()
            .entry_type(metadata.file_type().into())
            .size(metadata.len())
            .modified(metadata.modified().ok().map(From::from))
            .build()
    }
}

/// Possible entry types in an archive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryType {
    /// A regular file.
    File,

    /// A regular directory.
    ///
    /// Directories do not contain data and are purely informational. An archive
    /// could contain nested paths even if there are no corresponding directory
    /// entries for them.
    Dir,
}

impl Default for EntryType {
    fn default() -> Self {
        Self::File
    }
}

impl From<fs::FileType> for EntryType {
    fn from(file_type: fs::FileType) -> Self {
        if file_type.is_dir() {
            Self::Dir
        } else {
            Self::File
        }
    }
}

pub fn open(mut input: Input) -> Result<Option<Box<dyn ArchiveReader>>> {
    if let Some(format) = formats::for_bytes(input.fill_buf()?) {
        Ok(Some(format.open(input)?))
    } else {
        Ok(None)
    }
}

pub fn create<'o>(output: &'o mut Output) -> Result<Option<Box<dyn ArchiveWriter + 'o>>> {
    if let Some(path) = output.path() {
        if let Some(format) = formats::for_extension(path) {
            return Ok(Some(format.create(output)?));
        }
    }

    Ok(None)
}
