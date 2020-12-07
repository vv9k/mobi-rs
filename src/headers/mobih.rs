use crate::reader::Writer;
use crate::Reader;
use std::io;

const DRM_ON_FLAG: u32 = 0xFFFF_FFFF;
const EXTH_ON_FLAG: u32 = 0x40;

#[derive(Debug, PartialEq)]
pub enum TextEncoding {
    CP1252,
    UTF8,
}

impl Default for MobiHeader {
    fn default() -> Self {
        MobiHeader {
            identifier: 0,
            header_length: 0,
            mobi_type: 0,
            text_encoding: 0,
            id: 0,
            gen_version: 0,
            ortho_index: 0xFFFF_FFFF,
            inflect_index: 0xFFFF_FFFF,
            index_names: 0xFFFF_FFFF,
            index_keys: 0xFFFF_FFFF,
            extra_indices: [0xFFFF_FFFF; 6],
            first_non_book_index: 0,
            name_offset: 0,
            name_length: 0,
            unused: 0,
            locale: 0,
            language_code: 0,
            input_language: 0,
            output_language: 0,
            format_version: 0,
            first_image_index: 0,
            first_huff_record: 0,
            huff_record_count: 0,
            huff_table_offset: 0,
            huff_table_length: 0,
            exth_flags: 0,
            unused_0: Box::new([0; 32]),
            unused_1: 0xFFFF_FFFF,
            drm_offset: 0,
            drm_count: 0,
            drm_size: 0,
            drm_flags: 0,
            unused_2: Box::new([0; 8]),
            first_content_record: 1,
            last_content_record: 0,
            unused_3: 1,
            fcis_record: 0,
            unused_4: 1,
            flis_record: 0,
            unused_5: vec![],
        }
    }
}

#[derive(Debug, PartialEq)]
/// Strcture that holds Mobi header information
pub struct MobiHeader {
    pub identifier: u32,
    pub header_length: u32,
    pub mobi_type: u32,
    pub text_encoding: u32,
    pub id: u32,
    pub gen_version: u32,
    pub ortho_index: u32,
    pub inflect_index: u32,
    pub index_names: u32,
    pub index_keys: u32,
    pub extra_indices: [u32; 6],
    pub first_non_book_index: u32,
    pub name_offset: u32,
    pub name_length: u32,
    unused: u16,
    pub locale: u8,
    pub language_code: u8,
    pub input_language: u32,
    pub output_language: u32,
    pub format_version: u32,
    pub first_image_index: u32,
    pub first_huff_record: u32,
    pub huff_record_count: u32,
    pub huff_table_offset: u32,
    pub huff_table_length: u32,
    pub exth_flags: u32,
    unused_0: Box<[u8; 32]>,
    unused_1: u32,
    pub drm_offset: u32,
    pub drm_count: u32,
    pub drm_size: u32,
    pub drm_flags: u32,
    unused_2: Box<[u8; 8]>,
    pub first_content_record: u16,
    pub last_content_record: u16,
    unused_3: u32,
    pub fcis_record: u32,
    unused_4: u32,
    pub flis_record: u32,
    unused_5: Vec<u8>,
}

impl MobiHeader {
    /// Parse a Mobi header from the content. The reader must be advanced to the starting
    /// position of the Mobi header.
    pub(crate) fn parse<R: io::Read>(reader: &mut Reader<R>) -> io::Result<MobiHeader> {
        let identifier = reader.read_u32_be()?;
        let header_length = reader.read_u32_be()?;

        Ok(MobiHeader {
            identifier,
            header_length,
            mobi_type: reader.read_u32_be()?,
            text_encoding: reader.read_u32_be()?,
            id: reader.read_u32_be()?,
            gen_version: reader.read_u32_be()?,
            ortho_index: reader.read_u32_be()?,
            inflect_index: reader.read_u32_be()?,
            index_names: reader.read_u32_be()?,
            index_keys: reader.read_u32_be()?,
            extra_indices: [
                reader.read_u32_be()?,
                reader.read_u32_be()?,
                reader.read_u32_be()?,
                reader.read_u32_be()?,
                reader.read_u32_be()?,
                reader.read_u32_be()?,
            ],
            first_non_book_index: reader.read_u32_be()?,
            name_offset: reader.read_u32_be()?,
            name_length: reader.read_u32_be()?,
            unused: reader.read_u16_be()?,
            locale: reader.read_u8()?,
            language_code: reader.read_u8()?,
            input_language: reader.read_u32_be()?,
            output_language: reader.read_u32_be()?,
            format_version: reader.read_u32_be()?,
            first_image_index: reader.read_u32_be()?,
            first_huff_record: reader.read_u32_be()?,
            huff_record_count: reader.read_u32_be()?,
            huff_table_offset: reader.read_u32_be()?,
            huff_table_length: reader.read_u32_be()?,
            exth_flags: reader.read_u32_be()?,
            unused_0: {
                let mut bytes = [0; 32];
                reader.read_exact(&mut bytes)?;
                Box::new(bytes)
            },
            unused_1: reader.read_u32_be()?,
            drm_offset: reader.read_u32_be()?,
            drm_count: reader.read_u32_be()?,
            drm_size: reader.read_u32_be()?,
            drm_flags: reader.read_u32_be()?,
            unused_2: {
                let mut bytes = [0; 8];
                reader.read_exact(&mut bytes)?;
                Box::new(bytes)
            },
            first_content_record: reader.read_u16_be()?,
            last_content_record: reader.read_u16_be()?,
            unused_3: reader.read_u32_be()?,
            fcis_record: reader.read_u32_be()?,
            unused_4: reader.read_u32_be()?,
            flis_record: reader.read_u32_be()?,
            unused_5: {
                let mut unused = vec![0; header_length as usize - 196];
                reader.read_exact(&mut unused)?;
                unused
            },
        })
    }

    /// Parse a Mobi header from the content. The reader must be advanced to the starting
    /// position of the Mobi header.
    pub(crate) fn write<W: io::Write>(&self, w: &mut Writer<W>) -> io::Result<()> {
        w.write_be(self.identifier)?;
        w.write_be(self.header_length)?;
        w.write_be(self.mobi_type)?;
        w.write_be(self.text_encoding)?;
        w.write_be(self.id)?;
        w.write_be(self.gen_version)?;
        w.write_be(self.ortho_index)?;
        w.write_be(self.inflect_index)?;
        w.write_be(self.index_names)?;
        w.write_be(self.index_keys)?;
        for &i in &self.extra_indices {
            w.write_be(i)?;
        }
        w.write_be(self.first_non_book_index)?;
        w.write_be(self.name_offset)?;
        w.write_be(self.name_length)?;
        w.write_be(self.unused)?;
        w.write_be(self.locale)?;
        w.write_be(self.language_code)?;
        w.write_be(self.input_language)?;
        w.write_be(self.output_language)?;
        w.write_be(self.format_version)?;
        w.write_be(self.first_image_index)?;
        w.write_be(self.first_huff_record)?;
        w.write_be(self.huff_record_count)?;
        w.write_be(self.huff_table_offset)?;
        w.write_be(self.huff_table_length)?;
        w.write_be(self.exth_flags)?;
        w.write_be(self.unused_0.as_ref().as_ref())?;
        w.write_be(self.unused_1)?;
        w.write_be(self.drm_offset)?;
        w.write_be(self.drm_count)?;
        w.write_be(self.drm_size)?;
        w.write_be(self.drm_flags)?;
        w.write_be(self.unused_2.as_ref().as_ref())?;
        w.write_be(self.first_content_record)?;
        w.write_be(self.last_content_record)?;
        w.write_be(self.unused_3)?;
        w.write_be(self.fcis_record)?;
        w.write_be(self.unused_4)?;
        w.write_be(self.flis_record)?;
        w.write_be(self.unused_5.as_slice())
    }

    /// Checks if there is a Exth Header and changes the parameter
    pub fn has_exth_header(&self) -> bool {
        (self.exth_flags & EXTH_ON_FLAG) != 0
    }

    /// Checks if there is DRM on this book
    pub fn has_drm(&self) -> bool {
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
    use crate::book::MOBIHEADER;
    use crate::reader::Writer;
    use crate::{book, Mobi, TextEncoding};

    #[test]
    fn test_parse() {
        let mobiheader = MobiHeader {
            identifier: 1297039945,
            header_length: 232,
            mobi_type: 2,
            text_encoding: 65001,
            id: 3428045761,
            gen_version: 6,
            ortho_index: 0xFFFF_FFFF,
            inflect_index: 0xFFFF_FFFF,
            index_names: 0xFFFF_FFFF,
            index_keys: 0xFFFF_FFFF,
            extra_indices: [0xFFFF_FFFF; 6],
            first_non_book_index: 284,
            name_offset: 1360,
            name_length: 42,
            unused: 0,
            locale: 8,
            language_code: 9,
            input_language: 0,
            output_language: 0,
            format_version: 6,
            first_image_index: 287,
            first_huff_record: 0,
            huff_record_count: 0,
            huff_table_offset: 0,
            huff_table_length: 0,
            exth_flags: 80,
            drm_offset: 4294967295,
            drm_count: 0,
            drm_size: 0,
            drm_flags: 0,
            first_content_record: 1,
            last_content_record: 288,
            fcis_record: 290,
            flis_record: 289,
            unused_0: Box::new([0; 32]),
            unused_1: 0xFFFF_FFFF,
            unused_2: Box::new([0, 0, 0, 0, 0, 0, 0, 0]),
            unused_3: 1,
            unused_4: 1,
            unused_5: vec![
                0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255,
                255, 0, 0, 0, 7, 0, 0, 1, 28,
            ],
        };

        let mut reader = book::u8_reader(book::MOBIHEADER.to_vec());
        let test_header = MobiHeader::parse(&mut reader).unwrap();

        assert_eq!(mobiheader, test_header);
    }

    #[test]
    fn test_drm() {
        let mobiheader = MobiHeader {
            drm_offset: 1,
            ..Default::default()
        };

        assert!(mobiheader.has_drm());
    }

    #[test]
    fn test_no_drm() {
        let mobiheader = MobiHeader {
            drm_offset: 0xFFFF_FFFF,
            ..Default::default()
        };

        assert!(!mobiheader.has_drm());
    }

    #[test]
    fn test_write() {
        let input_bytes = book::MOBIHEADER.to_vec();

        let mobiheader = MobiHeader::parse(&mut book::u8_reader(input_bytes.clone())).unwrap();

        let mut output_bytes = vec![];
        assert!(mobiheader.write(&mut Writer::new(&mut output_bytes)).is_ok());
        assert_eq!(input_bytes.len(), output_bytes.len());
        assert_eq!(input_bytes, output_bytes);
    }

    mod text_encoding {
        use super::*;
        #[test]
        fn utf_8() {
            let m = MobiHeader {
                text_encoding: 65001,
                ..Default::default()
            };
            assert_eq!(m.text_encoding(), TextEncoding::UTF8)
        }
        #[test]
        fn win_latin1() {
            let m = MobiHeader {
                text_encoding: 1252,
                ..Default::default()
            };
            assert_eq!(m.text_encoding(), TextEncoding::CP1252)
        }
    }

    #[test]
    fn parses_mobi_types() {
        macro_rules! mtype {
            ($mt: expr, $s: expr) => {
                let m = MobiHeader {
                    mobi_type: $mt,
                    ..Default::default()
                };
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
                let m = MobiHeader {
                    language_code: $lc,
                    ..Default::default()
                };
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
