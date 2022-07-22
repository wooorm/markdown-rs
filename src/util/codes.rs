//! Utilities to deal with character codes.

use crate::constant::TAB_SIZE;
use crate::tokenizer::Code;

/// Turn a string into codes.
pub fn parse(value: &str) -> Vec<Code> {
    // Note: It’ll grow a bit bigger with each `Code::VirtualSpace`, smaller
    // with `Code::CarriageReturnLineFeed`.
    let mut codes = Vec::with_capacity(value.len());
    let mut at_start = true;
    let mut at_carriage_return = false;
    let mut column = 1;

    for char in value.chars() {
        if at_start {
            at_start = false;

            if char == '\u{feff}' {
                // Ignore.
                continue;
            }
        }

        // Send a CRLF.
        if at_carriage_return && '\n' == char {
            at_carriage_return = false;
            codes.push(Code::CarriageReturnLineFeed);
        } else {
            // Send the previous CR: we’re not at a next `\n`.
            if at_carriage_return {
                at_carriage_return = false;
                codes.push(Code::Char('\r'));
            }

            match char {
                // Send a replacement character.
                '\0' => {
                    column += 1;
                    codes.push(Code::Char(char::REPLACEMENT_CHARACTER));
                }
                // Send a tab and virtual spaces.
                '\t' => {
                    let remainder = column % TAB_SIZE;
                    let mut virtual_spaces = if remainder == 0 {
                        0
                    } else {
                        TAB_SIZE - remainder
                    };
                    codes.push(Code::Char(char));
                    column += 1;
                    while virtual_spaces > 0 {
                        codes.push(Code::VirtualSpace);
                        column += 1;
                        virtual_spaces -= 1;
                    }
                }
                // Send an LF.
                '\n' => {
                    column = 1;
                    codes.push(Code::Char(char));
                }
                // Don’t send anything yet.
                '\r' => {
                    column = 1;
                    at_carriage_return = true;
                }
                // Send the char.
                _ => {
                    column += 1;
                    codes.push(Code::Char(char));
                }
            }
        };
    }

    // Send the last CR: we’re not at a next `\n`.
    if at_carriage_return {
        codes.push(Code::Char('\r'));
    }

    codes
}

/// Serialize codes, optionally expanding tabs.
pub fn serialize(codes: &[Code], expand_tabs: bool) -> String {
    let mut at_tab = false;
    // Note: It’ll grow a bit smaller with each
    // `Code::Char('\t') | Code::VirtualSpace` if `expand_tabs` is false,
    // and bigger with `Code::CarriageReturnLineFeed`,
    let mut value = String::with_capacity(codes.len());

    for code in codes {
        let mut at_tab_next = false;

        match code {
            Code::CarriageReturnLineFeed => {
                value.push_str("\r\n");
            }
            Code::Char(char) if *char == '\n' || *char == '\r' => {
                value.push(*char);
            }
            Code::Char(char) if *char == '\t' => {
                at_tab_next = true;
                value.push(if expand_tabs { ' ' } else { *char });
            }
            Code::VirtualSpace => {
                if !expand_tabs && at_tab {
                    continue;
                }
                value.push(' ');
            }
            Code::Char(char) => {
                value.push(*char);
            }
            Code::None => {
                unreachable!("unexpected EOF code in codes");
            }
        }

        at_tab = at_tab_next;
    }

    value
}
