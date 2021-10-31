//! An implementation of [MOBI](https://wiki.mobileread.com/wiki/MOBI) format data parsing and manipulation, written in Rust.
//!
//! The code is available on [GitHub](https://github.com/vv9k/mobi-rs)
//!
//! License: [*MIT*](https://github.com/vv9k/mobi-rs/blob/master/license)
//!
//! ## Examples
//! ### Access basic info
//! ```no_run
//! use mobi::Mobi;
//! fn main() -> Result<(), std::io::Error> {
//!     let book = vec![0, 0, 0];
//!     // You can either create a Mobi struct from a slice
//!     let m = Mobi::new(&book)?;
//!     // Or from an instance of io::Read
//!     let stdin = std::io::stdin();
//!     let mut handle = stdin.lock();
//!     let m = Mobi::from_read(&mut handle)?;
//!     // Or from filesystem
//!     let m = Mobi::from_path("/some/path/to/book.mobi")?;
//!
//!     // Access metadata
//!     let title = m.title();
//!     let author = m.author().unwrap_or_default();
//!     let publisher = m.publisher().unwrap_or_default();
//!     let desc = m.description().unwrap_or_default();
//!     let isbn = m.isbn().unwrap_or_default();
//!     let pub_date = m.publish_date().unwrap_or_default();
//!     let contributor = m.contributor().unwrap_or_default();
//!
//!     // Access Headers
//!     let metadata = &m.metadata;
//!     let header = &metadata.header; // Normal Header
//!     let pdheader = &metadata.palmdoc; // PalmDOC Header
//!     let mheader = &metadata.mobi; // MOBI Header
//!     let exth = &metadata.exth; // Extra Header
//!
//!     // Access content
//!     let content = m.content_as_string();
//!
//!     Ok(())
//! }
//! ```

/// Module with headers from book containg more extracted data not
/// available through public API.
pub mod headers;
pub use crate::headers::MobiMetadata;
pub(crate) mod book;
pub(crate) mod huff;
pub(crate) mod lz77;
pub(crate) mod reader;
pub(crate) mod record;
pub(crate) mod writer;

use crate::headers::{Compression, Encryption, Language, MobiType, TextEncoding};
pub(crate) use crate::reader::Reader;
pub(crate) use crate::writer::Writer;
#[cfg(feature = "time")]
use chrono::NaiveDateTime;
use record::{DecodeError, RawRecords};
use std::{fs::File, io, io::BufReader, ops::Range, path::Path};

#[derive(Debug, Default)]
/// Structure that holds parsed ebook information and contents
pub struct Mobi {
    pub content: Vec<u8>,
    pub metadata: MobiMetadata,
}

impl Mobi {
    /// Construct a Mobi object from a slice of bytes
    pub fn new<B: AsRef<Vec<u8>>>(bytes: B) -> io::Result<Mobi> {
        Mobi::from_reader(&mut Reader::new(std::io::Cursor::new(bytes.as_ref())))
    }

    /// Construct a Mobi object from passed file path
    pub fn from_path<P: AsRef<Path>>(file_path: P) -> io::Result<Mobi> {
        let mut reader = Reader::new(BufReader::new(File::open(file_path)?));
        Mobi::from_reader(&mut reader)
    }

    /// Construct a Mobi object from an object that implements a Read trait
    pub fn from_read<R: io::Read>(reader: R) -> io::Result<Mobi> {
        Mobi::from_reader(&mut Reader::new(reader))
    }

    fn from_reader<R: io::Read>(reader: &mut Reader<R>) -> io::Result<Mobi> {
        let metadata = MobiMetadata::from_reader(reader)?;
        Ok(Mobi {
            content: reader.read_to_end()?,
            metadata,
        })
    }

    #[allow(dead_code)]
    fn write(&self, writer: &mut impl io::Write) -> io::Result<()> {
        let mut w = Writer::new(writer);

        self.metadata.write_into(&mut w)?;

        let first_offset = self.metadata.records.records[1].offset as usize;
        let fill = first_offset - w.bytes_written();
        w.write_be(vec![0; fill])?;
        // TODO: Consider record compression and everything else.
        w.write_be(&self.content[first_offset..])
    }

    /// Returns an author of this book
    pub fn author(&self) -> Option<String> {
        self.metadata.author()
    }

    /// Returns this books publisher
    pub fn publisher(&self) -> Option<String> {
        self.metadata.publisher()
    }

    /// Returns description record if such exists
    pub fn description(&self) -> Option<String> {
        self.metadata.description()
    }

    /// Returns isbn record if such exists
    pub fn isbn(&self) -> Option<String> {
        self.metadata.isbn()
    }

    /// Returns publish_date record if such exists
    pub fn publish_date(&self) -> Option<String> {
        self.metadata.publish_date()
    }

    /// Returns contributor record if such exists
    pub fn contributor(&self) -> Option<String> {
        self.metadata.contributor()
    }

    /// Returns title record if such exists
    pub fn title(&self) -> String {
        self.metadata.title()
    }

    /// Returns text encoding used in ebook
    pub fn text_encoding(&self) -> TextEncoding {
        self.metadata.text_encoding()
    }

    /// Returns type of this ebook
    pub fn mobi_type(&self) -> MobiType {
        self.metadata.mobi_type()
    }

    /// Returns language of the ebook
    pub fn language(&self) -> Language {
        self.metadata.language()
    }

    #[cfg(feature = "time")]
    /// Returns creation datetime
    /// This field is only available using `time` feature
    pub fn created_datetime(&self) -> NaiveDateTime {
        self.metadata.created_datetime()
    }

    #[cfg(feature = "time")]
    /// Returns modification datetime
    /// This field is only available using `time` feature
    pub fn mod_datetime(&self) -> NaiveDateTime {
        self.metadata.mod_datetime()
    }

    #[cfg(not(feature = "time"))]
    /// Returns creation time as u32 timestamp
    pub fn created_time(&self) -> u32 {
        self.metadata.created_time()
    }

    #[cfg(not(feature = "time"))]
    /// Returns last modification time as u32 timestamp
    pub fn mod_time(&self) -> u32 {
        self.metadata.mod_time()
    }

    /// Returns compression method used on this file
    pub fn compression(&self) -> Compression {
        self.metadata.compression()
    }
    /// Returns encryption method used on this file
    pub fn encryption(&self) -> Encryption {
        self.metadata.encryption()
    }

    /// Returns last readable index of the book
    fn last_index(&self) -> usize {
        (self.metadata.palmdoc.record_count - 1) as usize
    }

    fn readable_records_range(&self) -> Range<usize> {
        1..self.last_index()
    }

    fn records(&self) -> io::Result<Vec<Record>> {
        Record::parse_records(
            &self.content,
            &self.metadata.records.records,
            self.metadata.records.extra_bytes(),
            self.metadata.palmdoc.compression(),
        )
    }

    /// Returns all readable records content decompressed as a String.
    /// There are only two supported encodings in mobi format (UTF8, WIN1252)
    /// and both are losely converted by this function
    pub fn content_as_string_lossy(&self) -> io::Result<String> {
        Ok(self.records()?[self.readable_records_range()]
            .iter()
            .map(|record| record.to_string_lossy(self.text_encoding()))
            .collect())
    }

    /// Returns all readable records content decompressed as a String.
    /// This function is a strict version returning error on first encountered
    /// decoding error.
    pub fn content_as_string(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut content = String::new();
        for record in &self.records()?[self.readable_records_range()] {
            content.push_str(&record.to_string(self.text_encoding())?);
        }

        Ok(content)
    }

    /// Returns all readable records content decompressed as a Vec
    pub fn content(&self) -> io::Result<Vec<u8>> {
        let records = &self.records()?[self.readable_records_range()];
        let mut record_data = Vec::with_capacity(records.iter().map(|r| r.record_data.len()).sum());
        for record in records {
            record_data.extend_from_slice(&record.record_data);
        }
        Ok(record_data)
    }
}
