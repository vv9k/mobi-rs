#![allow(dead_code)]
use crate::Reader;

use std::fmt;

type HuffmanResult<T> = Result<T, HuffmanError>;

#[derive(Debug)]
pub enum HuffmanError {
    IoError(std::io::Error),
    CodeLenOutOfBounds,
    BadTerm,
    InvalidHuffHeader,
    InvalidCDICHeader,
    InvalidDictionaryIndex,
}

impl fmt::Display for HuffmanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "todo...")
    }
}

impl std::error::Error for HuffmanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            HuffmanError::IoError(error) => error.source(),
            _ => None,
        }
    }
}

impl From<std::io::Error> for HuffmanError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

type HuffmanDictionary = Vec<Option<(Vec<u8>, bool)>>;
type CodeDictionary = [(u8, bool, u32); 256];
type MinCodesMapping = [u32; 33];
type MaxCodesMapping = [u32; 33];

#[derive(Debug)]
struct HuffmanDecoder {
    dictionary: HuffmanDictionary,
    code_dict: CodeDictionary,
    min_codes: MinCodesMapping,
    max_codes: MaxCodesMapping,
}

impl Default for HuffmanDecoder {
    fn default() -> Self {
        Self {
            dictionary: vec![],
            code_dict: [(0, false, 0); 256],
            min_codes: [0; 33],
            max_codes: [u32::MAX; 33],
        }
    }
}

impl HuffmanDecoder {
    fn load_code_dictionary<R: std::io::Read>(
        &mut self,
        reader: &mut Reader<R>,
        offset: usize,
    ) -> HuffmanResult<()> {
        reader.set_position(offset)?;

        for code in self.code_dict.iter_mut() {
            let v = reader.read_u32_be()?;
            // 0 < code_len <= 32, term is T or F, max_code is u24 pretending to be u32.
            let (code_len, term, mut max_code) = ((v & 0x1F) as u8, (v & 0x80) == 0x80, v >> 8);
            if code_len == 0 {
                return Err(HuffmanError::CodeLenOutOfBounds);
            }
            if code_len <= 8 && !term {
                return Err(HuffmanError::BadTerm);
            }
            max_code = ((max_code + 1) << (32u8.saturating_sub(code_len))).saturating_sub(1);
            *code = (code_len, term, max_code);
        }

        Ok(())
    }

    fn load_min_max_codes<R: std::io::Read>(
        &mut self,
        reader: &mut Reader<R>,
        offset: usize,
    ) -> HuffmanResult<()> {
        reader.set_position(offset)?;

        for code_len in 1..=32 {
            self.min_codes[code_len] = reader.read_u32_be()? << (32 - code_len);
            self.max_codes[code_len] =
                ((reader.read_u32_be()? + 1) << (32 - code_len)).saturating_sub(1);
        }
        Ok(())
    }

    // Loads the code dictionary, min and max code values from the HUFF record
    fn load_huff(&mut self, huff: &[u8]) -> HuffmanResult<()> {
        let mut r = Reader::new(std::io::Cursor::new(huff));

        if &r.read_u32_be()?.to_be_bytes() != b"HUFF" || r.read_u32_be()? != 0x18 {
            return Err(HuffmanError::InvalidHuffHeader);
        }

        let cache_offset = r.read_u32_be()?;
        let base_offset = r.read_u32_be()?;

        self.load_code_dictionary(&mut r, cache_offset as usize)?;
        self.load_min_max_codes(&mut r, base_offset as usize)?;

        Ok(())
    }

    // Loads a CDIC record into the huffman dictionary
    fn load_cdic_record(&mut self, cdic: &[u8]) -> HuffmanResult<()> {
        let mut r = Reader::new(std::io::Cursor::new(cdic));

        if &r.read_u32_be()?.to_be_bytes() != b"CDIC" || r.read_u32_be()? != 0x10 {
            return Err(HuffmanError::InvalidCDICHeader);
        }

        let num_phrases = r.read_u32_be()?;
        let bits = r.read_u32_be()?;

        let n = (1 << bits).min(num_phrases - self.dictionary.len() as u32);

        let mut offsets = Vec::with_capacity(n as usize);
        for _ in 0..n {
            offsets.push(r.read_u16_be()?);
        }

        for offset in offsets {
            r.set_position(16 + offset as usize)?;
            let num_bytes = r.read_u16_be()?;
            let bytes = {
                let mut slice = vec![0; (num_bytes as usize) & 0x7FFF];
                r.read_exact(&mut slice)?;
                slice
            };
            self.dictionary
                .push(Some((bytes, (num_bytes & 0x8000) == 0x8000)));
        }

        Ok(())
    }

    fn load_cdic_records(&mut self, records: &[&[u8]]) -> HuffmanResult<()> {
        for cdic in records {
            self.load_cdic_record(cdic)?;
        }
        Ok(())
    }

    // Unpacks data of a section (?)
    fn unpack(&mut self, data: &[u8]) -> HuffmanResult<Vec<u8>> {
        // Need len.
        let mut bits_left = data.len() * 8;

        let mut r = Reader::new(std::io::Cursor::new(&data));

        // X is a sliding window of 64 bits from data.
        let mut x = r.read_u64_be()?;
        // -32 < n <= 32
        let mut n = 32i8;
        let mut unpacked = vec![];

        loop {
            // The top 32 bits are now stale, read next 32 bits.
            if n <= 0 {
                // Can not read another 32 bits.
                if bits_left < 32 {
                    // Can read up to 3 bytes.
                    for _ in 0..bits_left / 8 {
                        x = (x << 8) | u64::from(r.read_u8()?);
                    }
                    // Pad last bits with 0.
                    x <<= 32 - bits_left;
                } else {
                    x = (x << 32) | u64::from(r.read_u32_be()?);
                }
                n += 32;
            }

            // Read maximum of 32 bits from x.
            let code = (x >> n) as u32;
            // Get value from dict1.
            let (code_len, term, mut max_code) = self.code_dict[(code >> 24) as usize];

            // 32 > code_len > 0.
            let mut code_len = code_len as usize;
            if !term {
                // Last min_code is guaranteed to be 0, so no unwrap.
                code_len += self.min_codes[code_len..]
                    .iter()
                    .position(|&min_code| code >= min_code)
                    .unwrap();
                max_code = self.max_codes[code_len];
            }

            let index = ((max_code - code) >> (32 - code_len)) as usize;
            println!(
                "max_code: {}, code: {}, code_len: {}, index: {}, dict_len: {}",
                max_code,
                code,
                code_len,
                index,
                self.dictionary.len()
            );
            let (mut slice, flag) = std::mem::take(
                self.dictionary
                    .get_mut(index)
                    .ok_or(HuffmanError::InvalidDictionaryIndex)?,
            )
            .ok_or(HuffmanError::InvalidDictionaryIndex)?;
            if !flag {
                slice = self.unpack(&slice)?;
            }
            unpacked.extend_from_slice(&slice);
            self.dictionary[index] = Some((slice, true));

            // code_len <= 32, so this is safe.
            n -= code_len as i8;
            bits_left = match bits_left.checked_sub(code_len) {
                // No more bits left to read.
                None | Some(0) => break,
                Some(i) => i,
            };
        }

        println!("unpacked: {:?}", unpacked);

        Ok(unpacked)
    }

    fn unpack_sections(&mut self, sections: &[&[u8]]) -> HuffmanResult<Vec<Vec<u8>>> {
        let mut output = vec![];
        for section in sections {
            output.push(self.unpack(section)?);
        }
        Ok(output)
    }

    fn init(huffs: &[&[u8]]) -> HuffmanResult<Self> {
        let mut decoder = Self::default();
        decoder.load_huff(huffs[0])?;
        decoder.load_cdic_records(&huffs[1..])?;
        eprintln!("{:#?}", decoder);
        Ok(decoder)
    }
}

pub fn decompress(huffs: &[&[u8]], sections: &[&[u8]]) -> HuffmanResult<Vec<Vec<u8>>> {
    let mut decoder = HuffmanDecoder::init(huffs)?;
    decoder.unpack_sections(sections)
}
