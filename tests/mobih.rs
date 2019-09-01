mod book;
use book::BOOK;
use mobi::{Header, HeaderData, MobiHeader};
#[cfg(test)]
mod mobi_header {
    use super::*;
    #[test]
    fn has_exth_header() {
        assert_eq!(true, MobiHeader::has_exth_header(80));
    }
    #[test]
    fn parse() {
        let mobiheader = MobiHeader {
            identifier: 1297039945,
            header_length: 232,
            mobi_type: 2,
            text_encoding: 65001,
            id: 3428045761,
            gen_version: 6,
            first_non_book_index: 284,
            name: String::from("Lord of the Rings - Fellowship of the Ring"),
            name_offset: 1360,
            name_length: 42,
            language_code: 9,
            input_language: 0,
            output_language: 0,
            format_version: 6,
            first_image_index: 287,
            first_huff_record: 0,
            huff_record_count: 0,
            first_data_record: 0,
            data_record_count: 0,
            exth_flags: 80,
            has_exth_header: true,
            has_drm: false,
            drm_offset: 4294967295,
            drm_count: 0,
            drm_size: 0,
            drm_flags: 0,
            last_image_record: 288,
            fcis_record: 290,
            flis_record: 289,
            extra_bytes: 22,
        };
        let parsed_header = MobiHeader::parse(
            BOOK,
            Header::get_headers_u16(BOOK, HeaderData::NumOfRecords).unwrap(),
        )
        .unwrap();
        assert_eq!(mobiheader, parsed_header);
    }
    mod text_encoding {
        use super::*;
        #[test]
        fn utf_8() {
            let mut m = MobiHeader::default();
            m.text_encoding = 65001;
            assert_eq!(m.text_encoding(), Some(String::from("UTF-8")))
        }
        #[test]
        fn win_latin1() {
            let mut m = MobiHeader::default();
            m.text_encoding = 1252;
            assert_eq!(m.text_encoding(), Some(String::from("CP1252 (WinLatin1)")))
        }
    }
    mod mobi_type {
        use super::*;
        macro_rules! mtype {
            ($mt: expr, $s: expr) => {
                let mut m = MobiHeader::default();
                m.mobi_type = $mt;
                assert_eq!(m.mobi_type(), Some(String::from($s)))
            };
        }
        #[test]
        fn mobipocket_book() {
            mtype!(2, "Mobipocket Book");
        }
        #[test]
        fn palmdoc_book() {
            mtype!(3, "PalmDoc Book");
        }
        #[test]
        fn audio() {
            mtype!(4, "Audio");
        }
        #[test]
        fn news() {
            mtype!(257, "News");
        }
        #[test]
        fn news_feed() {
            mtype!(258, "News Feed");
        }
        #[test]
        fn news_magazine() {
            mtype!(259, "News Magazine");
        }
        #[test]
        fn pics() {
            mtype!(513, "PICS");
        }
        #[test]
        fn word() {
            mtype!(514, "WORD");
        }
        #[test]
        fn xls() {
            mtype!(515, "XLS");
        }
        #[test]
        fn ppt() {
            mtype!(516, "PPT");
        }
        #[test]
        fn text() {
            mtype!(517, "TEXT");
        }
        #[test]
        fn html() {
            mtype!(518, "HTML");
        }
    }
    mod language {
        use super::*;
        macro_rules! lang {
            ($lc: expr, $s: expr) => {
                let mut m = MobiHeader::default();
                m.language_code = $lc;
                assert_eq!(m.language(), Some(String::from($s)))
            };
        }
        #[test]
        fn neutral() {
            lang!(0, "NEUTRAL");
        }
        #[test]
        fn afrikaans() {
            lang!(54, "AFRIKAANS");
        }
        #[test]
        fn albanian() {
            lang!(28, "ALBANIAN");
        }
        #[test]
        fn arabic() {
            lang!(1, "ARABIC");
        }
        #[test]
        fn armenian() {
            lang!(43, "ARMENIAN");
        }
        #[test]
        fn assamese() {
            lang!(77, "ASSAMESE");
        }
        #[test]
        fn azeri() {
            lang!(44, "AZERI");
        }
        #[test]
        fn basque() {
            lang!(45, "BASQUE");
        }
        #[test]
        fn belarusian() {
            lang!(35, "BELARUSIAN");
        }
        #[test]
        fn bengali() {
            lang!(69, "BENGALI");
        }
        #[test]
        fn bulgarian() {
            lang!(2, "BULGARIAN");
        }
        #[test]
        fn catalan() {
            lang!(3, "CATALAN");
        }
        #[test]
        fn chinese() {
            lang!(4, "CHINESE");
        }
        #[test]
        fn czech() {
            lang!(5, "CZECH");
        }
        #[test]
        fn danish() {
            lang!(6, "DANISH");
        }
        #[test]
        fn dutch() {
            lang!(19, "DUTCH");
        }
        #[test]
        fn english() {
            lang!(9, "ENGLISH");
        }
        #[test]
        fn estonian() {
            lang!(37, "ESTONIAN");
        }
        #[test]
        fn faeroese() {
            lang!(56, "FAEROESE");
        }
        #[test]
        fn farsi() {
            lang!(41, "FARSI");
        }
        #[test]
        fn finnish() {
            lang!(11, "FINNISH");
        }
        #[test]
        fn french() {
            lang!(12, "FRENCH");
        }
        #[test]
        fn georgian() {
            lang!(55, "GEORGIAN");
        }
        #[test]
        fn german() {
            lang!(7, "GERMAN");
        }
        #[test]
        fn greek() {
            lang!(8, "GREEK");
        }
        #[test]
        fn gujarati() {
            lang!(71, "GUJARATI");
        }
        #[test]
        fn hebrew() {
            lang!(13, "HEBREW");
        }
        #[test]
        fn hindi() {
            lang!(57, "HINDI");
        }
        #[test]
        fn hungarian() {
            lang!(14, "HUNGARIAN");
        }
        #[test]
        fn icelandic() {
            lang!(15, "ICELANDIC");
        }
        #[test]
        fn indonesian() {
            lang!(33, "INDONESIAN");
        }
        #[test]
        fn italian() {
            lang!(16, "ITALIAN");
        }
        #[test]
        fn japanese() {
            lang!(17, "JAPANESE");
        }
        #[test]
        fn kannada() {
            lang!(75, "KANNADA");
        }
        #[test]
        fn kazak() {
            lang!(63, "KAZAK");
        }
        #[test]
        fn konkani() {
            lang!(87, "KONKANI");
        }
        #[test]
        fn korean() {
            lang!(18, "KOREAN");
        }
        #[test]
        fn latvian() {
            lang!(38, "LATVIAN");
        }
        #[test]
        fn lithuanian() {
            lang!(39, "LITHUANIAN");
        }
        #[test]
        fn macedonian() {
            lang!(47, "MACEDONIAN");
        }
        #[test]
        fn malay() {
            lang!(62, "MALAY");
        }
        #[test]
        fn malayalam() {
            lang!(76, "MALAYALAM");
        }
        #[test]
        fn maltese() {
            lang!(58, "MALTESE");
        }
        #[test]
        fn marathi() {
            lang!(78, "MARATHI");
        }
        #[test]
        fn nepali() {
            lang!(97, "NEPALI");
        }
        #[test]
        fn norwegian() {
            lang!(20, "NORWEGIAN");
        }
        #[test]
        fn oriya() {
            lang!(72, "ORIYA");
        }
        #[test]
        fn polish() {
            lang!(21, "POLISH");
        }
        #[test]
        fn portuguese() {
            lang!(22, "PORTUGUESE");
        }
        #[test]
        fn punjabi() {
            lang!(70, "PUNJABI");
        }
        #[test]
        fn rhaetoromanic() {
            lang!(23, "RHAETOROMANIC");
        }
        #[test]
        fn romanian() {
            lang!(24, "ROMANIAN");
        }
        #[test]
        fn russian() {
            lang!(25, "RUSSIAN");
        }
        #[test]
        fn sami() {
            lang!(59, "SAMI");
        }
        #[test]
        fn sanskrit() {
            lang!(79, "SANSKRIT");
        }
        #[test]
        fn serbian() {
            lang!(26, "SERBIAN");
        }
        #[test]
        fn slovak() {
            lang!(27, "SLOVAK");
        }
        #[test]
        fn slovenian() {
            lang!(36, "SLOVENIAN");
        }
        #[test]
        fn sorbian() {
            lang!(46, "SORBIAN");
        }
        #[test]
        fn spanish() {
            lang!(10, "SPANISH");
        }
        #[test]
        fn sutu() {
            lang!(48, "SUTU");
        }
        #[test]
        fn swahili() {
            lang!(65, "SWAHILI");
        }
        #[test]
        fn swedish() {
            lang!(29, "SWEDISH");
        }
        #[test]
        fn tamil() {
            lang!(73, "TAMIL");
        }
        #[test]
        fn tatar() {
            lang!(68, "TATAR");
        }
        #[test]
        fn telugu() {
            lang!(74, "TELUGU");
        }
        #[test]
        fn thai() {
            lang!(30, "THAI");
        }
        #[test]
        fn tsonga() {
            lang!(49, "TSONGA");
        }
        #[test]
        fn tswana() {
            lang!(50, "TSWANA");
        }
        #[test]
        fn turkish() {
            lang!(31, "TURKISH");
        }
        #[test]
        fn ukrainian() {
            lang!(34, "UKRAINIAN");
        }
        #[test]
        fn urdu() {
            lang!(32, "URDU");
        }
        #[test]
        fn uzbek() {
            lang!(67, "UZBEK");
        }
        #[test]
        fn vietnamese() {
            lang!(42, "VIETNAMESE");
        }
        #[test]
        fn xhosa() {
            lang!(52, "XHOSA");
        }
        #[test]
        fn zulu() {
            lang!(53, "ZULU");
        }
    }
}
