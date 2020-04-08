use crate::{
    archive::ArchiveReader,
    input::Input,
};
use std::{
    io::Result,
    path::Path,
};

mod ar;
mod tar;
mod zip;

pub const AR: &dyn Format = &ar::Ar;
pub const TAR: &dyn Format = &tar::Tar;
pub const ZIP: &dyn Format = &zip::Zip;

const ALL_FORMATS: &[&dyn Format] = &[
    AR,
    TAR,
    ZIP
];

/// A provider implementation for a specific archive format.
pub trait Format {
    /// Get all file extensions that this format uses.
    fn file_extensions(&self) -> &[&str];

    /// Check the given starting bytes of a stream to detect if they match this
    /// format's magic signatures.
    fn match_bytes(&self, bytes: &[u8]) -> bool;

    /// Open the given input for reading.
    fn open(&self, input: Input) -> Result<Box<dyn ArchiveReader>>;
}

/// Get an appropriate format provider for a file beginning with the given
/// bytes.
pub fn for_bytes(bytes: &[u8]) -> Option<&'static dyn Format> {
    ALL_FORMATS.iter()
        .filter(|format| format.match_bytes(bytes))
        .map(|format| *format)
        .next()
}

/// Get an appropriate format provider for a file with the given file extension.
pub fn for_extension(path: &Path) -> Option<&'static dyn Format> {
    if let Some(file_name) = path.file_name()?.to_str() {
        for format in ALL_FORMATS {
            for ext in format.file_extensions() {
                if file_name.ends_with(ext) {
                    return Some(*format);
                }
            }
        }
    }

    None
}
