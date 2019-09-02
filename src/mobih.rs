//! A module about mobi header
use super::*;
macro_rules! return_or_err {
    ($x:expr) => {
        match $x {
            Ok(data) => data,
            Err(e) => return Err(e),
        }
    };
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
pub enum MobiHeaderData {
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
    HasDrm,
    DrmOffset,
    DrmCount,
    DrmSize,
    DrmFlags,
    LastImageRecord,
    FcisRecord,
    FlisRecord,
    ExtraBytes,
}
impl fmt::Display for MobiHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MOBI HEADER
Identifier:             {}
HeaderLength:           {}
Mobi type:              {}
Text encoding:          {}
Id:                     {}
Gen version:            v{}
First non book index:   {}
Name:                   {}
Name offset:            {}
Name length:            {}
Language:               {}
Input language:         {}
Output language:        {}
Format version:         {}
First image index:      {}
First huff record:      {}
Huff record count:      {}
First data record:      {}
Data record count:      {}
Exth flags:             {}
Has Exth header:        {}
Has DRM:                {}
DRM offset:             {}
DRM count:              {}
DRM size:               {}
DRM flags:              {}
Last image record:      {}
Fcis record:            {}
Flis record:            {}",
            self.identifier,
            self.header_length,
            self.mobi_type().unwrap_or(String::from("")),
            self.text_encoding().unwrap_or(String::from("")),
            self.id,
            self.gen_version,
            self.first_non_book_index,
            self.name,
            self.name_offset,
            self.name_length,
            self.language().unwrap_or(String::from("")),
            self.input_language,
            self.output_language,
            self.format_version,
            self.first_image_index,
            self.first_huff_record,
            self.huff_record_count,
            self.first_data_record,
            self.data_record_count,
            self.exth_flags,
            self.has_exth_header,
            self.has_drm,
            self.drm_offset,
            self.drm_count,
            self.drm_size,
            self.drm_flags,
            self.last_image_record,
            self.fcis_record,
            self.flis_record,
        )
    }
}
impl MobiHeader {
    /// Parse a Mobi header from the content
    pub fn parse(content: &[u8], num_of_records: u16) -> Result<MobiHeader, std::io::Error> {
        macro_rules! mobiheader {
            ($method:ident($enum:ident)) => {
                return_or_err!(MobiHeader::$method(
                    content,
                    MobiHeaderData::$enum,
                    num_of_records
                ))
            };
        }
        Ok(MobiHeader {
            identifier: mobiheader!(get_headers_u32(Identifier)),
            header_length: mobiheader!(get_headers_u32(HeaderLength)),
            mobi_type: mobiheader!(get_headers_u32(MobiType)),
            text_encoding: mobiheader!(get_headers_u32(TextEncoding)),
            id: mobiheader!(get_headers_u32(Id)),
            gen_version: mobiheader!(get_headers_u32(GenVersion)),
            first_non_book_index: mobiheader!(get_headers_u32(FirstNonBookIndex)),
            name: return_or_err!(MobiHeader::name(content, num_of_records)),
            name_offset: mobiheader!(get_headers_u32(NameOffset)),
            name_length: mobiheader!(get_headers_u32(NameLength)),
            language_code: MobiHeader::lang_code(mobiheader!(get_headers_u32(LanguageCode))),
            input_language: mobiheader!(get_headers_u32(InputLanguage)),
            output_language: mobiheader!(get_headers_u32(OutputLanguage)),
            format_version: mobiheader!(get_headers_u32(FormatVersion)),
            first_image_index: mobiheader!(get_headers_u32(FirstImageIndex)),
            first_huff_record: mobiheader!(get_headers_u32(FirstHuffRecord)),
            huff_record_count: mobiheader!(get_headers_u32(HuffRecordCount)),
            first_data_record: mobiheader!(get_headers_u32(FirstDataRecord)),
            data_record_count: mobiheader!(get_headers_u32(DataRecordCount)),
            exth_flags: mobiheader!(get_headers_u32(ExthFlags)),
            has_exth_header: MobiHeader::has_exth_header(mobiheader!(get_headers_u32(ExthFlags))),
            drm_offset: mobiheader!(get_headers_u32(DrmOffset)),
            drm_count: mobiheader!(get_headers_u32(DrmCount)),
            drm_size: mobiheader!(get_headers_u32(DrmSize)),
            drm_flags: mobiheader!(get_headers_u32(DrmFlags)),
            has_drm: MobiHeader::has_drm(mobiheader!(get_headers_u32(DrmOffset))),
            last_image_record: mobiheader!(get_headers_u16(LastImageRecord)),
            fcis_record: mobiheader!(get_headers_u32(FcisRecord)),
            flis_record: mobiheader!(get_headers_u32(FlisRecord)),
            extra_bytes: return_or_err!(MobiHeader::extra_bytes(content, num_of_records)),
        })
    }
    /// Gets u32 header value from specific location
    fn get_headers_u32(
        content: &[u8],
        mheader: MobiHeaderData,
        num_of_records: u16,
    ) -> Result<u32, std::io::Error> {
        let mut reader = Cursor::new(content);
        let position = match mheader {
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
            MobiHeaderData::FcisRecord => 280,
            MobiHeaderData::FlisRecord => 288,
            _ => 0,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u32::<BigEndian>()
    }
    /// Gets u16 header value from specific location
    fn get_headers_u16(
        content: &[u8],
        mheader: MobiHeaderData,
        num_of_records: u16,
    ) -> Result<u16, std::io::Error> {
        let mut reader = Cursor::new(content);
        let position = match mheader {
            MobiHeaderData::LastImageRecord => 274,
            _ => 0,
        };
        reader.set_position(position + u64::from(num_of_records * 8));
        reader.read_u16::<BigEndian>()
    }
    /// Returns the book name
    pub fn name(content: &[u8], num_of_records: u16) -> Result<String, std::io::Error> {
        let name_offset = return_or_err!(MobiHeader::get_headers_u32(
            content,
            MobiHeaderData::NameOffset,
            num_of_records
        ));
        let name_length = return_or_err!(MobiHeader::get_headers_u32(
            content,
            MobiHeaderData::NameLength,
            num_of_records
        ));
        let offset = name_offset as usize + (num_of_records * 8) as usize + 80;
        Ok(
            String::from_utf8_lossy(&content[offset..offset + name_length as usize])
                .to_owned()
                .to_string(),
        )
    }
    /// Checks if there is a Exth Header and changes the parameter
    pub fn has_exth_header(exth_flags: u32) -> bool {
        (exth_flags & 0x40) != 0
    }
    /// Checks if there is DRM on this book
    fn has_drm(drm_offset: u32) -> bool {
        drm_offset != 0xFFFF_FFFF
    }
    /// Returns extra bytes for reading records
    fn extra_bytes(content: &[u8], num_of_records: u16) -> Result<u32, std::io::Error> {
        let ex_bytes = return_or_err!(MobiHeader::get_headers_u16(
            content,
            MobiHeaderData::ExtraBytes,
            num_of_records
        ));
        Ok(2 * (ex_bytes & 0xFFFE).count_ones())
    }
    /// Converts numerical value into a type
    pub fn mobi_type(&self) -> Option<String> {
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
    pub fn text_encoding(&self) -> Option<String> {
        match self.text_encoding {
            1252 => Some(String::from("CP1252 (WinLatin1)")),
            65001 => Some(String::from("UTF-8")),
            _ => None,
        }
    }
    fn lang_code(code: u32) -> u16 {
        (code & 0xFF) as u16
    }
    pub fn language(&self) -> Option<String> {
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