use alloc::string::String;
use markdown::mdast::{Image, Node};

use crate::{
    construct_name::ConstructName,
    message::Message,
    state::{Info, State},
    util::{check_quote::check_quote, safe::SafeConfig},
};

use super::Handle;

impl Handle for Image {
    fn handle(
        &self,
        state: &mut State,
        _info: &Info,
        _parent: Option<&Node>,
        _node: &Node,
    ) -> Result<alloc::string::String, Message> {
        let quote = check_quote(state)?;
        state.enter(ConstructName::Image);
        state.enter(ConstructName::Label);

        let mut value = String::new();

        value.push_str("![");

        value.push_str(&state.safe(
            &self.alt,
            &SafeConfig::new(Some(value.as_str()), Some("]"), None),
        ));

        value.push_str("](");
        state.exit();

        if self.url.is_empty() && self.title.is_some()
            || contain_control_char_or_whitespace(&self.url)
        {
            state.enter(ConstructName::DestinationLiteral);
            value.push('<');
            value.push_str(&state.safe(&self.url, &SafeConfig::new(Some(&value), Some(">"), None)));
            value.push('>');
        } else {
            state.enter(ConstructName::DestinationRaw);
            let after = if self.title.is_some() { " " } else { ")" };
            value.push_str(
                &state.safe(&self.url, &SafeConfig::new(Some(&value), Some(after), None)),
            );
        }

        state.exit();

        if let Some(title) = &self.title {
            let title_construct_name = if quote == '"' {
                ConstructName::TitleQuote
            } else {
                ConstructName::TitleApostrophe
            };

            state.enter(title_construct_name);
            value.push(' ');
            value.push(quote);

            let mut before_buffer = [0u8; 4];
            let before = quote.encode_utf8(&mut before_buffer);
            value.push_str(
                &state.safe(title, &SafeConfig::new(Some(&self.url), Some(before), None)),
            );

            value.push(quote);
            state.exit();
        }

        value.push(')');
        state.exit();

        Ok(value)
    }
}

fn contain_control_char_or_whitespace(value: &str) -> bool {
    value.chars().any(|c| c.is_whitespace() || c.is_control())
}

pub fn peek_image() -> char {
    '!'
}
