//! An implementation of [MOBI](https://wiki.mobileread.com/wiki/MOBI) format data parsing and manipulation, written in Rust.
//!
//! The code is available on [GitHub](https://github.com/wojciechkepka/mobi-rs)
//!
//! License: [*Apache-2.0*](https://github.com/wojciechkepka/mobi-rs/blob/master/license)
//!
//!## Examples
//!### Print the whole book into stdout
//!```rust,ignore
//!use mobi::Mobi;
//!fn main() {
//!    let m = Mobi::init("/home/wojtek/Downloads/lotr.mobi").unwrap();
//!    println!("{}", m.content_raw().unwrap());
//!}
//!```
//!### Access basic info
//!- `src/main.rs`
//!```rust,ignore
//!use mobi::Mobi;
//!fn main() {
//!    let m = Mobi::init("/home/wojtek/Downloads/lotr.mobi").unwrap();
//!    let title = m.title().unwrap();
//!    let author = m.author().unwrap();
//!    let publisher = m.publisher().unwrap();
//!    let desc = m.description().unwrap();
//!    let isbn = m.isbn().unwrap();
//!    let pub_date = m.publish_date().unwrap();
//!    let contributor = m.contributor().unwrap();
//!    println!("{}\n{}\n{}\n{}\n{}\n{}\n{}\n", title, author, publisher, isbn, pub_date, desc, contributor);
//!    // Access Headers
//!    let header = m.header; // Normal Header
//!    let pdheader = m.palmdoc; // PalmDOC Header
//!    let mheader = m.mobi; // MOBI Header
//!    let exth = m.exth // Extra Header
//!}
//!```
//!### Print all info
//!- `src/main.rs`
//!```rust,ignore
//!use mobi::Mobi;
//!
//!fn main() {
//!    let m = Mobi::init("/home/wojtek/Downloads/lotr.mobi").unwrap();
//!    println!("{}", m)
//!}
//!```
mod lz77;
pub mod exth;
pub mod mobih;
pub mod palmdoch;
pub mod header;
use crate::header::Header;
use crate::palmdoch::{Compression, PalmDocHeader};
use crate::exth::{ExtHeader, BookInfo};
use crate::mobih::MobiHeader;
use byteorder::{BigEndian, ReadBytesExt};
use chrono::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::Cursor;
use std::path::Path;
#[derive(Debug, Default)]
/// Structure that holds parsed ebook information and contents
pub struct Mobi {
    pub contents: Vec<u8>,
    pub header: Header,
    pub palmdoc: PalmDocHeader,
    pub mobi: MobiHeader,
    pub exth: ExtHeader,
    pub records: Vec<Record>,
}
impl Mobi {
    /// Construct a Mobi object from passed file path
    /// Returns std::io::Error if there is a problem with the provided path
    pub fn init<P: AsRef<Path>>(file_path: P) -> Result<Mobi, std::io::Error> {
        let contents = fs::read(file_path)?;
        let header = Header::parse(&contents)?;
        let palmdoc = PalmDocHeader::parse(&contents, header.num_of_records)?;
        let mobi = MobiHeader::parse(&contents, header.num_of_records)?;
        let exth = {
            if mobi.has_exth_header {
                ExtHeader::parse(&contents, header.num_of_records)?
            } else {
                ExtHeader::default()
            }
        };
        let records = Record::parse_records(
            &contents,
            header.num_of_records,
            mobi.extra_bytes,
            palmdoc.compression_en()
        )?;
        Ok(Mobi {
            contents,
            header,
            palmdoc,
            mobi,
            exth,
            records,
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
    /// Returns the whole content as String
    pub fn content_raw(&self) -> Option<String> {
        let mut content = String::new();
        for i in 1..self.palmdoc.record_count - 1 {
            let s = &self.records[i as usize].to_string().replace("â", "").replace("", "");

            content.push_str(s);
        }
        Some(content)
    }
    /// Returns a slice of the content where b is beginning index and e is ending index.
    /// Usually readable indexes are between 1-300(+-50)
    pub fn content_slice(&self, b: usize, e: usize) -> Option<String> {
        let mut content = String::new();
        if (b >= 1) && (b <= e) && (e < (self.palmdoc.record_count - 1) as usize) {
            for i in b..e {
                content.push_str(&self.records[i as usize].to_string());
            }
        }
        Some(content)
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
#[derive(Debug, Clone)]
/// A "cell" in the whole books content
pub struct Record {
    record_data_offset: u32,
    id: u32,
    pub record_data: String,
}
impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.record_data)
    }
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
    fn record_data(
        record_data_offset: u32,
        next_record_data_offset: u32,
        extra_bytes: u32,
        compression_type: &Compression,
        content: &[u8],
    ) -> Result<String, std::io::Error> {
        match compression_type {
            Compression::No => Ok(String::from_utf8_lossy(
                &content
                    [record_data_offset as usize..next_record_data_offset as usize],
            )
            .to_owned()
            .to_string()),
            Compression::PalmDoc => {
                if record_data_offset < content.len() as u32 {
                    if record_data_offset < next_record_data_offset - extra_bytes {
                        let s = &content[record_data_offset as usize
                            ..(next_record_data_offset - extra_bytes) as usize];
                        lz77::decompress_lz77(s)
                    } else {
                        Ok(String::from(""))
                    }
                } else {
                    Ok(String::from(""))
                }
            }
            Compression::Huff => Ok(String::from("")),
        }
    }
    /// Parses a record from the reader at current position
    fn parse_record(reader: &mut Cursor<&[u8]>) -> Result<Record, std::io::Error> {
        let record_data_offset = reader.read_u32::<BigEndian>()?;
        let id = reader.read_u32::<BigEndian>()?;
        let record = Record {
            record_data_offset,
            id,
            record_data: String::new(),
        };
        Ok(record)
    }
    /// Gets all records in the specified content
    fn parse_records(
        content: &[u8],
        num_of_records: u16,
        _extra_bytes: u32,
        compression_type: Compression,
    ) -> Result<Vec<Record>, std::io::Error> {
        let mut records_content = vec![];
        let mut reader = Cursor::new(content);
        reader.set_position(78);
        for _i in 0..num_of_records {
            let record = Record::parse_record(&mut reader)?;
            records_content.push(record);
        }
        for i in 0..records_content.len() {
            let mut current_rec = records_content[i].clone();
            if i != records_content.len() - 1 {
                let next_offset = records_content[i + 1].record_data_offset;
                if _extra_bytes < next_offset {
                    current_rec.record_data = match Record::record_data(
                        current_rec.record_data_offset,
                        next_offset,
                        _extra_bytes,
                        &compression_type,
                        content,
                    ) {
                        Ok(n) => n,
                        Err(e) => panic!(e),
                    };
                }
                records_content.insert(i, current_rec);
                records_content.remove(i + 1);
            }
        }
        Ok(records_content)
    }
}
