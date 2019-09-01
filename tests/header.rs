use mobi::Header;
mod book;
use book::BOOK;
#[cfg(test)]
mod header {
    use super::*;
    #[test]
    fn parse() {
        let header = Header {
            name: String::from("Lord_of_the_Rings_-_Fellowship_\u{0}"),
            attributes: 0,
            version: 0,
            created: 1299709979,
            modified: 1299709979,
            backup: 0,
            modnum: 0,
            app_info_id: 0,
            sort_info_id: 0,
            typ_e: String::from("BOOK"),
            creator: String::from("MOBI"),
            unique_id_seed: 292,
            next_record_list_id: 0,
            num_of_records: 292,
        };
        let parsed_header = Header::parse(BOOK);
        assert_eq!(header, parsed_header.unwrap())
    }
}
