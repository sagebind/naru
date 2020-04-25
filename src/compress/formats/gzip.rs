//! The gzip compression format as defined in [RFC 1952].
//!
//! [RFC 1952]: https://tools.ietf.org/html/rfc1952

use crate::{
    format::Format,
    input::Input,
};
use std::{fmt, io::{Read, Result}};

pub struct Gzip;

impl Format for Gzip {
    fn id(&self) -> &str {
        "gzip"
    }

    fn file_extensions(&self) -> &[&str] {
        &["gz"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_gz(bytes)
    }
}

impl fmt::Display for Gzip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GZIP")
    }
}

impl super::CompressionFormat for Gzip {
    fn new_decoder<'r>(&self, input: Input<'r>) -> Result<Box<dyn Read + 'r>> {
        Ok(Box::new(flate2::read::GzDecoder::new(input)))
    }
}
