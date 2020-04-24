//! https://en.wikipedia.org/wiki/Compress

use crate::{
    format::Format,
    input::Input,
};
use std::{fmt, io::{Read, Result}};

pub struct Compress;

impl Format for Compress {
    fn id(&self) -> &str {
        "COMPRESS"
    }

    fn file_extensions(&self) -> &[&str] {
        &["Z"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_z(bytes)
    }
}

impl fmt::Display for Compress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("COMPRESS")
    }
}

impl super::CompressionFormat for Compress {
}
