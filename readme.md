# mobi-rs
A library written in rust to extract data from `.mobi` format ebooks It's purely for the sake of learning. 
[Crates.io](https://crates.io/crates/mobi)
## Usage
- add to `Cargo.toml`
```toml
[dependencies]
mobi = "0.1.6"
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
    // Access Headers
    let header = m.header; // Normal Header
    let pdheader = m.palmdoc; // PalmDOC Header
    let mheader = m.mobi; // MOBI Header
    let exth = m.exth // Extra Header
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
### Print all info
- `src/main.rs`
```rust
use mobi::Mobi;

fn main() {
    let m = Mobi::init(Path::new("/home/wojtek/Downloads/lotr.mobi"));
    println!("{}", m)
}
```
Running `cargo run` would yield (different data based on the file ofcourse):
```
------------------------------------------------------------------------------------
Title:          The Fellowship of the Ring
Author:         J. R. R. Tolkien
Publisher:      Houghton Mifflin
Description:    SUMMARY: For over fifty years, J.R.R. Tolkien’s peerless fantasy has accumulated worldwide acclaim as the greatest adventure tale ever written.No other writer has created a world as distinct as Middle-earth, complete with its own geography, history, languages, and legends. And no one has created characters as endearing as Tolkien’s large-hearted, hairy-footed hobbits. Tolkien’s The Lord of the Rings continues to seize the imaginations of readers of all ages, and this new three-volume paperback edition is designed to appeal to the youngest of them.In ancient times the Rings of Power were crafted by the Elvensmiths, and Sauron, the Dark Lord, forged the One Ring, filling it with his own power so that he could rule all others. But the One Ring was taken from him, and though he sought it throughout Middle-earth, still it remained lost to him . . .
ISBN:           9780618574940
Publish Date:   2005-07-15T07:00:00+00:00
Contributor:    calibre (0.7.23) [http://calibre-ebook.com]
------------------------------------------------------------------------------------
HEADER
Name:                   The_Fellowship_of_the_Ring
Attributes:             0
Version:                0
Created:                1286664537
Modified:               1286664537
Backup:                 0
Modnum:                 0
App_info_id:            0
Sort_info_id:           0
Typ_e:                  BOOK
Creator:                MOBI
Unique_id_seed:         326
Next_record_list_id:    0
Num_of_records:         326
------------------------------------------------------------------------------------
PALMDOC HEADER
Compression:            2
Text length:            1213227
Record count:           297
Record size:            4096
Encryption type:        0
------------------------------------------------------------------------------------
MOBI HEADER
Identifier:             1297039945
HeaderLength:           232
Mobi type:              2
Text encoding:          65001
Id:                     1826426250
Gen version:            6
First non book index:   299
Name:                   The Fellowship of the Ring
Name offset:            1840
Name length:            26
Language:               9
Input language:         0
Output language:        0
Format version:         6
First image index:      299
First huff record:      0
Huff record count:      0
First data record:      0
Data record count:      0
Exth flags:             80
Has Exth header:        true
Drm offset:             4294967295
Drm count:              0
Drm size:               0
Drm flags:              0
Last image record:      322
Fcis record:            324
Flis record:            323
------------------------------------------------------------------------------------
EXTHEADER
Identifier:             1163416648
Header_length:          1588
Record_count:           29
Records:                {
    100: "J. R. R. Tolkien",
    503: "The Fellowship of the Ring",
    108: "calibre (0.7.23) [http://calibre-ebook.com]",
    104: "9780618574940",
    201: "\u{0}\u{0}\u{0}\u{c}",
    101: "Houghton Mifflin",
    202: "\u{0}\u{0}\u{0}\u{17}",
    106: "2005-07-15T07:00:00+00:00",
    103: "SUMMARY: For over fifty years, J.R.R. Tolkien’s peerless fantasy has accumulated worldwide acclaim as the greatest adventure tale ever written.No other writer has created a world as distinct as Middle-earth, complete with its own geography, history, languages, and legends. And no one has created characters as endearing as Tolkien’s large-hearted, hairy-footed hobbits. Tolkien’s The Lord of the Rings continues to seize the imaginations of readers of all ages, and this new three-volume paperback edition is designed to appeal to the youngest of them.In ancient times the Rings of Power were crafted by the Elvensmiths, and Sauron, the Dark Lord, forged the One Ring, filling it with his own power so that he could rule all others. But the One Ring was taken from him, and though he sought it throughout Middle-earth, still it remained lost to him . . .",
    105: "Gandalf (Fictitious character)",
    203: "\u{0}\u{0}\u{0}\u{0}",
}
------------------------------------------------------------------------------------
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