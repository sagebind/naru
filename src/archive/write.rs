use std::{
    io::{
        Read,
        Result,
        Write,
    },
    path::Path,
};

/// An incremental writer for some archive format.
pub trait ArchiveWriter {
    fn add_file(&mut self, path: &Path, file: &mut dyn Read) -> Result<()>;

    fn add_directory(&mut self, path: &Path) -> Result<()>;

    fn finish(&mut self) -> Result<()>;
}
