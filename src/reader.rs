use super::HeaderField;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{self, Cursor};

/// Helper struct for reading header values from content
pub(crate) struct Reader<'r> {
    pub cursor: Cursor<&'r [u8]>,
    pub num_of_records: u16,
}
impl<'r> Reader<'r> {
    pub(crate) fn new(content: &'r [u8]) -> Reader<'r> {
        Reader {
            cursor: Cursor::new(content),
            num_of_records: 0,
        }
    }
    pub(crate) fn content(&self) -> Vec<u8> {
        self.cursor.clone().into_inner().to_vec()
    }
    pub(crate) fn content_ref(&self) -> &[u8] {
        self.cursor.clone().into_inner()
    }
    #[inline]
    pub(crate) fn set_num_of_records(&mut self, n: u16) {
        self.num_of_records = n;
    }
    #[inline]
    pub(crate) fn set_position(&mut self, n: u64) {
        self.cursor.set_position(n);
    }
    #[inline]
    pub(crate) fn read_u32_be(&mut self) -> io::Result<u32> {
        self.cursor.read_u32::<BigEndian>()
    }
    #[inline]
    pub(crate) fn read_u16_be(&mut self) -> io::Result<u16> {
        self.cursor.read_u16::<BigEndian>()
    }
    #[inline]
    pub(crate) fn read_i16_be(&mut self) -> io::Result<i16> {
        self.cursor.read_i16::<BigEndian>()
    }
    #[inline]
    pub(crate) fn read_u8(&mut self) -> io::Result<u8> {
        self.cursor.read_u8()
    }

    #[inline]
    pub(crate) fn read_i16_header<F: HeaderField>(&mut self, field: F) -> io::Result<i16> {
        self.set_position(field.position() as u64 + u64::from(self.num_of_records * 8));
        self.read_i16_be()
    }
    #[inline]
    pub(crate) fn read_u16_header<F: HeaderField>(&mut self, field: F) -> io::Result<u16> {
        self.set_position(field.position() as u64 + u64::from(self.num_of_records * 8));
        self.read_u16_be()
    }
    #[inline]
    pub(crate) fn read_u32_header<F: HeaderField>(&mut self, field: F) -> io::Result<u32> {
        self.set_position(field.position() as u64 + u64::from(self.num_of_records * 8));
        self.read_u32_be()
    }
    pub(crate) fn read_string_header<F: HeaderField>(&mut self, field: F, len: u64) -> String {
        let position = field.position();
        String::from_utf8_lossy(&self.cursor.get_ref()[position as usize..(position as u64 + len) as usize])
            .to_owned()
            .to_string()
    }
}
