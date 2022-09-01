use crate::{Reader, Writer};

#[cfg(feature = "time")]
use chrono::NaiveDateTime;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeaderParseError {
    #[error("this book is an Amazon Topaz book and it cannot be processed")]
    IsTopazError,
    #[error("this book is an Amazon KFX book and it cannot be processed")]
    IsKfxError,
    #[error("expected type header identifier BOOK or TEXT")]
    InvalidTypeIdentifier,
    #[error("expected creator header identifier MOBI or READ")]
    InvalidCreatorIdentifier,
    #[error(transparent)]
    IoError(#[from] io::Error),
}

#[derive(Debug, PartialEq, Eq, Default)]
/// Structure that holds header information
pub struct Header {
    pub name: Vec<u8>,
    pub attributes: u16,
    pub version: u16,
    pub created: u32,
    pub modified: u32,
    pub backup: u32,
    pub modnum: u32,
    pub app_info_id: u32,
    pub sort_info_id: u32,
    pub type_: [u8; 4],
    pub creator: [u8; 4],
    pub unique_id_seed: u32,
    pub next_record_list_id: u32,
    pub num_records: u16,
}

impl Header {
    /// Parse a header from the content. The reader must be advanced to the starting position of the
    /// header, at byte 0.
    pub(crate) fn parse<R: io::Read>(reader: &mut Reader<R>) -> Result<Header, HeaderParseError> {
        Ok(Header {
            name: {
                let bytes = reader.read_vec_header(32)?;
                if bytes.starts_with(b"TPZ") {
                    return Err(HeaderParseError::IsTopazError);
                } else if bytes.starts_with(b"\xeaDRMION\xee") {
                    return Err(HeaderParseError::IsKfxError);
                }
                bytes
            },
            attributes: reader.read_u16_be()?,
            version: reader.read_u16_be()?,
            created: reader.read_u32_be()?,
            modified: reader.read_u32_be()?,
            backup: reader.read_u32_be()?,
            modnum: reader.read_u32_be()?,
            app_info_id: reader.read_u32_be()?,
            sort_info_id: reader.read_u32_be()?,
            type_: {
                let ty = reader.read_u32_be()?.to_be_bytes();
                if &ty != b"BOOK" && &ty != b"TEXT" {
                    return Err(HeaderParseError::InvalidTypeIdentifier);
                }
                ty
            },
            creator: {
                let creator = reader.read_u32_be()?.to_be_bytes();
                if &creator != b"MOBI" && &creator != b"READ" {
                    return Err(HeaderParseError::InvalidCreatorIdentifier);
                }
                creator
            },
            unique_id_seed: reader.read_u32_be()?,
            next_record_list_id: reader.read_u32_be()?,
            num_records: reader.read_u16_be()?,
        })
    }

    pub(crate) fn write<W: io::Write>(
        &self,
        w: &mut Writer<W>,
        num_records: u16,
    ) -> io::Result<()> {
        w.write_be(&self.name)?;
        w.write_be(self.attributes)?;
        w.write_be(self.version)?;
        w.write_be(self.created)?;
        // User should change this themselves?
        w.write_be(self.modified)?;
        w.write_be(self.backup)?;
        w.write_be(self.modnum)?;
        w.write_be(self.app_info_id)?;
        w.write_be(self.sort_info_id)?;
        w.write_be(self.type_.as_slice())?;
        w.write_be(self.creator.as_slice())?;
        w.write_be(self.unique_id_seed)?;
        w.write_be(self.next_record_list_id)?;
        w.write_be(num_records)
    }

    #[cfg(feature = "time")]
    /// Returns a chrono::NaiveDateTime timestamp of file creation
    /// This field is only available using `time` feature
    pub(crate) fn created_datetime(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(i64::from(self.created), 0)
    }

    #[cfg(feature = "time")]
    /// Returns a chrono::NaiveDateTime timestamp of file modification
    /// This field is only available using `time` feature
    pub(crate) fn mod_datetime(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(i64::from(self.modified), 0)
    }

    #[cfg(not(feature = "time"))]
    /// Returns a u32 timestamp of creation. This is a fallback
    /// method when `time` feature is disabled.
    pub(crate) fn created_datetime(&self) -> u32 {
        self.created
    }

    #[cfg(not(feature = "time"))]
    /// Returns a u32 timestamp of last modification. This is a fallback
    /// method when `time` feature is disabled.
    pub(crate) fn mod_datetime(&self) -> u32 {
        self.modified
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::book;
    use crate::writer::Writer;

    #[test]
    fn parse() {
        let header = Header {
            name: b"Lord_of_the_Rings_-_Fellowship_\0".to_vec(),
            attributes: 0,
            version: 0,
            created: 1299709979,
            modified: 1299709979,
            backup: 0,
            modnum: 0,
            app_info_id: 0,
            sort_info_id: 0,
            type_: *b"BOOK",
            creator: *b"MOBI",
            unique_id_seed: 292,
            next_record_list_id: 0,
            num_records: 292,
        };

        let mut reader = book::u8_reader(book::HEADER.to_vec());
        let parsed_header = Header::parse(&mut reader);
        assert_eq!(header, parsed_header.unwrap());
    }

    #[test]
    fn write() {
        let header = book::HEADER.to_vec();

        let mut reader = book::u8_reader(header.clone());
        let parsed_header = Header::parse(&mut reader).unwrap();

        let mut buf = vec![];

        parsed_header
            .write(&mut Writer::new(&mut buf), 292)
            .unwrap();
        assert_eq!(header.len(), buf.len());
        assert_eq!(header, buf);
    }
}
