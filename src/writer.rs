use std::io;

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

impl WriteBeBytes for u64 {
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
}
