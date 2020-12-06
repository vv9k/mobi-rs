pub(crate) mod exth;
pub(crate) mod header;
pub(crate) mod mobih;
pub(crate) mod palmdoch;
pub(crate) mod records;

pub use self::{
    exth::{ExtHeader, ExthRecord},
    header::Header,
    mobih::{MobiHeader, TextEncoding},
    palmdoch::PalmDocHeader,
};

use crate::headers::records::Records;
use crate::reader::{MobiReader, ReaderPrime};
use crate::Reader;
#[cfg(feature = "time")]
use chrono::NaiveDateTime;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

#[derive(Debug, Default)]
/// Holds all headers containing low level metadata of a mobi book
pub struct MobiMetadata {
    pub header: Header,
    pub records: Records,
    pub palmdoc: PalmDocHeader,
    pub mobi: MobiHeader,
    pub exth: ExtHeader,
}
impl MobiMetadata {
    /// Construct a Metadata object from a slice of bytes
    pub fn new<B: AsRef<Vec<u8>>>(bytes: B) -> io::Result<MobiMetadata> {
        MobiMetadata::from_reader(&mut ReaderPrime::new(std::io::Cursor::new(bytes.as_ref())))
    }

    /// Construct a Metadata object from passed file path
    pub fn from_path<P: AsRef<Path>>(file_path: P) -> io::Result<MobiMetadata> {
        let mut reader = ReaderPrime::new(BufReader::new(File::open(file_path)?));
        MobiMetadata::from_reader(&mut reader)
    }

    /// Construct a Metadata object from an object that implements a Read trait
    pub fn from_read<R: Read>(reader: R) -> io::Result<MobiMetadata> {
        let content: Vec<_> = reader.bytes().flatten().collect();
        MobiMetadata::from_reader(&mut Reader::new(&content))
    }

    pub(crate) fn from_reader(reader: &mut impl MobiReader) -> io::Result<MobiMetadata> {
        let header = Header::parse(reader)?;
        reader.set_num_records(header.num_records);
        let records = Records::parse(reader)?;
        let palmdoc = PalmDocHeader::parse(reader)?;
        let mut mobi = MobiHeader::partial_parse(reader)?;

        let exth = {
            if mobi.has_exth_header() {
                ExtHeader::parse(reader, mobi.header_length)?
            } else {
                ExtHeader::default()
            }
        };

        let offset1 = reader.position_after_records() + 80 + mobi.name_offset as u64;
        let offset = records.records[0].0 + mobi.name_offset;

        assert_eq!(offset as u64, offset1);
        mobi.finish_parse(reader)?;

        Ok(MobiMetadata {
            header,
            records,
            palmdoc,
            mobi,
            exth,
        })
    }

    //################################################################################//
    // Not available in Mobi

    /// Returns raw ExthRecord data located at appropriate position if it exists. It is
    /// highly recommended to use public api provided here to access those records but
    /// in case where lower level access is needed this method provides access to all fields.
    pub fn exth_record(&self, record: ExthRecord) -> Option<&Vec<u8>> {
        self.exth.get_record(record)
    }

    /// Returns raw ExthRecord data located at passed position if it exists.
    ///
    /// If unsure where the wanted record is located at use exth_record method that
    /// limits possible position to those commonly available on mobi books.
    ///
    /// It is highly recommended to instead use public api provided here to access
    /// those records but in case where lower level access is needed this method
    /// provides access to all fields.
    pub fn exth_record_at(&self, position: u32) -> Option<&Vec<u8>> {
        self.exth.get_record_position(position)
    }

    //################################################################################//
    // Available in Mobi

    /// Returns an author of this book
    pub fn author(&self) -> Option<String> {
        self.exth.get_record_string_lossy(exth::ExthRecord::Author)
    }

    /// Returns this books publisher
    pub fn publisher(&self) -> Option<String> {
        self.exth.get_record_string_lossy(exth::ExthRecord::Publisher)
    }

    /// Returns description record if such exists
    pub fn description(&self) -> Option<String> {
        self.exth.get_record_string_lossy(exth::ExthRecord::Description)
    }

    /// Returns isbn record if such exists
    pub fn isbn(&self) -> Option<String> {
        self.exth.get_record_string_lossy(exth::ExthRecord::Isbn)
    }

    /// Returns publish_date record if such exists
    pub fn publish_date(&self) -> Option<String> {
        self.exth.get_record_string_lossy(exth::ExthRecord::PublishDate)
    }

    /// Returns contributor record if such exists
    pub fn contributor(&self) -> Option<String> {
        self.exth.get_record_string_lossy(exth::ExthRecord::Contributor)
    }

    /// Returns title record if such exists
    pub fn title(&self) -> Option<String> {
        self.exth.get_record_string_lossy(exth::ExthRecord::Title)
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

    #[cfg(not(feature = "time"))]
    /// Returns creation time as u32 timestamp
    pub fn created_time(&self) -> u32 {
        self.header.created_datetime()
    }

    #[cfg(not(feature = "time"))]
    /// Returns last modification time as u32 timestamp
    pub fn mod_time(&self) -> u32 {
        self.header.mod_datetime()
    }

    /// Returns compression method used on this file
    pub fn compression(&self) -> String {
        self.palmdoc.compression()
    }

    /// Returns encryption method used on this file
    pub fn encryption(&self) -> String {
        self.palmdoc.encryption()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::book;

    #[test]
    fn test_mobi_metadata() {
        let book = book::full_book();
        let mut reader = Reader::new(&book);
        assert!(MobiMetadata::from_reader(&mut reader).is_ok());
    }
}
