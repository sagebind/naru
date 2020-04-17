//! The common ZIP file format.
//!
//! https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT

use crate::{
    archive::{ArchiveReader, ArchiveWriter, Entry, EntryType, Metadata},
    input::Input,
    output::Output,
};
use chrono::prelude::*;
use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use std::{
    fmt,
    io::{copy, Read, Result, Seek, Write},
    path::Path,
};
use zip::{
    read::{ZipArchive, ZipFile},
    result::ZipError,
    write::ZipWriter,
};

const DEFAULT_COMPRESSION_METHOD: zip::CompressionMethod = zip::CompressionMethod::Bzip2;

pub struct Zip;

impl super::Format for Zip {
    fn id(&self) -> &str {
        "zip"
    }

    fn file_extensions(&self) -> &[&str] {
        &["jar", "zip"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_zip(bytes)
    }
}

impl fmt::Display for Zip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ZIP")
    }
}

impl super::ArchiveFormat for Zip {
    fn open(&self, input: Input) -> Result<Box<dyn ArchiveReader>> {
        Ok(Box::new(ZipReader::open(input)?))
    }

    fn create<'w>(&self, output: &'w mut Output) -> Result<Box<dyn ArchiveWriter + 'w>> {
        Ok(Box::new(ZipWriter::new(output)))
    }
}

pub struct ZipReader<R: Read + Seek> {
    archive: ZipArchive<R>,
    index: usize,
}

impl<R: Read + Seek> ZipReader<R> {
    fn open(reader: R) -> Result<Self> {
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

    fn entry(&mut self) -> Result<Option<Box<dyn Entry + '_>>> {
        match self.archive.by_index(self.index) {
            Ok(entry) => {
                self.index += 1;

                Ok(Some(Box::new(ZipEntry(entry))))
            },

            Err(ZipError::FileNotFound) => Ok(None),
            Err(e) => Err(convert_err(e)),
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
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl<W: Write + Seek> ArchiveWriter for ZipWriter<W> {
    fn add_directory(&mut self, path: &Path, metadata: Metadata) -> Result<()> {
        self.add_directory_from_path(path, create_file_options(metadata))?;

        Ok(())
    }

    fn add_file(&mut self, path: &Path, metadata: Metadata, file: &mut dyn Read) -> Result<()> {
        // TODO: Handle encoding better.
        self.start_file(path.to_string_lossy(), create_file_options(metadata))?;
        copy(file, self)?;

        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        self.finish().map(|_| ()).map_err(convert_err)
    }
}

fn create_file_options(metadata: Metadata) -> zip::write::FileOptions {
    let mut options = zip::write::FileOptions::default()
    .compression_method(DEFAULT_COMPRESSION_METHOD);

    if let Some(datetime) = metadata.last_modified {
        if let Ok(datetime) = zip::DateTime::from_date_and_time(
            datetime.year() as u16,
            datetime.month() as u8,
            datetime.day() as u8,
            datetime.hour() as u8,
            datetime.minute() as u8,
            datetime.second() as u8,
        ) {
            options = options.last_modified_time(datetime);
        }
    }

    options
}

fn convert_err(error: ZipError) -> std::io::Error {
    match error {
        ZipError::FileNotFound => std::io::ErrorKind::NotFound.into(),
        ZipError::Io(e) => e,
        e => std::io::Error::new(std::io::ErrorKind::InvalidData, e),
    }
}
