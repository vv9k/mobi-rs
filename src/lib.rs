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
pub(crate) mod book;
pub(crate) mod exth;
pub(crate) mod header;
pub(crate) mod lz77;
pub(crate) mod mobih;
pub(crate) mod palmdoch;
pub(crate) mod record;
use byteorder::{BigEndian, ReadBytesExt};
use chrono::prelude::*;
use exth::BookInfo;
pub use exth::ExtHeader;
pub use header::Header;
pub use mobih::MobiHeader;
use palmdoch::Compression;
pub use palmdoch::PalmDocHeader;
pub use record::Record;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::Cursor;
use std::io::Read;
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
            palmdoc.compression_en(),
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
    /// Construct a Mobi object from an object that implements a Read trait
    pub fn from_reader<R: Read>(reader: R) -> Result<Mobi, std::io::Error> {
        let mut contents = vec![];
        for byte in reader.bytes() {
            contents.push(byte?);
        }
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
            palmdoc.compression_en(),
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
    pub fn content_as_string(&self) -> String {
        (1..self.palmdoc.record_count - 1)
            .map(|i| self.records[i as usize].to_string())
            .collect()
    }
    /// Returns a slice of the content where b is beginning index and e is ending index.
    /// Usually readable indexes are between 1-300(+-50)
    pub fn content_slice(&self, b: usize, e: usize) -> Option<String> {
        if (b >= 1) && (b <= e) && (e < (self.palmdoc.record_count - 1) as usize) {
            Some(
                (b..e)
                    .map(|i| self.records[i as usize].to_string())
                    .collect(),
            )
        } else {
            None
        }
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
