use markdown::mdast::Code;
use regex::Regex;

use crate::state::State;

pub fn format_code_as_indented(code: &Code, state: &State) -> bool {
    let white_space = Regex::new(r"[^ \r\n]").unwrap();
    let blank = Regex::new(r"^[\t ]*(?:[\r\n]|$)|(?:^|[\r\n])[\t ]*$").unwrap();

    !state.options.fences
        && !code.value.is_empty()
        && code.lang.is_none()
        && white_space.is_match(&code.value)
        && !blank.is_match(&code.value)
}
