use std::path::PathBuf;
use structopt::StructOpt;

mod archive;
mod compress;
mod create;
mod extract;
mod format;
mod io;
mod list;

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
    #[structopt(short, long)]
    quiet: bool,

    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long, parse(from_occurrences))]
    verbose: usize,

    /// Change to directory DIR before running command.
    #[structopt(short = "C", parse(from_os_str))]
    directory: Option<PathBuf>,

    /// Display base 10 file size units.
    #[structopt(long)]
    base_10: bool,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Show information about supported formats.
    Formats,

    #[structopt(visible_alias = "a")]
    Add,

    /// Create a new archive.
    #[structopt(visible_alias = "c")]
    Create {
        /// When adding a directory recursively, skip any child directory that
        /// is on a different file system than the starting directory.
        #[structopt(long)]
        one_file_system: bool,

        /// Archive file ("-" for stdin)
        #[structopt(parse(from_os_str))]
        output: PathBuf,

        /// Files to add
        #[structopt(parse(from_os_str))]
        files: Vec<PathBuf>,
    },

    /// List the contents of an archive.
    #[structopt(visible_alias = "l")]
    List {
        /// Input file ("-" for stdin)
        #[structopt(parse(from_os_str))]
        input: PathBuf,
    },

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

fn show_formats() -> Result<(), Box<dyn std::error::Error>> {
    println!("Archive formats:");

    for format in archive::formats::all() {
        println!("  {}", format);
    }

    println!("Compression stream formats:");

    for format in compress::formats::all() {
        println!("  {}", format);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::from_args();

    stderrlog::new()
        // .module(module_path!())
        .quiet(options.flags.quiet)
        .verbosity(options.flags.verbose)
        .init()
        .unwrap();

    match options.command {
        Command::Formats => show_formats(),
        Command::Create {output, files, ..} => create::create(&output, &files),
        Command::Extract(options) => options.run(),
        Command::List {input} => list::list(&options.flags, &input),
        _ => unimplemented!(),
    }
}
