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
