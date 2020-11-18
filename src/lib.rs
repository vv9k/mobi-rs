//! An implementation of [MOBI](https://wiki.mobileread.com/wiki/MOBI) format data parsing and manipulation, written in Rust.
//!
//! The code is available on [GitHub](https://github.com/wojciechkepka/mobi-rs)
//!
//! License: [*MIT*](https://github.com/wojciechkepka/mobi-rs/blob/master/license)
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
//!     let empty = "".to_string();
//!     // Access metadata
//!     let title = m.title().unwrap_or(&empty);
//!     let author = m.author().unwrap_or(&empty);
//!     let publisher = m.publisher().unwrap_or(&empty);
//!     let desc = m.description().unwrap_or(&empty);
//!     let isbn = m.isbn().unwrap_or(&empty);
//!     let pub_date = m.publish_date().unwrap_or(&empty);
//!     let contributor = m.contributor().unwrap_or(&empty);
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
pub use headers::Metadata;
pub use record::Record;

pub(crate) mod book;
#[cfg(feature = "fmt")]
mod display;
pub(crate) mod lz77;
pub(crate) mod reader;
pub(crate) mod record;
#[cfg(feature = "time")]
use chrono::NaiveDateTime;
use headers::TextEncoding;
pub(crate) use reader::Reader;
use std::{fs, io, io::Read, ops::Range, path::Path};

#[derive(Debug, Default)]
/// Structure that holds parsed ebook information and contents
pub struct Mobi {
    pub content: Vec<u8>,
    pub metadata: Metadata,
}
impl Mobi {
    /// Construct a Mobi object from a slice of bytes
    pub fn new<B: AsRef<Vec<u8>>>(bytes: B) -> io::Result<Mobi> {
        Mobi::from_reader(Reader::new(bytes.as_ref()))
    }

    /// Construct a Mobi object from passed file path
    pub fn from_path<P: AsRef<Path>>(file_path: P) -> io::Result<Mobi> {
        Mobi::new(&fs::read(file_path)?)
    }

    /// Construct a Mobi object from an object that implements a Read trait
    pub fn from_read<R: Read>(reader: R) -> io::Result<Mobi> {
        // Temporary solution
        let content: Vec<_> = reader.bytes().flatten().collect();
        Mobi::from_reader(Reader::new(&content))
    }

    fn from_reader(mut reader: Reader) -> io::Result<Mobi> {
        let metadata = Metadata::from_reader(&mut reader)?;
        Ok(Mobi {
            content: reader.content(),
            metadata,
        })
    }

    /// Returns an author of this book
    pub fn author(&self) -> Option<&String> {
        self.metadata.author()
    }

    /// Returns this books publisher
    pub fn publisher(&self) -> Option<&String> {
        self.metadata.publisher()
    }

    /// Returns description record if such exists
    pub fn description(&self) -> Option<&String> {
        self.metadata.description()
    }

    /// Returns isbn record if such exists
    pub fn isbn(&self) -> Option<&String> {
        self.metadata.isbn()
    }

    /// Returns publish_date record if such exists
    pub fn publish_date(&self) -> Option<&String> {
        self.metadata.publish_date()
    }

    /// Returns contributor record if such exists
    pub fn contributor(&self) -> Option<&String> {
        self.metadata.contributor()
    }

    /// Returns title record if such exists
    pub fn title(&self) -> Option<&String> {
        self.metadata.title()
    }

    /// Returns text encoding used in ebook
    pub fn text_encoding(&self) -> TextEncoding {
        self.metadata.text_encoding()
    }

    /// Returns type of this ebook
    pub fn mobi_type(&self) -> Option<String> {
        self.metadata.mobi_type()
    }

    /// Returns language of the ebook
    pub fn language(&self) -> Option<String> {
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
    pub fn compression(&self) -> String {
        self.metadata.compression()
    }
    /// Returns encryption method used on this file
    pub fn encryption(&self) -> String {
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
            self.metadata.header.num_of_records,
            self.metadata.mobi.extra_bytes,
            self.metadata.palmdoc.compression_enum(),
        )
    }

    /// Returns all readable records content decompressed as a String.
    /// There are only two supported encodings in mobi format (UTF8, WIN1252)
    /// and both are losely converted by this function
    pub fn content_as_string(&self) -> io::Result<String> {
        let records = self.records()?;
        Ok(self
            .readable_records_range()
            .map(|i| records[i as usize].to_string(self.text_encoding()))
            .collect())
    }

    /// Returns all readable records content decompressed as a Vec
    pub fn content(&self) -> io::Result<Vec<u8>> {
        let records = self.records()?;
        Ok(self
            .readable_records_range()
            .map(|i| records[i as usize].record_data.clone())
            .flatten()
            .collect())
    }
}
