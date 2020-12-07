use crate::reader::Reader;
use std::io;

/// Compression types available in MOBI format.
pub(crate) enum Compression {
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
impl ToString for Compression {
    fn to_string(&self) -> String {
        match self {
            Compression::No => String::from("No Compression"),
            Compression::PalmDoc => String::from("PalmDOC Compression"),
            Compression::Huff => String::from("HUFF/CFIC Compression"),
        }
    }
}

/// Encryption types available in MOBI format.
pub(crate) enum Encryption {
    No,
    OldMobipocket,
    Mobipocket,
}
impl From<u16> for Encryption {
    fn from(n: u16) -> Encryption {
        match n {
            2 => Encryption::Mobipocket,
            1 => Encryption::OldMobipocket,
            _ => Encryption::No,
        }
    }
}
impl ToString for Encryption {
    fn to_string(&self) -> String {
        match self {
            Encryption::No => String::from("No Encryption"),
            Encryption::OldMobipocket => String::from("Old Mobipocket Encryption"),
            Encryption::Mobipocket => String::from("Mobipocket Encryption"),
        }
    }
}

#[derive(Debug, PartialEq, Default)]
/// Strcture that holds PalmDOC header information
pub struct PalmDocHeader {
    pub compression: u16,
    pub text_length: u32,
    pub record_count: u16,
    pub record_size: u16,
    pub encryption_type: u16,
}

impl PalmDocHeader {
    /// Parse a PalmDOC header from a reader. Reader must be advanced to the starting position
    /// of the PalmDocHeader, at byte 80 + 8 * num_records.
    pub(crate) fn parse<R: io::Read>(reader: &mut Reader<R>) -> io::Result<PalmDocHeader> {
        Ok(PalmDocHeader {
            compression: reader.read_u16_be()?,
            text_length: {
                reader.read_u16_be()?;
                reader.read_u32_be()?
            },
            record_count: reader.read_u16_be()?,
            record_size: reader.read_u16_be()?,
            encryption_type: {
                let b = reader.read_u16_be()?;
                reader.read_u16_be()?;
                b
            },
        })
    }

    pub(crate) fn compression(&self) -> String {
        Compression::from(self.compression).to_string()
    }

    pub(crate) fn encryption(&self) -> String {
        Encryption::from(self.encryption_type).to_string()
    }

    pub(crate) fn compression_enum(&self) -> Compression {
        Compression::from(self.compression)
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
            encryption_type: 0,
        };

        let mut reader = book::u8_reader(book::PALMDOCHEADER.to_vec());

        assert_eq!(pdheader, PalmDocHeader::parse(&mut reader).unwrap());
    }

    mod compression_type {
        use super::*;
        macro_rules! compression {
            ($et: expr, $s: expr) => {
                let mut pdheader = PalmDocHeader::default();
                pdheader.compression = $et;
                assert_eq!(pdheader.compression(), String::from($s))
            };
        }
        #[test]
        fn no_compression() {
            compression!(1, "No Compression");
        }
        #[test]
        fn palmdoc_compression() {
            compression!(2, "PalmDOC Compression");
        }
        #[test]
        fn huff_compression() {
            compression!(17480, "HUFF/CFIC Compression");
        }
    }

    mod encryption_type {
        use super::*;
        macro_rules! encryption {
            ($et: expr, $s: expr) => {
                let mut pdheader = PalmDocHeader::default();
                pdheader.encryption_type = $et;
                assert_eq!(pdheader.encryption(), String::from($s))
            };
        }
        #[test]
        fn no_encryption() {
            encryption!(0, "No Encryption");
        }
        #[test]
        fn old_mobipocket_encryption() {
            encryption!(1, "Old Mobipocket Encryption");
        }
        #[test]
        fn mobipocket_encryption() {
            encryption!(2, "Mobipocket Encryption");
        }
    }
}
