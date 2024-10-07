use core::mem;

use alloc::string::String;
use markdown::{
    mdast::{Link, Node},
    message::Message,
};

use crate::{
    construct_name::ConstructName,
    state::{Info, State},
    util::{
        check_quote::check_quote, format_link_as_auto_link::format_link_as_auto_link,
        safe::SafeConfig,
    },
};

use super::Handle;

impl Handle for Link {
    fn handle(
        &self,
        state: &mut State,
        _info: &Info,
        _parent: Option<&Node>,
        node: &Node,
    ) -> Result<alloc::string::String, Message> {
        let quote = check_quote(state)?;

        if format_link_as_auto_link(self, node, state) {
            let old_stack = mem::take(&mut state.stack);
            state.enter(ConstructName::Autolink);
            let mut value = String::from("<");
            value.push_str(&state.container_phrasing(node, &Info::new(&value, ">"))?);
            value.push('>');
            state.exit();
            state.stack = old_stack;
            return Ok(value);
        }

        state.enter(ConstructName::Link);
        state.enter(ConstructName::Label);
        let mut value = String::from("[");
        value.push_str(&state.container_phrasing(node, &Info::new(&value, "]("))?);
        value.push_str("](");
        state.exit();

        if self.url.is_empty() && self.title.is_some()
            || contain_control_char_or_whitespace(&self.url)
        {
            state.enter(ConstructName::DestinationLiteral);
            value.push('<');
            value.push_str(&state.safe(&self.url, &SafeConfig::new(&value, ">", None)));
            value.push('>');
        } else {
            state.enter(ConstructName::DestinationRaw);
            let after = if self.title.is_some() { " " } else { ")" };
            value.push_str(&state.safe(&self.url, &SafeConfig::new(&value, after, None)))
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
            value.push_str(&state.safe(title, &SafeConfig::new(&self.url, before, None)));

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

pub fn peek_link(link: &Link, node: &Node, state: &State) -> char {
    if format_link_as_auto_link(link, node, state) {
        '>'
    } else {
        '['
    }
}
