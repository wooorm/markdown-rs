use crate::construct_name::ConstructName;
use crate::handle::strong::peek_strong;
use crate::handle::Handle;
use crate::message::Message;
use crate::Options;
use crate::{
    parents::Parent,
    r#unsafe::Unsafe,
    util::{
        format_code_as_indented::format_code_as_indented,
        format_heading_as_setext::format_heading_as_setext,
        pattern_in_scope::pattern_in_scope,
        safe::{escape_backslashes, EscapeInfos, SafeConfig},
    },
};
use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use markdown::mdast::Node;
use regex::Regex;

enum Join {
    Number(usize),
    Bool(bool),
}

#[allow(dead_code)]
pub struct State<'a> {
    stack: Vec<ConstructName>,
    // We use i64 for index_stack because -1 is used to mark the absense of children.
    // We don't use index_stack values to index into any child.
    index_stack: Vec<i64>,
    bullet_last_used: Option<String>,
    r#unsafe: Vec<Unsafe<'a>>,
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

#[allow(dead_code)]
impl<'a> State<'a> {
    pub fn new(options: &'a Options) -> Self {
        State {
            stack: Vec::new(),
            index_stack: Vec::new(),
            bullet_last_used: None,
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

    pub fn handle(&mut self, node: &Node, info: &Info) -> Result<String, Message> {
        match node {
            Node::Paragraph(paragraph) => paragraph.handle(self, info),
            Node::Text(text) => text.handle(self, info),
            Node::Strong(strong) => strong.handle(self, info),
            _ => Err(Message {
                reason: "Cannot handle node".into(),
            }),
        }
    }

    pub fn safe(&mut self, input: &String, config: &SafeConfig) -> String {
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
                    let full_match = m.get(0).unwrap();
                    let captured_group_len = if let Some(captured_group) = m.get(1) {
                        captured_group.len()
                    } else {
                        0
                    };

                    let before = pattern.before.is_some() || pattern.at_break.unwrap_or(false);
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
            // This will never panic because we're checking the correct bounds, and we
            // gurantee to have the positions as key in the infos map before reaching this
            // execution.
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
                    Self::encode(config, char_at_pos, &mut result)
                }
                Some(character) => {
                    let code = u32::from(character);
                    let hex_string = format!("{:X}", code);
                    result.push_str(&format!("&#x{};", hex_string));
                    start += 1;
                }
                _ => (),
            };
        }

        result.push_str(&escape_backslashes(&value[start..end], config.after));

        result
    }

    fn encode(config: &SafeConfig, char_at_pos: Option<char>, result: &mut String) {
        match &config.encode {
            Some(encode) => {
                if encode.contains(&char_at_pos.unwrap()) {
                    result.push('\\');
                }
            }
            None => result.push('\\'),
        }
    }

    fn compile_pattern(pattern: &mut Unsafe) {
        if pattern.compiled.is_none() {
            let before = if pattern.at_break.unwrap_or(false) {
                "[\\r\\n][\\t ]*"
            } else {
                ""
            };

            let before = format!(
                "{}{}",
                before,
                pattern
                    .before
                    .map_or(String::new(), |before| format!("(?:{})", before))
            );

            let before = if before.is_empty() {
                String::new()
            } else {
                format!("({})", before)
            };

            let after = pattern
                .after
                .map_or(String::new(), |after| format!("(?:{})", after));

            let special_char = if Regex::new(r"[\|\{}\()\[\]\\\^\$\+\*\?\.\-]")
                .unwrap()
                .is_match(pattern.character)
            {
                "\\"
            } else {
                ""
            };

            let regex = format!("{}{}{}{}", before, special_char, pattern.character, after);
            pattern.set_compiled(Regex::new(&regex).unwrap());
        }
    }

    pub fn container_phrasing<T: Parent>(
        &mut self,
        parent: &T,
        info: &Info,
    ) -> Result<String, Message> {
        let mut results: String = String::new();
        let mut children_iter = parent.children().iter().peekable();
        let mut index = 0;

        self.index_stack.push(-1);

        while let Some(child) = children_iter.next() {
            if let Some(top) = self.index_stack.last_mut() {
                *top = index;
            }

            let mut new_info = Info::new(info.before, info.after);
            let mut buffer = [0u8; 4];
            if let Some(child) = children_iter.peek() {
                if let Some(first_char) = self.determine_first_char(child) {
                    new_info.after = first_char.encode_utf8(&mut buffer);
                } else {
                    self.handle(child, &Info::new("", ""))?
                        .chars()
                        .nth(0)
                        .unwrap_or_default()
                        .encode_utf8(&mut buffer);
                }
            }

            if !results.is_empty() {
                new_info.before = &results[results.len() - 1..];
            }

            results.push_str(&self.handle(child, &new_info)?);
            index += 1;
        }

        self.index_stack.pop();

        Ok(results)
    }

    fn determine_first_char(&self, node: &Node) -> Option<char> {
        match node {
            Node::Strong(_) => Some(peek_strong(self)),
            _ => None,
        }
    }

    fn container_flow<T: Parent>(&mut self, parent: &T, _info: &Info) -> Result<String, Message> {
        let mut results: String = String::new();
        let mut children_iter = parent.children().iter().peekable();
        let mut index = 0;

        self.index_stack.push(-1);

        while let Some(child) = children_iter.next() {
            if let Some(top) = self.index_stack.last_mut() {
                *top = index;
            }

            if matches!(child, Node::List(_)) {
                self.bullet_last_used = None;
            }

            results.push_str(&self.handle(child, &Info::new("\n", "\n"))?);

            if let Some(next_child) = children_iter.peek() {
                self.set_between(child, next_child, parent, &mut results);
            }

            index += 1;
        }

        self.index_stack.pop();

        Ok(results)
    }

    fn set_between<T: Parent>(&self, left: &Node, right: &Node, parent: &T, results: &mut String) {
        match self.join_defaults(left, right, parent) {
            Some(Join::Number(num)) => {
                if num == 1 {
                    results.push_str("\n\n");
                } else {
                    results.push_str("\n".repeat(1 + num).as_ref());
                }
            }
            Some(Join::Bool(bool)) => {
                if bool {
                    results.push_str("\n\n");
                } else {
                    results.push_str("\n\n<!---->\n\n");
                }
            }
            None => results.push_str("\n\n"),
        }
    }

    fn join_defaults<T: Parent>(&self, left: &Node, right: &Node, parent: &T) -> Option<Join> {
        if format_code_as_indented(right, self)
            && (matches!(left, Node::List(_)) || format_code_as_indented(left, self))
        {
            return Some(Join::Bool(false));
        }

        if let Some(spread) = parent.spreadable() {
            if matches!(left, Node::Paragraph(_)) && Self::matches((left, right))
                || matches!(right, Node::Definition(_))
                || format_heading_as_setext(right, self)
            {
                return None;
            }

            if spread {
                return Some(Join::Number(1));
            }

            return Some(Join::Number(0));
        }

        Some(Join::Bool(true))
    }

    fn matches(nodes: (&Node, &Node)) -> bool {
        matches!(
            nodes,
            (Node::Root(_), Node::Root(_))
                | (Node::BlockQuote(_), Node::BlockQuote(_))
                | (Node::FootnoteDefinition(_), Node::FootnoteDefinition(_))
                | (Node::Heading(_), Node::Heading(_))
                | (Node::List(_), Node::List(_))
                | (Node::ListItem(_), Node::ListItem(_))
                | (Node::Paragraph(_), Node::Paragraph(_))
                | (Node::Table(_), Node::Table(_))
        )
    }
}
