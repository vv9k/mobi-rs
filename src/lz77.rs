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
        if byte == 0 {
            text.push(byte as char);
        } else if byte <= 8 {
            if offset + byte as usize <= length {
                for ch in &data[offset..(offset + byte as usize)] {
                    text.push(*ch as char);
                }
                offset += byte as usize;
            }
        } else if byte <= 0x7f {
            text.push(byte as char);
        } else if byte <= 0xbf {
            offset += 1;
            if offset > length {
                return Ok(text.iter().collect::<String>());
            }
            reader.set_position((offset - 2) as u64);
            let mut lz77 = reader.read_u16::<BigEndian>().unwrap();
            lz77 &= 0x3fff;
            let lz77length = (lz77 & 0x0007) + 3;
            let lz77offset = lz77 >> 3;

            if lz77offset < 1 {
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
        } else {
            text.push(' ');
            text.push((byte ^ 0x80) as char);
        }
    }
    Ok(text.iter().collect::<String>())
}
