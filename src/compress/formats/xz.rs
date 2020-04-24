//! https://tukaani.org/xz/format.html

use crate::{
    format::Format,
    input::Input,
};
use std::{fmt, io::{Read, Result}};

pub struct Xz;

impl Format for Xz {
    fn id(&self) -> &str {
        "xz"
    }

    fn file_extensions(&self) -> &[&str] {
        &["xz"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_xz(bytes)
    }
}

impl fmt::Display for Xz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("xz")
    }
}

impl super::CompressionFormat for Xz {
    fn new_decoder(&self, reader: Box<dyn Read>) -> Result<Box<dyn Read>> {
        Ok(Box::new(xz2::read::XzDecoder::new(reader)))
    }
}
