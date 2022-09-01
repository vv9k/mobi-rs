#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate mobi;

fuzz_target!(|data: &[u8]| {
    let _ = mobi::Mobi::new(Vec::from(data));
});
