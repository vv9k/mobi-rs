//! An implementation of [MOBI](https://wiki.mobileread.com/wiki/MOBI) format data parsing and manipulation, written in Rust.
//!
//! The code is available on [GitHub](https://github.com/wojciechkepka/mobi-rs)
//!
//! License: [*MIT*](https://github.com/wojciechkepka/mobi-rs/blob/master/license)
//!
//! ## Examples
//! Examples are available on the GitHub repository
pub(crate) mod book;
pub(crate) mod exth;
pub(crate) mod header;
pub(crate) mod lz77;
pub(crate) mod mobih;
pub(crate) mod palmdoch;
pub(crate) mod record;
use byteorder::{BigEndian, ReadBytesExt};
#[cfg(feature = "time")]
use chrono::prelude::*;
use exth::BookInfo;
pub use exth::ExtHeader;
pub use header::Header;
pub use mobih::{MobiHeader, TextEncoding};
use palmdoch::Compression;
pub use palmdoch::PalmDocHeader;
pub use record::Record;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fs;
use std::io;
use std::io::{Cursor, Read};
use std::path::Path;
#[derive(Debug, Default)]
/// Structure that holds parsed ebook information and contents
pub struct Mobi {
    pub content: Vec<u8>,
    pub header: Header,
    pub palmdoc: PalmDocHeader,
    pub mobi: MobiHeader,
    pub exth: ExtHeader,
    pub records: Vec<Record>,
}
impl TryFrom<&[u8]> for Mobi {
    type Error = io::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::new(&bytes)
    }
}
impl Mobi {
    pub fn new(bytes: &[u8]) -> Result<Mobi, io::Error> {
        let header = Header::parse(&bytes)?;
        let palmdoc = PalmDocHeader::parse(&bytes, header.num_of_records)?;
        let mobi = MobiHeader::parse(&bytes, header.num_of_records)?;
        let exth = {
            if mobi.has_exth_header {
                ExtHeader::parse(&bytes, header.num_of_records)?
            } else {
                ExtHeader::default()
            }
        };
        let records = Record::parse_records(
            &bytes,
            header.num_of_records,
            mobi.extra_bytes,
            palmdoc.compression_en(),
            mobi.text_encoding(),
        )?;
        Ok(Mobi {
            content: bytes.to_vec(),
            header,
            palmdoc,
            mobi,
            exth,
            records,
        })
    }
    /// Construct a Mobi object from passed file path
    pub fn from_path<P: AsRef<Path>>(file_path: P) -> Result<Mobi, io::Error> {
        Self::new(&fs::read(file_path)?)
    }
    /// Construct a Mobi object from an object that implements a Read trait
    pub fn from_reader<R: Read>(reader: R) -> Result<Mobi, io::Error> {
        let mut content = Vec::new();
        for byte in reader.bytes() {
            content.push(byte?);
        }
        Self::new(&content)
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
    pub fn text_encoding(&self) -> TextEncoding {
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
    #[cfg(feature = "time")]
    /// Returns creation datetime
    /// This field is only available using `time` feature
    pub fn created_datetime(&self) -> NaiveDateTime {
        self.header.created_datetime()
    }
    #[cfg(feature = "time")]
    /// Returns modification datetime
    /// This field is only available using `time` feature
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
    /// Returns last readable index of the book
    pub fn last_index(&self) -> usize {
        (self.palmdoc.record_count - 1) as usize
    }
    /// Returns a slice of the content where b is beginning index and e is ending index.
    /// Use `last_index` function to find out the last readable index
    pub fn content_slice(&self, b: usize, e: usize) -> Option<String> {
        if (b >= 1) && (b <= e) && (e < self.last_index()) {
            Some((b..e).map(|i| self.records[i as usize].to_string()).collect())
        } else {
            None
        }
    }
}
#[cfg(feature = "fmt")]
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

/// Helper trait to group all enums containing header fields corresponding to each possible header
/// (MobiHeaderData, ExtHeaderData, PalmDocHeaderData, HeaderData)
pub(crate) trait FieldHeaderEnum {}
/// Trait allowing generic reading of header fields
pub(crate) trait HeaderField<T: FieldHeaderEnum> {
    /// Returns a position in the text where this field can be read
    fn position(self) -> u16;
}

/// Helper struct for reading header values from content
pub(crate) struct Reader<'r> {
    cursor: Cursor<&'r [u8]>,
    num_of_records: u16,
}
impl<'r> Reader<'r> {
    pub(crate) fn new(content: &[u8], num_of_records: u16) -> Reader {
        Reader {
            cursor: Cursor::new(content),
            num_of_records,
        }
    }
    pub(crate) fn read_i16_header<T: FieldHeaderEnum, F: HeaderField<T>>(
        &mut self,
        field: F,
    ) -> Result<i16, io::Error> {
        self.cursor
            .set_position(field.position() as u64 + u64::from(self.num_of_records * 8));
        self.cursor.read_i16::<BigEndian>()
    }
    pub(crate) fn read_u16_header<T: FieldHeaderEnum, F: HeaderField<T>>(
        &mut self,
        field: F,
    ) -> Result<u16, io::Error> {
        self.cursor
            .set_position(field.position() as u64 + u64::from(self.num_of_records * 8));
        self.cursor.read_u16::<BigEndian>()
    }
    pub(crate) fn read_u32_header<T: FieldHeaderEnum, F: HeaderField<T>>(
        &mut self,
        field: F,
    ) -> Result<u32, io::Error> {
        self.cursor
            .set_position(field.position() as u64 + u64::from(self.num_of_records * 8));
        self.cursor.read_u32::<BigEndian>()
    }
    pub(crate) fn read_string_header<T: FieldHeaderEnum, F: HeaderField<T>>(&mut self, field: F, len: u64) -> String {
        let position = field.position();
        String::from_utf8_lossy(&self.cursor.get_ref()[position as usize..(position as u64 + len) as usize])
            .to_owned()
            .to_string()
    }
}
