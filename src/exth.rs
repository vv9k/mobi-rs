use super::{FieldHeaderEnum, HeaderField, Reader};
use std::collections::HashMap;

const RECORDS_OFFSET: u16 = 340;

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
impl FieldHeaderEnum for ExtHeaderData {}
impl HeaderField<ExtHeaderData> for ExtHeaderData {
    fn position(self) -> u16 {
        match self {
            ExtHeaderData::Identifier => 328,
            ExtHeaderData::HeaderLength => 332,
            ExtHeaderData::RecordCount => 336,
        }
    }
}
#[derive(Debug, Default, PartialEq)]
/// Strcture that holds Exth header information
pub struct ExtHeader {
    pub identifier: u32,
    pub header_length: u32,
    pub record_count: u32,
    pub records: HashMap<u32, String>,
}
impl ExtHeader {
    /// Parse a Exth header from the content
    pub(crate) fn parse(mut reader: &mut Reader) -> Result<ExtHeader, std::io::Error> {
        use ExtHeaderData::*;

        let mut extheader = ExtHeader {
            identifier: reader.read_u32_header(Identifier)?,
            header_length: reader.read_u32_header(HeaderLength)?,
            record_count: reader.read_u32_header(RecordCount)?,
            records: HashMap::new(),
        };
        extheader.get_records(&mut reader);
        Ok(extheader)
    }
    /// Gets header records
    fn get_records(&mut self, reader: &mut Reader) {
        let mut records = HashMap::new();
        let position: u64 = RECORDS_OFFSET as u64 + u64::from(reader.num_of_records * 8);
        reader.set_position(position);
        for _i in 0..self.record_count {
            let mut record_data = vec![];
            let record_type = reader.read_u32_be().unwrap_or(0);
            let record_len = reader.read_u32_be().unwrap_or(0);
            for _j in 0..record_len - 8 {
                record_data.push(reader.read_u8().unwrap_or(0));
            }
            records.insert(
                record_type,
                String::from_utf8_lossy(&record_data[..]).to_owned().to_string(),
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
mod tests {
    use super::*;
    use crate::book;
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
        let parsed_header = ExtHeader::parse(&mut reader).unwrap();
        assert_eq!(extheader, parsed_header);
    }
    mod records {
        use super::*;
        use crate::book;
        macro_rules! info {
            ($t: ident, $s: expr) => {
                let mut reader = book::test_reader();
                reader.set_num_of_records(292);
                let exth = ExtHeader::parse(&mut reader).unwrap();
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
