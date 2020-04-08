use crate::{
    archive::{ArchiveReader, Entry, EntryType},
    io::Input,
    formats::Format,
};
use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use std::{
    io::{Read, Seek},
    path::Path,
};
use zip::{
    read::{ZipArchive, ZipFile},
    result::ZipError,
};

pub struct Zip;

impl Format for Zip {
    fn file_extensions(&self) -> &[&str] {
        &["jar", "zip"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_zip(bytes)
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
    fn len(&mut self) -> Option<u64> {
        Some(self.archive.len() as u64)
    }

    fn entry(&mut self) -> std::io::Result<Option<Box<dyn Entry + '_>>> {
        match self.archive.by_index(self.index) {
            Ok(entry) => {
                self.index += 1;

                Ok(Some(Box::new(ZipEntry(entry))))
            },

            Err(ZipError::FileNotFound) => Ok(None),
            Err(ZipError::Io(e)) => Err(e),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }
}

struct ZipEntry<'a>(ZipFile<'a>);

impl<'a> Entry for ZipEntry<'a> {
    fn path(&self) -> std::borrow::Cow<'_, Path> {
        self.0.sanitized_name().into()
    }

    fn entry_type(&self) -> EntryType {
        if self.0.is_dir() {
            EntryType::Dir
        } else {
            EntryType::File
        }
    }

    fn size(&self) -> u64 {
        self.0.size()
    }

    fn compressed_size(&self) -> Option<u64> {
        Some(self.0.compressed_size())
    }

    fn modified(&self) -> Option<NaiveDateTime> {
        let dt = self.0.last_modified();

        Some(NaiveDateTime::new(
            NaiveDate::from_ymd(
                dt.year().into(),
                dt.month().into(),
                dt.day().into(),
            ),
            NaiveTime::from_hms(
                dt.hour().into(),
                dt.minute().into(),
                dt.second().into(),
            ),
        ))
    }
}

impl<'a> Read for ZipEntry<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}
