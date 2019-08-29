# mobi-rs
A library written in rust to extract data from `.mobi` format ebooks It's purely for the sake of learning. 
[Crates.io](https://crates.io/crates/mobi)
## Usage
- add to `Cargo.toml`
```toml
[dependencies]
mobi = "0.1.4"
```
## Examples
### Access basic info
- `src/main.rs`
```rust
use mobi::Mobi;
fn main() {
    let m = Mobi::init(Path::new("/home/wojtek/Downloads/lotr.mobi")).unwrap();
    let title = m.title().unwrap();
    let author = m.author().unwrap();
    let publisher = m.publisher().unwrap();
    let desc = m.description().unwrap();
    let isbn = m.isbn().unwrap();
    let pub_date = m.publish_date().unwrap();
    let contributor = m.contributor().unwrap();
    println!("{}\n{}\n{}\n{}\n{}\n{}\n{}\n", title, author, publisher, isbn, pub_date, desc, contributor);
}
```
Output:
```
The Fellowship of the Ring
J. R. R. Tolkien
Houghton Mifflin
9780618574940
2005-07-15T07:00:00+00:00
SUMMARY: For over fifty years, J.R.R. Tolkienâs peerless fantasy has accumulated worldwide acclaim as the greatest adventure tale ever written.No other writer has created a world as distinct as Middle-earth, complete with its own geography, history, languages, and legends. And no one has created characters as endearing as Tolkienâs large-hearted, hairy-footed hobbits. Tolkienâs The Lord of the Rings continues to seize the imaginations of readers of all ages, and this new three-volume paperback edition is designed to appeal to the youngest of them.In ancient times the Rings of Power were crafted by the Elvensmiths, and Sauron, the Dark Lord, forged the One Ring, filling it with his own power so that he could rule all others. But the One Ring was taken from him, and though he sought it throughout Middle-earth, still it remained lost to him . . .
calibre (0.7.23) [http://calibre-ebook.com]
```
### Print nice summary
```rust
use mobi::Mobi;
fn main() {
    let m = Mobi::init(Path::new("/home/wojtek/Downloads/lotr.mobi")).unwrap();
    m.print_book_info();
}
```
yields:
```
----------------------------------------------------------
Title:          The Fellowship of the Ring
Author:         J. R. R. Tolkien
Publisher:      Houghton Mifflin
Description:    SUMMARY: For over fifty years, J.R.R. Tolkien’s peerless fantasy has accumulated worldwide acclaim as the greatest adventure tale ever written.No other writer has created a world as distinct as Middle-earth, complete with its own geography, history, languages, and legends. And no one has created characters as endearing as Tolkien’s large-hearted, hairy-footed hobbits. Tolkien’s The Lord of the Rings continues to seize the imaginations of readers of all ages, and this new three-volume paperback edition is designed to appeal to the youngest of them.In ancient times the Rings of Power were crafted by the Elvensmiths, and Sauron, the Dark Lord, forged the One Ring, filling it with his own power so that he could rule all others. But the One Ring was taken from him, and though he sought it throughout Middle-earth, still it remained lost to him . . .
ISBN:           9780618574940
Publish Date:   2005-07-15T07:00:00+00:00
Contributor:    calibre (0.7.23) [http://calibre-ebook.com]
----------------------------------------------------------
```
### Print headers
- `src/main.rs`
```rust
use mobi::Mobi;

fn main() {
    let m = Mobi::init(Path::new("/home/wojtek/Downloads/lotr.mobi"));
    println!(
        "{:#?}\n{:#?}\n{:#?}\n{:#?}",
        m.header, m.palmdoc, m.mobi, m.exth
    );
}
```
Running `cargo run` would yield (different data based on the file ofcourse):
```
Header {
    name: "The_Fellowship_of_the_Ring\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}",
    attributes: 0,
    version: 0,
    created: 1286664537,
    modified: 1286664537,
    backup: 0,
    modnum: 0,
    app_info_id: 0,
    sort_info_id: 0,
    typ_e: "BOOK",
    creator: "MOBI",
    unique_id_seed: 326,
    next_record_list_id: 0,
    num_of_records: 326,
}
PalmDocHeader {
    compression: 2,
    text_length: 1213227,
    record_count: 297,
    record_size: 4096,
    encryption_type: 0,
}
MobiHeader {
    identifier: 1297039945,
    header_length: 232,
    mobi_type: 2,
    text_encoding: 65001,
    id: 1826426250,
    gen_version: 6,
    first_non_book_index: 299,
    name: "The Fellowship of the Ring",
    name_offset: 1840,
    name_length: 26,
    language: 9,
    input_language: 0,
    output_language: 0,
    format_version: 6,
    first_image_index: 299,
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
    last_image_record: 322,
    fcis_record: 324,
    flis_record: 323,
}
ExtHeader {
    identifier: 1163416648,
    header_length: 1588,
    record_count: 29,
    records: {
        503: "The Fellowship of the Ring",
        101: "Houghton Mifflin",
        106: "2005-07-15T07:00:00+00:00",
        201: "\u{0}\u{0}\u{0}\u{c}",
        100: "J. R. R. Tolkien",
        203: "\u{0}\u{0}\u{0}\u{0}",
        202: "\u{0}\u{0}\u{0}\u{17}",
        104: "9780618574940",
        103: "SUMMARY: For over fifty years, J.R.R. Tolkien’s peerless fantasy has accumulated worldwide acclaim as the greatest adventure tale ever written.No other writer has created a world as distinct as Middle-earth, complete with its own geography, history, languages, and legends. And no one has created characters as endearing as Tolkien’s large-hearted, hairy-footed hobbits. Tolkien’s The Lord of the Rings continues to seize the imaginations of readers of all ages, and this new three-volume paperback edition is designed to appeal to the youngest of them.In ancient times the Rings of Power were crafted by the Elvensmiths, and Sauron, the Dark Lord, forged the One Ring, filling it with his own power so that he could rule all others. But the One Ring was taken from him, and though he sought it throughout Middle-earth, still it remained lost to him . . .",
        105: "Gandalf (Fictitious character)",
        108: "calibre (0.7.23) [http://calibre-ebook.com]",
    },
}

```
## TODO:
- [X] Implement lz77 decompression (almost done)
- [ ] Implement reading records
- [ ] Comments!
## License
                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/
## Thanks to
[kroo](https://github.com/kroo/mobi-python) for inspiration and idea.