use crate::{
    archive,
    io::input::Input,
};
use indicatif::ProgressBar;
use std::{
    borrow::Cow,
    error::Error,
    fs,
    path::Path,
};

/// Handles the `extract` command.
pub fn extract(input: &Path, dest: Option<&Path>) -> Result<(), Box<dyn Error>> {
    let input = Input::open(&input)?;

    let dest = match dest {
        Some(path) => Cow::Borrowed(path),

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

            Cow::Owned(path)
        },
    };

    if let Some(mut reader) = archive::open(input)? {
        let progress_bar = match reader.len() {
            Some(len) => ProgressBar::new(len).with_style(super::progress_bar_style()),
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
