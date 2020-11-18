pub(crate) mod exth;
pub(crate) mod header;
pub(crate) mod mobih;
pub(crate) mod palmdoch;

pub use self::{
    exth::ExtHeader,
    header::Header,
    mobih::{MobiHeader, TextEncoding},
    palmdoch::PalmDocHeader,
};
use crate::Reader;
use std::io;

/// Trait allowing generic reading of header fields
pub(crate) trait HeaderField {
    /// Returns a position in the text where this field can be read
    fn position(self) -> u64;
}

#[derive(Debug, Default)]
/// Holds all headers containing low level metadata of a mobi book
pub struct Metadata {
    pub header: Header,
    pub palmdoc: PalmDocHeader,
    pub mobi: MobiHeader,
    pub exth: ExtHeader,
}
impl Metadata {
    /// Construct a Metadata object from a slice of bytes
    pub fn new<B: AsRef<Vec<u8>>>(bytes: B) -> io::Result<Metadata> {
        Metadata::from_reader(&mut Reader::new(bytes.as_ref()))
    }

    pub(crate) fn from_reader(mut reader: &mut Reader) -> io::Result<Metadata> {
        let header = Header::parse(&mut reader)?;
        reader.set_num_of_records(header.num_of_records);
        let palmdoc = PalmDocHeader::parse(&mut reader)?;
        let mobi = MobiHeader::parse(&mut reader)?;
        let exth = {
            if mobi.has_exth_header {
                ExtHeader::parse(&mut reader, mobi.header_length)?
            } else {
                ExtHeader::default()
            }
        };
        Ok(Metadata {
            header,
            palmdoc,
            mobi,
            exth,
        })
    }
}
