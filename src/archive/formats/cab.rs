use crate::{
    archive::{ArchiveReader, Entry, EntryType, Metadata},
    input::Input,
};
use chrono::prelude::*;
use std::{
    borrow::Cow,
    fmt,
    io::{Read, Result, Seek},
    path::{Path, PathBuf},
};

/// Format provider for CAB.
pub struct Cab;

impl super::Format for Cab {
    fn id(&self) -> &str {
        "cab"
    }

    fn file_extensions(&self) -> &[&str] {
        &["cab"]
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        infer::archive::is_cab(bytes)
    }
}

impl fmt::Display for Cab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("cabinet")
    }
}

impl super::ArchiveFormat for Cab {
    fn open<'r>(&self, input: Input<'r>) -> Result<Box<dyn ArchiveReader + 'r>> {
        Ok(Box::new(CabReader::new(input)?))
    }
}

struct CabReader<R: Read + Seek> {
    cab: cab::Cabinet<R>,
    entries: Vec<(String, Metadata)>,
    offset: usize,
}

impl<R: Read + Seek> CabReader<R> {
    fn new(reader: R) -> Result<Self> {
        let cab = cab::Cabinet::new(reader)?;

        let entries = cab.folder_entries()
            .flat_map(|folder| folder.file_entries())
            .map(|file| (
                file.name().to_owned(),
                Metadata::builder()
                    .entry_type(EntryType::File)
                    .size(file.uncompressed_size().into())
                    .modified(Local.from_local_datetime(&file.datetime()).single())
                    .build()
            ))
            .collect::<Vec<_>>();

        Ok(Self {
            cab,
            entries,
            offset: 0,
        })
    }
}

impl<R: Read + Seek> ArchiveReader for CabReader<R> {
    fn len(&mut self) -> Option<u64> {
        Some(self.entries.len() as u64)
    }

    fn entry(&mut self) -> Result<Option<Box<dyn Entry + '_>>> {
        if let Some(entry)= self.entries.get(self.offset) {
            self.offset += 1;

            Ok(Some(Box::new(CabEntry {
                name: &entry.0,
                metadata: &entry.1,
                reader: self.cab.read_file(&entry.0)?,
            })))
        } else {
            Ok(None)
        }
    }
}

struct CabEntry<'a, R: Read + Seek> {
    name: &'a str,
    metadata: &'a Metadata,
    reader: cab::FileReader<'a, R>,

}

impl<'a, R: Read + Seek> Entry for CabEntry<'a, R> {
    fn path(&self) -> Cow<'_, Path> {
        Path::new(self.name).into()
    }

    fn metadata(&self) -> Metadata {
        self.metadata.clone()
    }
}

impl<'a, R: Read + Seek> Read for CabEntry<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.reader.read(buf)
    }
}
