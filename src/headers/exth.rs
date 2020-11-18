#![allow(dead_code)]
use super::HeaderField;
use crate::Reader;
use std::{collections::HashMap, io};

const RECORDS_OFFSET: u64 = 108;

// Records available in EXTH header
pub(crate) enum ExthRecord {
    // source - https://wiki.mobileread.com/wiki/MOBI#EXTH_Header
    DrmServerId = 1,
    DrmCommerceId = 2,
    DrmEbookbaseBookId = 3,
    Author = 100,
    Publisher = 101,
    Imprint = 102,
    Description = 103,
    Isbn = 104,
    Subject = 105,
    PublishDate = 106,
    Review = 107,
    Contributor = 108,
    Rights = 109,
    Subjectcode = 110,
    Type = 111,
    Source = 112,
    Asin = 113,
    VersionNumber = 114,
    /// 0x0001 if the book content is only a sample of the full book
    Sample = 115,
    /// Position (4-byte offset) in file at which to open when first opened
    Startreading = 116,
    /// Mobipocket Creator adds this if Adult only is checked on its GUI; contents: "yes"
    Adult = 117,
    /// As text, e.g. "4.99"
    RetailPrice = 118,
    /// As text, e.g. "USD"
    RetailPriceCurrency = 119,
    KF8BoundaryOffset = 121,
    CountOfResources = 125,
    KF8CoverURI = 129,
    DictionaryShortName = 200,
    /// Add to first image field in Mobi Header to find PDB record containing the cover image
    CoverOffset = 201,
    /// Add to first image field in Mobi Header to find PDB record containing the thumbnail cover image
    ThumbOffset = 202,
    HasFakeCover = 203,
    ///Known Values: 1=mobigen, 2=Mobipocket Creator, 200=kindlegen (Windows), 201=kindlegen (Linux), 202=kindlegen (Mac). Warning: Calibre creates fake creator entries, pretending to be a Linux kindlegen 1.2 (201, 1, 2, 33307) for normal ebooks and a non-public Linux kindlegen 2.0 (201, 2, 0, 101) for periodicals.
    CreatorSoftware = 204,
    CreatoreMajorVersion = 205,
    CreatorMinorVersion = 206,
    CreatorBuildNumber = 207,
    Watermark = 208,
    /// Used by the Kindle (and Android app) for generating book-specific PIDs.
    TamperProofKeys = 209,
    FontSignature = 300,
    /// Integer percentage of the text allowed to be clipped. Usually 10.
    ClippingLimit = 401,
    PublisherLimit = 402,
    /// 1 - Text to Speech disabled; 0 - Text to Speech enabled
    TtsFlag = 404,

    // This fields are unsure
    /// 1 in this field seems to indicate a rental book
    IsRented = 405,
    /// If this field is removed from a rental, the book says it expired in 1969
    BorrowExpirationDate = 406,
    //
    ///PDOC - Personal Doc; EBOK - ebook; EBSP - ebook sample;
    Cdetype = 501,
    LastUpdateTime = 502,
    Title = 503,
    Language = 524,
}

/// Parameters of Exth Header
pub(crate) enum ExtHeaderData {
    Identifier = 96,
    HeaderLength = 100,
    RecordCount = 104,
}

impl HeaderField for ExtHeaderData {
    fn position(self) -> u64 {
        self as u64
    }
}

#[derive(Debug, Default, PartialEq)]
/// Optional header containing extended information. If the MOBI header
/// indicates that there's an EXTH header, it follows immediately after
/// the MOBI header.
pub struct ExtHeader {
    pub identifier: u32,
    pub header_length: u32,
    pub record_count: u32,
    pub records: HashMap<u32, Vec<u8>>,
}

impl ExtHeader {
    /// Parse a EXTH header from the content
    pub(crate) fn parse(mut reader: &mut Reader, header_length: u32) -> io::Result<ExtHeader> {
        use ExtHeaderData::*;

        let header_length = header_length as u64;
        let mut extheader = ExtHeader {
            identifier: reader.read_u32_header_offset(Identifier.position() + header_length)?,
            header_length: reader.read_u32_header_offset(HeaderLength.position() + header_length)?,
            record_count: reader.read_u32_header_offset(RecordCount.position() + header_length)?,
            records: HashMap::new(),
        };

        extheader.populate_records(&mut reader, header_length)?;
        Ok(extheader)
    }

    /// Gets header records
    fn populate_records(&mut self, reader: &mut Reader, header_length: u64) -> io::Result<()> {
        let position = RECORDS_OFFSET + u64::from(reader.num_of_records * 8) + header_length;

        reader.set_position(position);

        for _i in 0..self.record_count {
            let record_type = reader.read_u32_be()?;
            let record_len = reader.read_u32_be()?;

            let mut record_data = Vec::with_capacity(record_len as usize - 8);
            for _j in 0..record_len - 8 {
                record_data.push(reader.read_u8()?);
            }
            self.records.insert(record_type, record_data);
        }

        Ok(())
    }

    pub(crate) fn get_record(&self, record: ExthRecord) -> Option<&Vec<u8>> {
        self.records.get(&(record as u32))
    }

    pub(crate) fn get_record_string_lossy(&self, record: ExthRecord) -> Option<String> {
        self.get_record(record).map(|r| String::from_utf8_lossy(r).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::book;
    use crate::headers::MobiHeader;

    #[test]
    fn parse() {
        let mut records = HashMap::new();
        #[rustfmt::skip]
        let _records = vec![
            (503u32, vec![76u8, 111, 114, 100, 32, 111, 102, 32, 116, 104, 101, 32, 82, 105, 110, 103, 115, 32, 45, 32, 70, 101, 108, 108, 111, 119, 115, 104, 105, 112, 32, 111, 102, 32, 116, 104, 101, 32, 82, 105, 110, 103]),
(108, vec![99, 97, 108, 105, 98, 114, 101, 32, 40, 48, 46, 55, 46, 51, 49, 41, 32, 91, 104, 116, 116, 112, 58, 47, 47, 99, 97, 108, 105, 98, 114, 101, 45, 101, 98, 111, 111, 107, 46, 99, 111, 109, 93]),
(100, vec![74, 46, 32, 82, 46, 32, 82, 46, 32, 84, 111, 108, 107, 105, 101, 110]),
(201, vec![0, 0, 0, 0]),
(103, vec![60, 104, 51, 62, 70, 114, 111, 109, 32, 76, 105, 98, 114, 97, 114, 121, 32, 74, 111, 117, 114, 110, 97, 108, 60, 47, 104, 51, 62, 60, 112, 62, 78, 101, 119, 32, 76, 105, 110, 101, 32, 67, 105, 110, 101, 109, 97, 32, 119, 105, 108, 108, 32, 98, 101, 32, 114, 101, 108, 101, 97, 115, 105, 110, 103, 32, 34, 84, 104, 101, 32, 76, 111, 114, 100, 32, 111, 102, 32, 116, 104, 101, 32, 82, 105, 110, 103, 115, 34, 32, 116, 114, 105, 108, 111, 103, 121, 32, 105, 110, 32, 116, 104, 114, 101, 101, 32, 115, 101, 112, 97, 114, 97, 116, 101, 32, 105, 110, 115, 116, 97, 108, 108, 109, 101, 110, 116, 115, 44, 32, 97, 110, 100, 32, 72, 111, 117, 103, 104, 116, 111, 110, 32, 77, 105, 102, 102, 108, 105, 110, 32, 84, 111, 108, 107, 105, 101, 110, 39, 115, 32, 85, 46, 83, 46, 32, 112, 117, 98, 108, 105, 115, 104, 101, 114, 32, 115, 105, 110, 99, 101, 32, 116, 104, 101, 32, 114, 101, 108, 101, 97, 115, 101, 32, 111, 102, 32, 84, 104, 101, 32, 72, 111, 98, 98, 105, 116, 32, 105, 110, 32, 49, 57, 51, 56, 32, 119, 105, 108, 108, 32, 98, 101, 32, 114, 101, 45, 114, 101, 108, 101, 97, 115, 105, 110, 103, 32, 101, 97, 99, 104, 32, 118, 111, 108, 117, 109, 101, 32, 111, 102, 32, 116, 104, 101, 32, 116, 114, 105, 108, 111, 103, 121, 32, 115, 101, 112, 97, 114, 97, 116, 101, 108, 121, 32, 97, 110, 100, 32, 105, 110, 32, 97, 32, 98, 111, 120, 101, 100, 32, 115, 101, 116, 32, 40, 73, 83, 66, 78, 32, 48, 45, 54, 49, 56, 45, 49, 53, 51, 57, 55, 45, 55, 46, 32, 36, 50, 50, 59, 32, 112, 97, 112, 46, 32, 73, 83, 66, 78, 32, 48, 45, 54, 49, 56, 45, 49, 53, 51, 57, 54, 45, 57, 46, 32, 36, 49, 50, 41, 46, 32, 60, 98, 114, 32, 47, 62, 67, 111, 112, 121, 114, 105, 103, 104, 116, 32, 50, 48, 48, 49, 32, 82, 101, 101, 100, 32, 66, 117, 115, 105, 110, 101, 115, 115, 32, 73, 110, 102, 111, 114, 109, 97, 116, 105, 111, 110, 44, 32, 73, 110, 99, 46, 32, 60, 47, 112, 62, 60, 104, 51, 62, 82, 101, 118, 105, 101, 119, 60, 47, 104, 51, 62, 60, 112, 62, 39, 65, 110, 32, 101, 120, 116, 114, 97, 111, 114, 100, 105, 110, 97, 114, 121, 32, 98, 111, 111, 107, 46, 32, 73, 116, 32, 100, 101, 97, 108, 115, 32, 119, 105, 116, 104, 32, 97, 32, 115, 116, 117, 112, 101, 110, 100, 111, 117, 115, 32, 116, 104, 101, 109, 101, 46, 32, 73, 116, 32, 108, 101, 97, 100, 115, 32, 117, 115, 32, 116, 104, 114, 111, 117, 103, 104, 32, 97, 32, 115, 117, 99, 99, 101, 115, 115, 105, 111, 110, 32, 111, 102, 32, 115, 116, 114, 97, 110, 103, 101, 32, 97, 110, 100, 32, 97, 115, 116, 111, 110, 105, 115, 104, 105, 110, 103, 32, 101, 112, 105, 115, 111, 100, 101, 115, 44, 32, 115, 111, 109, 101, 32, 111, 102, 32, 116, 104, 101, 109, 32, 109, 97, 103, 110, 105, 102, 105, 99, 101, 110, 116, 44, 32, 105, 110, 32, 97, 32, 114, 101, 103, 105, 111, 110, 32, 119, 104, 101, 114, 101, 32, 101, 118, 101, 114, 121, 116, 104, 105, 110, 103, 32, 105, 115, 32, 105, 110, 118, 101, 110, 116, 101, 100, 44, 32, 102, 111, 114, 101, 115, 116, 44, 32, 109, 111, 111, 114, 44, 32, 114, 105, 118, 101, 114, 44, 32, 119, 105, 108, 100, 101, 114, 110, 101, 115, 115, 44, 32, 116, 111, 119, 110, 32, 97, 110, 100, 32, 116, 104, 101, 32, 114, 97, 99, 101, 115, 32, 119, 104, 105, 99, 104, 32, 105, 110, 104, 97, 98, 105, 116, 32, 116, 104, 101, 109, 46, 39, 32, 84, 104, 101, 32, 79, 98, 115, 101, 114, 118, 101, 114, 32, 39, 65, 109, 111, 110, 103, 32, 116, 104, 101, 32, 103, 114, 101, 97, 116, 101, 115, 116, 32, 119, 111, 114, 107, 115, 32, 111, 102, 32, 105, 109, 97, 103, 105, 110, 97, 116, 105, 118, 101, 32, 102, 105, 99, 116, 105, 111, 110, 32, 111, 102, 32, 116, 104, 101, 32, 116, 119, 101, 110, 116, 105, 101, 116, 104, 32, 99, 101, 110, 116, 117, 114, 121, 46, 39, 32, 83, 117, 110, 100, 97, 121, 32, 84, 101, 108, 101, 103, 114, 97, 112, 104, 32, 60, 47, 112, 62]),
(101, vec![72, 97, 114, 112, 101, 114, 67, 111, 108, 108, 105, 110, 115, 32, 80, 117, 98, 108, 105, 115, 104, 101, 114, 115, 32, 76, 116, 100]),
(106, vec![50, 48, 49, 48, 45, 49, 50, 45, 50, 49, 84, 48, 48, 58, 48, 48, 58, 48, 48, 43, 48, 48, 58, 48, 48]),
(104, vec![57, 55, 56, 48, 50, 54, 49, 49, 48, 50, 51, 49, 54]),
(203, vec![0, 0, 0, 0]),
(202, vec![0, 0, 0, 1]),
    ];
        _records.into_iter().for_each(|(k, v)| {
            records.insert(k, v);
        });

        let extheader = ExtHeader {
            identifier: 1163416648,
            header_length: 1109,
            record_count: 11,
            records,
        };
        let mut reader = book::test_reader_after_header();
        let mobi = MobiHeader::parse(&mut reader).unwrap();
        let parsed_header = ExtHeader::parse(&mut reader, mobi.header_length).unwrap();
        assert_eq!(extheader, parsed_header);
    }

    mod records {
        use super::*;
        use crate::book;
        macro_rules! info {
            ($t: ident, $s: expr) => {
                let mut reader = book::test_reader();
                reader.set_num_of_records(292);
                let mobi = MobiHeader::parse(&mut reader).unwrap();
                let exth = ExtHeader::parse(&mut reader, mobi.header_length).unwrap();
                let data = exth.get_record_string_lossy(ExthRecord::$t);
                assert_eq!(data, Some(String::from($s)));
            };
        }

        #[test]
        fn author() {
            info!(Author, "J. R. R. Tolkien");
        }

        #[test]
        fn publisher() {
            info!(Publisher, "HarperCollins Publishers Ltd");
        }

        #[test]
        fn description() {
            info!(Description, "<h3>From Library Journal</h3><p>New Line Cinema will be releasing \"The Lord of the Rings\" trilogy in three separate installments, and Houghton Mifflin Tolkien\'s U.S. publisher since the release of The Hobbit in 1938 will be re-releasing each volume of the trilogy separately and in a boxed set (ISBN 0-618-15397-7. $22; pap. ISBN 0-618-15396-9. $12). <br />Copyright 2001 Reed Business Information, Inc. </p><h3>Review</h3><p>\'An extraordinary book. It deals with a stupendous theme. It leads us through a succession of strange and astonishing episodes, some of them magnificent, in a region where everything is invented, forest, moor, river, wilderness, town and the races which inhabit them.\' The Observer \'Among the greatest works of imaginative fiction of the twentieth century.\' Sunday Telegraph </p>");
        }

        #[test]
        fn isbn() {
            info!(Isbn, "9780261102316");
        }

        #[test]
        fn publish_date() {
            info!(PublishDate, "2010-12-21T00:00:00+00:00");
        }

        #[test]
        fn contributor() {
            info!(Contributor, "calibre (0.7.31) [http://calibre-ebook.com]");
        }

        #[test]
        fn title() {
            info!(Title, "Lord of the Rings - Fellowship of the Ring");
        }
    }
}
