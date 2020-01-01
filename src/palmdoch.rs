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

#[derive(Debug, PartialEq, Default)]
/// Strcture that holds PalmDOC header information
pub struct PalmDocHeader {
    pub compression: u16,
    pub text_length: u32,
    pub record_count: u16,
    pub record_size: u16,
    pub encryption_type: u16,
}
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
        macro_rules! pdheader {
            ($method:ident($type:ident)) => {
                PalmDocHeader::$method(content, PalmDocHeaderData::$type, num_of_records)?
            };
        }
        Ok(PalmDocHeader {
            compression: pdheader!(get_headers_u16(Compression)),
            text_length: pdheader!(get_headers_u32(TextLength)),
            record_count: pdheader!(get_headers_u16(RecordCount)),
            record_size: pdheader!(get_headers_u16(RecordSize)),
            encryption_type: pdheader!(get_headers_u16(EncryptionType)),
        })
    }
    /// Gets u16 header value from specific location
    fn get_headers_u16(
        content: &[u8],
        pdheader: PalmDocHeaderData,
        num_of_records: u16,
    ) -> Result<u16, std::io::Error> {
        let mut reader = Cursor::new(content);
        let position = match pdheader {
            PalmDocHeaderData::Compression => 80,
            PalmDocHeaderData::RecordCount => 88,
            PalmDocHeaderData::RecordSize => 90,
            PalmDocHeaderData::EncryptionType => 92,
            _ => 0,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u16::<BigEndian>()
    }
    /// Gets u32 header value from specific location
    fn get_headers_u32(
        content: &[u8],
        pdheader: PalmDocHeaderData,
        num_of_records: u16,
    ) -> Result<u32, std::io::Error> {
        let mut reader = Cursor::new(content);
        let position = match pdheader {
            PalmDocHeaderData::TextLength => 84,
            _ => 0,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u32::<BigEndian>()
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
