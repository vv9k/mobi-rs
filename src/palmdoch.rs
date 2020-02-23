use super::*;
pub(crate) enum Compression {
    No,
    PalmDoc,
    Huff,
}
/// Parameters of PalmDOC Header
pub(crate) enum PalmDocHeaderData {
    Compression,
    TextLength,
    RecordCount,
    RecordSize,
    EncryptionType,
}
impl FieldHeaderEnum for PalmDocHeaderData {}
impl HeaderField<PalmDocHeaderData> for PalmDocHeaderData {
    fn position(self) -> Option<u16> {
        match self {
            PalmDocHeaderData::Compression => Some(80),
            PalmDocHeaderData::RecordCount => Some(88),
            PalmDocHeaderData::RecordSize => Some(90),
            PalmDocHeaderData::EncryptionType => Some(92),
            PalmDocHeaderData::TextLength => Some(84),
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
#[cfg(feature = "fmt")]
impl fmt::Display for PalmDocHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PALMDOC HEADER
Compression:            {}
Text length:            {}
Record count:           {}
Record size:            {}
Encryption type:        {}",
            self.compression().unwrap_or_default(),
            self.text_length,
            self.record_count,
            self.record_size,
            self.encryption().unwrap_or_default(),
        )
    }
}
impl PalmDocHeader {
    /// Parse a PalmDOC header from the content
    pub(crate) fn parse(
        content: &[u8],
        num_of_records: u16,
    ) -> Result<PalmDocHeader, std::io::Error> {
        let mut reader = Reader::new(&content, num_of_records);
        use PalmDocHeaderData::*;
        Ok(PalmDocHeader {
            compression: reader.read_u16_header(Compression)?,
            text_length: reader.read_u32_header(TextLength)?,
            record_count: reader.read_u16_header(RecordCount)?,
            record_size: reader.read_u16_header(RecordSize)?,
            encryption_type: reader.read_u16_header(EncryptionType)?,
        })
    }
    pub(crate) fn compression(&self) -> Option<String> {
        match self.compression {
            1 => Some(String::from("No Compression")),
            2 => Some(String::from("PalmDOC Compression")),
            17480 => Some(String::from("HUFF/CFIC Compression")),
            _ => None,
        }
    }
    pub(crate) fn encryption(&self) -> Option<String> {
        match self.encryption_type {
            0 => Some(String::from("No Encryption")),
            1 => Some(String::from("Old Mobipocket Encryption")),
            2 => Some(String::from("Mobipocket Encryption")),
            _ => None,
        }
    }
    pub(crate) fn compression_en(&self) -> Compression {
        match self.compression {
            2 => Compression::PalmDoc,
            17480 => Compression::Huff,
            _ => Compression::No,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use book::BOOK;
    use header::HeaderData;
    #[test]
    fn parse() {
        let pdheader = PalmDocHeader {
            compression: 2,
            text_length: 1151461,
            record_count: 282,
            record_size: 4096,
            encryption_type: 0,
        };
        let mut reader = Reader::new(&BOOK, 0);
        let parsed_header = PalmDocHeader::parse(
            BOOK,
            reader.read_u16_header(HeaderData::NumOfRecords).unwrap(),
        )
        .unwrap();
        assert_eq!(pdheader, parsed_header);
    }
    mod compression_type {
        use super::*;
        macro_rules! compression {
            ($et: expr, $s: expr) => {
                let mut pdheader = PalmDocHeader::default();
                pdheader.compression = $et;
                assert_eq!(pdheader.compression(), Some(String::from($s)))
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
                assert_eq!(pdheader.encryption(), Some(String::from($s)))
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
