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
    fn parse_record(reader: &mut Cursor<&[u8]>) -> io::Result<Record> {
        Ok(Record {
            record_data_offset: reader.read_u32::<BigEndian>()?,
            id: reader.read_u32::<BigEndian>()?,
            record_data: Vec::new(),
            length: 0,
        })
    }

    /// Gets all records in the specified content
    pub(crate) fn parse_records(
        content: &[u8],
        num_of_records: u16,
        _extra_bytes: u32,
        compression_type: Compression,
    ) -> io::Result<Vec<Record>> {
        let mut records_content = vec![];
        let mut reader = Cursor::new(content);
        reader.set_position(RECORDS_START_INDEX);
        for _i in 0..num_of_records {
            records_content.push(Record::parse_record(&mut reader)?);
        }
        for i in 0..records_content.len() {
            let mut current_rec = records_content[i].clone();
            if i != records_content.len() - 1 {
                let next_offset = records_content[i + 1].record_data_offset;
                if _extra_bytes < next_offset {
                    current_rec.record_data = match Record::record_data(
                        current_rec.record_data_offset,
                        next_offset,
                        _extra_bytes,
                        &compression_type,
                        content,
                    ) {
                        Ok(record) => record,
                        Err(_) => Vec::new(),
                    };

                    current_rec.length = current_rec.record_data.len();
                }
                records_content.insert(i, current_rec);
                records_content.remove(i + 1);
            }
        }
        Ok(records_content)
    }

    pub(crate) fn to_string(&self, encoding: TextEncoding) -> String {
        match encoding {
            TextEncoding::UTF8 => String::from_utf8_lossy(&self.record_data).to_owned().to_string(),
            TextEncoding::CP1252 => WINDOWS_1252.decode(&self.record_data, DecoderTrap::Ignore).unwrap(),
        }
    }
}
