use crate::compression::palmdoc;
use crate::headers::TextEncoding;
use crate::{Reader, Writer};

use encoding::{all::WINDOWS_1252, DecoderTrap, Encoding};
use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::io;
use std::ops::{Bound, RangeBounds};
use std::string::FromUtf8Error;

const EXTRA_BYTES_FLAG: u16 = 0xFFFE;

#[derive(Debug, Clone)]
/// A wrapper error type for unified error across multiple encodings.
pub enum DecodeError {
    UTF8(FromUtf8Error),
    CP1252(Cow<'static, str>),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeError::UTF8(e) => write!(f, "Failed decoding utf8 content - {}", e),
            DecodeError::CP1252(e) => write!(f, "Failed decoding win-cp1252 content - {}", e),
        }
    }
}

impl Error for DecodeError {}

#[derive(Debug, Default, Copy, Clone)]
pub struct RawRecord<'a> {
    pub record: PdbRecord,
    pub content: &'a [u8],
}

impl<'a> RawRecord<'a> {
    pub(crate) fn decompress_palmdoc(&self) -> DecompressedRecord {
        DecompressedRecord(palmdoc::decompress(self.content))
    }

    pub(crate) fn is_image_record(&self) -> bool {
        if self.content.len() < 4 {
            return false;
        }
        let bytes = &self.content[..4];

        bytes != b"FLIS"
            && bytes != b"FCIS"
            && bytes != b"SRCS"
            && bytes != b"RESC"
            && bytes != b"BOUN"
            && bytes != b"FDST"
            && bytes != b"DATP"
            && bytes != b"AUDI"
            && bytes != b"VIDE"
            && bytes != b"\xe9\x8e\r\n"
    }
}

#[derive(Debug, Default)]
pub struct RawRecords<'a>(pub(crate) Vec<RawRecord<'a>>);

impl<'a> IntoIterator for RawRecords<'a> {
    type Item = RawRecord<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> RawRecords<'a> {
    pub fn records(&self) -> &[RawRecord<'a>] {
        &self.0
    }

    pub fn range(&self, range: impl RangeBounds<usize>) -> &[RawRecord<'a>] {
        let len = self.0.len();
        if len == 0 {
            return &[];
        }
        let start = match range.start_bound() {
            Bound::Excluded(b) | Bound::Included(b) => (*b).min(len - 1),
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Excluded(b) => (*b - 1).min(len - 1),
            Bound::Included(b) => (*b).min(len - 1),
            Bound::Unbounded => 0,
        };
        &self.0[start..end]
    }
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct PdbRecord {
    pub id: u32,
    pub offset: u32,
}

#[derive(Debug, PartialEq, Default)]
pub struct PdbRecords {
    pub records: Vec<PdbRecord>,
    extra_bytes: u16,
}

impl PdbRecords {
    /// Parse the records from a reader. Reader must be advanced to the starting position
    /// of the records, at byte 78.
    pub(crate) fn new<R: io::Read>(
        reader: &mut Reader<R>,
        num_records: u16,
    ) -> io::Result<PdbRecords> {
        let mut records = Vec::with_capacity(num_records as usize);

        for _ in 0..num_records {
            records.push(PdbRecord {
                offset: reader.read_u32_be()?,
                id: reader.read_u32_be()?,
            });
        }

        Ok(PdbRecords {
            records,
            extra_bytes: reader.read_u16_be()?,
        })
    }

    /// Parses content returing raw records that contain slices of content based on their offset.
    pub(crate) fn parse<'a>(&self, content: &'a [u8]) -> RawRecords<'a> {
        let mut crecords = RawRecords::default();
        let extra_bytes = self.extra_bytes as usize;
        let mut records = self.records.iter().peekable();

        while let Some(record) = records.next() {
            let curr_offset = record.offset as usize;

            let content = if let Some(next) = records.peek() {
                let next_offset = next.offset as usize;

                if extra_bytes < next_offset {
                    &content[curr_offset..(next_offset - extra_bytes)]
                } else {
                    &[]
                }
            } else {
                &content[curr_offset..]
            };

            crecords.0.push(RawRecord {
                record: *record,
                content,
            });
        }
        crecords
    }

    pub fn extra_bytes(&self) -> u32 {
        2 * (self.extra_bytes & EXTRA_BYTES_FLAG).count_ones() as u32
    }

    pub fn num_records(&self) -> u16 {
        self.records.len() as u16
    }

    pub(crate) fn write<W: io::Write>(&self, w: &mut Writer<W>) -> io::Result<()> {
        for record in &self.records {
            w.write_be(record.offset)?;
            w.write_be(record.id)?;
        }
        w.write_be(self.extra_bytes)
    }
}

pub(crate) fn content_to_string_lossy(content: &[u8], encoding: TextEncoding) -> String {
    match encoding {
        TextEncoding::UTF8 | TextEncoding::Unknown(_) => {
            String::from_utf8_lossy(content).to_owned().to_string()
        }
        TextEncoding::CP1252 => WINDOWS_1252.decode(content, DecoderTrap::Ignore).unwrap(),
    }
}

pub(crate) fn content_to_string(
    content: &[u8],
    encoding: TextEncoding,
) -> Result<String, DecodeError> {
    match encoding {
        TextEncoding::UTF8 | TextEncoding::Unknown(_) => {
            String::from_utf8(content.to_vec()).map_err(DecodeError::UTF8)
        }
        TextEncoding::CP1252 => WINDOWS_1252
            .decode(content, DecoderTrap::Strict)
            .map_err(DecodeError::CP1252),
    }
}

#[derive(Debug, Default)]
pub(crate) struct DecompressedRecord(pub Vec<u8>);

impl DecompressedRecord {
    pub(crate) fn to_string_lossy(&self, encoding: TextEncoding) -> String {
        content_to_string_lossy(&self.0, encoding)
    }

    pub(crate) fn to_string(&self, encoding: TextEncoding) -> Result<String, DecodeError> {
        content_to_string(&self.0, encoding)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::book;

    #[test]
    fn parse() {
        let mut reader = book::u8_reader(book::RECORDS.to_vec());
        let _records = PdbRecords::new(&mut reader, 292).unwrap();
    }

    #[test]
    fn test_write() {
        let records = book::RECORDS.to_vec();
        let mut reader = book::u8_reader(records.clone());
        let record = PdbRecords::new(&mut reader, 292).unwrap();
        let mut written = Vec::new();
        record.write(&mut Writer::new(&mut written)).unwrap();
        assert_eq!(records.len(), written.len());
        assert_eq!(records, written);
    }
}
