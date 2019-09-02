mod book;
use book::BOOK;
mod exth_header {
    use super::*;
    use mobi::header::{Header, HeaderData};
    use mobi::exth::{ExtHeader, BookInfo};
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
