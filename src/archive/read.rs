use chrono::naive::NaiveDateTime;
use std::{
    borrow::Cow,
    fs,
    fs::File,
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

    fn entry_type(&self) -> EntryType;

    fn is_dir(&self) -> bool {
        self.entry_type() == EntryType::Dir
    }

    fn size(&self) -> u64;

    fn compressed_size(&self) -> Option<u64> {
        None
    }

    fn modified(&self) -> Option<NaiveDateTime> {
        None
    }

    /// Extract this entry into the file system within the given path.
    ///
    /// The entire path of this entry within the archive will be recreated in
    /// the destination path.
    fn extract(&mut self, dir: &Path) -> io::Result<()> {
        let dest = dir.join(self.path());

        // Create parent directories if required.
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        match self.entry_type() {
            EntryType::Dir => fs::create_dir(dest)?,
            EntryType::File => {
                // Create the file and stream this entry's bytes into it.
                let mut file = File::create(dest)?;
                io::copy(self, &mut file)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryType {
    File,
    Dir,
}
