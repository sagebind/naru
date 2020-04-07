use crate::{
    archive::ArchiveReader,
    io::Input,
};
use std::io::{
    Read,
    Seek,
    Result as IoResult,
};

mod tar;
pub mod zip;

pub trait Format {
    fn detect(reader: &mut impl Read) -> bool;

    fn open(&self, input: Input) -> IoResult<Box<dyn ArchiveReader>>;
}

