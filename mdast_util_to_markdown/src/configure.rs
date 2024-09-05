#[allow(dead_code)]
pub struct Options {
    pub bullet: char,
    pub bullet_other: char,
    pub bullet_orderd: char,
    pub emphasis: char,
    pub fences: char,
    pub list_item_indent: IndentOptions,
    pub quote: char,
    pub rule: char,
    pub strong: char,
    pub increment_list_marker: bool,
    pub close_atx: bool,
    pub resource_link: bool,
    pub rule_spaces: bool,
    pub setext: bool,
    pub tight_definitions: bool,
    pub rule_repetition: u32,
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
            setext: false,
            tight_definitions: false,
        }
    }
}
