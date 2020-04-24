use crate::{
    format::Format,
    input::Input,
    util::MaybeBoxedMut,
};
use std::io::{Read, Result, Write};

mod bzip2;
mod compress;
mod gzip;
mod lzip;
mod xz;
mod zlib;

pub trait CompressionFormat: Format {
    fn new_decoder(&self, reader: Box<dyn Read>) -> Result<Box<dyn Read>> {
        unimplemented!()
    }

    fn new_encoder<'w>(&self, writer: MaybeBoxedMut<'w, dyn Write>) -> Result<Box<dyn Write + 'w>> {
        unimplemented!()
    }
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
