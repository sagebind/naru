//! Archive format APIs for reading and writing.

use crate::{
    input::Input,
    output::Output,
};
use chrono::prelude::*;
use std::{
    fs,
    io::{BufRead, Result},
};

mod formats;
mod read;
mod write;

pub use self::{
    read::*,
    write::*,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Metadata {
    last_modified: Option<DateTime<Local>>,
}

impl From<fs::Metadata> for Metadata {
    fn from(metadata: fs::Metadata) -> Self {
        Self {
            last_modified: metadata.modified().ok().map(From::from),
        }
    }
}

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
