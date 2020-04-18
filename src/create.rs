use crate::{
    archive,
    io::output::Output,
};
use indicatif::ProgressBar;
use std::{
    error::Error,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Handles the `create` command.
pub fn create(output: &Path, files: &[PathBuf]) -> Result<(), Box<dyn Error>> {
    let mut output = Output::create(&output)?;

    if let Some(mut writer) = archive::create(&mut output)? {
        let entries = collect_paths(&files)?;

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

fn collect_paths(paths: &[PathBuf]) -> Result<Vec<walkdir::DirEntry>, Box<dyn Error>> {
    paths.iter()
        .flat_map(|path| WalkDir::new(path).follow_links(true))
        .map(|result| result.map_err(|e| Box::new(e) as Box<dyn std::error::Error>))
        .collect()
}
