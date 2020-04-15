//! Archive format APIs for reading and writing.

use crate::{
    input::Input,
    output::Output,
};
use std::io::{BufRead, Result};

mod formats;
mod read;
mod write;

pub use self::{
    read::*,
    write::*,
};

pub fn open(mut input: Input) -> Result<Option<Box<dyn ArchiveReader>>> {
    if let Some(format) = formats::for_bytes(input.fill_buf()?) {
        Ok(Some(format.open(input)?))
    } else {
        Ok(None)
    }
}

pub fn create<'o>(output: &'o mut Output) -> Result<Option<Box<dyn ArchiveWriter + 'o>>> {
    if let Some(path) = output.path() {
        if let Some(format) = formats::for_extension(path) {
            return Ok(Some(format.create(output)?));
        }
    }

    Ok(None)
}
