pub fn u8_as_string(byte_arr: &[u8]) -> String {
    let mut out_str = String::new();
    for byte in byte_arr {
        out_str.push(*byte as char);
    }
    out_str
}
