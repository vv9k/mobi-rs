use mobi::Mobi;
use std::path::Path;
fn main() {
    let m = Mobi::init(Path::new("/home/wojtek/Downloads/lotr.mobi"));
    println!(
        "{:#?} {:#?} {:#?} {:?}",
        m.header, m.palmdoc, m.mobi, m.exth
    );
}
