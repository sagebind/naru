//! Stream-oriented compression and decompression.
//!
//! Formats such as TAR are often wrapped with stream compression since it is
//! not supported by the archive format itself.

use flate2::bufread::GzDecoder;
use std::io::{BufRead, Read};

pub mod formats;

pub fn create_decompressor<'r, R: BufRead + Read + 'r>(magic: &[u8], reader: R) -> Box<dyn Read + 'r> {
    if infer::archive::is_gz(magic) {
        return Box::new(GzDecoder::new(reader));
    }

    Box::new(reader)
}
