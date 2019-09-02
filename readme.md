# mobi-rs
A library written in rust to extract data from `.mobi` format ebooks.
- [Crates.io](https://crates.io/crates/mobi)
- [Documentation](https://docs.rs/mobi)
## Usage
- add to `Cargo.toml`
```toml
[dependencies]
mobi = "0.2.4"
```
## Examples
### Print the whole book into stdout
```rust
use mobi::Mobi;
fn main() {
    let m = Mobi::init("/home/wojtek/Downloads/lotr.mobi").unwrap();
    println!("{}", m.content_raw().unwrap());
}
```
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
Title:                  Lord of the Rings - Fellowship of the Ring
Author:                 J. R. R. Tolkien
Publisher:              HarperCollins Publishers Ltd
Description:            <h3>From Library Journal</h3><p>New Line Cinema will be releasing "The Lord of the Rings" trilogy in three separate installments, and Houghton Mifflin Tolkien's U.S. publisher since the release of The Hobbit in 1938 will be re-releasing each volume of the trilogy separately and in a boxed set (ISBN 0-618-15397-7. $22; pap. ISBN 0-618-15396-9. $12). <br />Copyright 2001 Reed Business Information, Inc. </p><h3>Review</h3><p>'An extraordinary book. It deals with a stupendous theme. It leads us through a succession of strange and astonishing episodes, some of them magnificent, in a region where everything is invented, forest, moor, river, wilderness, town and the races which inhabit them.' The Observer 'Among the greatest works of imaginative fiction of the twentieth century.' Sunday Telegraph </p>
ISBN:                   9780261102316
Publish Date:           2010-12-21T00:00:00+00:00
Contributor:            calibre (0.7.31) [http://calibre-ebook.com]
------------------------------------------------------------------------------------
HEADER
Name:                   Lord_of_the_Rings_-_Fellowship_
Attributes:             0
Version:                0
Created:                2011-03-09 22:32:59
Modified:               2011-03-09 22:32:59
Backup:                 0
Modnum:                 0
App_info_id:            0
Sort_info_id:           0
Typ_e:                  BOOK
Creator:                MOBI
Unique_id_seed:         292
Next_record_list_id:    0
Num_of_records:         292
------------------------------------------------------------------------------------
PALMDOC HEADER
Compression:            2
Text length:            1151461
Record count:           282
Record size:            4096
Encryption type:        0
------------------------------------------------------------------------------------
MOBI HEADER
Identifier:             1297039945
HeaderLength:           232
Mobi type:              Mobipocket Book
Text encoding:          UTF-8
Id:                     3428045761
Gen version:            v6
First non book index:   284
Name:                   Lord of the Rings - Fellowship of the Ring
Name offset:            1360
Name length:            42
Language:               ENGLISH
Input language:         0
Output language:        0
Format version:         6
First image index:      287
First huff record:      0
Huff record count:      0
First data record:      0
Data record count:      0
Exth flags:             80
Has Exth header:        true
Has DRM:                false
DRM offset:             4294967295
DRM count:              0
DRM size:               0
DRM flags:              0
Last image record:      288
Fcis record:            290
Flis record:            289
------------------------------------------------------------------------------------
EXTHEADER
Identifier:             1163416648
Header_length:          1109
Record_count:           11
Records:                {
    203: "\u{0}\u{0}\u{0}\u{0}",
    202: "\u{0}\u{0}\u{0}\u{1}",
    101: "HarperCollins Publishers Ltd",
    100: "J. R. R. Tolkien",
    201: "\u{0}\u{0}\u{0}\u{0}",
    106: "2010-12-21T00:00:00+00:00",
    503: "Lord of the Rings - Fellowship of the Ring",
    108: "calibre (0.7.31) [http://calibre-ebook.com]",
    104: "9780261102316",
    103: "<h3>From Library Journal</h3><p>New Line Cinema will be releasing \"The Lord of the Rings\" trilogy in three separate installments, and Houghton Mifflin Tolkien\'s U.S. publisher since the release of The Hobbit in 1938 will be re-releasing each volume of the trilogy separately and in a boxed set (ISBN 0-618-15397-7. $22; pap. ISBN 0-618-15396-9. $12). <br />Copyright 2001 Reed Business Information, Inc. </p><h3>Review</h3><p>\'An extraordinary book. It deals with a stupendous theme. It leads us through a succession of strange and astonishing episodes, some of them magnificent, in a region where everything is invented, forest, moor, river, wilderness, town and the races which inhabit them.\' The Observer \'Among the greatest works of imaginative fiction of the twentieth century.\' Sunday Telegraph </p>",
}
------------------------------------------------------------------------------------

```
## TODO:
- [X] Implement lz77 decompression
- [X] Implement reading records
- [ ] Comments!
- [ ] format the content
- [ ] into PDF
- [ ] into epub
## License
[**The MIT License (MIT)**](https://github.com/wojciechkepka/mobi-rs/blob/master/license)
## Thanks to
[kroo](https://github.com/kroo/mobi-python) for inspiration and idea.