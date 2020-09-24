use crate::{FieldHeaderEnum, HeaderField, Reader};
use std::io;

/// Parameters of Header
pub(crate) enum HeaderData {
    Name,
    Attributes,
    Version,
    Created,
    Modified,
    Backup,
    Modnum,
    AppInfoId,
    SortInfoId,
    TypE,
    Creator,
    UniqueIdSeed,
    NextRecordListId,
    NumOfRecords,
}
impl FieldHeaderEnum for HeaderData {}
impl HeaderField<HeaderData> for HeaderData {
    fn position(self) -> u16 {
        match self {
            HeaderData::Name => 0,
            HeaderData::Attributes => 32,
            HeaderData::Version => 34,
            HeaderData::Created => 36,
            HeaderData::Modified => 40,
            HeaderData::Backup => 44,
            HeaderData::Modnum => 48,
            HeaderData::AppInfoId => 52,
            HeaderData::SortInfoId => 56,
            HeaderData::TypE => 60,
            HeaderData::Creator => 64,
            HeaderData::UniqueIdSeed => 68,
            HeaderData::NextRecordListId => 72,
            HeaderData::NumOfRecords => 76,
        }
    }
}
#[derive(Debug, PartialEq, Default)]
/// Strcture that holds header information
pub struct Header {
    pub name: String,
    pub attributes: i16,
    pub version: i16,
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
    pub num_of_records: u16,
}
impl Header {
    /// Parse a header from the content
    pub(crate) fn parse(reader: &mut Reader) -> io::Result<Header> {
        use HeaderData::*;
        Ok(Header {
            name: reader.read_string_header(Name, 32),
            attributes: reader.read_i16_header(Attributes)?,
            version: reader.read_i16_header(Version)?,
            created: reader.read_u32_header(Created)?,
            modified: reader.read_u32_header(Modified)?,
            backup: reader.read_u32_header(Backup)?,
            modnum: reader.read_u32_header(Modnum)?,
            app_info_id: reader.read_u32_header(AppInfoId)?,
            sort_info_id: reader.read_u32_header(SortInfoId)?,
            typ_e: reader.read_string_header(TypE, 4),
            creator: reader.read_string_header(Creator, 4),
            unique_id_seed: reader.read_u32_header(UniqueIdSeed)?,
            next_record_list_id: reader.read_u32_header(NextRecordListId)?,
            num_of_records: reader.read_u16_header(NumOfRecords)?,
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
            num_of_records: 292,
        };
        let mut reader = book::test_reader();
        let parsed_header = Header::parse(&mut reader);
        assert_eq!(header, parsed_header.unwrap())
    }
}
