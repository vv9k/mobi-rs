use super::*;
#[derive(Debug, Clone)]
/// A "cell" in the whole books content
pub struct Record {
    record_data_offset: u32,
    id: u32,
    pub record_data: String,
    pub length: usize,
}
impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.record_data)
    }
}
impl Record {
    #[allow(dead_code)]
    fn new() -> Record {
        Record {
            record_data_offset: 0,
            id: 0,
            record_data: String::new(),
            length: 0,
        }
    }
    /// Reads into a string the record data based on record_data_offset
    fn record_data(
        record_data_offset: u32,
        next_record_data_offset: u32,
        extra_bytes: u32,
        compression_type: &Compression,
        content: &[u8],
    ) -> Result<String, std::io::Error> {
        match compression_type {
            Compression::No => Ok(String::from_utf8_lossy(
                &content[record_data_offset as usize..next_record_data_offset as usize],
            )
            .to_owned()
            .to_string()),
            Compression::PalmDoc => {
                if record_data_offset < content.len() as u32 {
                    if record_data_offset < next_record_data_offset - extra_bytes {
                        lz77::decompress_lz77(
                            &content[record_data_offset as usize
                                ..(next_record_data_offset - extra_bytes) as usize],
                        )
                    } else {
                        Ok(String::from(""))
                    }
                } else {
                    Ok(String::from(""))
                }
            }
            Compression::Huff => Ok(String::from("")),
        }
    }
    /// Parses a record from the reader at current position
    fn parse_record(reader: &mut Cursor<&[u8]>) -> Result<Record, std::io::Error> {
        let record_data_offset = reader.read_u32::<BigEndian>()?;
        let id = reader.read_u32::<BigEndian>()?;
        let record = Record {
            record_data_offset,
            id,
            record_data: String::new(),
            length: 0,
        };
        Ok(record)
    }
    /// Gets all records in the specified content
    pub(crate) fn parse_records(
        content: &[u8],
        num_of_records: u16,
        _extra_bytes: u32,
        compression_type: Compression,
    ) -> Result<Vec<Record>, std::io::Error> {
        let mut records_content = vec![];
        let mut reader = Cursor::new(content);
        reader.set_position(78);
        for _i in 0..num_of_records {
            records_content.push(Record::parse_record(&mut reader)?);
        }
        for i in 0..records_content.len() {
            let mut current_rec = records_content[i].clone();
            if i != records_content.len() - 1 {
                let next_offset = records_content[i + 1].record_data_offset;
                if _extra_bytes < next_offset {
                    current_rec.record_data = Record::record_data(
                        current_rec.record_data_offset,
                        next_offset,
                        _extra_bytes,
                        &compression_type,
                        content,
                    )?;
                    current_rec.length = current_rec.record_data.len();
                }
                records_content.insert(i, current_rec);
                records_content.remove(i + 1);
            }
        }
        Ok(records_content)
    }
}
