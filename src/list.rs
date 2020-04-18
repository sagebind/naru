use crate::{
    archive,
    io::input::Input,
};
use std::{
    error::Error,
    fmt,
    path::Path,
};

/// Handles the `list` command.
pub fn list(input: &Path) -> Result<(), Box<dyn Error>> {
    let input_file = Input::open(&input)?;

    if let Some(mut reader) = archive::open(input_file)? {
        while let Some(entry) = reader.entry()? {
            println!(
                "{:>19}  {:>7}  {}",
                EmptyFormat(entry.modified()),
                size::Size::Bytes(entry.size()).to_string(size::Base::Base10, size::Style::Abbreviated),
                entry.path().display(),
            );
        }
    } else {
        eprintln!("Unknown format: {}", input.display());
    }

    Ok(())
}

#[derive(Debug)]
struct EmptyFormat<T>(Option<T>);

impl<T: fmt::Display> fmt::Display for EmptyFormat<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(inner) = self.0.as_ref() {
            inner.fmt(f)
        } else {
            '-'.fmt(f)
        }
    }
}
