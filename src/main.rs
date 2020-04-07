use crate::{
    archive::ArchiveReader,
    formats::Format,
};
use std::{
    path::{Path, PathBuf},
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
    },

    #[structopt(visible_alias = "u")]
    Update,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::from_args();

    match options.command {
        Some(Command::List {input}) => {
            let input = io::Input::open(input)?;
            let mut reader = formats::zip::Zip.open(input)?;

            while let Some(entry) = reader.entry() {
                println!(
                    "{}  {:>7}  {}",
                    entry.modified,
                    size::Size::Bytes(entry.size).to_string(size::Base::Base10, size::Style::Abbreviated),
                    entry.name.to_string_lossy(),
                );
            }

            Ok(())
        }
        _ => Ok(()),
    }
}
