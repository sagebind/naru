use std::fmt::Display;

/// A definition for a file format.
pub trait Format: Display {
    /// A unique identifier for referring to this format.
    fn id(&self) -> &str;

    /// Get all file extensions that this format uses.
    ///
    /// This is used for detecting the use of a file format automatically by
    /// file extension.
    fn file_extensions(&self) -> &[&str] {
        &[]
    }

    /// Check the given starting bytes of a stream to detect if they match this
    /// format's magic signatures.
    fn match_bytes(&self, bytes: &[u8]) -> bool;
}
