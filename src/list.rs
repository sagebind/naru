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
pub(crate) fn list(flags: &super::Flags, input: &Path) -> Result<(), Box<dyn Error>> {
    let input_file = Input::open(&input)?;

    if let Some(mut reader) = archive::open(input_file)? {
        let mut files = 0;
        let mut dirs = 0;
        let mut bytes = 0;

        while let Some(entry) = reader.entry()? {
            if entry.is_dir() {
                dirs += 1;
            } else {
                files += 1;
                bytes += entry.size();
            }

            println!(
                "{:>19}  {:>8}  {}",
                EmptyFormat(entry.modified()),
                EmptyFormat(if entry.is_dir() {
                    None
                } else {
                    Some(size::Size::Bytes(entry.size()).to_string(flags.base(), size::Style::Abbreviated))
                }),
                entry.path().display(),
            );
        }

        println!("{} files, {} directories, totalling {}", files, dirs, size::Size::Bytes(bytes).to_string(flags.base(), size::Style::Smart));
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
