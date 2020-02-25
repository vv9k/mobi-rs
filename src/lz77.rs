use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
pub fn decompress_lz77(data: &[u8]) -> Result<String, std::io::Error> {
    let length = data.len();
    let mut reader = Cursor::new(data);
    let mut offset: usize = 0;
    let mut text: Vec<char> = vec![];
    while offset < length {
        let byte = data[offset];
        offset += 1;
        match byte {
            // The first character is a null which are literal
            // Chars from range 0x09..=0x7f are also literal
            0x0 | 0x09..=0x7f => {
                text.push(byte as char);
            }
            // next $byte bytes are also literal
            0x1..=0x8 => {
                if offset + byte as usize <= length {
                    &data[offset..(offset + byte as usize)]
                        .iter()
                        .for_each(|ch| {
                            text.push(*ch as char);
                        });
                    offset += byte as usize;
                }
            }
            // Data is LZ77-compressed
            0x80..=0xbf => {
                offset += 1;
                if offset > length {
                    return Ok(text.iter().collect::<String>());
                }
                reader.set_position((offset - 2) as u64);
                let mut lz77 = reader.read_u16::<BigEndian>().unwrap();

                lz77 &= 0x3fff; // Leftmost two bits are ID bits and need to be dropped
                let lz77length = (lz77 & 0x0007) + 3; // Length is  rightmost three bits + 3
                let lz77offset = lz77 >> 3; // Remaining 11 bits are offset

                if lz77offset < 1 {
                    // Decompression offset is invalid?
                    return Ok(text.iter().collect::<String>());
                }

                for _lz77pos in 0..lz77length {
                    let mut textpos: usize = text.len();
                    if textpos >= lz77offset as usize {
                        textpos -= lz77offset as usize;
                    } else {
                        break;
                    }
                    let ch = &text[textpos..=textpos].to_owned();
                    for c in ch {
                        text.push(*c);
                    }
                }
            }
            // 0xc0..= 0xff are single charaters XOR 0x80 preceded by a space
            _ => {
                text.push(' ');
                text.push((byte ^ 0x80) as char);
            }
        }
    }
    Ok(text.iter().collect::<String>())
}
