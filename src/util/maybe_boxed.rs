pub enum MaybeBoxedMut<'a, T: ?Sized> {
    Borrowed(&'a mut T),
    Owned(Box<T>),
}

impl<'a, T: ?Sized> From<&'a mut T> for MaybeBoxedMut<'a, T> {
    fn from(reference: &'a mut T) -> Self {
        Self::Borrowed(reference)
    }
}

impl<T> From<T> for MaybeBoxedMut<'_, T> {
    fn from(value: T) -> Self {
        Self::Owned(Box::new(value))
    }
}

impl<T: ?Sized> AsMut<T> for MaybeBoxedMut<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        match self {
            Self::Borrowed(v) => v,
            Self::Owned(v) => v.as_mut(),
        }
    }
}

impl<T: std::io::Read + ?Sized> std::io::Read for MaybeBoxedMut<'_, T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.as_mut().read(buf)
    }
}

impl<T: std::io::Write + ?Sized> std::io::Write for MaybeBoxedMut<'_, T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.as_mut().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.as_mut().flush()
    }
}
