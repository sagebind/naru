use crate::formats::Format;
use indicatif::ProgressBar;
use std::{
    fmt,
    path::PathBuf,
};
use structopt::StructOpt;

mod archive;
mod formats;
mod io;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(visible_alias = "a")]
    Add,

    #[structopt(visible_alias = "c")]
    Create,

    #[structopt(visible_alias = "l")]
    List {
        /// Input file ("-" for stdin)
        #[structopt(parse(from_os_str))]
        input: PathBuf,
    },

    #[structopt(visible_alias = "x")]
    Extract {
        /// Input file ("-" for stdin)
        #[structopt(parse(from_os_str))]
        input: PathBuf,

        /// Directory to extract into
        #[structopt(parse(from_os_str))]
        dest: Option<PathBuf>,
    },

    #[structopt(visible_alias = "u")]
    Update,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::from_args();

    match options.command {
        Some(Command::Extract {input, dest}) => {
            let dest = match dest {
                Some(path) => path,
                None => std::env::current_dir()?,
            };

            let input = io::Input::open(input)?;
            let mut reader = formats::zip::Zip.open(input)?;

            let progress_bar = match reader.len() {
                Some(len) => ProgressBar::new(len),
                None => ProgressBar::new_spinner(),
            };

            while let Some(mut entry) = reader.entry()? {
                progress_bar.set_message(&entry.path().to_string_lossy());
                entry.extract(&dest)?;
                progress_bar.inc(1);
            }

            progress_bar.finish();

            Ok(())
        }

        Some(Command::List {input}) => {
            let input = io::Input::open(input)?;
            let mut reader = formats::zip::Zip.open(input)?;

            while let Some(entry) = reader.entry()? {
                println!(
                    "{:>19}  {:>7}  {}",
                    EmptyFormat(entry.modified()),
                    size::Size::Bytes(entry.size()).to_string(size::Base::Base10, size::Style::Abbreviated),
                    entry.path().display(),
                );
            }

            Ok(())
        }

        _ => Ok(()),
    }
}
