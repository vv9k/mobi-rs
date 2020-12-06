use super::{lz77, TextEncoding};
use crate::headers::palmdoch::Compression;
use encoding::{all::WINDOWS_1252, DecoderTrap, Encoding};
use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone)]
/// A wrapper error type for unified error across multiple encodings.
pub enum DecodeError {
    UTF8(String),
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

#[derive(Debug, Clone)]
/// A "cell" in the whole books content
pub struct Record {
    record_data_offset: u32,
    id: u32,
    pub record_data: Vec<u8>,
    pub length: usize,
}
impl Record {
    #[allow(dead_code)]
    fn new() -> Record {
        Record {
            record_data_offset: 0,
            id: 0,
            record_data: Vec::new(),
            length: 0,
        }
    }

    /// Reads the content of a record at specified offset
    fn record_data(
        record_data_offset: u32,
        next_record_data_offset: u32,
        extra_bytes: u32,
        compression_type: &Compression,
        content: &[u8],
    ) -> io::Result<Vec<u8>> {
        // #TODO: reconsider using string here due to possible different encodings?
        match compression_type {
            Compression::No => Ok(content[record_data_offset as usize..next_record_data_offset as usize].to_vec()),
            Compression::PalmDoc => {
                if record_data_offset < content.len() as u32
                    && record_data_offset < next_record_data_offset - extra_bytes
                {
                    Ok(lz77::decompress_lz77(
                        &content[record_data_offset as usize..(next_record_data_offset - extra_bytes) as usize],
                    ))
                } else {
                    Err(io::Error::new(
                        ErrorKind::NotFound,
                        "record points to location out of bounds",
                    ))
                }
            }
            Compression::Huff => panic!("Huff compression is currently not supported"),
        }
    }

    /// Gets all records in the specified content
    pub(crate) fn parse_records(
        content: &[u8],
        record_info: &[(u32, u32)],
        _extra_bytes: u32,
        compression_type: Compression,
    ) -> io::Result<Vec<Record>> {
        let mut new_records = vec![];
        for records in record_info.windows(2) {
            let (curr_offset, id) = records[0];
            let (next_offset, _) = records[1];
            let record_data = if _extra_bytes < next_offset {
                match Record::record_data(curr_offset, next_offset, _extra_bytes, &compression_type, content) {
                    Ok(record) => record,
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            };

            let record = Record {
                record_data_offset: curr_offset,
                id,
                length: record_data.len(),
                record_data,
            };

            new_records.push(record);
        }

        if let Some(&(record_data_offset, id)) = record_info.last() {
            new_records.push(Record {
                record_data_offset,
                id,
                record_data: vec![],
                length: 0,
            });
        }

        Ok(new_records)
    }

    pub(crate) fn to_string_lossy(&self, encoding: TextEncoding) -> String {
        match encoding {
            TextEncoding::UTF8 => String::from_utf8_lossy(&self.record_data).to_owned().to_string(),
            TextEncoding::CP1252 => WINDOWS_1252.decode(&self.record_data, DecoderTrap::Ignore).unwrap(),
        }
    }

    pub(crate) fn to_string(&self, encoding: TextEncoding) -> Result<String, DecodeError> {
        match encoding {
            TextEncoding::UTF8 => {
                String::from_utf8(self.record_data.clone()).map_err(|e| DecodeError::UTF8(e.to_string()))
            }
            TextEncoding::CP1252 => WINDOWS_1252
                .decode(&self.record_data, DecoderTrap::Strict)
                .map_err(DecodeError::CP1252),
        }
    }
}
