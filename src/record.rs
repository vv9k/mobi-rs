use super::{lz77, TextEncoding};
use crate::headers::palmdoch::Compression;
use byteorder::{BigEndian, ReadBytesExt};
use encoding::{all::WINDOWS_1252, DecoderTrap, Encoding};
use std::io::{self, Cursor, ErrorKind};

const RECORDS_START_INDEX: u64 = 78;

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

    /// Parses a record from the reader at current position
    fn parse_record_info(reader: &mut Cursor<&[u8]>) -> io::Result<(u32, u32)> {
        Ok((reader.read_u32::<BigEndian>()?, reader.read_u32::<BigEndian>()?))
    }

    /// Gets all records in the specified content
    pub(crate) fn parse_records(
        content: &[u8],
        num_records: u16,
        _extra_bytes: u32,
        compression_type: Compression,
    ) -> io::Result<Vec<Record>> {
        let mut reader = Cursor::new(content);
        reader.set_position(RECORDS_START_INDEX);

        let mut record_info = vec![];

        for _i in 0..num_records {
            record_info.push(Record::parse_record_info(&mut reader)?);
        }

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

    pub(crate) fn to_string(&self, encoding: TextEncoding) -> String {
        match encoding {
            TextEncoding::UTF8 => String::from_utf8_lossy(&self.record_data).to_owned().to_string(),
            TextEncoding::CP1252 => WINDOWS_1252.decode(&self.record_data, DecoderTrap::Ignore).unwrap(),
        }
    }
}
