enum HeaderData {
    name,
    attributes,
    version,
    created,
    modified,
    backup,
    modnum,
    appInfoId,
    sortInfoId,
    typ_e,
    creator,
    uniqueIdSeed,
    nextRecordListId,
    numOfRecords,
}
struct Header {
    name: String,
    attributes: u16,
    version: u16,
    created: u32,
    modified: u32,
    backup: u32,
    modnum: u32,
    app_info_id: u32,
    sord_info_id: u32,
    typ_e: [u8; 4],
    creator: [u8; 4],
    unique_id_seed: u32,
    next_record_list_id: u32,
    num_of_records: u16,
}
impl Header {
    fn parse(content: &Vec<u8>) -> Header {
        unimplemented!();
    }
    fn name(content: &Vec<u8>) -> String {
        let mut name = String::new();
        for ch in &content[0..32] {
            name.push(*ch as char);
        }
        name
    }
    fn attributes(content: &Vec<u8>) -> u16 {
        let mut reader = Cursor::new(content);
        reader.set_position(32);
        reader.read_u16::<BigEndian>().unwrap()
    }
    fn attributes(content: &Vec<u8>) -> u16 {
        let mut reader = Cursor::new(content);
        reader.set_position(32);
        reader.read_u16::<BigEndian>().unwrap()
    }
}