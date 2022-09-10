#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate micromark;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = micromark::micromark(s);
        let _ = micromark::micromark_with_options(
            s,
            &micromark::Options {
                constructs: micromark::Constructs::gfm(),
                ..micromark::Options::default()
            },
        );
    }
});
