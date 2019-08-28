mod utils;
use byteorder::{BigEndian, ReadBytesExt};
use std::fs;
use std::io::Cursor;
use std::path::Path;

#[derive(Debug)]
pub struct Mobi {
    pub contents: Vec<u8>,
    pub header: Header,
    pub records: Vec<Record>,
    pub palmdoc: PalmDocHeader,
    pub mobi: MobiHeader,
    pub exth: ExtHeader,
}
impl Mobi {
    pub fn init(file_path: &Path) -> Mobi {
        let contents = fs::read(file_path).unwrap();
        let header = Header::parse(&contents);
        let records = Record::parse_records(&contents, header.num_of_records);
        let palmdoc = PalmDocHeader::parse(&contents, header.num_of_records);
        let mobi = MobiHeader::parse(&contents, header.num_of_records);
        let exth = ExtHeader::parse(&contents, header.num_of_records);
        Mobi {
            contents,
            header,
            records,
            palmdoc,
            mobi,
            exth,
        }
    }
}
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
enum PalmDocHeaderData {
    Compression,
    TextLength,
    RecordCount,
    RecordSize,
    EncryptionType,
}

#[derive(Debug, PartialEq)]
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
    pub fn parse(content: &[u8]) -> Header {
        Header {
            name: Header::get_headers_string(content, HeaderData::Name),
            attributes: Header::get_headers_i16(content, HeaderData::Attributes),
            version: Header::get_headers_i16(content, HeaderData::Version),
            created: Header::get_headers_u32(content, HeaderData::Created),
            modified: Header::get_headers_u32(content, HeaderData::Modified),
            backup: Header::get_headers_u32(content, HeaderData::Backup),
            modnum: Header::get_headers_u32(content, HeaderData::Modnum),
            app_info_id: Header::get_headers_u32(content, HeaderData::AppInfoId),
            sort_info_id: Header::get_headers_u32(content, HeaderData::SortInfoId),
            typ_e: Header::get_headers_string(content, HeaderData::TypE),
            creator: Header::get_headers_string(content, HeaderData::Creator),
            unique_id_seed: Header::get_headers_u32(content, HeaderData::UniqueIdSeed),
            next_record_list_id: Header::get_headers_u32(content, HeaderData::NextRecordListId),
            num_of_records: Header::get_headers_u16(content, HeaderData::NumOfRecords),
        }
    }
    fn get_headers_i16(content: &[u8], header: HeaderData) -> i16 {
        let mut reader = Cursor::new(content);
        let position = match header {
            HeaderData::Attributes => 32,
            HeaderData::Version => 34,
            _ => 0,
        };
        reader.set_position(position);
        reader.read_i16::<BigEndian>().unwrap()
    }
    pub fn get_headers_u16(content: &[u8], header: HeaderData) -> u16 {
        let mut reader = Cursor::new(content);
        let position = match header {
            HeaderData::NumOfRecords => 76,
            _ => 0,
        };
        reader.set_position(position);
        reader.read_u16::<BigEndian>().unwrap()
    }
    fn get_headers_u32(content: &[u8], header: HeaderData) -> u32 {
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
        reader.read_u32::<BigEndian>().unwrap()
    }
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
pub struct PalmDocHeader {
    pub compression: u16,
    pub text_length: u32,
    pub record_count: u16,
    pub record_size: u16,
    pub encryption_type: u16,
}
impl PalmDocHeader {
    pub fn parse(content: &[u8], num_of_records: u16) -> PalmDocHeader {
        PalmDocHeader {
            compression: PalmDocHeader::get_headers_u16(
                content,
                PalmDocHeaderData::Compression,
                num_of_records,
            ),
            text_length: PalmDocHeader::get_headers_u32(
                content,
                PalmDocHeaderData::TextLength,
                num_of_records,
            ),
            record_count: PalmDocHeader::get_headers_u16(
                content,
                PalmDocHeaderData::RecordCount,
                num_of_records,
            ),
            record_size: PalmDocHeader::get_headers_u16(
                content,
                PalmDocHeaderData::RecordSize,
                num_of_records,
            ),
            encryption_type: PalmDocHeader::get_headers_u16(
                content,
                PalmDocHeaderData::EncryptionType,
                num_of_records,
            ),
        }
    }
    fn get_headers_u16(content: &[u8], pdheader: PalmDocHeaderData, num_of_records: u16) -> u16 {
        let mut reader = Cursor::new(content);
        let position = match pdheader {
            PalmDocHeaderData::Compression => 80,
            PalmDocHeaderData::RecordCount => 88,
            PalmDocHeaderData::RecordSize => 90,
            PalmDocHeaderData::EncryptionType => 92,
            _ => 0,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u16::<BigEndian>().unwrap()
    }
    fn get_headers_u32(content: &[u8], pdheader: PalmDocHeaderData, num_of_records: u16) -> u32 {
        let mut reader = Cursor::new(content);
        let position = match pdheader {
            PalmDocHeaderData::TextLength => 84,
            _ => 0,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u32::<BigEndian>().unwrap()
    }
}

#[derive(Debug, PartialEq)]
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
    pub fn parse(content: &[u8], num_of_records: u16) -> MobiHeader {
        let mut m = MobiHeader {
            identifier: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::Identifier,
                num_of_records,
            ),
            header_length: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::HeaderLength,
                num_of_records,
            ),
            mobi_type: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::MobiType,
                num_of_records,
            ),
            text_encoding: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::TextEncoding,
                num_of_records,
            ),
            id: MobiHeader::get_headers_u32(&content, MobiHeaderData::Id, num_of_records),
            gen_version: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::GenVersion,
                num_of_records,
            ),
            first_non_book_index: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::FirstNonBookIndex,
                num_of_records,
            ),
            name: MobiHeader::name(&content, num_of_records),
            name_offset: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::NameOffset,
                num_of_records,
            ),
            name_length: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::NameLength,
                num_of_records,
            ),
            language: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::Language,
                num_of_records,
            ),
            input_language: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::InputLanguage,
                num_of_records,
            ),
            output_language: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::OutputLanguage,
                num_of_records,
            ),
            format_version: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::FormatVersion,
                num_of_records,
            ),
            first_image_index: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::FirstImageIndex,
                num_of_records,
            ),
            first_huff_record: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::FirstHuffRecord,
                num_of_records,
            ),
            huff_record_count: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::HuffRecordCount,
                num_of_records,
            ),
            first_data_record: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::FirstDataRecord,
                num_of_records,
            ),
            data_record_count: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::DataRecordCount,
                num_of_records,
            ),
            exth_flags: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::ExthFlags,
                num_of_records,
            ),
            has_exth_header: false,
            drm_offset: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::DrmOffset,
                num_of_records,
            ),
            drm_count: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::DrmCount,
                num_of_records,
            ),
            drm_size: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::DrmSize,
                num_of_records,
            ),
            drm_flags: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::DrmFlags,
                num_of_records,
            ),
            last_image_record: MobiHeader::get_headers_u16(
                &content,
                MobiHeaderData::LastImageRecord,
                num_of_records,
            ),
            fcis_record: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::FcisRecord,
                num_of_records,
            ),
            flis_record: MobiHeader::get_headers_u32(
                &content,
                MobiHeaderData::FlisRecord,
                num_of_records,
            ),
        };
        m.exth_header();
        m
    }
    fn get_headers_u32(content: &[u8], mheader: MobiHeaderData, num_of_records: u16) -> u32 {
        let mut reader = Cursor::new(content);
        let position = match mheader {
            MobiHeaderData::Identifier => 100,
            MobiHeaderData::HeaderLength => 104,
            MobiHeaderData::MobiType => 108,
            MobiHeaderData::TextEncoding => 112,
            MobiHeaderData::Id => 116,
            MobiHeaderData::GenVersion => 120,
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
        reader.read_u32::<BigEndian>().unwrap()
    }
    fn get_headers_u16(content: &[u8], mheader: MobiHeaderData, num_of_records: u16) -> u16 {
        let mut reader = Cursor::new(content);
        let position = match mheader {
            MobiHeaderData::LastImageRecord => 274,
            _ => 0,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u16::<BigEndian>().unwrap()
    }
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
    fn exth_header(&mut self) {
        self.has_exth_header = (self.exth_flags & 0x40) != 0;
    }
}
enum ExtHeaderData {
    Identifier,
    HeaderLength,
    RecordCount,
}
#[derive(Debug, Default, PartialEq)]
pub struct ExtHeader {
    pub identifier: u32,
    pub header_length: u32,
    pub record_count: u32,
    pub records: Vec<(u32, u32, String)>,
}
impl ExtHeader {
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
            records: vec![],
        };
        extheader.get_records(content, num_of_records);
        extheader
    }
    fn get_headers_u32(content: &[u8], extheader: ExtHeaderData, num_of_records: u16) -> u32 {
        let mut reader = Cursor::new(content);
        let position = match extheader {
            ExtHeaderData::Identifier => 328,
            ExtHeaderData::HeaderLength => 332,
            ExtHeaderData::RecordCount => 336,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u32::<BigEndian>().unwrap()
    }
    fn get_records(&mut self, content: &[u8], num_of_records: u16) {
        let mut records = vec![];
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
            records.push((
                record_len,
                record_type,
                utils::u8_as_string(&record_data[..]),
            ));
        }
        self.records = records;
    }
}

#[derive(Debug)]
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
    fn record_data(&mut self, content: &[u8]) {
        if self.record_data_offset + 8 < content.len() as u32 {
            let string =
                &content[self.record_data_offset as usize..(self.record_data_offset + 8) as usize];
            self.record_data = utils::u8_as_string(string);
        }
    }
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
        utils::u8_as_string(
            &content[self.record_data_offset as usize..next_record.record_data_offset as usize],
        )
    }
    // TODO
    // lz77 decompression
}
