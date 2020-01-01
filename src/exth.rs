use super::*;
pub(crate) enum BookInfo {
    Author,
    Publisher,
    Description,
    Isbn,
    PublishDate,
    Contributor,
    Title,
}
/// Parameters of Exth Header
pub(crate) enum ExtHeaderData {
    Identifier,
    HeaderLength,
    RecordCount,
}
#[derive(Debug, Default, PartialEq)]
/// Strcture that holds Exth header information
pub struct ExtHeader {
    pub identifier: u32,
    pub header_length: u32,
    pub record_count: u32,
    pub records: HashMap<u32, String>,
}
impl fmt::Display for ExtHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EXTHEADER
Identifier:             {}
Header_length:          {}
Record_count:           {}
Records:                {:#?}",
            self.identifier, self.header_length, self.record_count, self.records,
        )
    }
}
impl ExtHeader {
    /// Parse a Exth header from the content
    pub(crate) fn parse(content: &[u8], num_of_records: u16) -> Result<ExtHeader, std::io::Error> {
        let identifier =
            ExtHeader::get_headers_u32(content, ExtHeaderData::Identifier, num_of_records)?;
        let header_length =
            ExtHeader::get_headers_u32(content, ExtHeaderData::HeaderLength, num_of_records)?;
        let record_count =
            ExtHeader::get_headers_u32(content, ExtHeaderData::RecordCount, num_of_records)?;
        let mut extheader = ExtHeader {
            identifier,
            header_length,
            record_count,
            records: HashMap::new(),
        };
        extheader.get_records(content, num_of_records);
        Ok(extheader)
    }
    /// Gets u32 header value from specific location
    fn get_headers_u32(
        content: &[u8],
        extheader: ExtHeaderData,
        num_of_records: u16,
    ) -> Result<u32, std::io::Error> {
        let mut reader = Cursor::new(content);
        let position = match extheader {
            ExtHeaderData::Identifier => 328,
            ExtHeaderData::HeaderLength => 332,
            ExtHeaderData::RecordCount => 336,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u32::<BigEndian>()
    }
    /// Gets header records
    fn get_records(&mut self, content: &[u8], num_of_records: u16) {
        let mut records = HashMap::new();
        let mut reader = Cursor::new(content);
        let position: u64 = 340 + u64::from(num_of_records * 8);
        reader.set_position(position);
        for _i in 0..self.record_count {
            let mut record_data = vec![];
            let record_type = reader.read_u32::<BigEndian>().unwrap_or(0);
            let record_len = reader.read_u32::<BigEndian>().unwrap_or(0);
            for _j in 0..record_len - 8 {
                record_data.push(reader.read_u8().unwrap_or(0));
            }
            records.insert(
                record_type,
                String::from_utf8_lossy(&record_data[..])
                    .to_owned()
                    .to_string(),
            );
        }
        self.records = records;
    }
    pub(crate) fn get_book_info(&self, info: BookInfo) -> Option<&String> {
        let record: u32 = match info {
            BookInfo::Author => 100,
            BookInfo::Publisher => 101,
            BookInfo::Description => 103,
            BookInfo::Isbn => 104,
            BookInfo::PublishDate => 106,
            BookInfo::Contributor => 108,
            BookInfo::Title => 503,
        };
        self.records.get(&record)
    }
}

#[cfg(test)]
mod exth_header {
    use super::*;
    use book::BOOK;
    use exth::{BookInfo, ExtHeader};
    use header::{Header, HeaderData};
    use std::collections::HashMap;
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
        let parsed_header = ExtHeader::parse(
            BOOK,
            Header::get_headers_u16(BOOK, HeaderData::NumOfRecords).unwrap(),
        )
        .unwrap();
        assert_eq!(extheader, parsed_header);
    }
    mod records {
        use super::*;
        macro_rules! info {
            ($t: ident, $s: expr) => {
                let exth = ExtHeader::parse(BOOK, 292).unwrap();
                let data = exth.get_book_info(BookInfo::$t);
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
