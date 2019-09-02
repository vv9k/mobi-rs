//! A module about ext header
use super::*;
macro_rules! return_or_err {
    ($x:expr) => {
        match $x {
            Ok(data) => data,
            Err(e) => return Err(e),
        }
    };
}
pub enum BookInfo {
    Author,
    Publisher,
    Description,
    Isbn,
    PublishDate,
    Contributor,
    Title,
}
/// Parameters of Exth Header
pub enum ExtHeaderData {
    Identifier,
    HeaderLength,
    RecordCount,
}
#[derive(Debug, Default, PartialEq)]
/// Strcture that holds Exth header information
pub struct ExtHeader {
    pub identifier: u32,
    pub header_length: u32,
    pub record_count: u32,
    pub records: HashMap<u32, String>,
}
impl fmt::Display for ExtHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EXTHEADER
Identifier:             {}
Header_length:          {}
Record_count:           {}
Records:                {:#?}",
            self.identifier, self.header_length, self.record_count, self.records,
        )
    }
}
impl ExtHeader {
    /// Parse a Exth header from the content
    pub fn parse(content: &[u8], num_of_records: u16) -> Result<ExtHeader, std::io::Error> {
        let identifier = return_or_err!(ExtHeader::get_headers_u32(
            content,
            ExtHeaderData::Identifier,
            num_of_records
        ));
        let header_length = return_or_err!(ExtHeader::get_headers_u32(
            content,
            ExtHeaderData::HeaderLength,
            num_of_records
        ));
        let record_count = return_or_err!(ExtHeader::get_headers_u32(
            content,
            ExtHeaderData::RecordCount,
            num_of_records
        ));
        let mut extheader = ExtHeader {
            identifier,
            header_length,
            record_count,
            records: HashMap::new(),
        };
        extheader.get_records(content, num_of_records);
        Ok(extheader)
    }
    /// Gets u32 header value from specific location
    fn get_headers_u32(
        content: &[u8],
        extheader: ExtHeaderData,
        num_of_records: u16,
    ) -> Result<u32, std::io::Error> {
        let mut reader = Cursor::new(content);
        let position = match extheader {
            ExtHeaderData::Identifier => 328,
            ExtHeaderData::HeaderLength => 332,
            ExtHeaderData::RecordCount => 336,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u32::<BigEndian>()
    }
    /// Gets header records
    fn get_records(&mut self, content: &[u8], num_of_records: u16) {
        let mut records = HashMap::new();
        let mut reader = Cursor::new(content);
        let position: u64 = 340 + u64::from(num_of_records * 8);
        reader.set_position(position);
        for _i in 0..self.record_count {
            let mut record_data = vec![];
            let record_type = reader.read_u32::<BigEndian>().unwrap_or(0);
            let record_len = reader.read_u32::<BigEndian>().unwrap_or(0);
            for _j in 0..record_len - 8 {
                record_data.push(reader.read_u8().unwrap_or(0));
            }
            records.insert(
                record_type,
                String::from_utf8_lossy(&record_data[..])
                    .to_owned()
                    .to_string(),
            );
        }
        self.records = records;
    }
    pub fn get_book_info(&self, info: BookInfo) -> Option<&String> {
        let record: u32 = match info {
            BookInfo::Author => 100,
            BookInfo::Publisher => 101,
            BookInfo::Description => 103,
            BookInfo::Isbn => 104,
            BookInfo::PublishDate => 106,
            BookInfo::Contributor => 108,
            BookInfo::Title => 503,
        };
        self.records.get(&record)
    }
}
