mod book;
use book::BOOK;
use mobi::header::{Header, HeaderData};
use mobi::palmdoch::PalmDocHeader;

#[cfg(test)]
mod palmdoc_header {
    use super::*;
    #[test]
    fn parse() {
        let pdheader = PalmDocHeader {
            compression: 2,
            text_length: 1151461,
            record_count: 282,
            record_size: 4096,
            encryption_type: 0,
        };
        let parsed_header = PalmDocHeader::parse(
            BOOK,
            Header::get_headers_u16(BOOK, HeaderData::NumOfRecords).unwrap(),
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
