//! The zlib compression format as defined in [RFC 1950].
//!
//! [RFC 1950]: https://tools.ietf.org/html/rfc1950

use crate::{
    format::Format,
    input::Input,
};
use std::io::{Read, Result};

pub struct Zlib;

impl Format for Zlib {
    fn file_extensions(&self) -> &[&str] {
        &["zz"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        matches!(bytes, [0x78, 0x01, ..] | [0x78, 0x5E, ..] | [0x78, 0x9C, ..] | [0x78, 0xDA, ..])
    }
}

impl super::CompressionFormat for Zlib {
    fn new_encoder(&self, input: Input) -> Result<Box<dyn Read>> {
        Ok(Box::new(flate2::read::ZlibDecoder::new(input)))
    }
}
