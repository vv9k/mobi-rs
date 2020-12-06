use crate::reader::MobiReader;
use std::io;

const RECORDS_START_INDEX: u64 = 78;
const EXTRA_BYTES_FLAG: u16 = 0xFFFE;

// #[derive(Debug, PartialEq, Default)]
// pub struct RecordMetadata {
//     offset: u32,
//     id: u32,
// }

#[derive(Debug, PartialEq, Default)]
pub struct Records {
    pub records: Vec<(u32, u32)>,
    pub extra_bytes: u32,
}

impl Records {
    pub(crate) fn parse(reader: &mut impl MobiReader) -> io::Result<Records> {
        let mut records = Vec::with_capacity(reader.get_num_records() as usize);

        reader.set_position(RECORDS_START_INDEX);
        for _ in 0..reader.get_num_records() {
            records.push((reader.read_u32_be()?, reader.read_u32_be()?));
        }
        let extra_bytes = reader.read_u16_be()?;

        Ok(Records {
            records,
            extra_bytes: 2 * (extra_bytes & EXTRA_BYTES_FLAG).count_ones(),
        })
    }
}
