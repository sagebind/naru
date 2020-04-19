use crate::{
    archive,
    io::output::Output,
};
use indicatif::ProgressBar;
use std::{
    error::Error,
    fs::File,
    io::BufReader,
    path::PathBuf,
};
use structopt::StructOpt;
use walkdir::WalkDir;

/// Create a new archive.
#[derive(Debug, StructOpt)]
pub struct Command {
    /// When adding a directory recursively, skip any child directory that
    /// is on a different file system than the starting directory.
    #[structopt(long)]
    one_file_system: bool,

    /// If set, symbolic links will not be followed and instead stored in the
    /// archive as symbolic links.
    ///
    /// Not all formats support symbolic link entries. Setting this flag when
    /// trying to create an archive in a format that does not support it will
    /// produce an error.
    #[structopt(long)]
    preserve_symlinks: bool,

    /// Archive file ("-" for stdin)
    #[structopt(parse(from_os_str))]
    output: PathBuf,

    /// Files to add
    #[structopt(parse(from_os_str))]
    files: Vec<PathBuf>,
}

impl Command {
    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        let mut output = Output::create(&self.output)?;

        if let Some(mut writer) = archive::create(&mut output)? {
            let entries = collect_paths(&self.files)?;

            let progress_bar = ProgressBar::new(entries.len() as u64)
                .with_style(super::progress_bar_style());
            progress_bar.enable_steady_tick(1000);

            for entry in entries {
                let path = entry.path();

                progress_bar.set_message(&path.to_string_lossy());

                let metadata = entry.metadata()
                    .map(Into::into)
                    .unwrap_or_default();

                if entry.file_type().is_dir() {
                    writer.add_directory(path, metadata)?;
                } else {
                    writer.add_file(path, metadata, &mut BufReader::new(File::open(path)?))?;
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
}

fn collect_paths(paths: &[PathBuf]) -> Result<Vec<walkdir::DirEntry>, Box<dyn Error>> {
    paths.iter()
        .flat_map(|path| WalkDir::new(path).follow_links(true))
        .map(|result| result.map_err(|e| Box::new(e) as Box<dyn std::error::Error>))
        .collect()
}
