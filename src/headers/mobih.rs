use crate::writer::WriteBeBytes;
use crate::{Reader, Writer};

use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MobiHeaderParseError {
    #[error("MobiHeader length is less than 232 bytes")]
    MobiHeaderTooSmall,
    #[error("Expected header to be identifier as MOBI")]
    InvalidIdentifier,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

const DRM_ON_FLAG: u32 = 0xFFFF_FFFF;
const EXTH_ON_FLAG: u32 = 0x40;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum MobiType {
    MobiPocketBook,
    PalmDocBook,
    Audio,
    News,
    NewsFeed,
    NewsMagazine,
    PICS,
    WORD,
    XLS,
    PPT,
    TEXT,
    HTML,
    Unknown,
}

impl Default for MobiType {
    fn default() -> Self {
        MobiType::Unknown
    }
}

impl From<u32> for MobiType {
    fn from(ty: u32) -> Self {
        use MobiType::*;
        match ty {
            2 => MobiPocketBook,
            3 => PalmDocBook,
            4 => Audio,
            257 => News,
            258 => NewsFeed,
            259 => NewsMagazine,
            513 => PICS,
            514 => WORD,
            515 => XLS,
            516 => PPT,
            517 => TEXT,
            518 => HTML,
            _ => Unknown,
        }
    }
}

impl From<MobiType> for u32 {
    fn from(ty: MobiType) -> u32 {
        use MobiType::*;
        match ty {
            MobiPocketBook => 2,
            PalmDocBook => 3,
            Audio => 4,
            News => 257,
            NewsFeed => 258,
            NewsMagazine => 259,
            PICS => 513,
            WORD => 514,
            XLS => 515,
            PPT => 516,
            TEXT => 517,
            HTML => 518,
            Unknown => 0,
        }
    }
}

impl WriteBeBytes for MobiType {
    fn write_be_bytes<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        u32::from(*self).write_be_bytes(writer)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TextEncoding {
    CP1252,
    UTF8,
    Unknown(u32),
}

impl Default for TextEncoding {
    fn default() -> Self {
        TextEncoding::UTF8
    }
}

impl From<u32> for TextEncoding {
    fn from(num: u32) -> Self {
        match num {
            1252 => TextEncoding::CP1252,
            65001 => TextEncoding::UTF8,
            n => TextEncoding::Unknown(n),
        }
    }
}

impl From<TextEncoding> for u32 {
    fn from(encoding: TextEncoding) -> Self {
        match encoding {
            TextEncoding::CP1252 => 1252,
            TextEncoding::UTF8 => 65001,
            TextEncoding::Unknown(n) => n,
        }
    }
}

impl WriteBeBytes for TextEncoding {
    fn write_be_bytes<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        u32::from(*self).write_be_bytes(writer)
    }
}

#[derive(Debug, PartialEq, Eq)]
/// Strcture that holds Mobi header information
pub struct MobiHeader {
    pub identifier: u32,
    pub header_length: u32,
    pub mobi_type: MobiType,
    pub text_encoding: TextEncoding,
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
    pub language_code: Language,
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
    unused_4: u32, // fcis record count?
    pub flis_record: u32,
    unused_5: u32, // flis record count?
    unused_6: u64,
    unused_7: u32,
    first_compilation_data_section_count: u32,
    data_section_count: u32,
    unused_8: u32,
    extra_record_data_flags: u32,
    pub first_index_record: u32,
    unused_9: Vec<u8>,
}

impl Default for MobiHeader {
    fn default() -> Self {
        MobiHeader {
            identifier: 0,
            header_length: 0,
            mobi_type: MobiType::default(),
            text_encoding: TextEncoding::default(),
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
            language_code: Language::default(),
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
            unused_5: 1,
            unused_6: 0,
            unused_7: 0xFFFF_FFFF,
            first_compilation_data_section_count: 0,
            data_section_count: 0xFFFF_FFFF,
            unused_8: 0xFFFF_FFFF,
            extra_record_data_flags: 0,
            first_index_record: 0xFFFF_FFFF,
            unused_9: vec![],
        }
    }
}

impl MobiHeader {
    /// Parse a Mobi header from the content. The reader must be advanced to the starting
    /// position of the Mobi header.
    pub(crate) fn parse<R: io::Read>(
        reader: &mut Reader<R>,
    ) -> Result<MobiHeader, MobiHeaderParseError> {
        let identifier = reader.read_u32_be()?;
        if &identifier.to_be_bytes() != b"MOBI" {
            return Err(MobiHeaderParseError::InvalidIdentifier);
        }
        let header_length = reader.read_u32_be()?;
        if header_length < 232 {
            return Err(MobiHeaderParseError::MobiHeaderTooSmall);
        }
        Ok(MobiHeader {
            identifier,
            header_length,
            mobi_type: reader.read_u32_be()?.into(),
            text_encoding: reader.read_u32_be()?.into(),
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
            language_code: reader.read_u8()?.into(),
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
            unused_5: reader.read_u32_be()?,
            unused_6: reader.read_u64_be()?,
            unused_7: reader.read_u32_be()?,
            first_compilation_data_section_count: reader.read_u32_be()?,
            data_section_count: reader.read_u32_be()?,
            unused_8: reader.read_u32_be()?,
            extra_record_data_flags: reader.read_u32_be()?,
            first_index_record: reader.read_u32_be()?,
            unused_9: { reader.read_vec_header(header_length as usize - 232)? },
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
        w.write_be(self.unused_5)?;
        w.write_be(self.unused_6)?;
        w.write_be(self.unused_7)?;
        w.write_be(self.first_compilation_data_section_count)?;
        w.write_be(self.data_section_count)?;
        w.write_be(self.unused_8)?;
        w.write_be(self.extra_record_data_flags)?;
        w.write_be(self.first_index_record)?;
        w.write_be(self.unused_9.as_slice())
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
    pub fn mobi_type(&self) -> MobiType {
        self.mobi_type
    }

    // Mobi format only specifies this two encodings so
    // this should never panic
    pub fn text_encoding(&self) -> TextEncoding {
        self.text_encoding
    }

    pub fn language(&self) -> Language {
        self.language_code
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Language {
    Neutral,
    Afrikaans,
    Albanian,
    Arabic,
    Armenian,
    Assamese,
    Azeri,
    Basque,
    Belarusian,
    Bengali,
    Bulgarian,
    Catalan,
    Chinese,
    Czech,
    Danish,
    Dutch,
    English,
    Estonian,
    Faeroese,
    Farsi,
    Finnish,
    French,
    Georgian,
    German,
    Greek,
    Gujarati,
    Hebrew,
    Hindi,
    Hungarian,
    Icelandic,
    Indonesian,
    Italian,
    Japanese,
    Kannada,
    Kazak,
    Konkani,
    Korean,
    Latvian,
    Lithuanian,
    Macedonian,
    Malay,
    Malayalam,
    Maltese,
    Marathi,
    Nepali,
    Norwegian,
    Oriya,
    Polish,
    Portuguese,
    Punjabi,
    Rhaetoromanic,
    Romanian,
    Russian,
    Sami,
    Sanskrit,
    Serbian,
    Slovak,
    Slovenian,
    Sorbian,
    Spanish,
    Sutu,
    Swahili,
    Swedish,
    Tamil,
    Tatar,
    Telugu,
    Thai,
    Tsonga,
    Tswana,
    Turkish,
    Ukrainian,
    Urdu,
    Uzbek,
    Vietnamese,
    Xhosa,
    Zulu,
    Unknown,
}

impl From<u8> for Language {
    fn from(code: u8) -> Self {
        use Language::*;
        match code {
            0 => Neutral,
            54 => Afrikaans,
            28 => Albanian,
            1 => Arabic,
            43 => Armenian,
            77 => Assamese,
            44 => Azeri,
            45 => Basque,
            35 => Belarusian,
            69 => Bengali,
            2 => Bulgarian,
            3 => Catalan,
            4 => Chinese,
            5 => Czech,
            6 => Danish,
            19 => Dutch,
            9 => English,
            37 => Estonian,
            56 => Faeroese,
            41 => Farsi,
            11 => Finnish,
            12 => French,
            55 => Georgian,
            7 => German,
            8 => Greek,
            71 => Gujarati,
            13 => Hebrew,
            57 => Hindi,
            14 => Hungarian,
            15 => Icelandic,
            33 => Indonesian,
            16 => Italian,
            17 => Japanese,
            75 => Kannada,
            63 => Kazak,
            87 => Konkani,
            18 => Korean,
            38 => Latvian,
            39 => Lithuanian,
            47 => Macedonian,
            62 => Malay,
            76 => Malayalam,
            58 => Maltese,
            78 => Marathi,
            97 => Nepali,
            20 => Norwegian,
            72 => Oriya,
            21 => Polish,
            22 => Portuguese,
            70 => Punjabi,
            23 => Rhaetoromanic,
            24 => Romanian,
            25 => Russian,
            59 => Sami,
            79 => Sanskrit,
            26 => Serbian,
            27 => Slovak,
            36 => Slovenian,
            46 => Sorbian,
            10 => Spanish,
            48 => Sutu,
            65 => Swahili,
            29 => Swedish,
            73 => Tamil,
            68 => Tatar,
            74 => Telugu,
            30 => Thai,
            49 => Tsonga,
            50 => Tswana,
            31 => Turkish,
            34 => Ukrainian,
            32 => Urdu,
            67 => Uzbek,
            42 => Vietnamese,
            52 => Xhosa,
            53 => Zulu,
            _ => Unknown,
        }
    }
}

impl From<Language> for u8 {
    fn from(lang: Language) -> Self {
        use Language::*;
        match lang {
            Neutral => 0,
            Afrikaans => 54,
            Albanian => 28,
            Arabic => 1,
            Armenian => 43,
            Assamese => 77,
            Azeri => 44,
            Basque => 45,
            Belarusian => 35,
            Bengali => 69,
            Bulgarian => 2,
            Catalan => 3,
            Chinese => 4,
            Czech => 5,
            Danish => 6,
            Dutch => 19,
            English => 9,
            Estonian => 37,
            Faeroese => 56,
            Farsi => 41,
            Finnish => 11,
            French => 12,
            Georgian => 55,
            German => 7,
            Greek => 8,
            Gujarati => 71,
            Hebrew => 13,
            Hindi => 57,
            Hungarian => 14,
            Icelandic => 15,
            Indonesian => 33,
            Italian => 16,
            Japanese => 17,
            Kannada => 75,
            Kazak => 63,
            Konkani => 87,
            Korean => 18,
            Latvian => 38,
            Lithuanian => 39,
            Macedonian => 47,
            Malay => 62,
            Malayalam => 76,
            Maltese => 58,
            Marathi => 78,
            Nepali => 97,
            Norwegian => 20,
            Oriya => 72,
            Polish => 21,
            Portuguese => 22,
            Punjabi => 70,
            Rhaetoromanic => 23,
            Romanian => 24,
            Russian => 25,
            Sami => 59,
            Sanskrit => 79,
            Serbian => 26,
            Slovak => 27,
            Slovenian => 36,
            Sorbian => 46,
            Spanish => 10,
            Sutu => 48,
            Swahili => 65,
            Swedish => 29,
            Tamil => 73,
            Tatar => 68,
            Telugu => 74,
            Thai => 30,
            Tsonga => 49,
            Tswana => 50,
            Turkish => 31,
            Ukrainian => 34,
            Urdu => 32,
            Uzbek => 67,
            Vietnamese => 42,
            Xhosa => 52,
            Zulu => 53,
            Unknown => u8::MAX,
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::Neutral
    }
}

impl WriteBeBytes for Language {
    fn write_be_bytes<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        u8::from(*self).write_be_bytes(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::{Language, MobiHeader, MobiType, TextEncoding};
    use crate::book;
    use crate::writer::Writer;

    #[test]
    fn test_parse() {
        let mobiheader = MobiHeader {
            identifier: 1297039945,
            header_length: 232,
            mobi_type: MobiType::MobiPocketBook,
            text_encoding: TextEncoding::UTF8,
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
            language_code: Language::English,
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
            unused_5: 1,
            unused_6: 0,
            unused_7: 0xFFFF_FFFF,
            first_compilation_data_section_count: 0,
            data_section_count: 0xFFFF_FFFF,
            unused_8: 0xFFFF_FFFF,
            extra_record_data_flags: 7,
            first_index_record: 284,
            unused_9: vec![],
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
        assert!(mobiheader
            .write(&mut Writer::new(&mut output_bytes))
            .is_ok());
        assert_eq!(input_bytes.len(), output_bytes.len());
        assert_eq!(input_bytes, output_bytes);
    }
}
