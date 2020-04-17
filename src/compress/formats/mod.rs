use crate::{
    format::Format,
    input::Input,
};
use std::io::{Read, Result};

mod bzip2;
mod compress;
mod gzip;
mod lzip;
mod xz;
mod zlib;

pub trait CompressionFormat: Format {
    fn new_encoder(&self, input: Input) -> Result<Box<dyn Read>>;
}

/// Get all enabled formats.
pub fn all() -> &'static [&'static dyn CompressionFormat] {
    &[
        &bzip2::Bzip2,
        &compress::Compress,
        &gzip::Gzip,
        &lzip::Lzip,
        &xz::Xz,
        &zlib::Zlib,
    ]
}
