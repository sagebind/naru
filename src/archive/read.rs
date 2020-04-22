use super::EntryType;
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

    /// Get the metadata for this entry.
    fn metadata(&self) -> super::Metadata;

    /// If this entry is a symbolic link of some kind, get the target path that
    /// the link points to.
    fn read_link(&mut self) -> io::Result<Option<Cow<'_, Path>>> {
        Ok(None)
    }

    /// Extract this entry into the file system within the given path.
    ///
    /// The entire path of this entry within the archive will be recreated in
    /// the destination path.
    fn extract(&mut self, dir: &Path) -> io::Result<()> {
        let path = self.path();
        let dest = dir.join(&path);

        // TODO: What if path points to a directory above the archive? (security)

        // Create parent directories if required.
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        let metadata = self.metadata();

        match metadata.entry_type {
            EntryType::Directory => fs::create_dir(dest)?,
            EntryType::File => {
                // Create the file and stream this entry's bytes into it.
                let mut file = File::create(dest)?;
                io::copy(self, &mut file)?;
            }
            _ => {
                log::warn!("skipping entry {}, unsupported type", path.display())
            }
        }

        Ok(())
    }
}
