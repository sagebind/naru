use std::{
    fs::File,
    io::{Read, Seek},
    path::{Path, PathBuf},
};
use structopt::StructOpt;

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

fn open(path: impl AsRef<Path>) -> std::io::Result<io::Input> {
    let path = path.as_ref();

    if path.to_str() == Some("-") {
        std::io::stdin().lock()
    } else {
        Ok(io::Input::new(File::open(path)?))
    }
}

fn main() {
    let opt = Options::from_args();
    println!("{:?}", opt);

    match opt.command {
        Some(Command::List {input}) => {
            println!("{:?}", input);
        }
        _ => {}
    }
}
