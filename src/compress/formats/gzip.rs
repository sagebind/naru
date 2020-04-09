//! The gzip compression format as defined in [RFC 1952].
//!
//! [RFC 1952]: https://tools.ietf.org/html/rfc1952

use crate::{
    format::Format,
    input::Input,
};
use std::io::{Read, Result};

pub struct Gzip;

impl Format for Gzip {
    fn file_extensions(&self) -> &[&str] {
        &["gz"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_gz(bytes)
    }
}

impl super::CompressionFormat for Gzip {
    fn new_encoder(&self, input: Input) -> Result<Box<dyn Read>> {
        Ok(Box::new(flate2::read::GzDecoder::new(input)))
    }
}
