//! https://en.wikipedia.org/wiki/Compress

use crate::{
    format::Format,
    input::Input,
};
use std::io::{Read, Result};

pub struct Compress;

impl Format for Compress {
    fn file_extensions(&self) -> &[&str] {
        &["Z"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_z(bytes)
    }
}

impl super::CompressionFormat for Compress {
    fn new_encoder(&self, input: Input) -> Result<Box<dyn Read>> {
        unimplemented!()
    }
}
