use std::fs;
use std::str;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use std::{thread, time};

struct Mobi {
    contents: Vec<u8>,
    header: Header,
    records: Records,
    palmdoc: PalmDocHeader,
    mobi: MobiHeader,
    exth: ExtHeader,
}
impl Mobi {

}
enum HeaderData {
    Name,
    Attributes,
    Version,
    Created,
    Modified,
    Backup,
    Modnum,
    AppInfoId,
    SortInfoId,
    Typ_e,
    Creator,
    UniqueIdSeed,
    NextRecordListId,
    NumOfRecords,
}
#[derive(Debug)]
struct Header {
    name: String,
    attributes: i16,
    version: i16,
    created: u32,
    modified: u32,
    backup: u32,
    modnum: u32,
    app_info_id: u32,
    sort_info_id: u32,
    typ_e: String,
    creator: String,
    unique_id_seed: u32,
    next_record_list_id: u32,
    num_of_records: u16,
}
impl Header {
    fn parse(content: &Vec<u8>) -> Header {
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
            typ_e: Header::get_headers_string(content, HeaderData::Typ_e),
            creator: Header::get_headers_string(content, HeaderData::Creator),
            unique_id_seed: Header::get_headers_u32(content, HeaderData::UniqueIdSeed),
            next_record_list_id: Header::get_headers_u32(content, HeaderData::NextRecordListId),
            num_of_records: Header::get_headers_u16(content, HeaderData::NumOfRecords),
        }
    }
    fn get_headers_i16(content: &Vec<u8>, header: HeaderData) -> i16 {
        let mut reader = Cursor::new(content);
        match header {
            HeaderData::Attributes => reader.set_position(32),
            HeaderData::Version => reader.set_position(34),
            _ => {}
        }
        reader.read_i16::<BigEndian>().unwrap()
    }
    fn get_headers_u16(content: &Vec<u8>, header: HeaderData) -> u16 {
        let mut reader = Cursor::new(content);
        match header {
            HeaderData::NumOfRecords => reader.set_position(76),
            _ => {}
        }
        reader.read_u16::<BigEndian>().unwrap()
    }    
    fn get_headers_u32(content: &Vec<u8>, header: HeaderData) -> u32 {
        let mut reader = Cursor::new(content);
        match header {
            HeaderData::Created => reader.set_position(36),
            HeaderData::Modified => reader.set_position(40),
            HeaderData::Backup => reader.set_position(44),
            HeaderData::Modnum => reader.set_position(48),
            HeaderData::AppInfoId => reader.set_position(52),
            HeaderData::SortInfoId => reader.set_position(56),
            HeaderData::UniqueIdSeed => reader.set_position(68),
            HeaderData::NextRecordListId => reader.set_position(72),
            _ => {}
        }
        reader.read_u32::<BigEndian>().unwrap()
    }
    fn get_headers_string(content: &Vec<u8>, header: HeaderData) -> String {
        match header {
            HeaderData::Name => u8_as_string(&content[0..32]),
            HeaderData::Typ_e => u8_as_string(&content[60..64]),
            HeaderData::Creator => u8_as_string(&content[64..68]),
            _ => String::new(),
        }
    }
}
fn u8_as_string(byte_vec: &[u8]) -> String {
    let mut out_str = String::new();
    for byte in byte_vec {
        out_str.push(*byte as char);
    }
    out_str
}
struct PalmDocHeader;
struct MobiHeader;
struct ExtHeader;
struct Records;
fn main() {
    let mut content = fs::read("/home/wojtek/Downloads/ania.mobi").unwrap();
    let mut display_str = String::new();
    let x = Header::parse(&content);
    println!("{:#?}", x);
}

// https://docs.python.org/2/library/struct.html
// https://github.com/crabhit/mobi-python/blob/master/mobi/__init__.py