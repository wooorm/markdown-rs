#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = markdown::to_html(s);
        let _ = markdown::to_html_with_options(s, &markdown::Options::gfm());
        let _ = markdown::to_mdast(s, &markdown::ParseOptions::default());
        let _ = markdown::to_mdast(s, &markdown::ParseOptions::gfm());
        let _ = markdown::to_mdast(s, &markdown::ParseOptions::mdx());
    }
});
