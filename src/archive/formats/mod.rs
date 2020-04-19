use crate::{
    archive::ArchiveReader,
    format::Format,
    input::Input,
    output::Output,
};
use std::{
    io::Result,
    path::Path,
};

mod ar;
mod cpio;
mod fat;
mod tar;
mod zip;

/// A provider implementation for a specific archive format.
pub trait ArchiveFormat: Format {
    /// Open the given input for reading.
    fn open(&self, input: Input) -> Result<Box<dyn ArchiveReader>>;

    /// Create a writer for writing an archive to a stream.
    fn create<'w>(&self, _sink: &'w mut Output) -> Result<Box<dyn super::ArchiveWriter + 'w>> {
        unimplemented!() // TODO: return equivalent error
    }
}

/// Get all enabled formats.
pub fn all() -> &'static [&'static dyn ArchiveFormat] {
    &[
        &ar::Ar,
        &cpio::Cpio,
        &fat::Fat,
        &tar::Tar,
        &zip::Zip,
    ]
}

/// Get an appropriate archive format provider for a file beginning with the given
/// bytes.
pub fn for_bytes(bytes: &[u8]) -> Option<&'static dyn ArchiveFormat> {
    all().iter()
        .filter(|format| format.match_bytes(bytes))
        .map(|format| *format)
        .next()
}

/// Get an appropriate format provider for a file with the given file extension.
pub fn for_extension(path: &Path) -> Option<&'static dyn ArchiveFormat> {
    if let Some(file_name) = path.file_name()?.to_str() {
        for format in all() {
            for ext in format.file_extensions() {
                if file_name.ends_with(ext) {
                    return Some(*format);
                }
            }
        }
    }

    None
}
