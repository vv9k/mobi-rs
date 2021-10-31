use crate::{Reader, Writer};

use indexmap::IndexMap;
use std::io;

// Records available in EXTH header
pub enum ExthRecord {
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
    /// Known Values: 1=mobigen, 2=Mobipocket Creator, 200=kindlegen (Windows), 201=kindlegen (Linux), 202=kindlegen (Mac). Warning: Calibre creates fake creator entries, pretending to be a Linux kindlegen 1.2 (201, 1, 2, 33307) for normal ebooks and a non-public Linux kindlegen 2.0 (201, 2, 0, 101) for periodicals.
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

#[derive(Debug, Default, PartialEq)]
/// Optional header containing extended information. If the MOBI header
/// indicates that there's an EXTH header, it follows immediately after
/// the MOBI header.
pub struct ExtHeader {
    pub identifier: u32,
    pub header_length: u32,
    pub record_count: u32,
    pub records: IndexMap<u32, Vec<u8>>,
}

impl ExtHeader {
    /// Parse a EXTH header from the content. Reader must be at starting
    /// location of exth header.
    pub(crate) fn parse<R: io::Read>(reader: &mut Reader<R>) -> io::Result<ExtHeader> {
        let mut extheader = ExtHeader {
            identifier: reader.read_u32_be()?,
            header_length: reader.read_u32_be()?,
            record_count: reader.read_u32_be()?,
            records: IndexMap::new(),
        };

        extheader.populate_records(reader)?;
        Ok(extheader)
    }

    /// Gets header records
    fn populate_records<R: io::Read>(&mut self, reader: &mut Reader<R>) -> io::Result<()> {
        for _i in 0..self.record_count {
            let record_type = reader.read_u32_be()?;
            let record_len = reader.read_u32_be()?;

            let mut record_data = vec![0; (record_len - 8) as usize];
            reader.read_exact(&mut record_data)?;

            self.records.insert(record_type, record_data);
        }

        Ok(())
    }

    pub(crate) fn write<W: io::Write>(&self, w: &mut Writer<W>) -> io::Result<()> {
        w.write_be(self.identifier)?;
        // 12 for first 12 bytes, 8 per record + len.
        w.write_be(12u32 + self.records.iter().map(|(_, d)| 8 + d.len() as u32).sum::<u32>())?;
        w.write_be(self.records.len() as u32)?;
        for (&id, record_data) in self.records.iter() {
            w.write_be(id)?;
            w.write_be(record_data.len() as u32 + 8)?;
            w.write_be(record_data)?;
        }
        Ok(())
    }

    /// Returns exth record data located at position. This is a low level function intended
    /// to use with wrapper get_record, but exposed for convienience.
    pub fn get_record_position(&self, position: u32) -> Option<&Vec<u8>> {
        self.records.get(&position)
    }

    /// Returns exth record data. This function limits possible queried records to only those
    /// commonly available among mobi ebooks.
    pub fn get_record(&self, record: ExthRecord) -> Option<&Vec<u8>> {
        self.get_record_position(record as u32)
    }

    pub(crate) fn get_record_string_lossy(&self, record: ExthRecord) -> Option<String> {
        self.get_record(record).map(|r| String::from_utf8_lossy(r).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::book;
    use crate::writer::Writer;

    #[test]
    fn test_parse() {
        let mut records = IndexMap::new();
        #[rustfmt::skip]
        let _records = vec![
            (104, b"9780261102316".to_vec()),
            (503, b"Lord of the Rings - Fellowship of the Ring".to_vec()),
            (203, b"\0\0\0\0".to_vec()),
            (103, b"<h3>From Library Journal</h3><p>New Line Cinema will be releasing \"The Lord of the Rings\" trilogy in three separate installments, and Houghton Mifflin Tolkien's U.S. publisher since the release of The Hobbit in 1938 will be re-releasing each volume of the trilogy separately and in a boxed set (ISBN 0-618-15397-7. $22; pap. ISBN 0-618-15396-9. $12). <br />Copyright 2001 Reed Business Information, Inc. </p><h3>Review</h3><p>'An extraordinary book. It deals with a stupendous theme. It leads us through a succession of strange and astonishing episodes, some of them magnificent, in a region where everything is invented, forest, moor, river, wilderness, town and the races which inhabit them.' The Observer 'Among the greatest works of imaginative fiction of the twentieth century.' Sunday Telegraph </p>".to_vec()),
            (201, b"\0\0\0\0".to_vec()),
            (101, b"HarperCollins Publishers Ltd".to_vec()),
            (106, b"2010-12-21T00:00:00+00:00".to_vec()),
            (100, b"J. R. R. Tolkien".to_vec()),
            (202, b"\0\0\0\x01".to_vec()),
            (108, b"calibre (0.7.31) [http://calibre-ebook.com]".to_vec()),
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

        let mut reader = book::u8_reader(book::BOOK.to_vec());
        let parsed_header = ExtHeader::parse(&mut reader).unwrap();
        for (k, v) in &extheader.records {
            let record = parsed_header.get_record_position(*k);
            assert!(record.is_some());
            assert_eq!(v, record.unwrap());
        }
        assert_eq!(extheader, parsed_header);
    }

    #[test]
    fn test_write() {
        // First ExtHeader has duplicated fields and will not match when written.
        let bookx = book::BOOK.to_vec();
        let mut reader = book::u8_reader(bookx);
        let parsed_header = ExtHeader::parse(&mut reader).unwrap();
        let mut buf = vec![];
        parsed_header.write(&mut Writer::new(&mut buf)).unwrap();

        // Create a new one with no duplicate fields, which will be identical
        let new_exth_header = ExtHeader::parse(&mut book::u8_reader(buf.clone())).unwrap();
        let mut buf2 = vec![];
        new_exth_header.write(&mut Writer::new(&mut buf2)).unwrap();

        assert_eq!(buf, buf2);
    }

    mod records {
        use super::*;
        use crate::book;

        macro_rules! info {
            ($t: ident, $s: expr) => {
                let mut reader = book::u8_reader(book::BOOK.to_vec());
                let exth = ExtHeader::parse(&mut reader).unwrap();
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
