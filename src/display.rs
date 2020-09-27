#![cfg(feature = "fmt")]
use super::{ExtHeader, Header, Mobi, MobiHeader, PalmDocHeader, TextEncoding};
use std::fmt;

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

impl fmt::Display for PalmDocHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PALMDOC HEADER
Compression:            {}
Text length:            {}
Record count:           {}
Record size:            {}
Encryption type:        {}",
            self.compression().unwrap_or_default(),
            self.text_length,
            self.record_count,
            self.record_size,
            self.encryption().unwrap_or_default(),
        )
    }
}

impl fmt::Display for Header {
    #[cfg(feature = "time")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HEADER
Name:                   {}
Attributes:             {}
Version:                {}
Created:                {}
Modified:               {}
Backup:                 {}
Modnum:                 {}
App_info_id:            {}
Sort_info_id:           {}
Typ_e:                  {}
Creator:                {}
Unique_id_seed:         {}
Next_record_list_id:    {}
Num_of_records:         {}",
            self.name,
            self.attributes,
            self.version,
            self.created_datetime(),
            self.mod_datetime(),
            self.backup,
            self.modnum,
            self.app_info_id,
            self.sort_info_id,
            self.typ_e,
            self.creator,
            self.unique_id_seed,
            self.next_record_list_id,
            self.num_of_records,
        )
    }
    #[cfg(not(feature = "time"))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HEADER
Name:                   {}
Attributes:             {}
Version:                {}
Backup:                 {}
Modnum:                 {}
App_info_id:            {}
Sort_info_id:           {}
Typ_e:                  {}
Creator:                {}
Unique_id_seed:         {}
Next_record_list_id:    {}
Num_of_records:         {}",
            self.name,
            self.attributes,
            self.version,
            self.backup,
            self.modnum,
            self.app_info_id,
            self.sort_info_id,
            self.typ_e,
            self.creator,
            self.unique_id_seed,
            self.next_record_list_id,
            self.num_of_records,
        )
    }
}

impl fmt::Display for TextEncoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TextEncoding::CP1252 => write!(f, "CP1252 (WINLATIN)"),
            TextEncoding::UTF8 => write!(f, "UTF-8"),
        }
    }
}

#[cfg(feature = "fmt")]
impl fmt::Display for MobiHeader {
    #[allow(clippy::or_fun_call)]
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
            self.mobi_type().unwrap_or_default(),
            self.text_encoding(),
            self.id,
            self.gen_version,
            self.first_non_book_index,
            self.name,
            self.name_offset,
            self.name_length,
            self.language().unwrap_or_default(),
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

impl fmt::Display for Mobi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let empty_str = String::from("");
        write!(
            f,
            "
------------------------------------------------------------------------------------
Title:                  {}
Author:                 {}
Publisher:              {}
Description:            {}
ISBN:                   {}
Publish Date:           {}
Contributor:            {}
------------------------------------------------------------------------------------
{}
------------------------------------------------------------------------------------
{}
------------------------------------------------------------------------------------
{}
------------------------------------------------------------------------------------
{}
------------------------------------------------------------------------------------",
            self.title().unwrap_or(&empty_str),
            self.author().unwrap_or(&empty_str),
            self.publisher().unwrap_or(&empty_str),
            self.description().unwrap_or(&empty_str),
            self.isbn().unwrap_or(&empty_str),
            self.publish_date().unwrap_or(&empty_str),
            self.contributor().unwrap_or(&empty_str),
            self.header,
            self.palmdoc,
            self.mobi,
            self.exth,
        )
    }
}
