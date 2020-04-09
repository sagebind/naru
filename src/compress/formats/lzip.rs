//! https://en.wikipedia.org/wiki/Lzip
//! https://www.nongnu.org/lzip/manual/lzip_manual.html#File-format

use crate::{
    format::Format,
    input::Input,
};
use std::io::{Read, Result};

pub struct Lzip;

impl Format for Lzip {
    fn file_extensions(&self) -> &[&str] {
        &["lz"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_lz(bytes)
    }
}

impl super::CompressionFormat for Lzip {
    fn new_encoder(&self, input: Input) -> Result<Box<dyn Read>> {
        unimplemented!()
    }
}

pub struct LzipDecoder {}

pub struct LzipEncoder {}
