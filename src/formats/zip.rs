use crate::{
    archive::{ArchiveReader, ArchiveWriter, EntryMetadata, EntryType},
    io::Input,
    formats::Format,
};
use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use std::{
    io::{Read, Seek},
};
use zip::{ZipArchive, result::ZipError};

pub struct Zip;

impl Format for Zip {
    fn detect(reader: &mut impl Read) -> bool {
        false
    }

    fn open(&self, input: Input) -> std::io::Result<Box<dyn ArchiveReader>> {
        Ok(Box::new(ZipReader::open(input)?))
    }
}

pub struct ZipReader<R: Read + Seek> {
    archive: ZipArchive<R>,
    index: usize,
}

impl<R: Read + Seek> ZipReader<R> {
    fn open(reader: R) -> std::io::Result<Self> {
        Ok(Self {
            archive: ZipArchive::new(reader)?,
            index: 0,
        })
    }
}

impl<R: Read + Seek> ArchiveReader for ZipReader<R> {
    fn len(&self) -> Option<u64> {
        Some(self.archive.len() as u64)
    }

    fn entry(&mut self) -> Option<EntryMetadata> {
        match self.archive.by_index(self.index) {
            Ok(entry) => {
                self.index += 1;

                Some(EntryMetadata {
                    name: entry.sanitized_name(),
                    file_type: if entry.is_dir() {
                        EntryType::Dir
                    } else {
                        EntryType::File
                    },
                    size: entry.size(),
                    compressed_size: Some(entry.compressed_size()),
                    modified: NaiveDateTime::new(
                        NaiveDate::from_ymd(
                            entry.last_modified().year().into(),
                            entry.last_modified().month().into(),
                            entry.last_modified().day().into(),
                        ),
                        NaiveTime::from_hms(
                            entry.last_modified().hour().into(),
                            entry.last_modified().minute().into(),
                            entry.last_modified().second().into(),
                        ),
                    ),
                })
            },

            Err(ZipError::FileNotFound) => None,

            Err(e) => panic!("{}", e),
        }
    }
}
