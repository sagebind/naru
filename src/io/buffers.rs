use std::{
    fs::File,
    io::{copy, Read, Result, Seek, SeekFrom, Write},
};

/// An I/O buffer that provides a [`Seek`] implementation for an arbitrary
/// reader by buffering data read to a temporary file on disk.
pub struct DiskCacheReader<R: Read> {
    /// Underlying reader.
    inner: R,

    /// Number of bytes read from the underlying reader so far.
    offset: u64,

    /// Temporary file where data is cached.
    file: File,

    /// Set to true when the end of the underlying reader is reached.
    eof: bool,
}

impl<R: Read> DiskCacheReader<R> {
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self {
            inner: reader,
            offset: 0,
            file: tempfile::tempfile()?,
            eof: false,
        })
    }

    fn fill_to_offset(&mut self, offset: u64) -> Result<u64> {
        if self.eof || offset <= self.offset {
            return Ok(0);
        }

        // Save current position.
        let previous_pos = self.position()?;
        self.file.seek(SeekFrom::End(0))?;

        // Create a temporary buffer for reading with.
        let mut buf = [0; 16384];

        // Read from the inner reader until EOF or `offset` is reached.
        let target = offset - self.offset;
        let result = self.read_inner(&mut buf, |n| n < target);

        // Restore previous position.
        self.file.seek(SeekFrom::Start(previous_pos))?;

        result
    }

    /// Get the current seek position.
    fn position(&mut self) -> Result<u64> {
        self.file.seek(SeekFrom::Current(0))
    }

    /// Read new data from the inner reader and copy it into the cache file.
    ///
    /// The `predicate` is a function that is used to determine when to stop
    /// reading. One read is always done first.
    fn read_inner(&mut self, buf: &mut [u8], predicate: impl Fn(u64) -> bool) -> Result<u64> {
        let mut read = 0;

        loop {
            match self.inner.read(buf)? {
                0 => {
                    self.eof = true;
                    break;
                }
                count => {
                    // TODO: If this fails but the above succeeded, then we lose
                    // this chunk of data from the stream forever. Can this be
                    // handled?
                    self.file.write_all(&buf[..count])?;
                    read += count as u64;
                }
            };

            if !predicate(read) {
                break;
            }
        }

        self.file.flush()?;
        self.offset += read;

        Ok(read)
    }
}

impl<R: Read> Read for DiskCacheReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.file.read(buf)? {
            // No more data in buffer, advance the underlying reader if not at
            // the end.
            0 if !self.eof => self.read_inner(buf, |_| false).map(|n| n as usize),
            n => Ok(n),
        }
    }
}

impl<R: Read> Seek for DiskCacheReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            // Report current position.
            SeekFrom::Current(0) => self.file.seek(pos),

            // Seek to an absolute position. Ensure the temp file is filled to
            // that point if possible.
            SeekFrom::Start(start) => {
                self.fill_to_offset(start)?;
                self.file.seek(pos)
            }

            // Seek to a relative position. Figure out the absolute position and
            // then do the same as above.
            SeekFrom::Current(relative) => {
                let current_pos = self.position()?;
                self.fill_to_offset((current_pos as i64 + relative) as u64)?;
                self.file.seek(pos)
            }

            // To seek from the end, we must consume the reader in order to know
            // the full length.
            SeekFrom::End(_) => {
                if !self.eof {
                    self.file.seek(SeekFrom::End(0))?;
                    self.offset += copy(&mut self.inner, &mut self.file)?;
                    self.eof = true;
                }

                self.file.seek(pos)
            }
        }
    }
}

/// Provides a [`Seek`] implementation for an arbitrary writer by buffering
/// bytes written to a temporary file and flushing the contents to the
/// underlying writer on drop.
pub struct DiskCacheWriter<W: Write> {
    /// Underlying writer.
    inner: W,

    /// Number of bytes written to the underlying writer so far.
    offset: u64,

    /// Temporary file where data is cached.
    file: File,
}

impl<W: Write> DiskCacheWriter<W> {
    pub fn new(writer: W) -> Result<Self> {
        Ok(Self {
            inner: writer,
            offset: 0,
            file: tempfile::tempfile()?,
        })
    }
}

impl<W: Write> Write for DiskCacheWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.file.flush()?;

        // Save current position.
        let pos = self.file.seek(SeekFrom::Current(0))?;

        // Jump to the last position we copied to the inner writer.
        self.file.seek(SeekFrom::Start(self.offset))?;

        // Write everything we have so far.
        self.offset += copy(&mut self.file, &mut self.inner)?;
        self.inner.flush()?;

        // Restore cursor.
        self.file.seek(SeekFrom::Start(pos))?;

        Ok(())
    }
}

impl<W: Write> Seek for DiskCacheWriter<W> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.file.seek(pos)
    }
}

impl<W: Write> Drop for DiskCacheWriter<W> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
