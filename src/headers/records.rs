use crate::{Reader, Writer};
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
    extra_bytes: u16,
}

impl Records {
    /// Parse the records from a reader. Reader must be advanced to the starting position
    /// of the records, at byte 78.
    pub(crate) fn parse<R: io::Read>(reader: &mut Reader<R>, num_records: u16) -> io::Result<Records> {
        let mut records = Vec::with_capacity(num_records as usize);

        for _ in 0..num_records {
            records.push((reader.read_u32_be()?, reader.read_u32_be()?));
        }

        Ok(Records {
            records,
            extra_bytes: reader.read_u16_be()?,
        })
    }

    pub fn extra_bytes(&self) -> u32 {
        2 * (self.extra_bytes & EXTRA_BYTES_FLAG).count_ones() as u32
    }

    pub fn num_records(&self) -> u16 {
        self.records.len() as u16
    }

    #[allow(dead_code)]
    pub(crate) fn write<W: io::Write>(&self, w: &mut Writer<W>) -> io::Result<()> {
        for &record in &self.records {
            w.write_be(record.0)?;
            w.write_be(record.1)?;
        }
        w.write_be(self.extra_bytes)
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

    #[test]
    fn test_write() {
        let records = book::RECORDS.to_vec();
        let mut reader = book::u8_reader(records.clone());
        let record = Records::parse(&mut reader, 292).unwrap();
        let mut written = Vec::new();
        record.write(&mut Writer::new(&mut written)).unwrap();
        assert_eq!(records.len(), written.len());
        assert_eq!(records, written);
    }
}
