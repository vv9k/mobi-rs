use super::*;
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
#[cfg(feature = "fmt")]
impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HEADER
Name:                   {}
Attributes:             {}
Version:                {}
Created:                {}
Modified:               {}
Backup:                 {}
Modnum:                 {}
App_info_id:            {}
Sort_info_id:           {}
Typ_e:                  {}
Creator:                {}
Unique_id_seed:         {}
Next_record_list_id:    {}
Num_of_records:         {}",
            self.name,
            self.attributes,
            self.version,
            self.created_datetime(),
            self.mod_datetime(),
            self.backup,
            self.modnum,
            self.app_info_id,
            self.sort_info_id,
            self.typ_e,
            self.creator,
            self.unique_id_seed,
            self.next_record_list_id,
            self.num_of_records,
        )
    }
}
impl Header {
    /// Parse a header from the content
    pub(crate) fn parse(content: &[u8]) -> Result<Header, std::io::Error> {
        macro_rules! header {
            ($method:ident($type:ident,$cursor:expr)) => {
                Header::$method($cursor, HeaderData::$type)?
            };
        }
        let mut reader = Cursor::new(content);
        Ok(Header {
            name: Header::get_headers_string(content, HeaderData::Name),
            attributes: header!(get_headers_i16(Attributes, &mut reader)),
            version: header!(get_headers_i16(Version, &mut reader)),
            created: header!(get_headers_u32(Created, &mut reader)),
            modified: header!(get_headers_u32(Modified, &mut reader)),
            backup: header!(get_headers_u32(Backup, &mut reader)),
            modnum: header!(get_headers_u32(Modnum, &mut reader)),
            app_info_id: header!(get_headers_u32(AppInfoId, &mut reader)),
            sort_info_id: header!(get_headers_u32(SortInfoId, &mut reader)),
            typ_e: Header::get_headers_string(content, HeaderData::TypE),
            creator: Header::get_headers_string(content, HeaderData::Creator),
            unique_id_seed: header!(get_headers_u32(UniqueIdSeed, &mut reader)),
            next_record_list_id: header!(get_headers_u32(NextRecordListId, &mut reader)),
            num_of_records: header!(get_headers_u16(NumOfRecords, &mut reader)),
        })
    }
    /// Gets i16 header value from specific location
    fn get_headers_i16(reader: &mut Cursor<&[u8]>, header: HeaderData) -> Result<i16, std::io::Error> {
        let position = match header {
            HeaderData::Attributes => 32,
            HeaderData::Version => 34,
            _ => 0,
        };
        reader.set_position(position);
        reader.read_i16::<BigEndian>()
    }
    /// Gets u16 header value from specific location
    pub(crate) fn get_headers_u16(
        reader: &mut Cursor<&[u8]>,
        header: HeaderData,
    ) -> Result<u16, std::io::Error> {
        let position = match header {
            HeaderData::NumOfRecords => 76,
            _ => 0,
        };
        reader.set_position(position);
        reader.read_u16::<BigEndian>()
    }
    /// Gets u32 header value from specific location
    fn get_headers_u32(reader: &mut Cursor<&[u8]>, header: HeaderData) -> Result<u32, std::io::Error> {
        let position = match header {
            HeaderData::Created => 36,
            HeaderData::Modified => 40,
            HeaderData::Backup => 44,
            HeaderData::Modnum => 48,
            HeaderData::AppInfoId => 52,
            HeaderData::SortInfoId => 56,
            HeaderData::UniqueIdSeed => 68,
            HeaderData::NextRecordListId => 72,
            _ => 0,
        };
        reader.set_position(position);
        reader.read_u32::<BigEndian>()
    }
    /// Creates a string based on header bytes from specific location
    fn get_headers_string(content: &[u8], header: HeaderData) -> String {
        match header {
            HeaderData::Name => String::from_utf8_lossy(&content[0..32])
                .to_owned()
                .to_string(),
            HeaderData::TypE => String::from_utf8_lossy(&content[60..64])
                .to_owned()
                .to_string(),
            HeaderData::Creator => String::from_utf8_lossy(&content[64..68])
                .to_owned()
                .to_string(),
            _ => String::new(),
        }
    }
    #[cfg(feature = "chrono")]
    /// Returns a chrono::NaiveDateTime timestamp of file creation
    /// This field is only available using `chrono` feature
    pub(crate) fn created_datetime(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(i64::from(self.created), 0)
    }
    #[cfg(feature = "chrono")]
    /// Returns a chrono::NaiveDateTime timestamp of file modification
    /// This field is only available using `chrono` feature
    pub(crate) fn mod_datetime(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(i64::from(self.modified), 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use book::BOOK;
    use header::Header;
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
        let parsed_header = Header::parse(BOOK);
        assert_eq!(header, parsed_header.unwrap())
    }
}
