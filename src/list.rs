use crate::{
    archive,
    io::input::Input,
};
use std::{
    error::Error,
    fmt,
    path::PathBuf,
};
use structopt::StructOpt;

/// List the contents of an archive.
#[derive(Debug, StructOpt)]
pub struct Command {
    /// Input file ("-" for stdin).
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

impl Command {
    pub(crate) fn execute(&self, flags: &super::Flags) -> Result<(), Box<dyn Error>> {
        let input_file = Input::open(&self.input)?;

        if let Some(mut reader) = archive::open(input_file)? {
            let mut files = 0;
            let mut dirs = 0;
            let mut bytes = 0;

            while let Some(entry) = reader.entry()? {
                let metadata = entry.metadata();

                if metadata.is_dir() {
                    dirs += 1;
                } else {
                    files += 1;
                    bytes += metadata.size;
                }

                println!(
                    "{:>19}  {:>8}  {}",
                    EmptyFormat(metadata.modified.as_ref()),
                    EmptyFormat(if metadata.is_dir() {
                        None
                    } else {
                        Some(size::Size::Bytes(metadata.size).to_string(flags.base(), size::Style::Abbreviated))
                    }),
                    entry.path().display(),
                );
            }

            println!("{} files, {} directories, totalling {}", files, dirs, size::Size::Bytes(bytes).to_string(flags.base(), size::Style::Smart));
        } else {
            eprintln!("Unknown format: {}", self.input.display());
        }

        Ok(())
    }
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
