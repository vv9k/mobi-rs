# mobi-rs
A library written in rust to extract data from `.mobi` format ebooks It's purely for the sake of learning. 
## TODO:
- [ ] Implement lz77 decompression
- [ ] Implement reading records
- [ ] Comments!
## Usage
- add to `Cargo.toml`
```toml
[dependencies]
mobi = "0.1.0"
```
## Example
- `src/main.rs`
```rust
use mobi::Mobi;

fn main() {
    let m = Mobi::init(Path::new("/home/wojtek/Downloads/lotr.mobi"));
    println!(
        "{:#?} {:#?} {:#?} {:#?}",
        m.header, m.palmdoc, m.mobi, m.exth
    );
}
```
Running `cargo run` would yield (different data based on the file ofcourse):
```
Header {
    name: "Lord_of_the_Rings_-_Fellowship_\u{0}",
    attributes: 0,
    version: 0,
    created: 1299709979,
    modified: 1299709979,
    backup: 0,
    modnum: 0,
    app_info_id: 0,
    sort_info_id: 0,
    typ_e: "BOOK",
    creator: "MOBI",
    unique_id_seed: 292,
    next_record_list_id: 0,
    num_of_records: 292,
} PalmDocHeader {
    compression: 2,
    text_length: 1151461,
    record_count: 282,
    record_size: 4096,
    encryption_type: 0,
} MobiHeader {
    identifier: 232,
    header_length: 2,
    mobi_type: 65001,
    text_encoding: 3428045761,
    id: 6,
    gen_version: 4294967295,
    first_non_book_index: 284,
    name: "Lord of the Rings - Fellowship of the Ring",
    name_offset: 1360,
    name_length: 42,
    language: 2057,
    input_language: 0,
    output_language: 0,
    format_version: 6,
    first_image_index: 287,
    first_huff_record: 0,
    huff_record_count: 0,
    first_data_record: 0,
    data_record_count: 0,
    exth_flags: 80,
    has_exth_header: true,
    drm_offset: 4294967295,
    drm_count: 0,
    drm_size: 0,
    drm_flags: 0,
    last_image_record: 288,
    fcis_record: 290,
    flis_record: 289,
} ExtHeader {
    identifier: 1163416648,
    header_length: 1109,
    record_count: 11,
    records: [
        "HarperCollins Publishers Ltd",
        "<h3>From Library Journal</h3><p>New Line Cinema will be releasing \"The Lord of the Rings\" trilogy in three separate installments, and Houghton Mifflin Tolkien\'s U.S. publisher since the release of The Hobbit in 1938 will be re-releasing each volume of the trilogy separately and in a boxed set (ISBN 0-618-15397-7. $22; pap. ISBN 0-618-15396-9. $12). <br />Copyright 2001 Reed Business Information, Inc. </p><h3>Review</h3><p>\'An extraordinary book. It deals with a stupendous theme. It leads us through a succession of strange and astonishing episodes, some of them magnificent, in a region where everything is invented, forest, moor, river, wilderness, town and the races which inhabit them.\' The Observer \'Among the greatest works of imaginative fiction of the twentieth century.\' Sunday Telegraph </p>",
        "J. R. R. Tolkien",
        "Lord of the Rings - Fellowship of the Ring",
        "2010-12-21T00:00:00+00:00",
        "calibre (0.7.31) [http://calibre-ebook.com]",
        "9780261102316",
        "2010-12-21T00:00:00+00:00",
        "\u{0}\u{0}\u{0}\u{0}",
        "\u{0}\u{0}\u{0}\u{0}",
        "\u{0}\u{0}\u{0}\u{1}",
    ],
}
```
## License
                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/
## Thanks to
[kroo](https://github.com/kroo/mobi-python) for inspiration and idea.