#[macro_use]
extern crate typed_builder;

use std::{
    error::Error,
    path::PathBuf,
};
use structopt::StructOpt;

mod archive;
mod compress;
mod create;
mod extract;
mod format;
mod io;
mod list;
mod paths;

pub use io::*;

/// Cross platform, intuitive file archiver command.
#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(flatten)]
    flags: Flags,

    #[structopt(subcommand)]
    command: Command,
}

// Flags shared by all commands.
#[derive(Debug, StructOpt)]
struct Flags {
    /// Silence all command output
    #[structopt(short, long, global = true)]
    quiet: bool,

    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long, parse(from_occurrences), global = true)]
    verbose: usize,

    /// Change to directory DIR before running command.
    #[structopt(short = "C", parse(from_os_str), global = true)]
    directory: Option<PathBuf>,

    /// Display base 10 file size units.
    #[structopt(long, global = true)]
    base_10: bool,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Show information about supported formats.
    Formats,

    #[structopt(visible_alias = "c")]
    Create(create::Command),

    #[structopt(visible_alias = "l")]
    List(list::Command),

    #[structopt(visible_alias = "x")]
    Extract(extract::Command),
}

impl Flags {
    fn base(&self) -> size::Base {
        if self.base_10 {
            size::Base::Base10
        } else {
            size::Base::Base2
        }
    }
}

fn progress_bar_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::default_bar()
        .template(concat!(
            "Name: {msg}\n",
            "Time remaining: {eta}\n",
            "{percent:>3}% [{bar:40}] {pos}/{len}",
        ))
        .progress_chars("=> ")
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::from_args();
    log::debug!("parsed arguments: {:?}", options);

    stderrlog::new()
        // .module(module_path!())
        .quiet(options.flags.quiet)
        .verbosity(options.flags.verbose)
        .init()
        .unwrap();

    match options.command {
        Command::Create(command) => command.execute(),
        Command::Extract(command) => command.execute(),
        Command::List(command) => command.execute(&options.flags),
        Command::Formats => {
            println!("Archive formats:");

            for format in archive::formats::all() {
                println!("  {}", format);
            }

            println!("Compression stream formats:");

            for format in compress::formats::all() {
                println!("  {}", format);
            }

            Ok(())
        },
    }
}
