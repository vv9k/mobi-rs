use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
pub fn decompress_lz77(data: &[u8]) -> Result<String, std::io::Error> {
    let length = data.len();
    let mut reader = Cursor::new(data);
    let mut offset: u64 = 0;
    let mut text: Vec<char> = vec![];
    while offset < length as u64 {
        let byte = data[offset as usize];
        offset += 1;
        if byte == 0 {
            text.push(byte as char);
        } else if byte <= 8 {
            if (offset + u64::from(byte)) as usize <= length {
                for ch in &data[offset as usize..(offset + u64::from(byte)) as usize] {
                    text.push(*ch as char);
                }
                offset += u64::from(byte);
            }
        } else if byte <= 0x7f {
            text.push(byte as char)
        } else if byte <= 0xbf {
            offset += 1;
            if offset > length as u64 {
                let t: String = text.iter().collect();
                return Ok(t);
            }
            reader.set_position(offset - 2);
            let mut lz77 = reader.read_u16::<BigEndian>().unwrap();
            lz77 &= 0x3fff;
            let lz77length = (lz77 & 0x0007) + 3;
            let lz77offset = lz77 >> 3;

            if lz77offset < 1 {
                let t: String = text.iter().collect();
                return Ok(t);
            }
            for _lz77pos in 0..lz77length {
                let text_length = text.len();
                let mut textpos: usize = text_length;
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
    let t: String = text.iter().collect();
    Ok(t)
}
