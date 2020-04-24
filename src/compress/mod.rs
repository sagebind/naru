//! Stream-oriented compression and decompression.
//!
//! Formats such as TAR are often wrapped with stream compression since it is
//! not supported by the archive format itself.

use crate::io::input::Input;
use std::io::{BufRead, Read, Result};

pub mod formats;

pub fn wrap_decompressors(input: &mut Input) -> Result<()> {

    'find: loop {
        let magic = input.fill_buf()?.to_vec();

        for format in formats::all() {
            if format.match_bytes(&magic) {
                log::debug!("creating {} decoder", format.id());
                input.try_wrap(|reader| format.new_decoder(reader))?;
                continue 'find;
            }
        }

        break;
    }

    Ok(())
}
