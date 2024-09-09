use alloc::string::String;
use regex::Regex;

pub fn indent_lines(value: &str, map: impl Fn(&str, usize, bool) -> String) -> String {
    let mut result = String::new();
    let mut start = 0;
    let mut line = 0;
    let eol = Regex::new(r"\r?\n|\r").unwrap();
    for m in eol.captures_iter(value) {
        let full_match = m.get(0).unwrap();
        let value_slice = &value[start..full_match.start()];
        result.push_str(&map(value_slice, line, value_slice.is_empty()));
        result.push_str(full_match.as_str());
        start = full_match.start() + full_match.len();
        line += 1;
    }
    result.push_str(&map(&value[start..], line, value.is_empty()));
    result
}
