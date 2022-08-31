use crate::{Reader, Writer};

use indexmap::IndexMap;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExthRecordParseError {
    #[error("Record length is less than 8 bytes")]
    RecordTooSmall,
    #[error("Expected header to be identifier as EXTH")]
    InvalidIdentifier,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
// Records available in EXTH header
pub enum ExthRecord {
    // source - https://wiki.mobileread.com/wiki/MOBI#EXTH_Header
    DrmServerId,
    DrmCommerceId,
    DrmEbookbaseBookId,
    Author,
    Publisher,
    Imprint,
    Description,
    Isbn,
    Subject,
    PublishDate,
    Review,
    Contributor,
    Rights,
    Subjectcode,
    Type,
    Source,
    Asin,
    VersionNumber,
    /// 0x0001 if the book content is only a sample of the full book
    Sample,
    /// Position (4-byte offset) in file at which to open when first opened
    Startreading,
    /// Mobipocket Creator adds this if Adult only is checked on its GUI; contents: "yes"
    Adult,
    /// As text, e.g. "4.99"
    RetailPrice,
    /// As text, e.g. "USD"
    RetailPriceCurrency,
    KF8BoundaryOffset,
    CountOfResources,
    KF8CoverURI,
    DictionaryShortName,
    /// Add to first image field in Mobi Header to find PDB record containing the cover image
    CoverOffset,
    /// Add to first image field in Mobi Header to find PDB record containing the thumbnail cover image
    ThumbOffset,
    HasFakeCover,
    /// Known Values: 1=mobigen, 2=Mobipocket Creator, 200=kindlegen (Windows), 201=kindlegen (Linux), 202=kindlegen (Mac). Warning: Calibre creates fake creator entries, pretending to be a Linux kindlegen 1.2 (201, 1, 2, 33307) for normal ebooks and a non-public Linux kindlegen 2.0 (201, 2, 0, 101) for periodicals.
    CreatorSoftware,
    CreatoreMajorVersion,
    CreatorMinorVersion,
    CreatorBuildNumber,
    Watermark,
    /// Used by the Kindle (and Android app) for generating book-specific PIDs.
    TamperProofKeys,
    FontSignature,
    /// Integer percentage of the text allowed to be clipped. Usually 10.
    ClippingLimit,
    PublisherLimit,
    /// 1 - Text to Speech disabled; 0 - Text to Speech enabled
    TtsFlag,

    // This fields are unsure
    /// 1 in this field seems to indicate a rental book
    IsRented,
    /// If this field is removed from a rental, the book says it expired in 1969
    BorrowExpirationDate,
    //
    ///PDOC - Personal Doc; EBOK - ebook; EBSP - ebook sample;
    Cdetype,
    LastUpdateTime,
    Title,
    Language,
    Other(u32),
}

impl ExthRecord {
    pub fn position(&self) -> u32 {
        (*self).into()
    }
}

impl From<ExthRecord> for u32 {
    fn from(r: ExthRecord) -> Self {
        use ExthRecord::*;
        match r {
            DrmServerId => 1,
            DrmCommerceId => 2,
            DrmEbookbaseBookId => 3,
            Author => 100,
            Publisher => 101,
            Imprint => 102,
            Description => 103,
            Isbn => 104,
            Subject => 105,
            PublishDate => 106,
            Review => 107,
            Contributor => 108,
            Rights => 109,
            Subjectcode => 110,
            Type => 111,
            Source => 112,
            Asin => 113,
            VersionNumber => 114,
            Sample => 115,
            Startreading => 116,
            Adult => 117,
            RetailPrice => 118,
            RetailPriceCurrency => 119,
            KF8BoundaryOffset => 121,
            CountOfResources => 125,
            KF8CoverURI => 129,
            DictionaryShortName => 200,
            CoverOffset => 201,
            ThumbOffset => 202,
            HasFakeCover => 203,
            CreatorSoftware => 204,
            CreatoreMajorVersion => 205,
            CreatorMinorVersion => 206,
            CreatorBuildNumber => 207,
            Watermark => 208,
            TamperProofKeys => 209,
            FontSignature => 300,
            ClippingLimit => 401,
            PublisherLimit => 402,
            TtsFlag => 404,
            IsRented => 405,
            BorrowExpirationDate => 406,
            Cdetype => 501,
            LastUpdateTime => 502,
            Title => 503,
            Language => 524,
            Other(n) => n,
        }
    }
}

impl From<u32> for ExthRecord {
    fn from(ty: u32) -> Self {
        use ExthRecord::*;
        match ty {
            1 => DrmServerId,
            2 => DrmCommerceId,
            3 => DrmEbookbaseBookId,
            100 => Author,
            101 => Publisher,
            102 => Imprint,
            103 => Description,
            104 => Isbn,
            105 => Subject,
            106 => PublishDate,
            107 => Review,
            108 => Contributor,
            109 => Rights,
            110 => Subjectcode,
            111 => Type,
            112 => Source,
            113 => Asin,
            114 => VersionNumber,
            115 => Sample,
            116 => Startreading,
            117 => Adult,
            118 => RetailPrice,
            119 => RetailPriceCurrency,
            121 => KF8BoundaryOffset,
            125 => CountOfResources,
            129 => KF8CoverURI,
            200 => DictionaryShortName,
            201 => CoverOffset,
            202 => ThumbOffset,
            203 => HasFakeCover,
            204 => CreatorSoftware,
            205 => CreatoreMajorVersion,
            206 => CreatorMinorVersion,
            207 => CreatorBuildNumber,
            208 => Watermark,
            209 => TamperProofKeys,
            300 => FontSignature,
            401 => ClippingLimit,
            402 => PublisherLimit,
            404 => TtsFlag,
            405 => IsRented,
            406 => BorrowExpirationDate,
            501 => Cdetype,
            502 => LastUpdateTime,
            503 => Title,
            524 => Language,
            n => Other(n),
        }
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
    pub records: IndexMap<ExthRecord, Vec<Vec<u8>>>,
}

impl ExtHeader {
    /// Parse a EXTH header from the content. Reader must be at starting
    /// location of exth header.
    pub(crate) fn parse<R: io::Read>(
        reader: &mut Reader<R>,
    ) -> Result<ExtHeader, ExthRecordParseError> {
        let mut extheader = ExtHeader {
            identifier: reader.read_u32_be()?,
            header_length: reader.read_u32_be()?,
            record_count: reader.read_u32_be()?,
            records: IndexMap::new(),
        };

        if &extheader.identifier.to_be_bytes() == b"EXTH" {
            extheader.populate_records(reader)?;
            Ok(extheader)
        } else {
            Err(ExthRecordParseError::InvalidIdentifier)
        }
    }

    /// Gets header records
    fn populate_records<R: io::Read>(
        &mut self,
        reader: &mut Reader<R>,
    ) -> Result<(), ExthRecordParseError> {
        for _i in 0..self.record_count {
            let record_type = ExthRecord::from(reader.read_u32_be()?);
            let record_len = reader.read_u32_be()?;

            let num_bytes = match record_len.checked_sub(8) {
                None => return Err(ExthRecordParseError::RecordTooSmall),
                Some(num_bytes) => num_bytes,
            };

            let record_data = reader.read_vec_header(num_bytes as usize)?;

            if let Some(record) = self.records.get_mut(&record_type) {
                record.push(record_data);
            } else {
                self.records.insert(record_type, vec![record_data]);
            }
        }

        Ok(())
    }

    pub(crate) fn write<W: io::Write>(&self, w: &mut Writer<W>) -> io::Result<()> {
        w.write_be(self.identifier)?;
        w.write_be(
            12u32
                + self
                    .records
                    .iter()
                    .map(|(_, d)| 8 + d.len() as u32)
                    .sum::<u32>(),
        )?;
        w.write_be(self.records.len() as u32)?;
        for (&id, records) in self.records.iter() {
            for record_data in records {
                w.write_be(id.position())?;
                w.write_be(record_data.len() as u32 + 8)?;
                w.write_be(record_data)?;
            }
        }
        Ok(())
    }

    /// Returns exth record data located at position. This is a low level function intended
    /// to use with wrapper get_record, but exposed for convienience.
    pub fn get_record_position(&self, position: u32) -> Option<&Vec<Vec<u8>>> {
        self.get_record(ExthRecord::from(position))
    }

    /// Returns exth record data. This function limits possible queried records to only those
    /// commonly available among mobi ebooks.
    pub fn get_record(&self, record: ExthRecord) -> Option<&Vec<Vec<u8>>> {
        self.records.get(&record)
    }

    /// Returns an iterator over all available raw EXTH records.
    pub fn raw_records(&self) -> impl Iterator<Item = (&ExthRecord, &Vec<Vec<u8>>)> {
        self.records.iter()
    }

    /// Returns an iterator over all available EXTH records and performs a loseless conversion of
    /// record data to string.
    pub fn records(&self) -> impl Iterator<Item = (&ExthRecord, Vec<String>)> {
        self.records.iter().map(|(r, data)| {
            (
                r,
                data.iter()
                    .map(|d| String::from_utf8_lossy(d).to_string())
                    .collect(),
            )
        })
    }

    pub(crate) fn get_record_string_lossy(&self, record: ExthRecord) -> Option<String> {
        self.get_record(record)
            .and_then(|r| r.first())
            .map(|r| String::from_utf8_lossy(r).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::book;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse() {
        let mut records = IndexMap::new();
        #[rustfmt::skip]
        let _records = vec![
            (104.into(), vec![b"9780261102316".to_vec()]),
            (503.into(), vec![b"Lord of the Rings - Fellowship of the Ring".to_vec()]),
            (203.into(), vec![b"\0\0\0\0".to_vec()]),
            (103.into(), vec![b"<h3>From Library Journal</h3><p>New Line Cinema will be releasing \"The Lord of the Rings\" trilogy in three separate installments, and Houghton Mifflin Tolkien's U.S. publisher since the release of The Hobbit in 1938 will be re-releasing each volume of the trilogy separately and in a boxed set (ISBN 0-618-15397-7. $22; pap. ISBN 0-618-15396-9. $12). <br />Copyright 2001 Reed Business Information, Inc. </p><h3>Review</h3><p>'An extraordinary book. It deals with a stupendous theme. It leads us through a succession of strange and astonishing episodes, some of them magnificent, in a region where everything is invented, forest, moor, river, wilderness, town and the races which inhabit them.' The Observer 'Among the greatest works of imaginative fiction of the twentieth century.' Sunday Telegraph </p>".to_vec()]),
            (201.into(), vec![b"\0\0\0\0".to_vec()]),
            (101.into(), vec![b"HarperCollins Publishers Ltd".to_vec()]),
            (106.into(), vec![b"2010-12-21T00:00:00+00:00".to_vec(),b"2010-12-21T00:00:00+00:00".to_vec()]),
            (100.into(), vec![b"J. R. R. Tolkien".to_vec()]),
            (202.into(), vec![b"\0\0\0\x01".to_vec()]),
            (108.into(), vec![b"calibre (0.7.31) [http://calibre-ebook.com]".to_vec()]),
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
            let record = parsed_header.get_record(*k);
            assert!(record.is_some());
            assert_eq!(v, record.unwrap());
        }
        assert_eq!(extheader, parsed_header);
    }

    #[test]
    fn test_exth_records() {
        let mut _records: Vec<(ExthRecord, Vec<String>)> = vec![
            (ExthRecord::Isbn, vec!["9780261102316".to_string()]),
            (ExthRecord::Title, vec!["Lord of the Rings - Fellowship of the Ring".to_string()]),
            (ExthRecord::HasFakeCover, vec!["\0\0\0\0".to_string()]),
            (ExthRecord::Description, vec!["<h3>From Library Journal</h3><p>New Line Cinema will be releasing \"The Lord of the Rings\" trilogy in three separate installments, and Houghton Mifflin Tolkien's U.S. publisher since the release of The Hobbit in 1938 will be re-releasing each volume of the trilogy separately and in a boxed set (ISBN 0-618-15397-7. $22; pap. ISBN 0-618-15396-9. $12). <br />Copyright 2001 Reed Business Information, Inc. </p><h3>Review</h3><p>'An extraordinary book. It deals with a stupendous theme. It leads us through a succession of strange and astonishing episodes, some of them magnificent, in a region where everything is invented, forest, moor, river, wilderness, town and the races which inhabit them.' The Observer 'Among the greatest works of imaginative fiction of the twentieth century.' Sunday Telegraph </p>".to_string()]),
            (ExthRecord::CoverOffset, vec!["\0\0\0\0".to_string()]),
            (ExthRecord::Publisher, vec!["HarperCollins Publishers Ltd".to_string()]),
            (ExthRecord::PublishDate, vec!["2010-12-21T00:00:00+00:00".to_string(),"2010-12-21T00:00:00+00:00".to_string()]),
            (ExthRecord::Author, vec!["J. R. R. Tolkien".to_string()]),
            (ExthRecord::ThumbOffset, vec!["\0\0\0\x01".to_string()]),
            (ExthRecord::Contributor, vec!["calibre (0.7.31) [http://calibre-ebook.com]".to_string()]),
        ];

        let mut reader = book::u8_reader(book::BOOK.to_vec());
        let parsed_header = ExtHeader::parse(&mut reader).unwrap();
        let mut records: Vec<_> = parsed_header.records().collect();

        _records.sort();
        records.sort();
        for (a, b) in records.into_iter().zip(_records) {
            assert_eq!(*a.0, b.0);
            assert_eq!(a.1, b.1);
        }
    }

    mod records {
        use crate::book;
        use crate::headers::{ExtHeader, ExthRecord};
        use pretty_assertions::assert_eq;

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
