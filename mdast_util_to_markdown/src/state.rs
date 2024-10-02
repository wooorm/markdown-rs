use crate::association::Association;
use crate::construct_name::ConstructName;
use crate::handle::emphasis::peek_emphasis;
use crate::handle::html::peek_html;
use crate::handle::image::peek_image;
use crate::handle::image_reference::peek_image_reference;
use crate::handle::inline_code::peek_inline_code;
use crate::handle::link::peek_link;
use crate::handle::link_reference::peek_link_reference;
use crate::handle::strong::peek_strong;
use crate::handle::Handle;
use crate::Options;
use crate::{
    r#unsafe::Unsafe,
    util::{
        format_code_as_indented::format_code_as_indented,
        format_heading_as_setext::format_heading_as_setext,
        pattern_in_scope::pattern_in_scope,
        safe::{escape_backslashes, EscapeInfos, SafeConfig},
    },
};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use markdown::mdast::Node;
use markdown::message::Message;
use regex::{Captures, Regex, RegexBuilder};

#[derive(Debug)]
enum Join {
    Break,
    HTMLComment,
    Lines(usize),
}

pub struct State<'a> {
    pub stack: Vec<ConstructName>,
    pub index_stack: Vec<usize>,
    pub bullet_last_used: Option<char>,
    pub bullet_current: Option<char>,
    pub r#unsafe: Vec<Unsafe<'a>>,
    pub options: &'a Options,
}

pub struct Info<'a> {
    pub before: &'a str,
    pub after: &'a str,
}

impl<'a> Info<'a> {
    pub fn new(before: &'a str, after: &'a str) -> Self {
        Info { before, after }
    }
}

impl<'a> State<'a> {
    pub fn new(options: &'a Options) -> Self {
        State {
            stack: Vec::new(),
            index_stack: Vec::new(),
            bullet_last_used: None,
            bullet_current: None,
            r#unsafe: Unsafe::get_default_unsafe(),
            options,
        }
    }

    pub fn enter(&mut self, name: ConstructName) {
        self.stack.push(name);
    }

    pub fn exit(&mut self) {
        self.stack.pop();
    }

    pub fn handle(
        &mut self,
        node: &Node,
        info: &Info,
        parent: Option<&Node>,
    ) -> Result<String, Message> {
        match node {
            Node::Root(root) => root.handle(self, info, parent, node),
            Node::Paragraph(paragraph) => paragraph.handle(self, info, parent, node),
            Node::Text(text) => text.handle(self, info, parent, node),
            Node::Strong(strong) => strong.handle(self, info, parent, node),
            Node::Emphasis(emphasis) => emphasis.handle(self, info, parent, node),
            Node::Heading(heading) => heading.handle(self, info, parent, node),
            Node::Break(r#break) => r#break.handle(self, info, parent, node),
            Node::Html(html) => html.handle(self, info, parent, node),
            Node::ThematicBreak(thematic_break) => thematic_break.handle(self, info, parent, node),
            Node::Code(code) => code.handle(self, info, parent, node),
            Node::Blockquote(block_quote) => block_quote.handle(self, info, parent, node),
            Node::List(list) => list.handle(self, info, parent, node),
            Node::ListItem(list_item) => list_item.handle(self, info, parent, node),
            Node::Image(image) => image.handle(self, info, parent, node),
            Node::Link(link) => link.handle(self, info, parent, node),
            Node::InlineCode(inline_code) => inline_code.handle(self, info, parent, node),
            Node::Definition(definition) => definition.handle(self, info, parent, node),
            Node::ImageReference(image_reference) => {
                image_reference.handle(self, info, parent, node)
            }
            Node::LinkReference(link_reference) => link_reference.handle(self, info, parent, node),
            _ => Err(Message {
                reason: String::from("Can't handle node"),
                rule_id: Box::new("unexpected-node".into()),
                source: Box::new("mdast-util-to-markdown".into()),
                place: None,
            }),
        }
    }

    pub fn safe(&mut self, input: &str, config: &SafeConfig) -> String {
        let value = format!("{}{}{}", config.before, input, config.after);
        let mut positions: Vec<usize> = Vec::new();
        let mut result: String = String::new();
        let mut infos: BTreeMap<usize, EscapeInfos> = BTreeMap::new();

        for pattern in &mut self.r#unsafe {
            if !pattern_in_scope(&self.stack, pattern) {
                continue;
            }

            Self::compile_pattern(pattern);

            if let Some(regex) = &pattern.compiled {
                for m in regex.captures_iter(&value) {
                    let full_match = m.get(0).expect("Guaranteed to have a match");
                    let captured_group_len = m
                        .get(1)
                        .map(|captured_group| captured_group.len())
                        .unwrap_or(0);
                    let before = pattern.before.is_some() || pattern.at_break;
                    let after = pattern.after.is_some();
                    let position = full_match.start() + if before { captured_group_len } else { 0 };

                    if positions.contains(&position) {
                        if let Some(entry) = infos.get_mut(&position) {
                            if entry.before && !before {
                                entry.before = false;
                            }
                            if entry.after && !after {
                                entry.after = false;
                            }
                        }
                    } else {
                        infos.insert(position, EscapeInfos { before, after });
                        positions.push(position);
                    }
                }
            }
        }

        positions.sort_unstable();

        let mut start = config.before.len();
        let end = value.len() - config.after.len();
        for (index, position) in positions.iter().enumerate() {
            if *position < start || *position >= end {
                continue;
            }

            // If this character is supposed to be escaped because it has a condition on
            // the next character, and the next character is definitly being escaped,
            // then skip this escape.
            // This will never panic because the bounds are properly checked, and we
            // guarantee that the positions are already keys in the `infos` map before this
            // point in execution.
            if index + 1 < positions.len()
                && position + 1 < end
                && positions[index + 1] == position + 1
                && infos[position].after
                && !infos[&(position + 1)].before
                && !infos[&(position + 1)].after
                || index > 0
                    && positions[index - 1] == position - 1
                    && infos[position].before
                    && !infos[&(position - 1)].before
                    && !infos[&(position - 1)].after
            {
                continue;
            }

            if start != *position {
                result.push_str(&escape_backslashes(&value[start..*position], r"\"));
            }
            start = *position;

            let char_at_pos = value.chars().nth(*position);
            match char_at_pos {
                Some('!'..='/') | Some(':'..='@') | Some('['..='`') | Some('{'..='~') => {
                    if let Some(encode) = &config.encode {
                        let character = char_at_pos.expect("To be a valid char");
                        if *encode != character {
                            result.push('\\');
                        } else {
                            let encoded_char = Self::encode_char(character);
                            result.push_str(&encoded_char);
                            start += character.len_utf8();
                        }
                    } else {
                        result.push('\\');
                    }
                }
                Some(character) => {
                    let encoded_char = Self::encode_char(character);
                    result.push_str(&encoded_char);
                    start += character.len_utf8();
                }
                _ => (),
            };
        }

        result.push_str(&escape_backslashes(&value[start..end], config.after));

        result
    }

    fn encode_char(character: char) -> String {
        let hex_code = u32::from(character);
        format!("&#x{:X};", hex_code)
    }

    pub fn compile_pattern(pattern: &mut Unsafe) {
        if pattern.compiled.is_none() {
            let mut pattern_to_compile = String::new();

            if let Some(pattern_before) = pattern.before {
                pattern_to_compile.push('(');
                if pattern.at_break {
                    pattern_to_compile.push_str("[\\r\\n][\\t ]*");
                }
                pattern_to_compile.push_str("(?:");
                pattern_to_compile.push_str(pattern_before);
                pattern_to_compile.push(')');
                pattern_to_compile.push(')');
            } else if pattern.at_break {
                pattern_to_compile.push('(');
                pattern_to_compile.push_str("[\\r\\n][\\t ]*");
                pattern_to_compile.push(')');
            }

            if matches!(
                pattern.character,
                '|' | '\\'
                    | '{'
                    | '}'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '^'
                    | '$'
                    | '+'
                    | '*'
                    | '?'
                    | '.'
                    | '-'
            ) {
                pattern_to_compile.push('\\');
            }

            pattern_to_compile.push(pattern.character);

            if let Some(pattern_after) = pattern.after {
                pattern_to_compile.push_str("(?:");
                pattern_to_compile.push_str(pattern_after);
                pattern_to_compile.push(')');
            }

            pattern.set_compiled(
                Regex::new(&pattern_to_compile).expect("A valid unsafe regex pattern"),
            );
        }
    }

    pub fn container_phrasing(&mut self, parent: &Node, info: &Info) -> Result<String, Message> {
        let children = parent
            .children()
            .expect("The node to be a phrasing parent.");

        if children.is_empty() {
            return Ok(String::new());
        }

        let mut results: String = String::new();
        let mut index = 0;
        let mut children_iter = children.iter().peekable();

        self.index_stack.push(0);

        while let Some(child) = children_iter.next() {
            if index > 0 {
                let top = self
                    .index_stack
                    .last_mut()
                    .expect("The stack is populated with at least one child position");
                *top = index;
            }

            let mut new_info = Info::new(info.before, info.after);
            let mut buffer = [0u8; 4];
            if let Some(child) = children_iter.peek() {
                if let Some(first_char) = self.peek_node(child) {
                    new_info.after = first_char.encode_utf8(&mut buffer);
                } else {
                    new_info.after = self
                        .handle(child, &Info::new("", ""), Some(parent))?
                        .chars()
                        .nth(0)
                        .unwrap_or_default()
                        .encode_utf8(&mut buffer);
                }
            }

            if !results.is_empty() {
                if info.before == "\r" || info.before == "\n" && matches!(child, Node::Html(_)) {
                    // TODO Remove this check here it might not be needed since we're
                    // checking for the before info.
                    if results.ends_with('\n') || results.ends_with('\r') {
                        results.pop();
                        if results.ends_with('\r') {
                            results.pop();
                        }
                    }
                    results.push(' ');
                    new_info.before = " ";
                } else {
                    new_info.before = &results[results.len() - 1..];
                }
            }

            results.push_str(&self.handle(child, &new_info, Some(parent))?);
            index += 1;
        }

        self.index_stack.pop();

        Ok(results)
    }

    fn peek_node(&self, node: &Node) -> Option<char> {
        match node {
            Node::Strong(_) => Some(peek_strong(self)),
            Node::Emphasis(_) => Some(peek_emphasis(self)),
            Node::Html(_) => Some(peek_html()),
            Node::Image(_) => Some(peek_image()),
            Node::Link(link) => Some(peek_link(link, node, self)),
            Node::InlineCode(_) => Some(peek_inline_code()),
            Node::ImageReference(_) => Some(peek_image_reference()),
            Node::LinkReference(_) => Some(peek_link_reference()),
            _ => None,
        }
    }

    pub fn container_flow(&mut self, parent: &Node) -> Result<String, Message> {
        let children = parent.children().expect("The node to be a flow parent.");

        if children.is_empty() {
            return Ok(String::new());
        }

        let mut results: String = String::new();
        let mut children_iter = children.iter().peekable();
        let mut index = 0;

        self.index_stack.push(0);

        while let Some(child) = children_iter.next() {
            if index > 0 {
                let top = self
                    .index_stack
                    .last_mut()
                    .expect("The stack is populated with at least one child position");
                *top = index;
            }

            if !matches!(child, Node::List(_)) {
                self.bullet_last_used = None;
            }

            results.push_str(&self.handle(child, &Info::new("\n", "\n"), Some(parent))?);

            if let Some(next_child) = children_iter.peek() {
                self.between(child, next_child, parent, &mut results);
            }

            index += 1;
        }

        self.index_stack.pop();

        Ok(results)
    }

    fn between(&self, left: &Node, right: &Node, parent: &Node, results: &mut String) {
        if self.options.tight_definitions {
            Self::set_between(&self.tight_definition(left, right), results)
        } else {
            Self::set_between(&self.join_defaults(left, right, parent), results)
        }
    }

    fn set_between(join: &Join, results: &mut String) {
        if let Join::Break = join {
            results.push_str("\n\n");
        } else if let Join::Lines(n) = join {
            if *n == 1 {
                results.push_str("\n\n");
                return;
            }
            results.push_str("\n".repeat(1 + n).as_ref());
            return;
        } else if let Join::HTMLComment = join {
            results.push_str("\n\n<!---->\n\n");
        }
    }

    fn tight_definition(&self, left: &Node, right: &Node) -> Join {
        if matches!(left, Node::Definition(_)) && Self::matches((left, right)) {
            return Join::Lines(0);
        }
        Join::Break
    }

    fn join_defaults(&self, left: &Node, right: &Node, parent: &Node) -> Join {
        if let Node::Code(code) = right {
            if format_code_as_indented(code, self) && matches!(left, Node::List(_)) {
                return Join::HTMLComment;
            }

            if let Node::Code(code) = left {
                if format_code_as_indented(code, self) {
                    return Join::HTMLComment;
                }
            }
        }

        if matches!(parent, Node::List(_) | Node::ListItem(_)) {
            if matches!(left, Node::Paragraph(_)) {
                if Self::matches((left, right)) {
                    return Join::Break;
                }

                if matches!(right, Node::Definition(_)) {
                    return Join::Break;
                }

                if let Node::Heading(heading) = right {
                    if format_heading_as_setext(heading, self) {
                        return Join::Break;
                    }
                }
            }

            let spread = if let Node::List(list) = parent {
                list.spread
            } else if let Node::ListItem(list_item) = parent {
                list_item.spread
            } else {
                false
            };

            if spread {
                return Join::Lines(1);
            }

            return Join::Lines(0);
        }

        Join::Break
    }

    fn matches(nodes: (&Node, &Node)) -> bool {
        matches!(
            nodes,
            (Node::Root(_), Node::Root(_))
                | (Node::Blockquote(_), Node::Blockquote(_))
                | (Node::Definition(_), Node::Definition(_))
                | (Node::FootnoteDefinition(_), Node::FootnoteDefinition(_))
                | (Node::Heading(_), Node::Heading(_))
                | (Node::List(_), Node::List(_))
                | (Node::ListItem(_), Node::ListItem(_))
                | (Node::Paragraph(_), Node::Paragraph(_))
                | (Node::Table(_), Node::Table(_))
        )
    }

    pub fn indent_lines(&self, value: &str, map: impl Fn(&str, usize, bool) -> String) -> String {
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

    pub fn association(&self, node: &impl Association) -> String {
        if node.label().is_some() || node.identifier().is_empty() {
            return node.label().clone().unwrap_or_default();
        }

        let character_escape_or_reference =
            RegexBuilder::new(r"\\([!-/:-@\[-`{-~])|&(#(?:\d{1,7}|x[\da-f]{1,6})|[\da-z]{1,31});")
                .case_insensitive(true)
                .build()
                .unwrap();

        character_escape_or_reference
            .replace_all(node.identifier(), Self::decode)
            .into_owned()
    }

    fn decode(caps: &Captures) -> String {
        if let Some(first_cap) = caps.get(1) {
            return String::from(first_cap.as_str());
        }

        if let Some(head) = &caps[2].chars().nth(0) {
            if *head == '#' {
                let radix = match caps[2].chars().nth(1) {
                    Some('x') | Some('X') => 16,
                    _ => 10,
                };
                let capture = &caps[2];
                let numeric_encoded = if radix == 16 {
                    &capture[2..]
                } else {
                    &capture[1..]
                };
                return markdown::decode_numeric(numeric_encoded, radix);
            }
        }

        markdown::decode_named(&caps[2], true).unwrap_or(caps[0].to_string())
    }
}
