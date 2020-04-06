use std::io::Read;

pub mod zip;

pub trait Format {
    fn detect(reader: &mut impl Read) -> bool;
}
