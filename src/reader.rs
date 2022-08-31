use std::io::{self, Read};

#[derive(Debug, Default, Clone)]
/// Helper struct for reading header values from content.
/// Only allows forward reads.
pub(crate) struct Reader<R> {
    reader: R,
    /// Invariant: position will be no larger than the number of bytes
    /// produced by the reader
    position: usize,
}

impl<R: Read> Reader<R> {
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
        if p >= self.position {
            let bytes_to_copy = (p - self.position) as u64;
            let copied_bytes = std::io::copy(
                &mut self.reader.by_ref().take(bytes_to_copy),
                &mut io::sink(),
            )?;

            if copied_bytes != bytes_to_copy {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Tried to set cursor position past EOF",
                ))
            } else {
                self.position = p;
                Ok(())
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "name seeked backwards",
            ))
        }
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

    /// Reads a header as u8 bytes. Designed to avoid OOM if len exceeds the length
    /// the underlying file.
    pub(crate) fn read_vec_header(&mut self, len: usize) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        let r = self.reader.by_ref();
        r.take(len as u64).read_to_end(&mut buf)?;
        if buf.len() != len {
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "Tried to read {} byte header, only {} bytes available",
                    len,
                    buf.len()
                )
                .as_str(),
            ))
        } else {
            self.position += buf.len();
            Ok(buf)
        }
    }
}
