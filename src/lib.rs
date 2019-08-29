//! An implementation of [MOBI](https://wiki.mobileread.com/wiki/MOBI) format data parsing and manipulation, written in Rust.
//!
//! The code is available on [GitHub](https://github.com/wojciechkepka/mobi-rs)
//!
//! License: [*Apache-2.0*](https://github.com/wojciechkepka/mobi-rs/blob/master/license)
//!
use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::Path;

macro_rules! return_or_err {
    ($x:expr) => {
        match $x {
            Ok(data) => data,
            Err(e) => return Err(e),
        }
    };
}
#[derive(Debug)]
/// Structure that holds parsed ebook information and contents
pub struct Mobi {
    pub contents: Vec<u8>,
    pub header: Header,
    pub records: Vec<Record>,
    pub palmdoc: PalmDocHeader,
    pub mobi: MobiHeader,
    pub exth: ExtHeader,
}
impl Mobi {
    /// Construct a Mobi object from passed file path
    /// Returns std::io::Error if there is a problem with the provided path
    pub fn init<P: AsRef<Path>>(file_path: P) -> Result<Mobi, std::io::Error> {
        let contents = return_or_err!(fs::read(file_path));
        let header = return_or_err!(Header::parse(&contents));
        let records = return_or_err!(Record::parse_records(&contents, header.num_of_records));
        let palmdoc = return_or_err!(PalmDocHeader::parse(&contents, header.num_of_records));
        let mobi = return_or_err!(MobiHeader::parse(&contents, header.num_of_records));
        let mut exth = ExtHeader::default();
        if mobi.has_exth_header {
            exth = return_or_err!(ExtHeader::parse(&contents, header.num_of_records));
        }
        Ok(Mobi {
            contents,
            header,
            records,
            palmdoc,
            mobi,
            exth,
        })
    }
    /// Returns author record if such exists
    pub fn author(&self) -> Option<&String> {
        self.exth.get_book_info(BookInfo::Author)
    }
    /// Returns publisher record if such exists
    pub fn publisher(&self) -> Option<&String> {
        self.exth.get_book_info(BookInfo::Publisher)
    }
    /// Returns description record if such exists
    pub fn description(&self) -> Option<&String> {
        self.exth.get_book_info(BookInfo::Description)
    }
    /// Returns isbn record if such exists
    pub fn isbn(&self) -> Option<&String> {
        self.exth.get_book_info(BookInfo::Isbn)
    }
    /// Returns publish_date record if such exists
    pub fn publish_date(&self) -> Option<&String> {
        self.exth.get_book_info(BookInfo::PublishDate)
    }
    /// Returns contributor record if such exists
    pub fn contributor(&self) -> Option<&String> {
        self.exth.get_book_info(BookInfo::Contributor)
    }
    /// Returns title record if such exists
    pub fn title(&self) -> Option<&String> {
        self.exth.get_book_info(BookInfo::Title)
    }
    /// Prints basic information about the book into stdout
    /// 
    /// # Example
    /// ```rust
    /// use mobi::Mobi;
    /// 
    /// fn main() {
    ///     let m = Mobi::init("/home/wojtek/Downloads/lotr.mobi");
    ///     m.print_book_info();
    /// }
    /// ```
    /// yields:
    /// ~~~
    /// ----------------------------------------------------------
    /// Title:          The Fellowship of the Ring
    /// Author:         J. R. R. Tolkien
    /// Publisher:      Houghton Mifflin
    /// Description:    SUMMARY: For over fifty years, J.R.R. Tolkien’s peerless fantasy has accumulated worldwide acclaim as the greatest adventure tale ever written.No other writer has created a world as distinct as Middle-earth, complete with its own geography, history, languages, and legends. And no one has created characters as endearing as Tolkien’s large-hearted, hairy-footed hobbits. Tolkien’s The Lord of the Rings continues to seize the imaginations of readers of all ages, and this new three-volume paperback edition is designed to appeal to the youngest of them.In ancient times the Rings of Power were crafted by the Elvensmiths, and Sauron, the Dark Lord, forged the One Ring, filling it with his own power so that he could rule all others. But the One Ring was taken from him, and though he sought it throughout Middle-earth, still it remained lost to him . . .
    /// ISBN:           9780618574940
    /// Publish Date:   2005-07-15T07:00:00+00:00
    /// Contributor:    calibre (0.7.23) [http://calibre-ebook.com]
    /// ----------------------------------------------------------
    /// ~~~
    pub fn print_book_info(&self) {
        let empty_str = String::from("");
        println!(
            "
----------------------------------------------------------
Title:          {}
Author:         {}
Publisher:      {}
Description:    {}
ISBN:           {}
Publish Date:   {}
Contributor:    {}
----------------------------------------------------------
",
            self.title().unwrap_or(&empty_str),
            self.author().unwrap_or(&empty_str),
            self.publisher().unwrap_or(&empty_str),
            self.description().unwrap_or(&empty_str),
            self.isbn().unwrap_or(&empty_str),
            self.publish_date().unwrap_or(&empty_str),
            self.contributor().unwrap_or(&empty_str)
        );
    }
}
/// Parameters of Header
pub enum HeaderData {
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
/// Parameters of PalmDOC Header
pub enum PalmDocHeaderData {
    Compression,
    TextLength,
    RecordCount,
    RecordSize,
    EncryptionType,
}

#[derive(Debug, PartialEq)]
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
    pub fn parse(content: &[u8]) -> Result<Header, std::io::Error> {
        macro_rules! header {
            ($method:ident($type:ident)) => {
                return_or_err!(Header::$method(content, HeaderData::$type))
            };
        }
        Ok(Header {
            name: Header::get_headers_string(content, HeaderData::Name),
            attributes: header!(get_headers_i16(Attributes)),
            version: header!(get_headers_i16(Version)),
            created: header!(get_headers_u32(Created)),
            modified: header!(get_headers_u32(Modified)),
            backup: header!(get_headers_u32(Backup)),
            modnum: header!(get_headers_u32(Modnum)),
            app_info_id: header!(get_headers_u32(AppInfoId)),
            sort_info_id: header!(get_headers_u32(SortInfoId)),
            typ_e: Header::get_headers_string(content, HeaderData::TypE),
            creator: Header::get_headers_string(content, HeaderData::Creator),
            unique_id_seed: header!(get_headers_u32(UniqueIdSeed)),
            next_record_list_id: header!(get_headers_u32(NextRecordListId)),
            num_of_records: header!(get_headers_u16(NumOfRecords)),
        })
    }
    /// Gets i16 header value from specific location
    fn get_headers_i16(content: &[u8], header: HeaderData) -> Result<i16, std::io::Error> {
        let mut reader = Cursor::new(content);
        let position = match header {
            HeaderData::Attributes => 32,
            HeaderData::Version => 34,
            _ => 0,
        };
        reader.set_position(position);
        reader.read_i16::<BigEndian>()
    }
    /// Gets u16 header value from specific location
    pub fn get_headers_u16(content: &[u8], header: HeaderData) -> Result<u16, std::io::Error> {
        let mut reader = Cursor::new(content);
        let position = match header {
            HeaderData::NumOfRecords => 76,
            _ => 0,
        };
        reader.set_position(position);
        reader.read_u16::<BigEndian>()
    }
    /// Gets u32 header value from specific location
    fn get_headers_u32(content: &[u8], header: HeaderData) -> Result<u32, std::io::Error> {
        let mut reader = Cursor::new(content);
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
            HeaderData::Name => String::from_utf8_lossy(&content[0..32]).to_owned().to_string(),
            HeaderData::TypE => String::from_utf8_lossy(&content[60..64]).to_owned().to_string(),
            HeaderData::Creator => String::from_utf8_lossy(&content[64..68]).to_owned().to_string(),
            _ => String::new(),
        }
    }
}
#[derive(Debug, PartialEq)]
/// Strcture that holds PalmDOC header information
pub struct PalmDocHeader {
    pub compression: u16,
    pub text_length: u32,
    pub record_count: u16,
    pub record_size: u16,
    pub encryption_type: u16,
}
impl PalmDocHeader {
    /// Parse a PalmDOC header from the content
    pub fn parse(content: &[u8], num_of_records: u16) -> Result<PalmDocHeader, std::io::Error> {
        macro_rules! pdheader {
            ($method:ident($type:ident)) => {
                return_or_err!(PalmDocHeader::$method(
                    content,
                    PalmDocHeaderData::$type,
                    num_of_records
                ))
            };
        }
        Ok(PalmDocHeader {
            compression: pdheader!(get_headers_u16(Compression)),
            text_length: pdheader!(get_headers_u32(TextLength)),
            record_count: pdheader!(get_headers_u16(RecordCount)),
            record_size: pdheader!(get_headers_u16(RecordSize)),
            encryption_type: pdheader!(get_headers_u16(EncryptionType)),
        })
    }
    /// Gets u16 header value from specific location
    fn get_headers_u16(
        content: &[u8],
        pdheader: PalmDocHeaderData,
        num_of_records: u16,
    ) -> Result<u16, std::io::Error> {
        let mut reader = Cursor::new(content);
        let position = match pdheader {
            PalmDocHeaderData::Compression => 80,
            PalmDocHeaderData::RecordCount => 88,
            PalmDocHeaderData::RecordSize => 90,
            PalmDocHeaderData::EncryptionType => 92,
            _ => 0,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u16::<BigEndian>()
    }
    /// Gets u32 header value from specific location
    fn get_headers_u32(
        content: &[u8],
        pdheader: PalmDocHeaderData,
        num_of_records: u16,
    ) -> Result<u32, std::io::Error> {
        let mut reader = Cursor::new(content);
        let position = match pdheader {
            PalmDocHeaderData::TextLength => 84,
            _ => 0,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u32::<BigEndian>()
    }
}

#[derive(Debug, PartialEq)]
/// Strcture that holds Mobi header information
pub struct MobiHeader {
    pub identifier: u32,
    pub header_length: u32,
    pub mobi_type: u32,
    pub text_encoding: u32,
    pub id: u32,
    pub gen_version: u32,
    pub first_non_book_index: u32,
    pub name: String,
    pub name_offset: u32,
    pub name_length: u32,
    pub language: u32,
    pub input_language: u32,
    pub output_language: u32,
    pub format_version: u32,
    pub first_image_index: u32,
    pub first_huff_record: u32,
    pub huff_record_count: u32,
    pub first_data_record: u32,
    pub data_record_count: u32,
    pub exth_flags: u32,
    pub has_exth_header: bool,
    pub drm_offset: u32,
    pub drm_count: u32,
    pub drm_size: u32,
    pub drm_flags: u32,
    pub last_image_record: u16,
    pub fcis_record: u32,
    pub flis_record: u32,
}
/// Parameters of Mobi Header
pub enum MobiHeaderData {
    Identifier,
    HeaderLength,
    MobiType,
    TextEncoding,
    Id,
    GenVersion,
    FirstNonBookIndex,
    NameOffset,
    NameLength,
    Language,
    InputLanguage,
    OutputLanguage,
    FormatVersion,
    FirstImageIndex,
    FirstHuffRecord,
    HuffRecordCount,
    FirstDataRecord,
    DataRecordCount,
    ExthFlags,
    DrmOffset,
    DrmCount,
    DrmSize,
    DrmFlags,
    LastImageRecord,
    FcisRecord,
    FlisRecord,
}
impl MobiHeader {
    /// Parse a Mobi header from the content
    pub fn parse(content: &[u8], num_of_records: u16) -> Result<MobiHeader, std::io::Error> {
        macro_rules! mobiheader {
            ($method:ident($enum:ident)) => {
                return_or_err!(MobiHeader::$method(
                    content,
                    MobiHeaderData::$enum,
                    num_of_records
                ))
            };
        }
        Ok(MobiHeader {
            identifier: mobiheader!(get_headers_u32(Identifier)),
            header_length: mobiheader!(get_headers_u32(HeaderLength)),
            mobi_type: mobiheader!(get_headers_u32(MobiType)),
            text_encoding: mobiheader!(get_headers_u32(TextEncoding)),
            id: mobiheader!(get_headers_u32(Id)),
            gen_version: mobiheader!(get_headers_u32(GenVersion)),
            first_non_book_index: mobiheader!(get_headers_u32(FirstNonBookIndex)),
            name: return_or_err!(MobiHeader::name(content, num_of_records)),
            name_offset: mobiheader!(get_headers_u32(NameOffset)),
            name_length: mobiheader!(get_headers_u32(NameLength)),
            language: mobiheader!(get_headers_u32(Language)),
            input_language: mobiheader!(get_headers_u32(InputLanguage)),
            output_language: mobiheader!(get_headers_u32(OutputLanguage)),
            format_version: mobiheader!(get_headers_u32(FormatVersion)),
            first_image_index: mobiheader!(get_headers_u32(FirstImageIndex)),
            first_huff_record: mobiheader!(get_headers_u32(FirstHuffRecord)),
            huff_record_count: mobiheader!(get_headers_u32(HuffRecordCount)),
            first_data_record: mobiheader!(get_headers_u32(FirstDataRecord)),
            data_record_count: mobiheader!(get_headers_u32(DataRecordCount)),
            exth_flags: mobiheader!(get_headers_u32(ExthFlags)),
            has_exth_header: MobiHeader::exth_header(mobiheader!(get_headers_u32(ExthFlags))),
            drm_offset: mobiheader!(get_headers_u32(DrmOffset)),
            drm_count: mobiheader!(get_headers_u32(DrmCount)),
            drm_size: mobiheader!(get_headers_u32(DrmSize)),
            drm_flags: mobiheader!(get_headers_u32(DrmFlags)),
            last_image_record: mobiheader!(get_headers_u16(LastImageRecord)),
            fcis_record: mobiheader!(get_headers_u32(FcisRecord)),
            flis_record: mobiheader!(get_headers_u32(FlisRecord)),
        })
    }
    /// Gets u32 header value from specific location
    fn get_headers_u32(
        content: &[u8],
        mheader: MobiHeaderData,
        num_of_records: u16,
    ) -> Result<u32, std::io::Error> {
        let mut reader = Cursor::new(content);
        let position = match mheader {
            MobiHeaderData::Identifier => 96,
            MobiHeaderData::HeaderLength => 100,
            MobiHeaderData::MobiType => 104,
            MobiHeaderData::TextEncoding => 108,
            MobiHeaderData::Id => 112,
            MobiHeaderData::GenVersion => 116,
            MobiHeaderData::FirstNonBookIndex => 160,
            MobiHeaderData::NameOffset => 164,
            MobiHeaderData::NameLength => 168,
            MobiHeaderData::Language => 172,
            MobiHeaderData::InputLanguage => 176,
            MobiHeaderData::OutputLanguage => 180,
            MobiHeaderData::FormatVersion => 184,
            MobiHeaderData::FirstImageIndex => 188,
            MobiHeaderData::FirstHuffRecord => 192,
            MobiHeaderData::HuffRecordCount => 196,
            MobiHeaderData::FirstDataRecord => 200,
            MobiHeaderData::DataRecordCount => 204,
            MobiHeaderData::ExthFlags => 208,
            MobiHeaderData::DrmOffset => 248,
            MobiHeaderData::DrmCount => 252,
            MobiHeaderData::DrmSize => 256,
            MobiHeaderData::DrmFlags => 260,
            MobiHeaderData::FcisRecord => 280,
            MobiHeaderData::FlisRecord => 288,
            _ => 0,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u32::<BigEndian>()
    }
    /// Gets u16 header value from specific location
    fn get_headers_u16(
        content: &[u8],
        mheader: MobiHeaderData,
        num_of_records: u16,
    ) -> Result<u16, std::io::Error> {
        let mut reader = Cursor::new(content);
        let position = match mheader {
            MobiHeaderData::LastImageRecord => 274,
            _ => 0,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u16::<BigEndian>()
    }
    /// Returns the book name
    pub fn name(content: &[u8], num_of_records: u16) -> Result<String, std::io::Error> {
        let name_offset = return_or_err!(MobiHeader::get_headers_u32(
            content,
            MobiHeaderData::NameOffset,
            num_of_records
        ));
        let name_length = return_or_err!(MobiHeader::get_headers_u32(
            content,
            MobiHeaderData::NameLength,
            num_of_records
        ));
        let offset = name_offset as usize + (num_of_records * 8) as usize + 80;
        Ok(String::from_utf8_lossy(&content[offset..offset + name_length as usize]).to_owned().to_string())
    }
    /// Checks if there is a Exth Header and changes the parameter
    fn exth_header(exth_flags: u32) -> bool {
        (exth_flags & 0x40) != 0
    }
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
            records.insert(record_type, String::from_utf8_lossy(&record_data[..]).to_owned().to_string());
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

#[derive(Debug)]
/// A "cell" in the whole books content
pub struct Record {
    record_data_offset: u32,
    id: u32,
    record_data: String,
}
impl Record {
    #[allow(dead_code)]
    fn new() -> Record {
        Record {
            record_data_offset: 0,
            id: 0,
            record_data: String::new(),
        }
    }
    /// Reads into a string the record data based on record_data_offset
    fn record_data(&mut self, content: &[u8]) {
        if self.record_data_offset + 8 < content.len() as u32 {
            let s = &content[self.record_data_offset as usize..(self.record_data_offset + 8) as usize];
            self.record_data = String::from_utf8_lossy(s).to_owned().to_string();
        }
    }
    /// Parses a record from the reader at current position
    fn parse_record(reader: &mut Cursor<&[u8]>) -> Result<Record, std::io::Error> {
        let record_data_offset = return_or_err!(reader.read_u32::<BigEndian>());
        let id = return_or_err!(reader.read_u32::<BigEndian>());
        let mut record = Record {
            record_data_offset,
            id,
            record_data: String::new(),
        };
        record.record_data(*reader.get_ref());
        Ok(record)
    }
    /// Gets all records in the specified content
    fn parse_records(content: &[u8], num_of_records: u16) -> Result<Vec<Record>, std::io::Error> {
        let mut reader = Cursor::new(content);
        let mut records = vec![];
        for _i in 0..num_of_records {
            let record = return_or_err!(Record::parse_record(&mut reader));
            records.push(record);
        }
        Ok(records)
    }
}
