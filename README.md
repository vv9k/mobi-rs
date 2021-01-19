# mobi-rs
[![Build Status](https://travis-ci.com/wojciechkepka/mobi-rs.svg?branch=master)](https://travis-ci.com/wojciechkepka/mobi-rs)
[![crates.io](https://img.shields.io/crates/v/mobi)](https://crates.io/crates/mobi)
[![crates.io](https://img.shields.io/crates/l/mobi)](https://github.com/wojciechkepka/mobi-rs/blob/master/LICENSE)
[![Docs](https://img.shields.io/badge/docs-master-brightgreen)](https://docs.rs/mobi)  
A crate to work with `.mobi` format ebooks.
## Usage
- add to `Cargo.toml`
```toml
[dependencies]
mobi = "0.5"
```
- `main.rs`
```rust
use mobi::Mobi;
fn main() -> Result<(), std::io::Error> {
    let book = vec![0, 0, 0];
    // You can either create a Mobi struct from a slice
    let m = Mobi::new(&book)?;
    // Or from an instance of io::Read
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let m = Mobi::from_read(&mut handle)?;
    // Or from filesystem
    let m = Mobi::from_path("/some/path/to/book.mobi")?;

    // Access metadata
    let title = m.title();
    let author = m.author().unwrap_or_default();
    let publisher = m.publisher().unwrap_or_default();
    let desc = m.description().unwrap_or_default();
    let isbn = m.isbn().unwrap_or_default();
    let pub_date = m.publish_date().unwrap_or_default();
    let contributor = m.contributor().unwrap_or_default();

    // Access Headers
    let metadata = &m.metadata;
    let header = &metadata.header; // Normal Header
    let pdheader = &metadata.palmdoc; // PalmDOC Header
    let mheader = &metadata.mobi; // MOBI Header
    let exth = &metadata.exth; // Extra Header

    // Access content
    let content = m.content_as_string();

    Ok(())
}
```
## License
[**The MIT License (MIT)**](https://github.com/wojciechkepka/mobi-rs/blob/master/LICENSE)
## Thanks to
[kroo](https://github.com/kroo/mobi-python) for inspiration and idea.
