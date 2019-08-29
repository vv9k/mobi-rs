//! An implementation of [MOBI](https://wiki.mobileread.com/wiki/MOBI) format data parsing and manipulation, written in Rust.
//!
//! The code is available on [GitHub](https://github.com/wojciechkepka/mobi-rs)
//!
//! License: [*Apache-2.0*](https://github.com/wojciechkepka/mobi-rs/blob/master/license)
//!
mod utils;
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
        macro_rules! get_header {
            ($type:ident, $method:ident) => {
                return_or_err!(Header::$method(content, HeaderData::$type))
            };
        }
        let name = Header::get_headers_string(content, HeaderData::Name);
        let attributes = get_header!(Attributes, get_headers_i16);
        let version = get_header!(Version, get_headers_i16);
        let created = get_header!(Created, get_headers_u32);
        let modified = get_header!(Modified, get_headers_u32);
        let backup = get_header!(Backup, get_headers_u32);
        let modnum = get_header!(Modnum, get_headers_u32);
        let app_info_id = get_header!(AppInfoId, get_headers_u32);
        let sort_info_id = get_header!(SortInfoId, get_headers_u32);
        let typ_e = Header::get_headers_string(content, HeaderData::TypE);
        let creator = Header::get_headers_string(content, HeaderData::Creator);
        let unique_id_seed = get_header!(UniqueIdSeed, get_headers_u32);
        let next_record_list_id = get_header!(NextRecordListId, get_headers_u32);
        let num_of_records = get_header!(NumOfRecords, get_headers_u16);
        Ok(Header {
            name,
            attributes,
            version,
            created,
            modified,
            backup,
            modnum,
            app_info_id,
            sort_info_id,
            typ_e,
            creator,
            unique_id_seed,
            next_record_list_id,
            num_of_records,
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
            HeaderData::Name => utils::u8_as_string(&content[0..32]),
            HeaderData::TypE => utils::u8_as_string(&content[60..64]),
            HeaderData::Creator => utils::u8_as_string(&content[64..68]),
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
        macro_rules! get_pdheader {
            ($type:ident, $method:ident) => {
                return_or_err!(PalmDocHeader::$method(
                    content,
                    PalmDocHeaderData::$type,
                    num_of_records
                ))
            };
        }
        let compression = get_pdheader!(Compression, get_headers_u16);
        let text_length = get_pdheader!(TextLength, get_headers_u32);
        let record_count = get_pdheader!(RecordCount, get_headers_u16);
        let record_size = get_pdheader!(RecordSize, get_headers_u16);
        let encryption_type = get_pdheader!(EncryptionType, get_headers_u16);
        Ok(PalmDocHeader {
            compression,
            text_length,
            record_count,
            record_size,
            encryption_type,
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
        macro_rules! get_headers {
            ($cont:expr, $nr:expr, $method:ident($enum:ident)) => {
                return_or_err!(MobiHeader::$method($cont, MobiHeaderData::$enum, $nr))
            };
        }
        let identifier = get_headers!(content, num_of_records, get_headers_u32(Identifier));
        let header_length = get_headers!(content, num_of_records, get_headers_u32(HeaderLength));
        let mobi_type = get_headers!(content, num_of_records, get_headers_u32(MobiType));
        let text_encoding = get_headers!(content, num_of_records, get_headers_u32(TextEncoding));
        let id = get_headers!(content, num_of_records, get_headers_u32(Id));
        let gen_version = get_headers!(content, num_of_records, get_headers_u32(GenVersion));
        let first_non_book_index =
            get_headers!(content, num_of_records, get_headers_u32(FirstNonBookIndex));
        let name = return_or_err!(MobiHeader::name(content, num_of_records));
        let name_offset = get_headers!(content, num_of_records, get_headers_u32(NameOffset));
        let name_length = get_headers!(content, num_of_records, get_headers_u32(NameLength));
        let language = get_headers!(content, num_of_records, get_headers_u32(Language));
        let input_language = get_headers!(content, num_of_records, get_headers_u32(InputLanguage));
        let output_language =
            get_headers!(content, num_of_records, get_headers_u32(OutputLanguage));
        let format_version = get_headers!(content, num_of_records, get_headers_u32(FormatVersion));
        let first_image_index =
            get_headers!(content, num_of_records, get_headers_u32(FirstImageIndex));
        let first_huff_record =
            get_headers!(content, num_of_records, get_headers_u32(FirstHuffRecord));
        let huff_record_count =
            get_headers!(content, num_of_records, get_headers_u32(HuffRecordCount));
        let first_data_record =
            get_headers!(content, num_of_records, get_headers_u32(FirstDataRecord));
        let data_record_count =
            get_headers!(content, num_of_records, get_headers_u32(DataRecordCount));
        let exth_flags = get_headers!(content, num_of_records, get_headers_u32(ExthFlags));
        let has_exth_header = MobiHeader::exth_header(exth_flags);
        let drm_offset = get_headers!(content, num_of_records, get_headers_u32(DrmOffset));
        let drm_count = get_headers!(content, num_of_records, get_headers_u32(DrmCount));
        let drm_size = get_headers!(content, num_of_records, get_headers_u32(DrmSize));
        let drm_flags = get_headers!(content, num_of_records, get_headers_u32(DrmFlags));
        let last_image_record =
            get_headers!(content, num_of_records, get_headers_u16(LastImageRecord));
        let fcis_record = get_headers!(content, num_of_records, get_headers_u32(FcisRecord));
        let flis_record = get_headers!(content, num_of_records, get_headers_u32(FlisRecord));
        Ok(MobiHeader {
            identifier,
            header_length,
            mobi_type,
            text_encoding,
            id,
            gen_version,
            first_non_book_index,
            name,
            name_offset,
            name_length,
            language,
            input_language,
            output_language,
            format_version,
            first_image_index,
            first_huff_record,
            huff_record_count,
            first_data_record,
            data_record_count,
            exth_flags,
            has_exth_header,
            drm_offset,
            drm_count,
            drm_size,
            drm_flags,
            last_image_record,
            fcis_record,
            flis_record,
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
        let mut name = String::new();
        let mut count = 0;
        for byte in &content[name_offset as usize + (num_of_records * 8) as usize + 80..] {
            if count == name_length {
                break;
            }
            name.push(*byte as char);
            count += 1;
        }
        Ok(name)
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
            let mut record_data = String::new();
            let record_type = reader.read_u32::<BigEndian>().unwrap_or(0);
            let record_len = reader.read_u32::<BigEndian>().unwrap_or(0);
            for _j in 0..record_len - 8 {
                record_data.push(reader.read_u8().unwrap_or(0) as char);
            }
            records.insert(record_type, record_data);
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
            let string =
                &content[self.record_data_offset as usize..(self.record_data_offset + 8) as usize];
            self.record_data = utils::u8_as_string(string);
        }
    }
    /// Parses a record from the reader at current position
    fn parse_record(reader: &mut Cursor<&[u8]>) -> Result<Record, std::io::Error> {
        let record_data_offset = return_or_err!(reader.read_u32::<BigEndian>());;
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
    #[allow(dead_code)]
    fn read(&self, content: &[u8], record_num: usize, records: &[Record]) -> String {
        let next_record = &records[record_num + 1];
        println!("{}", self.record_data_offset);
        println!("{}", next_record.record_data_offset);
        utils::u8_as_string(
            &content[self.record_data_offset as usize..next_record.record_data_offset as usize],
        )
    }
}
