use std::io::{self, Read};

#[derive(Debug, Default, Clone)]
/// Helper struct for reading header values from content
pub(crate) struct Reader<R> {
    pub reader: R,
    pub num_records: u16,
    position: usize,
}

impl<R: std::io::Read> Reader<R> {
    pub(crate) fn new(content: R) -> Reader<R> {
        Reader {
            reader: content,
            num_records: 0,
            position: 0,
        }
    }

    pub(crate) fn read_to_end(&mut self) -> io::Result<Vec<u8>> {
        let mut first_buf = vec![0; self.position];
        let mut second_buf = vec![];
        self.reader.read_to_end(&mut second_buf)?;
        first_buf.extend_from_slice(&second_buf);
        Ok(first_buf)
    }

    pub(crate) fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.reader.read_exact(buf)?;
        self.position += buf.len();
        Ok(())
    }

    pub(crate) fn get_num_records(&self) -> u16 {
        self.num_records
    }

    #[inline]
    pub(crate) fn set_num_records(&mut self, n: u16) {
        self.num_records = n;
    }

    pub(crate) fn position_after_records(&self) -> u64 {
        self.num_records as u64 * 8
    }

    pub(crate) fn get_position(&self) -> u64 {
        self.position as u64
    }

    #[inline]
    pub(crate) fn set_position(&mut self, n: u64) -> io::Result<()> {
        let p = n as usize;
        debug_assert!(p >= self.position, "{}, {}", p, self.position);

        if p >= self.position {
            std::io::copy(
                &mut self.reader.by_ref().take((p - self.position) as u64),
                &mut io::sink(),
            )?;
            self.position = p;
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn read_u32_be(&mut self) -> io::Result<u32> {
        let mut bytes = [0; 4];
        self.read_exact(&mut bytes)?;
        Ok(u32::from_be_bytes(bytes))
    }

    #[inline]
    pub(crate) fn read_u16_be(&mut self) -> io::Result<u16> {
        let mut bytes = [0; 2];
        self.read_exact(&mut bytes)?;
        Ok(u16::from_be_bytes(bytes))
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn read_u8(&mut self) -> io::Result<u8> {
        let mut bytes = [0; 1];
        self.read_exact(&mut bytes)?;
        Ok(u8::from_be_bytes(bytes))
    }

    pub(crate) fn read_string_header(&mut self, len: usize) -> io::Result<String> {
        let mut buf = vec![0; len];
        self.read_exact(&mut buf)?;

        Ok(String::from_utf8_lossy(&buf).to_owned().to_string())
    }
}
