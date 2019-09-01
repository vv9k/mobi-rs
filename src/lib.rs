//! An implementation of [MOBI](https://wiki.mobileread.com/wiki/MOBI) format data parsing and manipulation, written in Rust.
//!
//! The code is available on [GitHub](https://github.com/wojciechkepka/mobi-rs)
//!
//! License: [*Apache-2.0*](https://github.com/wojciechkepka/mobi-rs/blob/master/license)
//!
use byteorder::{BigEndian, ReadBytesExt};
use chrono::prelude::*;
use std::collections::HashMap;
use std::fmt;
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
#[derive(Debug, Default)]
/// Structure that holds parsed ebook information and contents
pub struct Mobi {
    pub contents: Vec<u8>,
    pub header: Header,
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
        let palmdoc = return_or_err!(PalmDocHeader::parse(&contents, header.num_of_records));
        let mobi = return_or_err!(MobiHeader::parse(&contents, header.num_of_records));
        let exth = {
            if mobi.has_exth_header {
                return_or_err!(ExtHeader::parse(&contents, header.num_of_records))
            } else {
                ExtHeader::default()
            }
        };
        Ok(Mobi {
            contents,
            header,
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
    /// Returns text encoding used in ebook
    pub fn text_encoding(&self) -> Option<String> {
        self.mobi.text_encoding()
    }
    /// Returns type of this ebook
    pub fn mobi_type(&self) -> Option<String> {
        self.mobi.mobi_type()
    }
    /// Returns language of the ebook
    pub fn language(&self) -> Option<String> {
        self.mobi.language()
    }
    /// Returns creation datetime
    pub fn created_datetime(&self) -> NaiveDateTime {
        self.header.created_datetime()
    }
    /// Returns modification datetime
    pub fn mod_datetime(&self) -> NaiveDateTime {
        self.header.mod_datetime()
    }
    /// Returns compression method used on this file
    pub fn compression(&self) -> Option<String> {
        self.palmdoc.compression()
    }
    /// Returns encryption method used on this file
    pub fn encryption(&self) -> Option<String> {
        self.palmdoc.encryption()
    }
}
impl fmt::Display for Mobi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let empty_str = String::from("");
        write!(
            f,
            "
------------------------------------------------------------------------------------
Title:                  {}
Author:                 {}
Publisher:              {}
Description:            {}
ISBN:                   {}
Publish Date:           {}
Contributor:            {}
------------------------------------------------------------------------------------
{}
------------------------------------------------------------------------------------
{}
------------------------------------------------------------------------------------
{}
------------------------------------------------------------------------------------
{}
------------------------------------------------------------------------------------",
            self.title().unwrap_or(&empty_str),
            self.author().unwrap_or(&empty_str),
            self.publisher().unwrap_or(&empty_str),
            self.description().unwrap_or(&empty_str),
            self.isbn().unwrap_or(&empty_str),
            self.publish_date().unwrap_or(&empty_str),
            self.contributor().unwrap_or(&empty_str),
            self.header,
            self.palmdoc,
            self.mobi,
            self.exth,
        )
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
    /// Returns a chrono::NaiveDateTime timestamp of file creation
    fn created_datetime(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(i64::from(self.created), 0)
    }
    /// Returns a chrono::NaiveDateTime timestamp of file modification
    fn mod_datetime(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(i64::from(self.modified), 0)
    }
}
#[derive(Debug, PartialEq, Default)]
/// Strcture that holds PalmDOC header information
pub struct PalmDocHeader {
    pub compression: u16,
    pub text_length: u32,
    pub record_count: u16,
    pub record_size: u16,
    pub encryption_type: u16,
}
impl fmt::Display for PalmDocHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PALMDOC HEADER
Compression:            {}
Text length:            {}
Record count:           {}
Record size:            {}
Encryption type:        {}",
            self.compression().unwrap_or_default(),
            self.text_length,
            self.record_count,
            self.record_size,
            self.encryption().unwrap_or_default(),
        )
    }
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
    pub fn compression(&self) -> Option<String> {
        match self.compression {
            1 => Some(String::from("No Compression")),
            2 => Some(String::from("PalmDOC Compression")),
            17480 => Some(String::from("HUFF/CFIC Compression")),
            _ => None,
        }
    }
    pub fn encryption(&self) -> Option<String> {
        match self.encryption_type {
            0 => Some(String::from("No Encryption")),
            1 => Some(String::from("Old Mobipocket Encryption")),
            2 => Some(String::from("Mobipocket Encryption")),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Default)]
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
    pub language_code: u16,
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
    pub has_drm: bool,
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
    LanguageCode,
    InputLanguage,
    OutputLanguage,
    FormatVersion,
    FirstImageIndex,
    FirstHuffRecord,
    HuffRecordCount,
    FirstDataRecord,
    DataRecordCount,
    ExthFlags,
    HasDrm,
    DrmOffset,
    DrmCount,
    DrmSize,
    DrmFlags,
    LastImageRecord,
    FcisRecord,
    FlisRecord,
}
impl fmt::Display for MobiHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MOBI HEADER
Identifier:             {}
HeaderLength:           {}
Mobi type:              {}
Text encoding:          {}
Id:                     {}
Gen version:            v{}
First non book index:   {}
Name:                   {}
Name offset:            {}
Name length:            {}
Language:               {}
Input language:         {}
Output language:        {}
Format version:         {}
First image index:      {}
First huff record:      {}
Huff record count:      {}
First data record:      {}
Data record count:      {}
Exth flags:             {}
Has Exth header:        {}
Has DRM:                {}
DRM offset:             {}
DRM count:              {}
DRM size:               {}
DRM flags:              {}
Last image record:      {}
Fcis record:            {}
Flis record:            {}",
            self.identifier,
            self.header_length,
            self.mobi_type().unwrap_or(String::from("")),
            self.text_encoding().unwrap_or(String::from("")),
            self.id,
            self.gen_version,
            self.first_non_book_index,
            self.name,
            self.name_offset,
            self.name_length,
            self.language().unwrap_or(String::from("")),
            self.input_language,
            self.output_language,
            self.format_version,
            self.first_image_index,
            self.first_huff_record,
            self.huff_record_count,
            self.first_data_record,
            self.data_record_count,
            self.exth_flags,
            self.has_exth_header,
            self.has_drm,
            self.drm_offset,
            self.drm_count,
            self.drm_size,
            self.drm_flags,
            self.last_image_record,
            self.fcis_record,
            self.flis_record,
        )
    }
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
            language_code: MobiHeader::lang_code(mobiheader!(get_headers_u32(LanguageCode))),
            input_language: mobiheader!(get_headers_u32(InputLanguage)),
            output_language: mobiheader!(get_headers_u32(OutputLanguage)),
            format_version: mobiheader!(get_headers_u32(FormatVersion)),
            first_image_index: mobiheader!(get_headers_u32(FirstImageIndex)),
            first_huff_record: mobiheader!(get_headers_u32(FirstHuffRecord)),
            huff_record_count: mobiheader!(get_headers_u32(HuffRecordCount)),
            first_data_record: mobiheader!(get_headers_u32(FirstDataRecord)),
            data_record_count: mobiheader!(get_headers_u32(DataRecordCount)),
            exth_flags: mobiheader!(get_headers_u32(ExthFlags)),
            has_exth_header: MobiHeader::has_exth_header(mobiheader!(get_headers_u32(ExthFlags))),
            drm_offset: mobiheader!(get_headers_u32(DrmOffset)),
            drm_count: mobiheader!(get_headers_u32(DrmCount)),
            drm_size: mobiheader!(get_headers_u32(DrmSize)),
            drm_flags: mobiheader!(get_headers_u32(DrmFlags)),
            has_drm: MobiHeader::has_drm(mobiheader!(get_headers_u32(DrmOffset))),
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
            MobiHeaderData::LanguageCode => 172,
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
        Ok(
            String::from_utf8_lossy(&content[offset..offset + name_length as usize])
                .to_owned()
                .to_string(),
        )
    }
    /// Checks if there is a Exth Header and changes the parameter
    fn has_exth_header(exth_flags: u32) -> bool {
        (exth_flags & 0x40) != 0
    }
    /// Checks if there is DRM on this book
    fn has_drm(drm_offset: u32) -> bool {
        drm_offset != 0xFFFF_FFFF
    }
    /// Converts numerical value into a type
    pub fn mobi_type(&self) -> Option<String> {
        macro_rules! mtype {
            ($s:expr) => {
                Some(String::from($s))
            };
        }
        match self.mobi_type {
            2 => mtype!("Mobipocket Book"),
            3 => mtype!("PalmDoc Book"),
            4 => mtype!("Audio"),
            257 => mtype!("News"),
            258 => mtype!("News Feed"),
            259 => mtype!("News Magazine"),
            513 => mtype!("PICS"),
            514 => mtype!("WORD"),
            515 => mtype!("XLS"),
            516 => mtype!("PPT"),
            517 => mtype!("TEXT"),
            518 => mtype!("HTML"),
            _ => None,
        }
    }
    pub fn text_encoding(&self) -> Option<String> {
        match self.text_encoding {
            1252 => Some(String::from("CP1252 (WinLatin1)")),
            65001 => Some(String::from("UTF-8")),
            _ => None,
        }
    }
    fn lang_code(code: u32) -> u16 {
        (code & 0xFF) as u16
    }
    pub fn language(&self) -> Option<String> {
        macro_rules! lang {
            ($s:expr) => {
                Some(String::from($s))
            };
        }
        match self.language_code {
            0 => lang!("NEUTRAL"),
            54 => lang!("AFRIKAANS"),
            28 => lang!("ALBANIAN"),
            1 => lang!("ARABIC"),
            43 => lang!("ARMENIAN"),
            77 => lang!("ASSAMESE"),
            44 => lang!("AZERI"),
            45 => lang!("BASQUE"),
            35 => lang!("BELARUSIAN"),
            69 => lang!("BENGALI"),
            2 => lang!("BULGARIAN"),
            3 => lang!("CATALAN"),
            4 => lang!("CHINESE"),
            5 => lang!("CZECH"),
            6 => lang!("DANISH"),
            19 => lang!("DUTCH"),
            9 => lang!("ENGLISH"),
            37 => lang!("ESTONIAN"),
            56 => lang!("FAEROESE"),
            41 => lang!("FARSI"),
            11 => lang!("FINNISH"),
            12 => lang!("FRENCH"),
            55 => lang!("GEORGIAN"),
            7 => lang!("GERMAN"),
            8 => lang!("GREEK"),
            71 => lang!("GUJARATI"),
            13 => lang!("HEBREW"),
            57 => lang!("HINDI"),
            14 => lang!("HUNGARIAN"),
            15 => lang!("ICELANDIC"),
            33 => lang!("INDONESIAN"),
            16 => lang!("ITALIAN"),
            17 => lang!("JAPANESE"),
            75 => lang!("KANNADA"),
            63 => lang!("KAZAK"),
            87 => lang!("KONKANI"),
            18 => lang!("KOREAN"),
            38 => lang!("LATVIAN"),
            39 => lang!("LITHUANIAN"),
            47 => lang!("MACEDONIAN"),
            62 => lang!("MALAY"),
            76 => lang!("MALAYALAM"),
            58 => lang!("MALTESE"),
            78 => lang!("MARATHI"),
            97 => lang!("NEPALI"),
            20 => lang!("NORWEGIAN"),
            72 => lang!("ORIYA"),
            21 => lang!("POLISH"),
            22 => lang!("PORTUGUESE"),
            70 => lang!("PUNJABI"),
            23 => lang!("RHAETOROMANIC"),
            24 => lang!("ROMANIAN"),
            25 => lang!("RUSSIAN"),
            59 => lang!("SAMI"),
            79 => lang!("SANSKRIT"),
            26 => lang!("SERBIAN"),
            27 => lang!("SLOVAK"),
            36 => lang!("SLOVENIAN"),
            46 => lang!("SORBIAN"),
            10 => lang!("SPANISH"),
            48 => lang!("SUTU"),
            65 => lang!("SWAHILI"),
            29 => lang!("SWEDISH"),
            73 => lang!("TAMIL"),
            68 => lang!("TATAR"),
            74 => lang!("TELUGU"),
            30 => lang!("THAI"),
            49 => lang!("TSONGA"),
            50 => lang!("TSWANA"),
            31 => lang!("TURKISH"),
            34 => lang!("UKRAINIAN"),
            32 => lang!("URDU"),
            67 => lang!("UZBEK"),
            42 => lang!("VIETNAMESE"),
            52 => lang!("XHOSA"),
            53 => lang!("ZULU"),
            _ => None,
        }
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
