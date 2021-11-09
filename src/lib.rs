//! An implementation of [MOBI](https://wiki.mobileread.com/wiki/MOBI) format data parsing and manipulation, written in Rust.
//!
//! The code is available on [GitHub](https://github.com/vv9k/mobi-rs)
//!
//! License: [*MIT*](https://github.com/vv9k/mobi-rs/blob/master/license)
//!
//! ## Examples
//! ### Access basic info
//! ```no_run
//! use mobi::{Mobi, MobiError};
//! fn main() -> Result<(), MobiError> {
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
pub mod record;
pub use crate::headers::MobiMetadata;
pub(crate) mod book;
pub(crate) mod compression;
pub(crate) mod reader;
pub(crate) mod writer;

use compression::huff;
use headers::{Compression, Encryption, Language, MobiType, TextEncoding};
pub(crate) use reader::Reader;
use record::{RawRecord, RawRecords};
pub(crate) use writer::Writer;

#[cfg(feature = "time")]
use chrono::NaiveDateTime;
use std::{fs::File, io, io::BufReader, ops::Range, path::Path};
use thiserror::Error;

pub type MobiResult<T> = std::result::Result<T, MobiError>;

#[derive(Debug, Error)]
pub enum MobiError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    MetadataParseError(#[from] headers::MetadataParseError),
    #[error(transparent)]
    DecodeError(#[from] record::DecodeError),
    #[error(transparent)]
    HuffmanError(#[from] huff::HuffmanError),
}

#[derive(Debug, Default)]
/// Structure that holds parsed ebook information and contents
pub struct Mobi {
    pub content: Vec<u8>,
    pub metadata: MobiMetadata,
}

impl Mobi {
    /// Construct a Mobi object from a slice of bytes
    pub fn new<B: AsRef<Vec<u8>>>(bytes: B) -> MobiResult<Mobi> {
        Mobi::from_reader(&mut Reader::new(std::io::Cursor::new(bytes.as_ref())))
    }

    /// Construct a Mobi object from passed file path
    pub fn from_path<P: AsRef<Path>>(file_path: P) -> MobiResult<Mobi> {
        let mut reader = Reader::new(BufReader::new(File::open(file_path)?));
        Mobi::from_reader(&mut reader)
    }

    /// Construct a Mobi object from an object that implements a Read trait
    pub fn from_read<R: io::Read>(reader: R) -> MobiResult<Mobi> {
        Mobi::from_reader(&mut Reader::new(reader))
    }

    fn from_reader<R: io::Read>(reader: &mut Reader<R>) -> MobiResult<Mobi> {
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

    /// Returns the readable reacord range - from first content record to first
    /// non book index.
    pub fn readable_records_range(&self) -> Range<usize> {
        self.metadata.mobi.first_content_record as usize
            ..self.metadata.mobi.first_non_book_index as usize
    }

    /// Returns raw records that contain compressed, encrypted and encoded content slices.
    pub fn raw_records(&self) -> RawRecords {
        self.metadata.records.parse(&self.content)
    }

    /// Returns all records classified as image records.
    pub fn image_records(&self) -> Vec<RawRecord> {
        self.raw_records()
            .range(self.metadata.mobi.first_image_index as usize..)
            .iter()
            .copied()
            .filter(|record| record.is_image_record())
            .collect()
    }

    fn palmdoc_string_lossy(&self) -> String {
        let encoding = self.text_encoding();
        self.raw_records()
            .range(self.readable_records_range())
            .iter()
            .map(|record| record.decompress_palmdoc().to_string_lossy(encoding))
            .collect()
    }

    fn palmdoc_string(&self) -> MobiResult<String> {
        let encoding = self.text_encoding();
        let mut s = String::new();

        for record in self.raw_records().range(self.readable_records_range()) {
            let content = record.decompress_palmdoc().to_string(encoding)?;
            s.push_str(&content);
        }
        Ok(s)
    }

    fn no_compression_string_lossy(&self) -> String {
        let encoding = self.text_encoding();
        self.raw_records()
            .range(self.readable_records_range())
            .iter()
            .map(|r| record::content_to_string_lossy(r.content, encoding))
            .collect()
    }

    fn no_compression_string(&self) -> MobiResult<String> {
        let encoding = self.text_encoding();
        let mut s = String::new();
        for record in self.raw_records().range(self.readable_records_range()) {
            let content = record::content_to_string(record.content, encoding)?;
            s.push_str(&content);
        }
        Ok(s)
    }

    fn huff_data(&self) -> MobiResult<Vec<Vec<u8>>> {
        let records = self.raw_records();
        let huff_start = self.metadata.mobi.first_huff_record as usize;
        let huff_count = self.metadata.mobi.huff_record_count as usize;
        let huffs: Vec<_> = records
            .range(huff_start..huff_start + huff_count)
            .iter()
            .map(|record| record.content)
            .collect();

        let sections: Vec<_> = records
            .range(self.readable_records_range())
            .iter()
            .map(|record| record.content)
            .collect();

        Ok(huff::decompress(&huffs, &sections)?)
    }

    fn huff_string_lossy(&self) -> MobiResult<String> {
        let encoding = self.text_encoding();
        let mut s = String::new();
        let data = self.huff_data()?;

        for section in data {
            let content = record::content_to_string_lossy(&section, encoding);
            s.push_str(&content);
        }
        Ok(s)
    }

    fn huff_string(&self) -> MobiResult<String> {
        let encoding = self.text_encoding();
        let mut s = String::new();
        let data = self.huff_data()?;

        for section in data {
            let content = record::content_to_string(&section, encoding)?;
            s.push_str(&content);
        }
        Ok(s)
    }

    /// Returns all readable records content decompressed as a String.
    /// There are only two supported encodings in mobi format (UTF8, WIN1252)
    /// and both are losely converted by this function
    pub fn content_as_string_lossy(&self) -> String {
        match self.compression() {
            Compression::No => self.no_compression_string_lossy(),
            Compression::PalmDoc => self.palmdoc_string_lossy(),
            Compression::Huff => self.huff_string_lossy().unwrap_or_default(),
        }
    }

    /// Returns all readable records content decompressed as a String.
    /// This function is a strict version returning error on first encountered
    /// decoding error.
    pub fn content_as_string(&self) -> MobiResult<String> {
        match self.compression() {
            Compression::No => self.no_compression_string(),
            Compression::PalmDoc => self.palmdoc_string(),
            Compression::Huff => self.huff_string(),
        }
    }
}
