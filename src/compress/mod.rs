//! Stream-oriented compression and decompression.
//!
//! Formats such as TAR are often wrapped with stream compression since it is
//! not supported by the archive format itself.

use crate::io::input::Input;
use std::io::{BufRead, Result};

pub mod formats;

/// Decode the given input stream automatically (if required), returning a new
/// deooded stream.
///
/// This function will attempt to detect if multiple layers of compression
/// algorithms are being used and decode them automatically.
pub fn detect_decode<'r>(mut input: Input<'r>) -> Result<Input<'r>> {
    'format: loop {
        let buf = input.fill_buf()?;

        // Detect a compression format, if any.
        for format in formats::all() {
            if format.match_bytes(&buf) {
                log::debug!("detected {} compression", format.id());

                // Wrap the input in a decoder.
                input = Input::from_reader(format.new_decoder(input)?)?;

                continue 'format;
            }
        }

        // No more compression formats detected.
        return Ok(input);
    }
}
