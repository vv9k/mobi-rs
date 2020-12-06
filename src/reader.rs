use crate::headers::HeaderField;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{self, Cursor};

#[derive(Debug, Default)]
/// Helper struct for reading header values from content
pub(crate) struct Reader<'r> {
    pub cursor: Cursor<&'r [u8]>,
    pub num_of_records: u16,
}

pub(crate) trait MobiReader {
    fn content(&mut self) -> Vec<u8>;

    fn set_num_of_records(&mut self, n: u16);

    fn get_num_records(&self) -> u16;

    fn set_position(&mut self, n: u64);

    fn read_u32_be(&mut self) -> io::Result<u32>;

    fn read_u16_be(&mut self) -> io::Result<u16>;

    fn read_i16_be(&mut self) -> io::Result<i16>;

    fn read_u8(&mut self) -> io::Result<u8>;

    fn position_after_records(&self) -> u64;

    fn read_i16_header<F: HeaderField>(&mut self, field: F) -> io::Result<i16>;

    fn read_u16_header<F: HeaderField>(&mut self, field: F) -> io::Result<u16>;

    fn read_u32_header<F: HeaderField>(&mut self, field: F) -> io::Result<u32>;

    fn read_u32_header_offset(&mut self, offset: u64) -> io::Result<u32>;

    fn read_string_header<F: HeaderField>(&mut self, field: F, len: u64) -> String;

    fn read_range(&mut self, start: u64, end: u64) -> String;
}

impl <'r> Reader<'r> {
    pub(crate) fn new(content: &'r [u8]) -> Reader<'r> {
        Reader {
            cursor: Cursor::new(content),
            num_of_records: 0,
        }
    }
}

impl<'r> MobiReader for Reader<'r> {
    fn content(&mut self) -> Vec<u8> {
        self.cursor.clone().into_inner().to_vec()
    }

    fn get_num_records(&self) -> u16 {
        self.num_of_records
    }

    #[inline]
    fn set_num_of_records(&mut self, n: u16) {
        self.num_of_records = n;
    }

    #[inline]
    fn set_position(&mut self, n: u64) {
        self.cursor.set_position(n);
    }

    #[inline]
    fn read_u32_be(&mut self) -> io::Result<u32> {
        self.cursor.read_u32::<BigEndian>()
    }

    #[inline]
    fn read_u16_be(&mut self) -> io::Result<u16> {
        self.cursor.read_u16::<BigEndian>()
    }

    #[inline]
    fn read_i16_be(&mut self) -> io::Result<i16> {
        self.cursor.read_i16::<BigEndian>()
    }

    #[inline]
    fn read_u8(&mut self) -> io::Result<u8> {
        self.cursor.read_u8()
    }

    fn position_after_records(&self) -> u64 {
        self.num_of_records as u64 * 8
    }

    #[inline]
    fn read_i16_header<F: HeaderField>(&mut self, field: F) -> io::Result<i16> {
        self.set_position(self.position_after_records() + field.position());
        self.read_i16_be()
    }

    #[inline]
    fn read_u16_header<F: HeaderField>(&mut self, field: F) -> io::Result<u16> {
        self.set_position(self.position_after_records() + field.position());
        self.read_u16_be()
    }

    #[inline]
    fn read_u32_header<F: HeaderField>(&mut self, field: F) -> io::Result<u32> {
        self.set_position(self.position_after_records() + field.position());
        self.read_u32_be()
    }

    #[inline]
    fn read_u32_header_offset(&mut self, offset: u64) -> io::Result<u32> {
        self.set_position(self.position_after_records() + offset);
        self.read_u32_be()
    }

    fn read_string_header<F: HeaderField>(&mut self, field: F, len: u64) -> String {
        let start = field.position();
        let end = start + len;

        self.read_range(start, end)
    }

    fn read_range(&mut self, start: u64, end: u64) -> String {
        String::from_utf8_lossy(&self.cursor.get_ref()[start as usize..end as usize])
            .to_owned()
            .to_string()
    }

}

#[derive(Debug, Default)]
/// Helper struct for reading header values from content
pub(crate) struct ReaderPrime<R: std::io::Read> {
    pub reader: R,
    pub num_of_records: u16,
    buf: Vec<u8>,
    position: usize,
}

impl<R: std::io::Read> ReaderPrime<R> {
    pub(crate) fn new(content: R) -> ReaderPrime<R> {
        ReaderPrime {
            reader: content,
            num_of_records: 0,
            buf: Vec::with_capacity(2 << 11),
            position: 0,
        }
    }

    // Will read from ?..p, so p itself will not be read, but p - 1 will exist.
    fn read_to_point(&mut self, p: usize) -> io::Result<()> {
        if p > self.buf.len() {
            let mut temp = vec![0; p - self.buf.len()];
            self.reader.read_exact(&mut temp)?;
            self.buf.extend(temp);
        }

        Ok(())
    }
}

impl<R: std::io::Read> MobiReader for ReaderPrime<R> {
    fn content(&mut self) -> Vec<u8> {
        self.reader.read_to_end(&mut self.buf);
        self.buf.clone()
    }

    fn get_num_records(&self) -> u16 {
        self.num_of_records
    }

    #[inline]
    fn set_num_of_records(&mut self, n: u16) {
        self.num_of_records = n;
    }

    #[inline]
    fn set_position(&mut self, n: u64) {
        self.position = n as usize;
    }

    #[inline]
    fn read_u32_be(&mut self) -> io::Result<u32> {
        let p = self.position;
        self.read_to_point(p + 4);
        let mut bytes = [0; 4];
        bytes.clone_from_slice(&self.buf[p..p+4]);
        self.position += 4;
        Ok(u32::from_be_bytes(bytes))
    }

    #[inline]
    fn read_u16_be(&mut self) -> io::Result<u16> {
        let p = self.position;
        self.read_to_point(p + 2);
        let mut bytes = [0; 2];
        bytes.clone_from_slice(&self.buf[p..p+2]);
        self.position += 2;
        Ok(u16::from_be_bytes(bytes))
    }

    #[inline]
    fn read_i16_be(&mut self) -> io::Result<i16> {
        let p = self.position;
        self.read_to_point(p + 2);
        let mut bytes = [0; 2];
        bytes.clone_from_slice(&self.buf[p..p+2]);
        self.position += 2;
        Ok(i16::from_be_bytes(bytes))
    }

    #[inline]
    fn read_u8(&mut self) -> io::Result<u8> {
        let p = self.position;
        self.read_to_point(p+1);
        self.position += 1;
        Ok(self.buf[p])
    }

    fn position_after_records(&self) -> u64 {
        self.num_of_records as u64 * 8
    }

    #[inline]
    fn read_i16_header<F: HeaderField>(&mut self, field: F) -> io::Result<i16> {
        self.set_position(self.position_after_records() + field.position());
        self.read_i16_be()
    }

    #[inline]
    fn read_u16_header<F: HeaderField>(&mut self, field: F) -> io::Result<u16> {
        self.set_position(self.position_after_records() + field.position());
        self.read_u16_be()
    }

    #[inline]
    fn read_u32_header<F: HeaderField>(&mut self, field: F) -> io::Result<u32> {
        self.set_position(self.position_after_records() + field.position());
        self.read_u32_be()
    }

    #[inline]
    fn read_u32_header_offset(&mut self, offset: u64) -> io::Result<u32> {
        self.set_position(self.position_after_records() + offset);
        self.read_u32_be()
    }

    fn read_string_header<F: HeaderField>(&mut self, field: F, len: u64) -> String {
        let start = field.position();
        let end = start + len;

        self.read_range(start, end)
    }

    fn read_range(&mut self, start: u64, end: u64) -> String {
        self.read_to_point(end as usize);

        String::from_utf8_lossy(&self.buf[start as usize..end as usize])
            .to_owned()
            .to_string()
    }
}
