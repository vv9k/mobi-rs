use crate::reader::Reader;
#[cfg(feature = "time")]
use chrono::NaiveDateTime;
use std::io;

#[derive(Debug, PartialEq, Default)]
/// Strcture that holds header information
pub struct Header {
    pub name: String,
    pub attributes: u16,
    pub version: u16,
    pub created: u32,
    pub modified: u32,
    pub backup: u32,
    pub modnum: u32,
    pub app_info_id: u32,
    pub sort_info_id: u32,
    pub typ_e: String,
    pub creator: String,
    pub unique_id_seed: u32,
    pub next_record_list_id: u32,
    pub num_records: u16,
}

impl Header {
    /// Parse a header from the content. The reader must be advanced to the starting position of the
    /// header, at byte 0.
    pub(crate) fn parse<R: io::Read>(reader: &mut Reader<R>) -> io::Result<Header> {
        Ok(Header {
            name: reader.read_string_header(32)?,
            attributes: reader.read_u16_be()?,
            version: reader.read_u16_be()?,
            created: reader.read_u32_be()?,
            modified: reader.read_u32_be()?,
            backup: reader.read_u32_be()?,
            modnum: reader.read_u32_be()?,
            app_info_id: reader.read_u32_be()?,
            sort_info_id: reader.read_u32_be()?,
            typ_e: reader.read_string_header(4)?,
            creator: reader.read_string_header(4)?,
            unique_id_seed: reader.read_u32_be()?,
            next_record_list_id: reader.read_u32_be()?,
            num_records: reader.read_u16_be()?,
        })
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
    use super::Header;
    use crate::book;

    #[test]
    fn parse() {
        let header = Header {
            name: String::from("Lord_of_the_Rings_-_Fellowship_\u{0}"),
            attributes: 0,
            version: 0,
            created: 1299709979,
            modified: 1299709979,
            backup: 0,
            modnum: 0,
            app_info_id: 0,
            sort_info_id: 0,
            typ_e: String::from("BOOK"),
            creator: String::from("MOBI"),
            unique_id_seed: 292,
            next_record_list_id: 0,
            num_records: 292,
        };
        let mut reader = book::u8_reader(book::HEADER.to_vec());
        let parsed_header = Header::parse(&mut reader);
        assert_eq!(header, parsed_header.unwrap())
    }
}
