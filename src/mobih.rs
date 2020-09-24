use super::{FieldHeaderEnum, HeaderField, Reader};
use std::io;

const DRM_ON_FLAG: u32 = 0xFFFF_FFFF;
const EXTH_ON_FLAG: u32 = 0x40;
const EXTRA_BYTES_FLAG: u16 = 0xFFFE;

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
    pub has_exth_header: bool,
    pub has_drm: bool,
    pub drm_offset: u32,
    pub drm_count: u32,
    pub drm_size: u32,
    pub drm_flags: u32,
    pub last_image_record: u16,
    pub fcis_record: u32,
    pub flis_record: u32,
    pub extra_bytes: u32,
}
/// Parameters of Mobi Header
pub(crate) enum MobiHeaderData {
    Identifier,
    HeaderLength,
    MobiType,
    TextEncoding,
    Id,
    GenVersion,
    FirstNonBookIndex,
    NameOffset,
    NameLength,
    LanguageCode,
    InputLanguage,
    OutputLanguage,
    FormatVersion,
    FirstImageIndex,
    FirstHuffRecord,
    HuffRecordCount,
    FirstDataRecord,
    DataRecordCount,
    ExthFlags,
    DrmOffset,
    DrmCount,
    DrmSize,
    DrmFlags,
    LastImageRecord,
    FcisRecord,
    FlisRecord,
    ExtraBytes,
}
impl FieldHeaderEnum for MobiHeaderData {}
impl HeaderField<MobiHeaderData> for MobiHeaderData {
    fn position(self) -> u16 {
        match self {
            // TODO: Check if this field is actually on position 0
            MobiHeaderData::ExtraBytes => 0,
            MobiHeaderData::Identifier => 96,
            MobiHeaderData::HeaderLength => 100,
            MobiHeaderData::MobiType => 104,
            MobiHeaderData::TextEncoding => 108,
            MobiHeaderData::Id => 112,
            MobiHeaderData::GenVersion => 116,
            MobiHeaderData::FirstNonBookIndex => 160,
            MobiHeaderData::NameOffset => 164,
            MobiHeaderData::NameLength => 168,
            MobiHeaderData::LanguageCode => 172,
            MobiHeaderData::InputLanguage => 176,
            MobiHeaderData::OutputLanguage => 180,
            MobiHeaderData::FormatVersion => 184,
            MobiHeaderData::FirstImageIndex => 188,
            MobiHeaderData::FirstHuffRecord => 192,
            MobiHeaderData::HuffRecordCount => 196,
            MobiHeaderData::FirstDataRecord => 200,
            MobiHeaderData::DataRecordCount => 204,
            MobiHeaderData::ExthFlags => 208,
            MobiHeaderData::DrmOffset => 248,
            MobiHeaderData::DrmCount => 252,
            MobiHeaderData::DrmSize => 256,
            MobiHeaderData::DrmFlags => 260,
            MobiHeaderData::LastImageRecord => 274,
            MobiHeaderData::FcisRecord => 280,
            MobiHeaderData::FlisRecord => 288,
        }
    }
}
impl MobiHeader {
    /// Parse a Mobi header from the content
    pub(crate) fn parse(mut reader: &mut Reader) -> io::Result<MobiHeader> {
        use MobiHeaderData::*;
        Ok(MobiHeader {
            identifier: reader.read_u32_header(Identifier)?,
            header_length: reader.read_u32_header(HeaderLength)?,
            mobi_type: reader.read_u32_header(MobiType)?,
            text_encoding: reader.read_u32_header(TextEncoding)?,
            id: reader.read_u32_header(Id)?,
            gen_version: reader.read_u32_header(GenVersion)?,
            first_non_book_index: reader.read_u32_header(FirstNonBookIndex)?,
            name: MobiHeader::name(&mut reader)?,
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
            has_exth_header: MobiHeader::has_exth_header(reader.read_u32_header(ExthFlags)?),
            drm_offset: reader.read_u32_header(DrmOffset)?,
            drm_count: reader.read_u32_header(DrmCount)?,
            drm_size: reader.read_u32_header(DrmSize)?,
            drm_flags: reader.read_u32_header(DrmFlags)?,
            has_drm: MobiHeader::has_drm(reader.read_u32_header(DrmOffset)?),
            last_image_record: reader.read_u16_header(LastImageRecord)?,
            fcis_record: reader.read_u32_header(FcisRecord)?,
            flis_record: reader.read_u32_header(FlisRecord)?,
            extra_bytes: MobiHeader::extra_bytes(&mut reader)?,
        })
    }
    /// Returns the book name
    pub(crate) fn name(reader: &mut Reader) -> io::Result<String> {
        let name_offset = reader.read_u32_header(MobiHeaderData::NameOffset)?;
        let name_length = reader.read_u32_header(MobiHeaderData::NameLength)?;
        // TODO: figure out why is this exactly `+ 80` and it works?
        let offset = name_offset as usize + (reader.num_of_records * 8) as usize + 80;
        Ok(
            String::from_utf8_lossy(&reader.cursor.get_mut()[offset..offset + name_length as usize])
                .to_owned()
                .to_string(),
        )
    }
    /// Checks if there is a Exth Header and changes the parameter
    pub(crate) fn has_exth_header(exth_flags: u32) -> bool {
        (exth_flags & EXTH_ON_FLAG) != 0
    }
    /// Checks if there is DRM on this book
    fn has_drm(drm_offset: u32) -> bool {
        drm_offset != DRM_ON_FLAG
    }
    /// Returns extra bytes for reading records
    fn extra_bytes(reader: &mut Reader) -> io::Result<u32> {
        let ex_bytes = reader.read_u16_header(MobiHeaderData::ExtraBytes)?;
        Ok(2 * (ex_bytes & EXTRA_BYTES_FLAG).count_ones())
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
    use crate::{book, mobih::MobiHeader, TextEncoding};

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

        let mut reader = book::test_reader_after_header();

        assert_eq!(mobiheader, MobiHeader::parse(&mut reader).unwrap());
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
