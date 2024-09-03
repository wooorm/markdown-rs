#[allow(dead_code)]
pub struct Options {
    bullet: char,
    bullet_other: char,
    bullet_orderd: char,
    emphasis: char,
    fences: char,
    list_item_indent: IndentOptions,
    quote: char,
    rule: char,
    strong: char,
    increment_list_marker: bool,
    close_atx: bool,
    resource_link: bool,
    rule_spaces: bool,
    set_text: bool,
    tight_definitions: bool,
    rule_repetition: u32,
}

#[allow(dead_code)]
pub enum IndentOptions {
    Mixed,
    One,
    Tab,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            bullet: '*',
            bullet_other: '-',
            bullet_orderd: '.',
            emphasis: '*',
            fences: '`',
            increment_list_marker: false,
            rule_repetition: 3,
            list_item_indent: IndentOptions::One,
            quote: '"',
            rule: '*',
            strong: '*',
            close_atx: false,
            rule_spaces: false,
            resource_link: false,
            set_text: false,
            tight_definitions: false,
        }
    }
}
