use super::HeaderField;
use crate::reader::MobiReader;
use std::io;

const DRM_ON_FLAG: u32 = 0xFFFF_FFFF;
const EXTH_ON_FLAG: u32 = 0x40;

#[derive(Debug, PartialEq)]
pub enum TextEncoding {
    CP1252,
    UTF8,
}

#[derive(Debug, PartialEq, Default)]
/// Strcture that holds Mobi header information
pub struct MobiHeader {
    pub identifier: u32,
    pub header_length: u32,
    pub mobi_type: u32,
    pub text_encoding: u32,
    pub id: u32,
    pub gen_version: u32,
    pub first_non_book_index: u32,
    pub name: String,
    pub name_offset: u32,
    pub name_length: u32,
    pub language_code: u16,
    pub input_language: u32,
    pub output_language: u32,
    pub format_version: u32,
    pub first_image_index: u32,
    pub first_huff_record: u32,
    pub huff_record_count: u32,
    pub first_data_record: u32,
    pub data_record_count: u32,
    pub exth_flags: u32,
    pub drm_offset: u32,
    pub drm_count: u32,
    pub drm_size: u32,
    pub drm_flags: u32,
    pub last_image_record: u16,
    pub fcis_record: u32,
    pub flis_record: u32,
}
/// Parameters of Mobi Header
pub(crate) enum MobiHeaderData {
    Identifier = 96,
    HeaderLength = 100,
    MobiType = 104,
    TextEncoding = 108,
    Id = 112,
    GenVersion = 116,
    FirstNonBookIndex = 160,
    NameOffset = 164,
    NameLength = 168,
    LanguageCode = 172,
    InputLanguage = 176,
    OutputLanguage = 180,
    FormatVersion = 184,
    FirstImageIndex = 188,
    FirstHuffRecord = 192,
    HuffRecordCount = 196,
    FirstDataRecord = 200,
    DataRecordCount = 204,
    ExthFlags = 208,
    DrmOffset = 248,
    DrmCount = 252,
    DrmSize = 256,
    DrmFlags = 260,
    LastImageRecord = 274,
    FcisRecord = 280,
    FlisRecord = 288,
}
impl HeaderField for MobiHeaderData {
    fn position(self) -> u64 {
        self as u64
    }
}
impl MobiHeader {
    /// Partially parse a Mobi header from the content
    pub(crate) fn partial_parse(reader: &mut impl MobiReader) -> io::Result<MobiHeader> {
        use MobiHeaderData::*;

        Ok(MobiHeader {
            identifier: reader.read_u32_header(Identifier)?,
            header_length: reader.read_u32_header(HeaderLength)?,
            mobi_type: reader.read_u32_header(MobiType)?,
            text_encoding: reader.read_u32_header(TextEncoding)?,
            id: reader.read_u32_header(Id)?,
            gen_version: reader.read_u32_header(GenVersion)?,
            first_non_book_index: reader.read_u32_header(FirstNonBookIndex)?,
            name_offset: reader.read_u32_header(NameOffset)?,
            name_length: reader.read_u32_header(NameLength)?,
            language_code: MobiHeader::lang_code(reader.read_u32_header(LanguageCode)?),
            input_language: reader.read_u32_header(InputLanguage)?,
            output_language: reader.read_u32_header(OutputLanguage)?,
            format_version: reader.read_u32_header(FormatVersion)?,
            first_image_index: reader.read_u32_header(FirstImageIndex)?,
            first_huff_record: reader.read_u32_header(FirstHuffRecord)?,
            huff_record_count: reader.read_u32_header(HuffRecordCount)?,
            first_data_record: reader.read_u32_header(FirstDataRecord)?,
            data_record_count: reader.read_u32_header(DataRecordCount)?,
            exth_flags: reader.read_u32_header(ExthFlags)?,
            drm_offset: reader.read_u32_header(DrmOffset)?,
            drm_count: reader.read_u32_header(DrmCount)?,
            drm_size: reader.read_u32_header(DrmSize)?,
            drm_flags: reader.read_u32_header(DrmFlags)?,
            last_image_record: reader.read_u16_header(LastImageRecord)?,
            fcis_record: reader.read_u32_header(FcisRecord)?,
            flis_record: reader.read_u32_header(FlisRecord)?,
            name: String::new(),
        })
    }

    pub(crate) fn finish_parse(&mut self, reader: &mut impl MobiReader) -> io::Result<()> {
        // TODO: figure out why is this exactly `+ 80` and it works?
        let offset = reader.position_after_records() + 80 + self.name_offset as u64;
        self.name = reader.read_range(offset as u64, offset + self.name_length as u64)?;
        Ok(())
    }

    /// Checks if there is a Exth Header and changes the parameter
    pub(crate) fn has_exth_header(&self) -> bool {
        (self.exth_flags & EXTH_ON_FLAG) != 0
    }

    /// Checks if there is DRM on this book
    pub(crate) fn has_drm(&self) -> bool {
        self.drm_offset != DRM_ON_FLAG
    }

    /// Converts numerical value into a type
    pub(crate) fn mobi_type(&self) -> Option<String> {
        macro_rules! mtype {
            ($s:expr) => {
                Some(String::from($s))
            };
        }
        match self.mobi_type {
            2 => mtype!("Mobipocket Book"),
            3 => mtype!("PalmDoc Book"),
            4 => mtype!("Audio"),
            257 => mtype!("News"),
            258 => mtype!("News Feed"),
            259 => mtype!("News Magazine"),
            513 => mtype!("PICS"),
            514 => mtype!("WORD"),
            515 => mtype!("XLS"),
            516 => mtype!("PPT"),
            517 => mtype!("TEXT"),
            518 => mtype!("HTML"),
            _ => None,
        }
    }

    // Mobi format only specifies this two encodings so
    // this should never panic
    pub(crate) fn text_encoding(&self) -> TextEncoding {
        match self.text_encoding {
            1252 => TextEncoding::CP1252,
            65001 => TextEncoding::UTF8,
            n => panic!("Unknown encoding {}", n),
        }
    }

    fn lang_code(code: u32) -> u16 {
        (code & 0xFF) as u16
    }

    pub(crate) fn language(&self) -> Option<String> {
        macro_rules! lang {
            ($s:expr) => {
                Some(String::from($s))
            };
        }
        match self.language_code {
            0 => lang!("NEUTRAL"),
            54 => lang!("AFRIKAANS"),
            28 => lang!("ALBANIAN"),
            1 => lang!("ARABIC"),
            43 => lang!("ARMENIAN"),
            77 => lang!("ASSAMESE"),
            44 => lang!("AZERI"),
            45 => lang!("BASQUE"),
            35 => lang!("BELARUSIAN"),
            69 => lang!("BENGALI"),
            2 => lang!("BULGARIAN"),
            3 => lang!("CATALAN"),
            4 => lang!("CHINESE"),
            5 => lang!("CZECH"),
            6 => lang!("DANISH"),
            19 => lang!("DUTCH"),
            9 => lang!("ENGLISH"),
            37 => lang!("ESTONIAN"),
            56 => lang!("FAEROESE"),
            41 => lang!("FARSI"),
            11 => lang!("FINNISH"),
            12 => lang!("FRENCH"),
            55 => lang!("GEORGIAN"),
            7 => lang!("GERMAN"),
            8 => lang!("GREEK"),
            71 => lang!("GUJARATI"),
            13 => lang!("HEBREW"),
            57 => lang!("HINDI"),
            14 => lang!("HUNGARIAN"),
            15 => lang!("ICELANDIC"),
            33 => lang!("INDONESIAN"),
            16 => lang!("ITALIAN"),
            17 => lang!("JAPANESE"),
            75 => lang!("KANNADA"),
            63 => lang!("KAZAK"),
            87 => lang!("KONKANI"),
            18 => lang!("KOREAN"),
            38 => lang!("LATVIAN"),
            39 => lang!("LITHUANIAN"),
            47 => lang!("MACEDONIAN"),
            62 => lang!("MALAY"),
            76 => lang!("MALAYALAM"),
            58 => lang!("MALTESE"),
            78 => lang!("MARATHI"),
            97 => lang!("NEPALI"),
            20 => lang!("NORWEGIAN"),
            72 => lang!("ORIYA"),
            21 => lang!("POLISH"),
            22 => lang!("PORTUGUESE"),
            70 => lang!("PUNJABI"),
            23 => lang!("RHAETOROMANIC"),
            24 => lang!("ROMANIAN"),
            25 => lang!("RUSSIAN"),
            59 => lang!("SAMI"),
            79 => lang!("SANSKRIT"),
            26 => lang!("SERBIAN"),
            27 => lang!("SLOVAK"),
            36 => lang!("SLOVENIAN"),
            46 => lang!("SORBIAN"),
            10 => lang!("SPANISH"),
            48 => lang!("SUTU"),
            65 => lang!("SWAHILI"),
            29 => lang!("SWEDISH"),
            73 => lang!("TAMIL"),
            68 => lang!("TATAR"),
            74 => lang!("TELUGU"),
            30 => lang!("THAI"),
            49 => lang!("TSONGA"),
            50 => lang!("TSWANA"),
            31 => lang!("TURKISH"),
            34 => lang!("UKRAINIAN"),
            32 => lang!("URDU"),
            67 => lang!("UZBEK"),
            42 => lang!("VIETNAMESE"),
            52 => lang!("XHOSA"),
            53 => lang!("ZULU"),
            _ => None,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::MobiHeader;
    use crate::{book, TextEncoding};

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
            drm_offset: 4294967295,
            drm_count: 0,
            drm_size: 0,
            drm_flags: 0,
            last_image_record: 288,
            fcis_record: 290,
            flis_record: 289,
        };

        let mut reader = book::test_reader_after_header();
        let mut test_header = MobiHeader::partial_parse(&mut reader).unwrap();
        test_header.finish_parse(&mut reader).expect("Should find name");

        assert_eq!(mobiheader, test_header);
    }

    mod text_encoding {
        use super::*;
        #[test]
        fn utf_8() {
            let mut m = MobiHeader::default();
            m.text_encoding = 65001;
            assert_eq!(m.text_encoding(), TextEncoding::UTF8)
        }
        #[test]
        fn win_latin1() {
            let mut m = MobiHeader::default();
            m.text_encoding = 1252;
            assert_eq!(m.text_encoding(), TextEncoding::CP1252)
        }
    }

    #[test]
    fn parses_mobi_types() {
        macro_rules! mtype {
            ($mt: expr, $s: expr) => {
                let mut m = MobiHeader::default();
                m.mobi_type = $mt;
                assert_eq!(m.mobi_type(), Some(String::from($s)))
            };
        }
        mtype!(2, "Mobipocket Book");
        mtype!(3, "PalmDoc Book");
        mtype!(4, "Audio");
        mtype!(257, "News");
        mtype!(258, "News Feed");
        mtype!(259, "News Magazine");
        mtype!(513, "PICS");
        mtype!(514, "WORD");
        mtype!(515, "XLS");
        mtype!(516, "PPT");
        mtype!(517, "TEXT");
        mtype!(518, "HTML");
    }

    #[test]
    fn parses_languages() {
        macro_rules! lang {
            ($lc: expr, $s: expr) => {
                let mut m = MobiHeader::default();
                m.language_code = $lc;
                assert_eq!(m.language(), Some(String::from($s)))
            };
        }

        lang!(0, "NEUTRAL");
        lang!(54, "AFRIKAANS");
        lang!(28, "ALBANIAN");
        lang!(1, "ARABIC");
        lang!(43, "ARMENIAN");
        lang!(77, "ASSAMESE");
        lang!(44, "AZERI");
        lang!(45, "BASQUE");
        lang!(35, "BELARUSIAN");
        lang!(69, "BENGALI");
        lang!(2, "BULGARIAN");
        lang!(3, "CATALAN");
        lang!(4, "CHINESE");
        lang!(5, "CZECH");
        lang!(6, "DANISH");
        lang!(19, "DUTCH");
        lang!(9, "ENGLISH");
        lang!(37, "ESTONIAN");
        lang!(56, "FAEROESE");
        lang!(41, "FARSI");
        lang!(11, "FINNISH");
        lang!(12, "FRENCH");
        lang!(55, "GEORGIAN");
        lang!(7, "GERMAN");
        lang!(8, "GREEK");
        lang!(71, "GUJARATI");
        lang!(13, "HEBREW");
        lang!(57, "HINDI");
        lang!(14, "HUNGARIAN");
        lang!(15, "ICELANDIC");
        lang!(33, "INDONESIAN");
        lang!(16, "ITALIAN");
        lang!(17, "JAPANESE");
        lang!(75, "KANNADA");
        lang!(63, "KAZAK");
        lang!(87, "KONKANI");
        lang!(18, "KOREAN");
        lang!(38, "LATVIAN");
        lang!(39, "LITHUANIAN");
        lang!(47, "MACEDONIAN");
        lang!(62, "MALAY");
        lang!(76, "MALAYALAM");
        lang!(58, "MALTESE");
        lang!(78, "MARATHI");
        lang!(97, "NEPALI");
        lang!(20, "NORWEGIAN");
        lang!(72, "ORIYA");
        lang!(21, "POLISH");
        lang!(22, "PORTUGUESE");
        lang!(70, "PUNJABI");
        lang!(23, "RHAETOROMANIC");
        lang!(24, "ROMANIAN");
        lang!(25, "RUSSIAN");
        lang!(59, "SAMI");
        lang!(79, "SANSKRIT");
        lang!(26, "SERBIAN");
        lang!(27, "SLOVAK");
        lang!(36, "SLOVENIAN");
        lang!(46, "SORBIAN");
        lang!(10, "SPANISH");
        lang!(48, "SUTU");
        lang!(65, "SWAHILI");
        lang!(29, "SWEDISH");
        lang!(73, "TAMIL");
        lang!(68, "TATAR");
        lang!(74, "TELUGU");
        lang!(30, "THAI");
        lang!(49, "TSONGA");
        lang!(50, "TSWANA");
        lang!(31, "TURKISH");
        lang!(34, "UKRAINIAN");
        lang!(32, "URDU");
        lang!(67, "UZBEK");
        lang!(42, "VIETNAMESE");
        lang!(52, "XHOSA");
        lang!(53, "ZULU");
    }
}
