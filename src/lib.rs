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

fn u8_as_string(byte_arr: &[u8]) -> String {
    let mut out_str = String::new();
    for byte in byte_arr {
        out_str.push(*byte as char);
    }
    out_str
}

// TODO: fix documentation
macro_rules! forward_book_info {
    ($($bi:ident => $method:ident),+) => {
        $(
            pub fn $method(&self) -> Option<&String> {
                self.exth.get_book_info(BookInfo::$bi)
            }
        )+
    }
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
    pub fn init(file_path: &Path) -> Mobi {
        let contents = fs::read(file_path).unwrap();
        let header = Header::parse(&contents);
        let records = Record::parse_records(&contents, header.num_of_records);
        let palmdoc = PalmDocHeader::parse(&contents, header.num_of_records);
        let mobi = MobiHeader::parse(&contents, header.num_of_records);
        let exth = if mobi.has_exth_header {
            ExtHeader::parse(&contents, header.num_of_records)
        } else {
            ExtHeader::default()
        };

        Mobi {
            contents,
            header,
            records,
            palmdoc,
            mobi,
            exth,
        }
    }
    forward_book_info!(
        Author => author,
        Publisher => publisher,
        Description => description,
        Isbn => isbn,
        PublishDate => publish_date,
        Contributor => contributor,
        Title => title
    );
    /// Prints basic information about the book into stdout
    pub fn print_book_info(&self) {
        let empty_str = &String::from("");
        macro_rules! get_book_info {
            ($method:ident) => {
                // NOTE: unwrap_or_default doesn't work,
                // because $method returns a reference
                self.$method().unwrap_or(empty_str)
            };
        };
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
            get_book_info!(title),
            get_book_info!(author),
            get_book_info!(publisher),
            get_book_info!(description),
            get_book_info!(isbn),
            get_book_info!(publish_date),
            get_book_info!(contributor),
        );
    }
}

macro_rules! get_headers_pmatch {
    ($input:expr, { $prefix:ident, $($elem:ident => $val:expr),+ }) => {
        get_headers_pmatch!($input, { $prefix, $($elem => $val,)+ })
    };
    ($input:expr, { $prefix:ident, $($elem:ident => $val:expr),+ , }) => {
        match $input {
            $(
                $prefix::$elem => $val,
            )+
            #[allow(unreachable_patterns)]
            _ => 0,
        }
    }
}

// TODO: fix documentation
macro_rules! get_headers_impl {
    ($method:ident($hdty:ty) -> $rty:ty, $read_fn:ident, $pmatch:tt) => {
        fn $method(content: &[u8], header_data: $hdty) -> $rty {
            let position = get_headers_pmatch!(header_data, $pmatch);
            let mut reader = Cursor::new(content);
            reader.set_position(position);
            reader.$read_fn::<BigEndian>().unwrap()
        }
    };
    ($method:ident($hdty:ty) -> $rty:ty, $num_of_records:ident, $read_fn:ident, $pmatch:tt) => {
        fn $method(content: &[u8], header_data: $hdty, $num_of_records: u16) -> $rty {
            let position = get_headers_pmatch!(header_data, $pmatch);
            let mut reader = Cursor::new(content);
            reader.set_position(position + u64::from($num_of_records * 8));
            reader.$read_fn::<BigEndian>().unwrap()
        }
    };
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
    pub fn parse(content: &[u8]) -> Header {
        macro_rules! parse_header {
            ($($hdat:ident => $elem:ident : $getter:ident),+) => {
                Header {
                    $(
                        $elem: Header::$getter(content, HeaderData::$hdat),
                    )+
                }
            }
        }
        parse_header!(
            Name             => name:                get_headers_string,
            Attributes       => attributes:          get_headers_i16,
            Version          => version:             get_headers_i16,
            Created          => created:             get_headers_u32,
            Modified         => modified:            get_headers_u32,
            Backup           => backup:              get_headers_u32,
            Modnum           => modnum:              get_headers_u32,
            AppInfoId        => app_info_id:         get_headers_u32,
            SortInfoId       => sort_info_id:        get_headers_u32,
            TypE             => typ_e:               get_headers_string,
            Creator          => creator:             get_headers_string,
            UniqueIdSeed     => unique_id_seed:      get_headers_u32,
            NextRecordListId => next_record_list_id: get_headers_u32,
            NumOfRecords     => num_of_records:      get_headers_u16
        )
    }

    get_headers_impl!(get_headers_i16(HeaderData) -> i16, read_i16, {
        HeaderData,
        Attributes => 32,
        Version => 34,
    });

    /// Gets u16 header value from specific location
    pub fn get_headers_u16(content: &[u8], header: HeaderData) -> u16 {
        let mut reader = Cursor::new(content);
        let position = match header {
            HeaderData::NumOfRecords => 76,
            _ => 0,
        };
        reader.set_position(position);
        reader.read_u16::<BigEndian>().unwrap()
    }

    get_headers_impl!(get_headers_u32(HeaderData) -> u32, read_u32, {
        HeaderData,
        Created => 36,
        Modified => 40,
        Backup => 44,
        Modnum => 48,
        AppInfoId => 52,
        SortInfoId => 56,
        UniqueIdSeed => 68,
        NextRecordListId => 72,
    });

    /// Creates a string based on header bytes from specific location
    fn get_headers_string(content: &[u8], header: HeaderData) -> String {
        match header {
            HeaderData::Name => u8_as_string(&content[0..32]),
            HeaderData::TypE => u8_as_string(&content[60..64]),
            HeaderData::Creator => u8_as_string(&content[64..68]),
            _ => String::new(),
        }
    }
}

/// Parameters of PalmDOC Header
enum PalmDocHeaderData {
    Compression,
    TextLength,
    RecordCount,
    RecordSize,
    EncryptionType,
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
    pub fn parse(content: &[u8], num_of_records: u16) -> PalmDocHeader {
        macro_rules! parse_header {
            ($($hdat:ident => $elem:ident : $getter:ident),+) => {
                PalmDocHeader {
                    $(
                        $elem: PalmDocHeader::$getter(
                            content,
                            PalmDocHeaderData::$hdat,
                            num_of_records,
                        ),
                    )+
                }
            }
        }
        parse_header!(
            Compression => compression:        get_headers_u16,
            TextLength => text_length:         get_headers_u32,
            RecordCount => record_count:       get_headers_u16,
            RecordSize => record_size:         get_headers_u16,
            EncryptionType => encryption_type: get_headers_u16
        )
    }

    get_headers_impl!(get_headers_u16(PalmDocHeaderData) -> u16, num_of_records, read_u16, {
        PalmDocHeaderData,
        Compression => 80,
        RecordCount => 88,
        RecordSize => 90,
        EncryptionType => 92,
    });

    get_headers_impl!(get_headers_u32(PalmDocHeaderData) -> u32, num_of_records, read_u32, {
        PalmDocHeaderData,
        TextLength => 84,
    });
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
enum MobiHeaderData {
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
    pub fn parse(content: &[u8], num_of_records: u16) -> MobiHeader {
        let get_header_u32 =
            |hdat: MobiHeaderData| MobiHeader::get_headers_u32(content, hdat, num_of_records);

        use MobiHeaderData::*;
        let mut m = MobiHeader {
            identifier: get_header_u32(Identifier),
            header_length: get_header_u32(HeaderLength),
            mobi_type: get_header_u32(MobiType),
            text_encoding: get_header_u32(TextEncoding),
            id: get_header_u32(Id),
            gen_version: get_header_u32(GenVersion),
            first_non_book_index: get_header_u32(FirstNonBookIndex),
            name: MobiHeader::name(content, num_of_records),
            name_offset: get_header_u32(NameOffset),
            name_length: get_header_u32(NameLength),
            language: get_header_u32(Language),
            input_language: get_header_u32(InputLanguage),
            output_language: get_header_u32(OutputLanguage),
            format_version: get_header_u32(FormatVersion),
            first_image_index: get_header_u32(FirstImageIndex),
            first_huff_record: get_header_u32(FirstHuffRecord),
            huff_record_count: get_header_u32(HuffRecordCount),
            first_data_record: get_header_u32(FirstDataRecord),
            data_record_count: get_header_u32(DataRecordCount),
            exth_flags: get_header_u32(ExthFlags),
            has_exth_header: false,
            drm_offset: get_header_u32(DrmOffset),
            drm_count: get_header_u32(DrmCount),
            drm_size: get_header_u32(DrmSize),
            drm_flags: get_header_u32(DrmFlags),
            last_image_record: MobiHeader::get_headers_u16(
                content,
                MobiHeaderData::LastImageRecord,
                num_of_records,
            ),
            fcis_record: get_header_u32(FcisRecord),
            flis_record: get_header_u32(FlisRecord),
        };
        m.exth_header();
        m
    }

    get_headers_impl!(get_headers_u32(MobiHeaderData) -> u32, num_of_records, read_u32, {
        MobiHeaderData,
        Identifier => 96,
        HeaderLength => 100,
        MobiType => 104,
        TextEncoding => 108,
        Id => 112,
        GenVersion => 116,
        FirstNonBookIndex => 160,
        NameOffset => 164,
        NameLength => 168,
        Language => 172,
        InputLanguage => 176,
        OutputLanguage => 180,
        FormatVersion => 184,
        FirstImageIndex => 188,
        FirstHuffRecord => 192,
        HuffRecordCount => 196,
        FirstDataRecord => 200,
        DataRecordCount => 204,
        ExthFlags => 208,
        DrmOffset => 248,
        DrmCount => 252,
        DrmSize => 256,
        DrmFlags => 260,
        FcisRecord => 280,
        FlisRecord => 288,
    });

    get_headers_impl!(get_headers_u16(MobiHeaderData) -> u16, num_of_records, read_u16, {
        MobiHeaderData,
        LastImageRecord => 274,
    });

    /// Returns the book name
    pub fn name(content: &[u8], num_of_records: u16) -> String {
        let name_offset =
            MobiHeader::get_headers_u32(content, MobiHeaderData::NameOffset, num_of_records);
        let name_length =
            MobiHeader::get_headers_u32(content, MobiHeaderData::NameLength, num_of_records);
        let mut name = String::new();
        let mut count = 0;
        for byte in &content[name_offset as usize + (num_of_records * 8) as usize + 80..] {
            if count == name_length {
                break;
            }
            name.push(*byte as char);
            count += 1;
        }
        name
    }
    /// Checks if there is a Exth Header and changes the parameter
    fn exth_header(&mut self) {
        self.has_exth_header = (self.exth_flags & 0x40) != 0;
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
enum ExtHeaderData {
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
    pub fn parse(content: &[u8], num_of_records: u16) -> ExtHeader {
        let mut extheader = ExtHeader {
            identifier: ExtHeader::get_headers_u32(
                content,
                ExtHeaderData::Identifier,
                num_of_records,
            ),
            header_length: ExtHeader::get_headers_u32(
                content,
                ExtHeaderData::HeaderLength,
                num_of_records,
            ),
            record_count: ExtHeader::get_headers_u32(
                content,
                ExtHeaderData::RecordCount,
                num_of_records,
            ),
            records: HashMap::new(),
        };
        extheader.get_records(content, num_of_records);
        extheader
    }

    get_headers_impl!(get_headers_u32(ExtHeaderData) -> u32, num_of_records, read_u32, {
        ExtHeaderData,
        Identifier => 328,
        HeaderLength => 332,
        RecordCount => 336,
    });

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
        use BookInfo::*;
        let record: u32 = match info {
            Author => 100,
            Publisher => 101,
            Description => 103,
            Isbn => 104,
            PublishDate => 106,
            Contributor => 108,
            Title => 503,
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
            self.record_data = u8_as_string(string);
        }
    }
    /// Parses a record from the reader at current position
    fn parse_record(reader: &mut Cursor<&[u8]>) -> Record {
        let record_data_offset = reader.read_u32::<BigEndian>().unwrap();
        let id = reader.read_u32::<BigEndian>().unwrap();
        let mut record = Record {
            record_data_offset,
            id,
            record_data: String::new(),
        };
        record.record_data(*reader.get_ref());
        record
    }
    /// Gets all records in the specified content
    fn parse_records(content: &[u8], num_of_records: u16) -> Vec<Record> {
        let mut reader = Cursor::new(content);
        let mut records = vec![];
        for _i in 0..num_of_records {
            let record = Record::parse_record(&mut reader);
            records.push(record);
        }
        records
    }
    #[allow(dead_code)]
    fn read(&self, content: &[u8], record_num: usize, records: &[Record]) -> String {
        let next_record = &records[record_num + 1];
        println!("{}", self.record_data_offset);
        println!("{}", next_record.record_data_offset);
        u8_as_string(
            &content[self.record_data_offset as usize..next_record.record_data_offset as usize],
        )
    }
}
