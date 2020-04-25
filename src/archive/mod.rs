//! Archive format APIs for reading and writing.

use crate::{
    compress,
    input::Input,
    output::Output
};
use chrono::prelude::*;
use std::{
    fs,
    io::{BufRead, Result},
};

pub mod formats;
mod read;
mod write;

pub use self::{read::*, write::*};

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
    Directory,

    /// A symbolic link to another file by name.
    SymbolicLink,

    /// Some other type of file not supported by Naru.
    Unsupported,
}

impl Default for EntryType {
    fn default() -> Self {
        Self::File
    }
}

impl From<fs::FileType> for EntryType {
    fn from(file_type: fs::FileType) -> Self {
        if file_type.is_dir() {
            Self::Directory
        } else {
            Self::File
        }
    }
}

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

    /// Flag indicating that this file is marked as read-only.
    ///
    /// The meaning of this flag can vary depending on the file system, archive
    /// format, or operating system that the entry came from.
    #[builder(default)]
    pub read_only: bool,

    /// Flag indicating that this file is marked as hidden.
    ///
    /// The meaning of this flag can vary depending on the file system, archive
    /// format, or operating system that the entry came from.
    #[builder(default)]
    pub hidden: bool,

    /// UNIX-mode permissions and file attributes.
    ///
    /// While files can only be extracted and inherit modes correctly on UNIX
    /// platforms, the information can certainly be visible on any platform.
    #[builder(default)]
    pub unix_mode: Option<u32>,
}

impl Metadata {
    pub fn is_dir(&self) -> bool {
        self.entry_type == EntryType::Directory
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

/// Attempt to read the given input stream as an archive file.
///
/// If the stream has any stream compression algorithms applied, this function
/// will first attempt to decode them first.
pub fn open<'r>(mut input: Input<'r>) -> Result<Option<Box<dyn ArchiveReader + 'r>>> {
    // Automatically decode any compression streams first.
    input = compress::detect_decode(input)?;

    // Detect archive format.
    for format in formats::all() {
        if format.match_bytes(input.fill_buf()?) {
            log::debug!("detected {} archive", format.id());
            return Ok(Some(format.open(input)?));
        }
    }

    Ok(None)
}

pub fn create<'o>(output: &'o mut Output) -> Result<Option<Box<dyn ArchiveWriter + 'o>>> {
    if let Some(path) = output.path() {
        if let Some(format) = formats::for_extension(path) {
            return Ok(Some(format.create(output)?));
        }
    }

    Ok(None)
}
