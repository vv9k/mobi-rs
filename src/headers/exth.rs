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
    pub records: HashMap<u32, String>,
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
            self.records.insert(
                record_type,
                String::from_utf8_lossy(&record_data[..]).to_owned().to_string(),
            );
        }

        Ok(())
    }

    pub(crate) fn get_record(&self, record: ExthRecord) -> Option<&String> {
        self.records.get(&(record as u32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::book;
    use crate::headers::MobiHeader;

    #[test]
    fn parse() {
        let records: HashMap<u32, String> = [
            (101, String::from("HarperCollins Publishers Ltd")),
            (103, String::from("<h3>From Library Journal</h3><p>New Line Cinema will be releasing \"The Lord of the Rings\" trilogy in three separate installments, and Houghton Mifflin Tolkien\'s U.S. publisher since the release of The Hobbit in 1938 will be re-releasing each volume of the trilogy separately and in a boxed set (ISBN 0-618-15397-7. $22; pap. ISBN 0-618-15396-9. $12). <br />Copyright 2001 Reed Business Information, Inc. </p><h3>Review</h3><p>\'An extraordinary book. It deals with a stupendous theme. It leads us through a succession of strange and astonishing episodes, some of them magnificent, in a region where everything is invented, forest, moor, river, wilderness, town and the races which inhabit them.\' The Observer \'Among the greatest works of imaginative fiction of the twentieth century.\' Sunday Telegraph </p>")),
            (100, String::from("J. R. R. Tolkien")),
            (503, String::from("Lord of the Rings - Fellowship of the Ring")),
            (106, String::from("2010-12-21T00:00:00+00:00")),
            (108, String::from("calibre (0.7.31) [http://calibre-ebook.com]")),
            (104, String::from("9780261102316")),
            (106, String::from("2010-12-21T00:00:00+00:00")),
            (201, String::from("\u{0}\u{0}\u{0}\u{0}")),
            (203, String::from("\u{0}\u{0}\u{0}\u{0}")),
            (202, String::from("\u{0}\u{0}\u{0}\u{1}")),
        ].iter().cloned().collect();

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
                let data = exth.get_record(ExthRecord::$t);
                assert_eq!(data, Some(&String::from($s)));
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
