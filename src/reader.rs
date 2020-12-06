use crate::headers::HeaderField;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{self, Cursor, Read};

#[derive(Debug, Default)]
/// Helper struct for reading header values from content
pub(crate) struct Reader<'r> {
    pub cursor: Cursor<&'r [u8]>,
    pub num_records: u16,
}

pub(crate) trait MobiReader {
    fn read_to_end(&mut self) -> io::Result<Vec<u8>>;

    fn set_num_records(&mut self, n: u16);

    fn get_num_records(&self) -> u16;

    fn set_position(&mut self, n: u64);
    fn get_position(&mut self) -> u64;

    fn read_u32_be(&mut self) -> io::Result<u32>;

    fn read_u16_be(&mut self) -> io::Result<u16>;

    fn read_i16_be(&mut self) -> io::Result<i16>;

    fn read_u8(&mut self) -> io::Result<u8>;

    fn position_after_records(&self) -> u64;

    fn read_i16_header<F: HeaderField>(&mut self, field: F) -> io::Result<i16>;

    fn read_u16_header<F: HeaderField>(&mut self, field: F) -> io::Result<u16>;

    fn read_u32_header<F: HeaderField>(&mut self, field: F) -> io::Result<u32>;

    fn read_u32_header_offset(&mut self, offset: u64) -> io::Result<u32>;

    fn read_string_header<F: HeaderField>(&mut self, field: F, len: u64) -> io::Result<String>;

    fn read_range(&mut self, start: u64, end: u64) -> io::Result<String>;
}

impl<'r> Reader<'r> {
    pub(crate) fn new(content: &'r [u8]) -> Reader<'r> {
        Reader {
            cursor: Cursor::new(content),
            num_records: 0,
        }
    }
}

impl<'r> MobiReader for Reader<'r> {
    fn read_to_end(&mut self) -> io::Result<Vec<u8>> {
        let mut first_buf = vec![0; self.cursor.position() as usize];
        let mut second_buf = vec![];
        self.cursor.read_to_end(&mut second_buf)?;
        first_buf.extend_from_slice(&second_buf);
        Ok(first_buf)
    }

    fn get_num_records(&self) -> u16 {
        self.num_records
    }

    #[inline]
    fn set_num_records(&mut self, n: u16) {
        self.num_records = n;
    }

    #[inline]
    fn set_position(&mut self, n: u64) {
        debug_assert!(n >= self.cursor.position(), "{}, {}", n, self.cursor.position());
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
        self.num_records as u64 * 8
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

    fn read_string_header<F: HeaderField>(&mut self, field: F, len: u64) -> io::Result<String> {
        let start = field.position();
        let end = start + len;

        self.read_range(start, end)
    }

    fn read_range(&mut self, start: u64, end: u64) -> io::Result<String> {
        Ok(
            String::from_utf8_lossy(&self.cursor.get_ref()[start as usize..end as usize])
                .to_owned()
                .to_string(),
        )
    }

    fn get_position(&mut self) -> u64 {
        self.cursor.position()
    }
}

#[derive(Debug, Default)]
/// Helper struct for reading header values from content
pub(crate) struct ReaderPrime<R> {
    pub reader: R,
    pub num_records: u16,
    position: usize,
}

impl<R: std::io::Read> ReaderPrime<R> {
    pub(crate) fn new(content: R) -> ReaderPrime<R> {
        ReaderPrime {
            reader: content,
            num_records: 0,
            position: 0,
        }
    }

    // Will read from ?..p, so p itself will not be read, but p - 1 will exist.
    fn read_to_point(&mut self, p: usize) -> io::Result<()> {
        debug_assert!(p >= self.position, "{}, {}", p, self.position);

        if p > self.position {
            std::io::copy(
                &mut self.reader.by_ref().take((p - self.position) as u64),
                &mut io::sink(),
            )?;
            self.position = p;
        }

        Ok(())
    }
}

impl<R: std::io::Read> MobiReader for ReaderPrime<R> {
    fn get_position(&mut self) -> u64 {
        self.position as u64
    }

    fn read_to_end(&mut self) -> io::Result<Vec<u8>> {
        let mut first_buf = vec![0; self.position];
        let mut second_buf = vec![];
        self.reader.read_to_end(&mut second_buf)?;
        first_buf.extend_from_slice(&second_buf);
        Ok(first_buf)
    }

    fn get_num_records(&self) -> u16 {
        self.num_records
    }

    #[inline]
    fn set_num_records(&mut self, n: u16) {
        self.num_records = n;
    }

    #[inline]
    fn set_position(&mut self, n: u64) {
        self.read_to_point(n as usize).unwrap();
    }

    #[inline]
    fn read_u32_be(&mut self) -> io::Result<u32> {
        let mut bytes = [0; 4];
        self.reader.read_exact(&mut bytes)?;
        self.position += 4;
        Ok(u32::from_be_bytes(bytes))
    }

    #[inline]
    fn read_u16_be(&mut self) -> io::Result<u16> {
        let mut bytes = [0; 2];
        self.reader.read_exact(&mut bytes)?;
        self.position += 2;
        Ok(u16::from_be_bytes(bytes))
    }

    #[inline]
    fn read_i16_be(&mut self) -> io::Result<i16> {
        let mut bytes = [0; 2];
        self.reader.read_exact(&mut bytes)?;
        self.position += 2;
        Ok(i16::from_be_bytes(bytes))
    }

    #[inline]
    fn read_u8(&mut self) -> io::Result<u8> {
        self.position += 1;
        self.reader.read_u8()
    }

    fn position_after_records(&self) -> u64 {
        self.num_records as u64 * 8
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

    fn read_string_header<F: HeaderField>(&mut self, field: F, len: u64) -> io::Result<String> {
        let start = field.position();
        let end = start + len;

        self.read_range(start, end)
    }

    fn read_range(&mut self, start: u64, end: u64) -> io::Result<String> {
        self.read_to_point(start as usize)?;
        let len = (end - start) as usize;
        let mut buf = vec![0; len];
        self.position += len;

        self.reader.read_exact(&mut buf)?;
        let s = String::from_utf8_lossy(&buf).to_owned().to_string();
        println!("{} : ({} {:?})", s, len, buf);
        Ok(s)
    }
}
