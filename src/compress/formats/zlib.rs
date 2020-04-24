//! The zlib compression format as defined in [RFC 1950].
//!
//! [RFC 1950]: https://tools.ietf.org/html/rfc1950

use crate::{
    format::Format,
    input::Input,
};
use std::{fmt, io::{Read, Result}};

pub struct Zlib;

impl Format for Zlib {
    fn id(&self) -> &str {
        "zlib"
    }

    fn file_extensions(&self) -> &[&str] {
        &["zz"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        matches!(bytes, [0x78, 0x01, ..] | [0x78, 0x5E, ..] | [0x78, 0x9C, ..] | [0x78, 0xDA, ..])
    }
}

impl fmt::Display for Zlib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ZLIB")
    }
}

impl super::CompressionFormat for Zlib {
    fn new_decoder(&self, reader: Box<dyn Read>) -> Result<Box<dyn Read>> {
        Ok(Box::new(flate2::read::ZlibDecoder::new(reader)))
    }
}
