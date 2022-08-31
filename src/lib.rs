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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_no_records() {
        let bytes = [
            173, 21, 58, 173, 252, 173, 173, 173, 173, 173, 173, 173, 165, 173, 173, 173, 0, 0, 0,
            255, 255, 255, 255, 255, 255, 255, 139, 0, 0, 0, 0, 0, 0, 50, 3, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 50, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 50, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 231, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 173, 173, 173, 173, 0, 0, 0, 0,
            0, 0, 0, 173, 173, 173, 33, 173, 173, 173, 173, 173, 173, 173, 173, 173, 173, 173, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 70, 70, 70, 70, 70, 70, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            173, 162, 162, 162, 173, 173, 84, 255,
        ];
        assert!(Mobi::new(bytes.to_vec()).is_err());
    }

    #[test]
    fn test_backwards_cursor_1() {
        let bytes = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 173, 21, 58, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 236, 0, 0, 3, 0, 173, 173, 173,
            173, 173, 173, 173, 173, 173, 173, 173, 173, 173, 173, 173, 162, 162, 162, 173, 162,
            255, 255, 255, 5, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 121, 121, 121, 121, 121, 121, 121,
            121, 121, 121, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244,
            244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 3, 0, 0, 0, 0, 0,
            0, 0, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244,
            244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 244, 121, 121, 0, 193,
            0, 0, 0, 0, 0, 65, 0, 0, 64, 0, 0, 0, 0, 0, 0, 10,
        ];
        assert!(Mobi::new(bytes.to_vec()).is_err());
    }

    #[test]
    fn test_backwards_cursor_2() {
        let bytes = [
            0, 0, 0, 0, 0, 0, 0, 198, 0, 0, 0, 0, 0, 0, 0, 0, 0, 50, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 50, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 50, 3,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 231, 0, 2, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 231, 0, 0, 0, 0, 0, 0, 0,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 231, 0, 2, 0, 0, 0, 0, 193, 2, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 10,
        ];
        assert!(Mobi::new(bytes.to_vec()).is_err());
    }

    #[test]
    fn test_offset_mismatch() {
        let bytes = [
            0, 0, 0, 0, 0, 0, 50, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 50, 3, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 231, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 231, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 231, 0,
            2, 0, 0, 0, 0, 193, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 50, 3, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 193, 0, 0, 0, 0, 0, 65, 0, 0, 0, 62, 0, 0, 0, 65, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 10,
        ];
        assert!(Mobi::new(bytes.to_vec()).is_err());
    }

    #[test]
    fn test_offset_overflow() {
        let bytes = [
            0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 151, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 65, 0,
            0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 193, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 251, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 245, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10,
        ];
        assert!(Mobi::new(bytes.to_vec()).is_err());
    }

    #[test]
    fn test_small_record() {
        let bytes = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 50, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            50, 3, 128, 0, 0, 0, 0, 0, 0, 0, 0, 50, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 50, 3, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 231, 0, 2, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 0, 0, 0, 0, 0, 0, 50, 3, 128, 0, 0, 0, 0, 0, 0, 0, 0, 50, 3, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 50, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45, 0, 0, 0, 0, 231, 0, 2, 0, 0, 0, 0,
            193, 0, 0, 0, 254, 255, 0, 173, 173, 173, 173, 0, 0, 0, 0, 0, 0, 0, 173, 173, 173, 33,
            173, 173, 173, 173, 173, 173, 173, 173, 173, 173, 173, 0, 0, 0, 0, 0, 0, 0, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 49, 0, 10,
        ];
        assert!(Mobi::new(bytes.to_vec()).is_err());
    }

    #[test]
    fn test_subtract_overflow() {
        let bytes = [
            211, 147, 90, 255, 64, 255, 211, 211, 211, 88, 84, 77, 79, 66, 73, 77, 79, 66, 73, 10,
            1, 23, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 211, 10, 211, 61, 45, 84, 69, 88,
            84, 77, 79, 66, 73, 10, 20, 0, 0, 0, 0, 0, 0, 0, 10, 211, 61, 45, 84, 69, 88, 84, 77,
            79, 66, 73, 77, 79, 66, 73, 10, 1, 23, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255,
            211, 10, 211, 61, 45, 84, 69, 88, 84, 77, 79, 66, 73, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 20, 0, 0, 248, 255,
            255, 255, 23, 0, 0, 0, 0, 0, 0, 0, 211, 61, 45, 84, 69, 88, 84, 77, 79, 66, 73, 79, 66,
            73, 10, 1, 23, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 211, 10, 211, 61, 45, 84,
            69, 88, 84, 77, 79, 66, 73, 10, 20, 0, 0, 77, 79, 66, 73, 211, 147, 90, 255, 64, 255,
            211, 211, 9, 0, 0, 0, 0, 0, 0, 0, 211, 10, 211, 211, 10, 1, 255, 0, 188, 255, 211, 1,
            23, 0, 0, 0, 0, 0, 0, 10, 20, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 10, 211, 61, 45, 84, 69, 88, 84, 77, 79, 66, 73,
            77, 79, 66, 73, 10, 1, 23, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 211, 0, 255,
            255, 255, 255, 255, 211, 10, 211, 61, 45, 84, 69, 88, 84, 77, 79, 66, 73, 10, 20, 0, 0,
            248, 255, 255, 255, 211, 61, 45, 84, 69, 88, 84, 77, 79, 66, 73, 77, 79, 66, 73, 10, 1,
            255, 0, 188, 255, 211, 10, 211, 61, 45, 84, 69, 10, 211, 61, 45, 84, 69, 79, 75,
        ];
        assert!(Mobi::new(bytes.to_vec()).is_err());
    }

    #[test]
    fn test_read_out_of_bounds() {
        let bytes = [
            211, 147, 90, 255, 64, 255, 211, 211, 211, 10, 211, 211, 211, 255, 255, 255, 255, 255,
            211, 10, 211, 61, 45, 84, 69, 88, 84, 1, 0, 0, 0, 188, 128, 255, 42, 0, 211, 207, 147,
            90, 255, 64, 255, 211, 211, 211, 10, 211, 211, 108, 255, 255, 255, 255, 255, 211, 10,
            211, 61, 45, 84, 69, 88, 84, 77, 79, 66, 73, 77, 79, 66, 73, 10, 1, 23, 0, 0, 0, 0, 0,
            0, 0, 255, 255, 0, 0, 0, 159, 10, 211, 61, 211, 255, 69, 88, 84, 77, 79, 66, 73, 0, 0,
            0, 232, 10, 20, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 211, 147, 90, 255,
            64, 255, 211, 211, 211, 88, 84, 77, 79, 66, 73, 77, 79, 66, 73, 10, 1, 23, 0, 0, 0, 0,
            0, 0, 0, 255, 255, 255, 255, 255, 211, 10, 211, 61, 45, 84, 69, 88, 84, 77, 79, 66, 73,
            10, 20, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 66, 73, 10, 1, 255, 0,
            211, 1, 23, 84, 69, 88, 84, 0, 0, 0, 0, 0, 0, 10, 20, 0, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 35, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 255, 0,
            0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 248, 255, 255, 255, 211, 61, 45, 84, 69, 88, 84, 77,
            1, 0, 0, 0, 0, 0, 0, 1, 79, 66, 73, 77, 79, 66, 255, 255, 255, 255, 211, 0, 255, 255,
            255, 255, 0, 188, 73, 10, 1, 255, 0, 188, 255, 211, 1, 23, 0, 0, 0, 0, 0, 0, 0, 255,
            255, 255, 255, 255, 211, 10, 211, 61, 45, 84, 69, 88, 84, 77, 79, 66, 73, 10, 20, 0, 0,
            248, 255, 255, 255, 211, 61, 255, 211, 10, 211, 61, 45, 84, 69, 10, 211, 45, 84, 69,
            88, 84, 77, 79, 66, 73, 77, 79, 66, 61, 45, 84, 73, 10, 1, 255, 0, 188, 255, 211, 10,
            69, 211, 61, 45, 84, 69, 10, 211, 61, 45, 84, 69, 79, 79, 75, 75,
        ];
        assert!(Mobi::new(bytes.to_vec()).is_err());
    }

    #[test]
    fn test_overflowing_record() {
        let bytes = [
            211, 147, 90, 255, 64, 255, 211, 211, 211, 88, 84, 77, 79, 66, 73, 77, 79, 66, 2, 0, 0,
            0, 73, 10, 1, 23, 0, 0, 0, 0, 0, 0, 77, 79, 66, 2, 0, 0, 0, 73, 10, 1, 23, 0, 0, 0,
            255, 255, 255, 22, 255, 255, 255, 255, 255, 211, 10, 211, 61, 45, 84, 69, 88, 84, 77,
            79, 66, 73, 10, 20, 0, 0, 0, 0, 0, 0, 0, 10, 211, 61, 45, 84, 69, 88, 93, 77, 79, 66,
            73, 77, 79, 66, 73, 10, 1, 23, 0, 11, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 211, 10, 211,
            61, 45, 255, 211, 10, 211, 61, 45, 84, 69, 88, 84, 77, 79, 66, 73, 10, 20, 0, 0, 0, 0,
            0, 0, 0, 10, 211, 61, 45, 84, 69, 88, 84, 93, 79, 66, 73, 77, 79, 66, 73, 84, 77, 79,
            66, 2, 0, 0, 0, 73, 10, 1, 0, 10, 20, 0, 255, 255, 255, 255, 45, 84, 69, 88, 84, 77,
            79, 66, 73, 0, 0, 0, 255, 255, 255, 22, 255, 255, 255, 255, 255, 211, 10, 211, 61, 45,
            84, 69, 88, 84, 77, 79, 66, 73, 10, 20, 0, 0, 0, 0, 0, 0, 0, 10, 211, 61, 45, 84, 69,
            88, 93, 77, 79, 66, 73, 77, 79, 66, 73, 10, 1, 23, 0, 11, 0, 0, 0, 0, 0, 0, 0, 255,
            255, 211, 61, 45, 255, 211, 10, 211, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 61, 45, 84, 69, 88, 84, 77, 79, 66, 73, 10, 20, 0, 0, 0, 0, 0, 0, 0, 10,
            211, 61, 45, 84, 69, 88, 84, 93, 79, 66, 73, 77, 79, 66, 73, 84, 77, 79, 66, 2, 0, 0,
            0, 73, 10, 1, 0, 10, 20, 0, 255, 255, 255, 255, 45, 84, 69, 88, 84, 77, 79, 66, 73,
            179, 176, 189, 182, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 75, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 10, 1, 255, 0, 230, 230, 230, 230, 230, 230,
            230, 0, 0, 255, 255, 255, 255, 255, 211, 10, 211, 61, 45, 255, 211, 10, 211, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 61, 45, 84, 69, 88, 84, 77,
            79, 66, 73, 10, 20, 0, 0, 0, 0, 0, 0, 0, 10, 211, 61, 45, 84, 69, 88, 84, 93, 79, 66,
            73, 77, 79, 66, 73, 84, 77, 79, 66, 2, 0, 0, 0, 73, 10, 1, 0, 10, 20, 0, 255, 255, 255,
            255, 45, 84, 69, 88, 84, 77, 79, 66, 73, 179, 176, 189, 182, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 75, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 179, 176, 189, 182, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 79, 66, 0, 0, 0, 0, 0, 0, 195, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230,
            230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230,
            230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230,
            230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 240, 230, 230, 230, 230,
            230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230,
            230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 0, 0, 255, 255, 255, 255, 255, 211,
            10, 211, 61, 45, 255, 211, 10, 211, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 61, 45, 84, 69, 88, 84, 77, 79, 66, 73, 10, 20, 0, 0, 0, 0, 0, 0, 0, 10,
            211, 61, 45, 84, 69, 88, 84, 93, 79, 66, 73, 77, 79, 66, 73, 84, 77, 79, 66, 2, 0, 0,
            0, 73, 10, 1, 0, 10, 20, 0, 255, 255, 255, 255, 45, 84, 69, 88, 84, 77, 79, 66, 73,
            179, 176, 189, 182, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 75, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 10, 1, 255, 0, 188, 255, 61, 45, 84, 69, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 39, 0, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230, 230,
            230, 230, 230, 230, 230, 230, 230, 230, 0, 0, 0, 0, 0, 79, 75,
        ];
        assert!(Mobi::new(bytes.to_vec()).is_err());
    }

    #[test]
    fn test_set_position_oom() {
        let bytes = [
            242, 242, 242, 242, 242, 242, 84, 80, 90, 55, 242, 242, 242, 242, 242, 242, 242, 242,
            242, 242, 242, 242, 242, 242, 242, 242, 242, 242, 130, 62, 178, 126, 126, 126, 126,
            130, 9, 68, 82, 77, 73, 79, 78, 238, 126, 126, 126, 126, 126, 126, 126, 126, 126, 126,
            126, 126, 126, 126, 126, 126, 84, 69, 88, 84, 82, 69, 65, 68, 126, 126, 126, 126, 242,
            136, 126, 1, 0, 8, 242, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
            15, 15, 15, 15, 15, 247, 247, 247, 247, 247, 247, 247, 247, 247, 247, 247, 247, 84, 69,
            255, 255, 255, 255, 9, 0, 0, 0, 0, 0, 0, 0, 255, 126, 126, 126, 126, 126, 126, 126,
            126, 242, 0, 126, 126, 126, 126, 126, 126, 126, 126, 126, 126, 84, 69, 88, 84, 82, 69,
            65, 68, 82, 175, 130, 129, 129, 77, 79, 66, 73, 0, 0, 0, 232, 122, 126, 255, 255, 255,
            255, 255, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
            15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
            15, 15, 1, 69, 88, 84, 15, 15, 15, 15, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
            15, 15, 15, 15, 15, 15, 15, 15, 15, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 48, 126, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163,
            163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163,
            163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 247,
            247, 247, 247, 247, 247, 247, 247, 247, 247, 247, 247, 247, 247, 247, 247, 247, 247,
            84, 69, 255, 255, 255, 255, 255, 255, 40, 255, 255, 255, 255, 255, 255, 126, 126, 126,
            126, 126, 93, 92, 92, 92, 92, 92, 92, 92, 163, 163, 163, 163, 163, 212, 163, 163, 163,
            163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163,
            163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163, 163,
        ];
        assert!(Mobi::new(bytes.to_vec()).is_err());
    }
}
