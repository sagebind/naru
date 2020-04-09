//! https://tukaani.org/xz/format.html

use crate::{
    format::Format,
    input::Input,
};
use std::io::{Read, Result};

pub struct Xz;

impl Format for Xz {
    fn file_extensions(&self) -> &[&str] {
        &["xz"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_xz(bytes)
    }
}

impl super::CompressionFormat for Xz {
    fn new_encoder(&self, input: Input) -> Result<Box<dyn Read>> {
        Ok(Box::new(xz2::read::XzDecoder::new(input)))
    }
}
