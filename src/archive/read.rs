use std::{
    borrow::Cow,
    io,
    io::Read,
    path::Path,
};

/// An incremental reader for some archive format.
pub trait ArchiveReader {
    /// Get the number of entries in this archive, if known.
    ///
    /// Not all formats and parsers are able to count the number of files in the
    /// archive via metadata (or by scanning).
    fn len(&mut self) -> Option<u64> {
        None
    }

    /// Read the next entry in this archive.
    fn entry(&mut self) -> io::Result<Option<Box<dyn Entry + '_>>>;
}

/// An entry in an archive being read.
pub trait Entry: Read {
    /// Get the full path of this entry, relative to the root of the archive.
    fn path(&self) -> Cow<'_, Path>;

    /// Get the metadata for this entry.
    fn metadata(&self) -> super::Metadata;

    /// If this entry is a symbolic link of some kind, get the target path that
    /// the link points to.
    fn read_link(&mut self) -> io::Result<Option<Cow<'_, Path>>> {
        Ok(None)
    }
}
