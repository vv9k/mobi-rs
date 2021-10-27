use crate::{Reader, Writer};

use std::io;

/// Compression types available in MOBI format.
pub enum Compression {
    No,
    PalmDoc,
    Huff,
}
impl From<u16> for Compression {
    fn from(n: u16) -> Compression {
        match n {
            2 => Compression::PalmDoc,
            17480 => Compression::Huff,
            _ => Compression::No,
        }
    }
}
impl AsRef<str> for Compression {
    fn as_ref(&self) -> &str {
        match self {
            Compression::No => "No Compression",
            Compression::PalmDoc => "PalmDOC Compression",
            Compression::Huff => "HUFF/CFIC Compression",
        }
    }
}

/// Encryption types available in MOBI format.
pub enum Encryption {
    No,
    OldMobiPocket,
    MobiPocket,
}
impl From<u16> for Encryption {
    fn from(n: u16) -> Encryption {
        match n {
            2 => Encryption::MobiPocket,
            1 => Encryption::OldMobiPocket,
            _ => Encryption::No,
        }
    }
}
impl AsRef<str> for Encryption {
    fn as_ref(&self) -> &str {
        match self {
            Encryption::No => "No Encryption",
            Encryption::OldMobiPocket => "Old MobiPocket Encryption",
            Encryption::MobiPocket => "MobiPocket Encryption",
        }
    }
}

#[derive(Debug, PartialEq, Default)]
/// Strcture that holds PalmDOC header information
pub struct PalmDocHeader {
    pub compression: u16,
    pub text_length: u32,
    unused0: u16,
    pub record_count: u16,
    pub record_size: u16,
    pub encryption: u16,
    unused1: u16,
}

impl PalmDocHeader {
    /// Parse a PalmDOC header from a reader. Reader must be advanced to the starting position
    /// of the PalmDocHeader, at byte 80 + 8 * num_records.
    pub(crate) fn parse<R: io::Read>(reader: &mut Reader<R>) -> io::Result<PalmDocHeader> {
        Ok(PalmDocHeader {
            compression: reader.read_u16_be()?,
            unused0: reader.read_u16_be()?,
            text_length: reader.read_u32_be()?,
            record_count: reader.read_u16_be()?,
            record_size: reader.read_u16_be()?,
            encryption: reader.read_u16_be()?,
            unused1: reader.read_u16_be()?,
        })
    }

    pub(crate) fn write<W: io::Write>(&self, w: &mut Writer<W>) -> io::Result<()> {
        w.write_be(self.compression)?;
        w.write_be(self.unused0)?;
        w.write_be(self.text_length)?;
        w.write_be(self.record_count)?;
        w.write_be(self.record_size)?;
        w.write_be(self.encryption)?;
        w.write_be(self.unused1)
    }

    pub fn compression(&self) -> Compression {
        self.compression.into()
    }

    pub fn encryption(&self) -> Encryption {
        self.encryption.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::book;

    #[test]
    fn parse() {
        let pdheader = PalmDocHeader {
            compression: 2,
            text_length: 1151461,
            record_count: 282,
            record_size: 4096,
            encryption: 0,
            ..Default::default()
        };

        let mut reader = book::u8_reader(book::PALMDOCHEADER.to_vec());

        assert_eq!(pdheader, PalmDocHeader::parse(&mut reader).unwrap());
    }

    #[test]
    fn test_write() {
        let input_bytes = book::PALMDOCHEADER.to_vec();

        let palmdoc = PalmDocHeader::parse(&mut book::u8_reader(input_bytes.clone())).unwrap();

        let mut output_bytes = vec![];
        assert!(palmdoc.write(&mut Writer::new(&mut output_bytes)).is_ok());
        assert_eq!(input_bytes, output_bytes);
    }
}
