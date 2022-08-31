pub(crate) mod exth;
pub(crate) mod header;
pub(crate) mod mobih;
pub(crate) mod palmdoch;

pub use self::{
    exth::{ExtHeader, ExthRecord},
    header::{Header, HeaderParseError},
    mobih::{Language, MobiHeader, MobiType, TextEncoding},
    palmdoch::{Compression, Encryption, PalmDocHeader},
};

use crate::headers::exth::ExthRecordParseError;
use crate::headers::mobih::MobiHeaderParseError;
use crate::record::PdbRecords;
use crate::{Reader, Writer};

#[cfg(feature = "time")]
use chrono::NaiveDateTime;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetadataParseError {
    #[error(transparent)]
    HeaderParseError(#[from] HeaderParseError),
    #[error(transparent)]
    MobiHeaderParseError(#[from] MobiHeaderParseError),
    #[error(transparent)]
    ExthRecordParseError(#[from] ExthRecordParseError),
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("No records present in file")]
    NoRecords,
}

#[derive(Debug, Default)]
/// Holds all headers containing low level metadata of a mobi book
pub struct MobiMetadata {
    pub name: Vec<u8>,
    pub header: Header,
    pub records: PdbRecords,
    pub palmdoc: PalmDocHeader,
    pub mobi: MobiHeader,
    pub exth: ExtHeader,
}
impl MobiMetadata {
    /// Construct a Metadata object from a slice of bytes
    pub fn new<B: AsRef<Vec<u8>>>(bytes: B) -> Result<MobiMetadata, MetadataParseError> {
        MobiMetadata::from_reader(&mut Reader::new(std::io::Cursor::new(bytes.as_ref())))
    }

    /// Construct a Metadata object from passed file path
    pub fn from_path<P: AsRef<Path>>(file_path: P) -> Result<MobiMetadata, MetadataParseError> {
        let mut reader = Reader::new(BufReader::new(File::open(file_path)?));
        MobiMetadata::from_reader(&mut reader)
    }

    /// Construct a Metadata object from an object that implements a Read trait
    pub fn from_read<R: Read>(reader: R) -> Result<MobiMetadata, MetadataParseError> {
        MobiMetadata::from_reader(&mut Reader::new(reader))
    }

    pub(crate) fn from_reader<R: Read>(
        reader: &mut Reader<R>,
    ) -> Result<MobiMetadata, MetadataParseError> {
        let header = Header::parse(reader)?;

        let records = PdbRecords::new(reader, header.num_records)?;
        if records.records.is_empty() {
            return Err(MetadataParseError::NoRecords);
        }

        let palmdoc = PalmDocHeader::parse(reader)?;
        let mobi = MobiHeader::parse(reader)?;

        let exth = if mobi.has_exth_header() {
            ExtHeader::parse(reader)?
        } else {
            ExtHeader::default()
        };

        let name_offset = match records.records[0].offset.checked_add(mobi.name_offset) {
            None => {
                return Err(MetadataParseError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    "attempted to seek with overflow",
                )))
            }
            Some(offset) => offset,
        };
        reader.set_position(name_offset as usize)?;
        let name = reader.read_vec_header(mobi.name_length as usize)?;

        Ok(MobiMetadata {
            name,
            header,
            records,
            palmdoc,
            mobi,
            exth,
        })
    }

    #[allow(dead_code)]
    fn write(&self, writer: &mut impl io::Write) -> io::Result<()> {
        self.write_into(&mut Writer::new(writer))
    }

    pub(crate) fn write_into<W: io::Write>(&self, w: &mut Writer<W>) -> io::Result<()> {
        self.header.write(w, self.records.num_records())?;
        self.records.write(w)?;
        self.palmdoc.write(w)?;
        self.mobi.write(w)?;
        if self.mobi.has_exth_header() {
            self.exth.write(w)?;
        }

        let fill = ((self.records.records[0].offset + self.mobi.name_offset) as usize)
            .saturating_sub(w.bytes_written());
        w.write_be(vec![0; fill])?;
        w.write_be(self.name.as_slice())
    }

    //################################################################################//
    // Not available in Mobi

    /// Returns raw ExthRecord data located at appropriate position if it exists. It is
    /// highly recommended to use public api provided here to access those records but
    /// in case where lower level access is needed this method provides access to all fields.
    ///
    /// Some records can occur multiple times thats why a list of data buffers is returned.
    pub fn exth_record(&self, record: ExthRecord) -> Option<&Vec<Vec<u8>>> {
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
    ///
    /// Some records can occur multiple times thats why a list of data buffers is returned.
    pub fn exth_record_at(&self, position: u32) -> Option<&Vec<Vec<u8>>> {
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
        self.exth
            .get_record_string_lossy(exth::ExthRecord::Publisher)
    }

    /// Returns description record if such exists
    pub fn description(&self) -> Option<String> {
        self.exth
            .get_record_string_lossy(exth::ExthRecord::Description)
    }

    /// Returns isbn record if such exists
    pub fn isbn(&self) -> Option<String> {
        self.exth.get_record_string_lossy(exth::ExthRecord::Isbn)
    }

    /// Returns publish_date record if such exists
    pub fn publish_date(&self) -> Option<String> {
        self.exth
            .get_record_string_lossy(exth::ExthRecord::PublishDate)
    }

    /// Returns contributor record if such exists
    pub fn contributor(&self) -> Option<String> {
        self.exth
            .get_record_string_lossy(exth::ExthRecord::Contributor)
    }

    /// Returns title record read from EXTH header if it exists
    /// or defaults to full book name read from location specified
    /// in MOBI header.
    pub fn title(&self) -> String {
        self.exth
            .get_record_string_lossy(exth::ExthRecord::Title)
            .map_or(String::from_utf8_lossy(&self.name).to_string(), |v| v)
    }

    /// Returns text encoding used in ebook
    pub fn text_encoding(&self) -> TextEncoding {
        self.mobi.text_encoding()
    }

    /// Returns type of this ebook
    pub fn mobi_type(&self) -> MobiType {
        self.mobi.mobi_type()
    }

    /// Returns language of the ebook
    pub fn language(&self) -> Language {
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
    pub fn compression(&self) -> Compression {
        self.palmdoc.compression()
    }

    /// Returns encryption method used on this file
    pub fn encryption(&self) -> Encryption {
        self.palmdoc.encryption()
    }

    /// Returns a list of subject records as a string if such records exist
    pub fn subjects(&self) -> Option<Vec<String>> {
        self.exth_record(ExthRecord::Subject).map(|s| {
            s.iter()
                .map(|s| String::from_utf8_lossy(s).to_string())
                .collect()
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::book;

    #[test]
    fn test_mobi_metadata() {
        let mut reader = book::u8_reader(book::full_book());
        assert!(MobiMetadata::from_reader(&mut reader).is_ok());
    }

    #[test]
    fn test_mobi_write() {
        // First write will lose duplicate ExtHeader records.
        let m = MobiMetadata::from_reader(&mut book::u8_reader(book::full_book())).unwrap();
        let mut bytes = vec![];
        assert!(m.write(&mut bytes).is_ok());
        assert_eq!(bytes, book::MOBI_METADATA);
    }
}
