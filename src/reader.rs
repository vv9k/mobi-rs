use std::io::{self, Read};

#[derive(Debug, Default, Clone)]
/// Helper struct for reading header values from content.
/// Only allows forward reads.
pub(crate) struct Reader<R> {
    reader: R,
    position: usize,
}

impl<R: std::io::Read> Reader<R> {
    pub(crate) fn new(content: R) -> Reader<R> {
        Reader {
            reader: content,
            position: 0,
        }
    }

    pub(crate) fn read_to_end(&mut self) -> io::Result<Vec<u8>> {
        // Zero-fill so that Records parsing works as expected.
        let mut first_buf = vec![0; self.position];
        // read_to_end appends to the end of the buffer.
        self.reader.read_to_end(&mut first_buf)?;
        self.position = first_buf.len();
        Ok(first_buf)
    }

    pub(crate) fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.reader.read_exact(buf)?;
        self.position += buf.len();
        Ok(())
    }

    #[inline]
    pub(crate) fn set_position(&mut self, p: usize) -> io::Result<()> {
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
    pub(crate) fn read_u64_be(&mut self) -> io::Result<u64> {
        let mut bytes = [0; 8];
        self.read_exact(&mut bytes)?;
        Ok(u64::from_be_bytes(bytes))
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

    pub(crate) fn read_vec_header(&mut self, len: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; len];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
}
