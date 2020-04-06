use super::Format;
use std::io::Read;

pub struct Zip;

impl Format for Zip {
    fn detect(reader: &mut impl Read) -> bool {
        false
    }
}
