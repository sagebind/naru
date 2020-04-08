use crate::{
    archive::ArchiveReader,
    io::Input,
};
use std::{
    io::Result as IoResult,
    path::Path,
};

mod tar;
mod zip;

pub const TAR: &dyn Format = &tar::Tar;
pub const ZIP: &dyn Format = &zip::Zip;

const ALL_FORMATS: &[&dyn Format] = &[
    TAR,
    ZIP
];

pub trait Format {
    /// Get all file extensions that this format uses.
    fn file_extensions(&self) -> &[&str];

    /// Check the given starting bytes of a stream to detect if they match this
    /// format's magic signatures.
    fn match_bytes(&self, bytes: &[u8]) -> bool;

    fn open(&self, input: Input) -> IoResult<Box<dyn ArchiveReader>>;
}

pub fn detect_bytes(bytes: &[u8]) -> Option<&'static dyn Format> {
    ALL_FORMATS.iter()
        .filter(|format| format.match_bytes(bytes))
        .map(|format| *format)
        .next()
}

pub fn detect_extension(path: &Path) -> Option<&'static dyn Format> {
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
