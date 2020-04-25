use crate::{
    archive,
    archive::{Entry, EntryType},
    io::input::Input,
};
use glob::Pattern;
use indicatif::ProgressBar;
use std::{
    borrow::Cow,
    error::Error,
    fs,
    fs::{File, OpenOptions},
    io,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

/// Extract files from an archive
#[derive(Debug, StructOpt)]
pub struct Command {
    /// Directory to extract into
    #[structopt(short = "d", parse(from_os_str))]
    dest: Option<PathBuf>,

    /// If extracting a file would overwrite an existing file, stop with an
    /// error.
    #[structopt(short, long)]
    keep_old_files: bool,

    #[structopt(short, long)]
    overwrite: bool,

    /// Do not restore file timestamps.
    #[structopt(long)]
    ignore_timestamp: bool,

    /// Do not restore UNIX file attributes.
    ///
    /// On Windows, only "readonly" is restored.
    #[structopt(long)]
    ignore_permissions: bool,

    /// Restore SUID/SGID bits.
    #[structopt(long)]
    preserve_extra_bits: bool,

    /// Extract slowly for testing purposes.
    #[cfg(debug_assertions)]
    #[structopt(long)]
    go_slow: bool,

    /// Input file ("-" for stdin)
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// File names to extract, by default entire archive is extracted.
    ///
    /// Names are paths relative to the root of the archive. Glob patterns can
    /// be used as paths to extract only files matching a pattern.
    ///
    /// If a name contains a slash (`/`) at the start or middle of a pattern,
    /// then it is treated as a path relative to the root of the archive. Only
    /// files whose full path matches the pattern will be extracted. If the
    /// pattern does not contain a slash (or it is at the end) then just file
    /// names are matched against the pattern.
    files: Vec<Pattern>,
}

impl Command {
    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        let input = Input::open(&self.input)?;

        let dest = match &self.dest {
            Some(path) => Cow::Borrowed(path),

            // If no destination path was given, assume one based on the
            // name of the input file.
            None => {
                let mut path = std::env::current_dir()?;

                if let Some(archive_file_name) = self.input.file_stem() {
                    if archive_file_name != "-" {
                        path = path.join(archive_file_name);
                    }
                }

                Cow::Owned(path)
            },
        };

        if let Some(mut reader) = archive::open(input)? {
            // Ensure the target directory is created if it does not already
            // exist.
            fs::create_dir_all(dest.as_ref())?;

            let progress_bar = match reader.len() {
                Some(len) => ProgressBar::new(len).with_style(super::progress_bar_style()),
                None => ProgressBar::new_spinner(),
            };

            while let Some(mut entry) = reader.entry()? {
                let path = entry.path();

                if self.should_extract(&path) {
                    progress_bar.set_message(&path.to_string_lossy());

                    #[cfg(debug_assertions)]
                    {
                        if self.go_slow {
                            std::thread::sleep_ms(1000);
                        }
                    }

                    self.extract(&mut *entry, &dest)?;
                }

                progress_bar.inc(1);
            }

            progress_bar.finish_and_clear();
        } else {
            eprintln!("Unknown format");
        }

        Ok(())
    }

    fn should_extract(&self, path: &Path) -> bool {
        if self.files.is_empty() {
            true
        } else {
            self.files.iter().any(|pattern| {
                // If a slash occurs anywhere other than the end of the pattern,
                // match against the whole path. Otherwise match against just
                // the name.
                let first_slash = pattern.as_str().find('/');

                if first_slash == None || first_slash == Some(pattern.as_str().len() - 1) {
                    match path.file_name() {
                        Some(file_name) => pattern.matches_path(Path::new(file_name)),
                        None => {
                            log::debug!("couldn't match path against pattern, no filename: {}", path.display());
                            false
                        },
                    }
                } else {
                    pattern.matches_path(path)
                }
            })
        }
    }

    /// Extract an entry into the file system within the given path.
    ///
    /// The entire path of this entry within the archive will be recreated in
    /// the destination path.
    fn extract(&self, entry: &mut dyn Entry, dir: &Path) -> Result<(), Box<dyn Error>> {
        let path = entry.path();
        let dest = dir.join(&path);

        // TODO: What if path points to a directory above the archive? (security)

        // Create parent directories if required.
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        let metadata = entry.metadata();

        match metadata.entry_type {
            EntryType::Directory => fs::create_dir_all(dest)?,

            EntryType::File => {
                // Create the file and stream this entry's bytes into it.
                let mut file = OpenOptions::new()
                    .create(true)
                    .create_new(self.keep_old_files)
                    .write(true)
                    .truncate(true)
                    .open(dest)?;

                io::copy(entry, &mut file)?;
            }
            _ => {
                log::warn!("skipping entry {}, unsupported type", path.display())
            }
        }

        Ok(())
    }
}
