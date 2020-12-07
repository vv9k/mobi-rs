use crate::reader::Reader;
use std::io;

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
    /// Parse the records from a reader. Reader must be advanced to the starting position
    /// of the records, at byte 78.
    pub(crate) fn parse<R: io::Read>(reader: &mut Reader<R>, num_records: u16) -> io::Result<Records> {
        let mut records = Vec::with_capacity(num_records as usize);

        for _ in 0..num_records {
            records.push((reader.read_u32_be()?, reader.read_u32_be()?));
        }

        let extra_bytes = reader.read_u16_be()?;

        Ok(Records {
            records,
            extra_bytes: 2 * (extra_bytes & EXTRA_BYTES_FLAG).count_ones(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::book;

    #[test]
    fn parse() {
        let mut reader = book::u8_reader(book::RECORDS.to_vec());
        assert!(Records::parse(&mut reader, 292).is_ok());
    }
}
