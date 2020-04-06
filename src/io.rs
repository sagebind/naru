use std::io::{BufRead, Cursor, Read, Result, Seek, SeekFrom};

trait ReadSeek: Read + Seek {}

impl<R: Read + Seek> ReadSeek for R {}

pub struct Input {
    inner: Box<dyn ReadSeek>,
}

impl Input {
    pub fn new(reader: impl Read + Seek + 'static) -> Self {
        Self {
            inner: Box::new(reader),
        }
    }
}

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl Seek for Input {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.inner.seek(pos)
    }
}

pub struct LazyReader<R> {
    inner: R,
    buffer: Cursor<Vec<u8>>,
}

impl<R: Read> LazyReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            inner: reader,
            buffer: Cursor::new(Vec::new()),
        }
    }

    fn read_once(&mut self) -> Result<usize> {
        struct ReadLimit<R>(R, usize);

        impl<R: Read> Read for ReadLimit<R> {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
                if let Some(n) = self.1.checked_sub(1) {
                    self.1 = n;
                    self.0.read(buf)
                } else {
                    Ok(0)
                }
            }
        }

        ReadLimit(&mut self.inner, 1).read_to_end(self.buffer.get_mut())
    }

    fn unread(&self) -> usize {
        self.buffer.get_ref().len() - self.buffer.position() as usize
    }

    fn fill_all(&mut self) -> Result<usize> {
        self.inner.read_to_end(self.buffer.get_mut())
    }
}

impl<R: Read> BufRead for LazyReader<R> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.unread() == 0 {
            self.read_once()?;
        }

        self.buffer.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.buffer.consume(amt);
    }
}

impl<R: Read> Read for LazyReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.fill_buf()?;
        self.buffer.read(buf)
    }
}

impl<R: Read> Seek for LazyReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            // Report current position
            SeekFrom::Current(0) => Ok(self.buffer.position()),

            SeekFrom::Start(start) => {
                // Read until requested number of bytes are available.
                while (self.buffer.get_ref().len() as u64) < start {
                    if self.read_once()? == 0 {
                        break;
                    }
                }

                self.buffer.seek(pos)
            }

            SeekFrom::Current(current) => {
                let target = self.buffer.position() as i64 + current;

                // Read until requested number of bytes are available.
                while (self.buffer.get_ref().len() as i64) < target {
                    if self.read_once()? == 0 {
                        break;
                    }
                }

                self.buffer.seek(pos)
            }

            // To seek from the end, we must consume the reader in order to know
            // the full length.
            SeekFrom::End(_) => {
                self.fill_all()?;
                self.buffer.seek(pos)
            }
        }
    }
}

impl<R: Read> From<R> for LazyReader<R> {
    fn from(reader: R) -> Self {
        Self::new(reader)
    }
}
