#![allow(dead_code)]
use crate::Reader;

type HuffmanResult<T> = Result<T, HuffmanError>;

pub enum HuffmanError {
    IoError(std::io::Error),
    CodeLenOutOfBounds,
    BadTerm,
    InvalidHuffHeader,
    InvalidCDICHeader,
    InvalidDictionaryIndex,
}

impl From<std::io::Error> for HuffmanError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

struct CodeDictionary([(u8, bool, u32); 256]);

struct HuffmanDecoder {
    dictionary: Vec<Option<(Vec<u8>, bool)>>,
    code_dict: [(u8, bool, u32); 256],
    min_codes: [u32; 33],
    max_codes: [u32; 33],
}

fn load_huff(huff: &[u8]) -> HuffmanResult<([(u8, bool, u32); 256], [u32; 33], [u32; 33])> {
    let mut r = Reader::new(std::io::Cursor::new(huff));

    if r.read_u32_be()? != u32::from_be_bytes(*b"HUFF") || r.read_u32_be()? != 0x18 {
        return Err(HuffmanError::InvalidHuffHeader);
    }

    let cache_offset = r.read_u32_be()?;
    let base_offset = r.read_u32_be()?;

    r.set_position(cache_offset as usize)?;

    let mut code_dict = [(0, false, 0); 256];
    for code in code_dict.iter_mut() {
        let v = r.read_u32_be()?;
        // 0 < code_len <= 32, term is T or F, max_code is u24 pretending to be u32.
        let (code_len, term, mut max_code) = ((v & 0x1F) as u8, (v & 0x80) == 0x80, v >> 8);
        if code_len == 0 {
            return Err(HuffmanError::CodeLenOutOfBounds);
        }
        if code_len <= 8 && !term {
            return Err(HuffmanError::BadTerm);
        }
        max_code = ((max_code + 1) << (32 - code_len)) - 1;
        *code = (code_len, term, max_code);
    }

    r.set_position(base_offset as usize)?;

    // First value is ignored, since code_len > 0.
    let mut min_codes = [0; 33];
    let mut max_codes = [u32::max_value(); 33];

    // Fill all other values.
    for code_len in 1..=32 {
        min_codes[code_len] = r.read_u32_be()? << (32 - code_len);
        max_codes[code_len] = ((r.read_u32_be()? + 1) << (32 - code_len)).wrapping_sub(1);
    }

    Ok((code_dict, min_codes, max_codes))
}

fn load_cdic(cdic: &[u8], dictionary: &mut Vec<Option<(Vec<u8>, bool)>>) -> HuffmanResult<()> {
    let mut r = Reader::new(std::io::Cursor::new(cdic));

    if r.read_u32_be()? != u32::from_be_bytes(*b"CDIC") || r.read_u32_be()? != 0x10 {
        return Err(HuffmanError::InvalidCDICHeader);
    }

    let num_phrases = r.read_u32_be()?;
    let bits = r.read_u32_be()?;

    let n = (1 << bits).min(num_phrases - dictionary.len() as u32);

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
        dictionary.push(Some((bytes, (num_bytes & 0x8000) == 0x8000)));
    }

    Ok(())
}

fn unpack(
    data: &[u8],
    dictionary: &mut [Option<(Vec<u8>, bool)>],
    code_dict: &[(u8, bool, u32); 256],
    min_codes: &[u32; 33],
    max_codes: &[u32; 33],
) -> HuffmanResult<Vec<u8>> {
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
        let (code_len, term, mut max_code) = code_dict[(code >> 24) as usize];

        // 32 > code_len > 0.
        let mut code_len = code_len as usize;
        if !term {
            // Last min_code is guaranteed to be 0, so no unwrap.
            code_len += min_codes[code_len..]
                .iter()
                .position(|&min_code| code >= min_code)
                .unwrap();
            max_code = max_codes[code_len];
        }

        let index = ((max_code - code) >> (32 - code_len)) as usize;
        let (mut slice, flag) = std::mem::take(dictionary.get_mut(index).ok_or(HuffmanError::InvalidDictionaryIndex)?)
            .ok_or(HuffmanError::InvalidDictionaryIndex)?;
        if !flag {
            slice = unpack(&slice, dictionary, code_dict, min_codes, max_codes)?;
        }
        unpacked.extend_from_slice(&slice);
        dictionary[index] = Some((slice, true));

        // code_len <= 32, so this is safe.
        n -= code_len as i8;
        bits_left = match bits_left.checked_sub(code_len) {
            // No more bits left to read.
            None | Some(0) => break,
            Some(i) => i,
        };
    }

    Ok(unpacked)
}

fn decompress(huffs: &[&[u8]], sections: &[&[u8]]) -> HuffmanResult<Vec<Vec<u8>>> {
    let (dict1, min_code, max_code) = load_huff(&huffs[0])?;
    let mut dictionary = Vec::new();
    for huff in huffs[1..].iter() {
        load_cdic(huff, &mut dictionary)?;
    }

    let mut output = Vec::new();
    for section in sections {
        output.push(unpack(section, &mut dictionary, &dict1, &min_code, &max_code)?);
    }
    Ok(output)
}
