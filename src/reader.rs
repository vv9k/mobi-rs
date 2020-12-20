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

pub(crate) trait WriteBeBytes {
    fn write_be_bytes<W: io::Write>(&self, writer: &mut W) -> io::Result<usize>;
}

impl WriteBeBytes for &[u8] {
    fn write_be_bytes<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        writer.write_all(self)?;
        Ok(self.len())
    }
}

impl WriteBeBytes for Vec<u8> {
    fn write_be_bytes<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        writer.write_all(self)?;
        Ok(self.len())
    }
}

impl WriteBeBytes for &Vec<u8> {
    fn write_be_bytes<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        writer.write_all(self)?;
        Ok(self.len())
    }
}

impl WriteBeBytes for u8 {
    fn write_be_bytes<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        writer.write_all(&self.to_be_bytes())?;
        Ok(self.to_be_bytes().len())
    }
}

impl WriteBeBytes for u16 {
    fn write_be_bytes<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        writer.write_all(&self.to_be_bytes())?;
        Ok(self.to_be_bytes().len())
    }
}

impl WriteBeBytes for u32 {
    fn write_be_bytes<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        writer.write_all(&self.to_be_bytes())?;
        Ok(self.to_be_bytes().len())
    }
}

#[derive(Debug, Default, Clone)]
/// Helper struct for reading header values from content.
/// Only allows forward reads.
pub(crate) struct Writer<W> {
    writer: W,
    bytes_written: usize,
}

impl<W: std::io::Write> Writer<W> {
    pub(crate) fn new(writer: W) -> Writer<W> {
        Writer {
            writer,
            bytes_written: 0,
        }
    }

    pub(crate) fn bytes_written(&self) -> usize {
        self.bytes_written
    }

    #[inline]
    pub(crate) fn write_be<B: WriteBeBytes>(&mut self, b: B) -> io::Result<()> {
        self.bytes_written += b.write_be_bytes(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    pub(crate) fn write_string_be<S: AsRef<str>>(&mut self, s: S, pad: usize) -> io::Result<()> {
        let mut s_bytes: Vec<_> = s.as_ref().bytes().collect();
        s_bytes.extend(std::iter::repeat(0).take(pad.saturating_sub(s_bytes.len())));
        self.write_be(s_bytes)
    }
}
