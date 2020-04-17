//! https://www.sourceware.org/bzip2/

use crate::{
    format::Format,
    input::Input,
};
use std::{fmt, io::{Read, Result}};

pub struct Bzip2;

impl Format for Bzip2 {
    fn id(&self) -> &str {
        "bzip2"
    }

    fn file_extensions(&self) -> &[&str] {
        &["bz2"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_bz2(bytes)
    }
}

impl fmt::Display for Bzip2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bzip2")
    }
}

impl super::CompressionFormat for Bzip2 {
    fn new_encoder(&self, input: Input) -> Result<Box<dyn Read>> {
        Ok(Box::new(bzip2::read::BzDecoder::new(input)))
    }
}
