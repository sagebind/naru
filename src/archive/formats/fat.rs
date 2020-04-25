//! https://www.win.tue.nl/~aeb/linux/fs/fat/fat-1.html

use crate::{
    archive::{ArchiveReader, Entry, EntryType, Metadata},
    input::Input,
};
use owning_ref::OwningHandle;
use std::{
    convert::TryInto,
    fmt,
    fs::File,
    io::{Read, Result, Seek, SeekFrom},
    path::{Path, PathBuf},
};

pub struct Fat;

impl super::Format for Fat {
    fn id(&self) -> &str {
        "fat"
    }

    fn match_bytes(&self, bytes: &[u8]) -> bool {
        matches!(bytes, [0xEB, 0x3C, 0x90, ..])
    }
}

impl fmt::Display for Fat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FAT")
    }
}

impl super::ArchiveFormat for Fat {
    fn open(&self, input: Input) -> Result<Box<dyn ArchiveReader>> {
        let input = input.into_file()
            .map_err(|_| ())
            .expect("only files are supported with fat");

        Ok(Box::new(FatReader::new(input)?))
    }
}

struct FatReader {
    iter: OwningHandle<Box<fatfs::FileSystem<File>>, Box<FatIterator<'static>>>,
}

impl FatReader {
    fn new(mut reader: File) -> Result<Self> {
        reader.seek(SeekFrom::Start(0))?;

        Ok(Self {
            iter: OwningHandle::new_with_fn(
                Box::new(fatfs::FileSystem::new(reader, fatfs::FsOptions::new())?),
                |fs| unsafe {
                    let fs = &mut *(fs as *mut fatfs::FileSystem<File>);
                    Box::new(FatIterator {
                        stack: vec![fs.root_dir().iter()],
                    })
                },
            ),
        })
    }
}

impl ArchiveReader for FatReader {
    fn entry(&mut self) -> Result<Option<Box<dyn Entry + '_>>> {
        match self.iter.next() {
            Some(Ok(entry)) => Ok(Some(Box::new(FatEntry(entry)))),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

struct FatEntry<'a>(fatfs::DirEntry<'a, File>);

impl<'a> Entry for FatEntry<'a> {
    fn path(&self) -> std::borrow::Cow<'_, Path> {
        PathBuf::from(self.0.file_name()).into()
    }

    fn metadata(&self) -> Metadata {
        Metadata::builder()
            .entry_type(if self.0.is_dir() {
                EntryType::Directory
            } else {
                EntryType::File
            })
            .size(self.0.len())
            .read_only(self.0.attributes().contains(fatfs::FileAttributes::READ_ONLY))
            .hidden(self.0.attributes().contains(fatfs::FileAttributes::HIDDEN))
            .build()
    }
}

impl<'a> Read for FatEntry<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.0.is_file() {
            self.0.to_file().read(buf)
        } else {
            Ok(0)
        }
    }
}

struct FatIterator<'a> {
    stack: Vec<fatfs::DirIter<'a, File>>,
}

impl<'a> Iterator for FatIterator<'a> {
    type Item = Result<fatfs::DirEntry<'a, File>>;

    fn next(&mut self) -> Option<Self::Item> {
        let iter = self.stack.last_mut()?;

        match iter.next() {
            Some(Ok(entry)) => {
                if entry.is_dir() {
                    self.stack.push(entry.to_dir().iter());
                }

                Some(Ok(entry))
            }
            _ => None,
        }
    }
}
