use indicatif::ProgressBar;
use std::{
    fmt,
    fs,
    io::BufRead,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use walkdir::WalkDir;

mod archive;
mod buffers;
mod compress;
mod format;
mod input;
mod output;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(visible_alias = "a")]
    Add,

    /// Create a new archive
    #[structopt(visible_alias = "c")]
    Create {
        /// Archive file ("-" for stdin)
        #[structopt(parse(from_os_str))]
        output: PathBuf,

        /// Files to add
        #[structopt(parse(from_os_str))]
        files: Vec<PathBuf>,
    },

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

fn progress_bar_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::default_bar()
        .template(concat!(
            "Name: {msg}\n",
            "Time remaining: {eta}\n",
            "{percent:>3}% [{bar:40}] {pos}/{len}",
        ))
        .progress_chars("=> ")
}

fn collect_paths(paths: &[PathBuf]) -> Result<Vec<walkdir::DirEntry>, Box<dyn std::error::Error>> {
    paths.iter()
        .flat_map(|path| WalkDir::new(path).follow_links(true))
        .map(|result| result.map_err(|e| Box::new(e) as Box<dyn std::error::Error>))
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::from_args();

    match options.command {
        Some(Command::Create {output, files}) => {
            let mut output = output::Output::create(&output)?;

            if let Some(mut writer) = archive::create(&mut output)? {
                let entries = collect_paths(&files)?;

                let progress_bar = ProgressBar::new(entries.len() as u64)
                    .with_style(progress_bar_style());
                progress_bar.enable_steady_tick(1000);

                for entry in entries {
                    let path = entry.path();

                    progress_bar.set_message(&path.to_string_lossy());

                    if entry.file_type().is_dir() {
                        writer.add_directory(path)?;
                    } else {
                        writer.add_file(path, &mut fs::File::open(path)?)?;
                    }

                    progress_bar.inc(1);
                }

                writer.finish()?;

                progress_bar.finish_and_clear();
            } else {
                eprintln!("Unrecognized file extension, specify format manually");
            }

            Ok(())
        }

        Some(Command::Extract {input, dest}) => {
            let input = input::Input::open(&input)?;

            let dest = match dest {
                Some(path) => path,

                // If no destination path was given, assume one based on the
                // name of the input file.
                None => {
                    let mut path = std::env::current_dir()?;

                    if let Some(input_path) = input.path() {
                        if let Some(archive_file_name) = input_path.file_stem() {
                            path = path.join(archive_file_name);
                            fs::create_dir(&path)?;
                        }
                    }

                    path
                },
            };

            if let Some(mut reader) = archive::open(input)? {
                let progress_bar = match reader.len() {
                    Some(len) => ProgressBar::new(len).with_style(progress_bar_style()),
                    None => ProgressBar::new_spinner(),
                };

                while let Some(mut entry) = reader.entry()? {
                    progress_bar.set_message(&entry.path().to_string_lossy());
                    entry.extract(&dest)?;
                    progress_bar.inc(1);
                }

                progress_bar.finish_and_clear();
            } else {
                eprintln!("Unknown format");
            }

            Ok(())
        }

        Some(Command::List {input}) => {
            let input_file = input::Input::open(&input)?;

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

        _ => Ok(()),
    }
}
