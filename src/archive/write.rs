use super::Metadata;
use std::{
    io::{Read, Result},
    path::Path,
};

/// An incremental writer for some archive format.
pub trait ArchiveWriter {
    /// Add a directory to the archive.
    ///
    /// The path for the directory specified is relative to the root of the
    /// archive.
    fn add_directory(&mut self, path: &Path, metadata: Metadata) -> Result<()>;

    /// Add a file to the archive from a byte stream.
    ///
    /// The path for the directory specified is relative to the root of the
    /// archive.
    fn add_file(&mut self, path: &Path, metadata: Metadata, file: &mut dyn Read) -> Result<()>;

    ///Finish writing the archive.
    fn finish(&mut self) -> Result<()>;
}
