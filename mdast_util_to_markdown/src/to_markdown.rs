use crate::{
    parents::Parent,
    r#unsafe::{Construct, Unsafe},
};
use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};
use markdown::mdast::{Node, Paragraph, Root, Strong, Text};
use regex::Regex;

use crate::ConstructName;

trait PeekNode {
    // TODO make it take a reference to the state options
    fn handle_peek(&self) -> String;
}

impl PeekNode for Strong {
    fn handle_peek(&self) -> String {
        "*".into()
    }
}

enum Join {
    Number(usize),
    Bool(bool),
}

#[allow(dead_code)]
struct State<'a> {
    stack: Vec<ConstructName>,
    // We use i64 for index_stack because -1 is used to mark the absense of children.
    // We don't use index_stack values to index into any child.
    index_stack: Vec<i64>,
    bullet_last_used: Option<String>,
    r#unsafe: Vec<Unsafe<'a>>,
}

#[allow(dead_code)]
struct Info<'a> {
    before: &'a str,
    after: &'a str,
}

impl<'a> Info<'a> {
    fn new(before: &'a str, after: &'a str) -> Self {
        Info { before, after }
    }
}

#[allow(dead_code)]
struct SafeConfig<'a> {
    before: &'a str,
    after: &'a str,
    encode: Option<Vec<&'a str>>,
}

impl<'a> SafeConfig<'a> {
    fn new(before: Option<&'a str>, after: Option<&'a str>, encode: Option<Vec<&'a str>>) -> Self {
        SafeConfig {
            before: before.unwrap_or(""),
            after: after.unwrap_or(""),
            encode,
        }
    }
}

struct EscapeInfos {
    before: bool,
    after: bool,
}

impl<'a> State<'a> {
    fn new() -> Self {
        State {
            stack: Vec::new(),
            index_stack: Vec::new(),
            bullet_last_used: None,
            r#unsafe: Unsafe::get_default_unsafe(),
        }
    }

    fn enter(&mut self, name: ConstructName) {
        self.stack.push(name);
    }

    fn exit(&mut self) {
        self.stack.pop();
    }

    fn handle(&mut self, node: &Node, info: &Info) -> String {
        match node {
            Node::Root(root) => self.handle_root(root, info),
            Node::Paragraph(paragraph) => self.handle_paragraph(paragraph, info),
            Node::Text(text) => self.handle_text(text, info),
            Node::Strong(strong) => self.handle_strong(strong, info),
            _ => panic!("Not handled yet"),
        }
    }

    fn handle_root(&mut self, node: &Root, info: &Info) -> String {
        self.container_flow(node, info)
    }

    fn handle_paragraph(&mut self, node: &Paragraph, info: &Info) -> String {
        self.enter(ConstructName::Paragraph);

        self.enter(ConstructName::Phrasing);
        let value = self.container_phrasing(node, info);
        // exit phrasing
        self.exit();
        // exit paragarph
        self.exit();
        value
    }

    fn handle_text(&mut self, text: &Text, info: &Info) -> String {
        self.safe(
            &text.value,
            &SafeConfig::new(Some(info.before), Some(info.after), None),
        )
    }

    fn handle_strong(&mut self, node: &Strong, info: &Info) -> String {
        let marker = check_strong(self);

        self.enter(ConstructName::Strong);

        let mut value = format!(
            "{}{}{}",
            marker,
            marker,
            self.container_phrasing(node, info)
        );
        value.push(marker);
        value.push(marker);

        self.exit();

        value
    }

    fn safe(&mut self, input: &String, config: &SafeConfig) -> String {
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

            let char_match = Regex::new(r"[!-/:-@\[-{-~]").unwrap();
            if let Some(char_at_pos) = char_match.find_at(&value, *position).iter().next() {
                match &config.encode {
                    Some(encode) => {
                        if encode.contains(&char_at_pos.as_str()) {
                            result.push('\\');
                        }
                    }
                    None => result.push('\\'),
                }
            } else if let Some(character) = value.chars().nth(*position) {
                let code = u32::from(character);
                let hex_string = format!("{:X}", code);
                result.push_str(&format!("&#x{};", hex_string));
                start += 1;
            }
        }

        result.push_str(&escape_backslashes(&value[start..end], config.after));

        result
    }

    fn compile_pattern(pattern: &mut Unsafe) {
        if !pattern.is_compiled() {
            let before = if pattern.at_break.unwrap_or(false) {
                r"[\\r\\n][\\t ]*"
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
                r"\"
            } else {
                ""
            };

            let regex = format!("{}{}{}{}", before, special_char, pattern.character, after);
            pattern.set_compiled(Regex::new(&regex).unwrap());
        }
    }

    fn container_phrasing<T: Parent>(&mut self, parent: &T, info: &Info) -> String {
        let mut results: String = String::new();
        let mut children_iter = parent.children().iter().peekable();
        let mut index = 0;

        self.index_stack.push(-1);

        while let Some(child) = children_iter.next() {
            if let Some(top) = self.index_stack.last_mut() {
                *top = index;
            }

            let after = if let Some(child) = children_iter.peek() {
                match Self::determine_first_char(child) {
                    Some(after_char) => after_char,
                    None => self
                        .handle(child, &Info::new("", ""))
                        .chars()
                        .nth(0)
                        .unwrap_or_default()
                        .to_string(),
                }
            } else {
                String::from(info.after)
            };

            if results.is_empty() {
                results.push_str(&self.handle(child, &Info::new(info.before, after.as_ref())));
            } else {
                results.push_str(&self.handle(
                    child,
                    &Info::new(&results[results.len() - 1..], after.as_ref()),
                ));
            }

            index += 1;
        }

        self.index_stack.pop();

        results
    }

    fn determine_first_char(node: &Node) -> Option<String> {
        match node {
            Node::Strong(strong) => Some(strong.handle_peek()),
            _ => None,
        }
    }

    fn container_flow<T: Parent>(&mut self, parent: &T, _info: &Info) -> String {
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

            results.push_str(&self.handle(child, &Info::new("\n", "\n")));

            if let Some(next_child) = children_iter.peek() {
                self.set_between(child, next_child, parent, &mut results);
            }

            index += 1;
        }

        self.index_stack.pop();

        results
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
                || format_heading_as_settext(right, self)
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

fn check_strong(_state: &State) -> char {
    '*'
}

fn escape_backslashes(value: &str, after: &str) -> String {
    let expression = Regex::new(r"\\[!-/:-@\[-`{-~]").unwrap();
    let mut results: String = String::new();
    let whole = format!("{}{}", value, after);

    let positions: Vec<usize> = expression.find_iter(&whole).map(|m| m.start()).collect();
    let mut start = 0;

    for position in &positions {
        if start != *position {
            results.push_str(&value[start..*position]);
        }

        results.push('\\');

        start = *position;
    }

    results.push_str(&value[start..]);

    results
}

fn pattern_in_scope(stack: &[ConstructName], pattern: &Unsafe) -> bool {
    list_in_scope(stack, &pattern.in_construct, true)
        && !list_in_scope(stack, &pattern.not_in_construct, false)
}

// This could use a better name
fn list_in_scope(stack: &[ConstructName], list: &Option<Construct>, none: bool) -> bool {
    let Some(list) = list else {
        return none;
    };
    match list {
        Construct::Single(construct_name) => {
            if stack.contains(construct_name) {
                return true;
            }

            false
        }
        Construct::List(constructs_names) => {
            if constructs_names.is_empty() {
                return none;
            }

            for construct_name in constructs_names {
                if stack.contains(construct_name) {
                    return true;
                }
            }

            false
        }
    }
}

fn format_code_as_indented(node: &Node, _state: &State) -> bool {
    if let Node::Code(code) = node {
        let white_space = Regex::new(r"[^ \r\n]").unwrap();
        let blank = Regex::new(r"^[\t ]*(?:[\r\n]|$)|(?:^|[\r\n])[\t ]*$").unwrap();

        return !code.value.is_empty()
            && code.lang.is_none()
            && white_space.is_match(&code.value)
            && !blank.is_match(&code.value);
    }

    false
}

fn format_heading_as_settext(node: &Node, _state: &State) -> bool {
    if let Node::Heading(heading) = node {
        let line_break = Regex::new(r"\r?\n|\r").unwrap();
        let mut literal_with_break = false;
        for child in &heading.children {
            if include_literal_with_break(child, &line_break) {
                literal_with_break = true;
                break;
            }
        }

        return heading.depth == 0
            || heading.depth < 3 && !node.to_string().is_empty() && literal_with_break;
    }

    false
}

fn include_literal_with_break(node: &Node, regex: &Regex) -> bool {
    match node {
        Node::Break(_) => true,
        Node::MdxjsEsm(x) => regex.is_match(&x.value),
        Node::Toml(x) => regex.is_match(&x.value),
        Node::Yaml(x) => regex.is_match(&x.value),
        Node::InlineCode(x) => regex.is_match(&x.value),
        Node::InlineMath(x) => regex.is_match(&x.value),
        Node::MdxTextExpression(x) => regex.is_match(&x.value),
        Node::Html(x) => regex.is_match(&x.value),
        Node::Text(x) => regex.is_match(&x.value),
        Node::Code(x) => regex.is_match(&x.value),
        Node::Math(x) => regex.is_match(&x.value),
        Node::MdxFlowExpression(x) => regex.is_match(&x.value),
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    if include_literal_with_break(child, regex) {
                        return true;
                    }
                }
            }

            false
        }
    }
}

pub fn serialize(tree: &Node) -> String {
    let mut state = State::new();
    let result = state.handle(tree, &Info::new("\n", "\n"));
    result
}

#[cfg(test)]
mod init_tests {
    use super::*;
    use alloc::{string::String, vec};

    use markdown::mdast::{Node, Paragraph, Text};

    #[test]
    fn it_works_for_simple_text() {
        let text_a = Node::Text(Text {
            value: String::from("a"),
            position: None,
        });
        let text_b = Node::Text(Text {
            value: String::from("b"),
            position: None,
        });
        let paragraph = Node::Paragraph(Paragraph {
            children: vec![text_a, text_b],
            position: None,
        });
        let actual = serialize(&paragraph);
        assert_eq!(actual, String::from("ab"));
    }

    #[test]
    fn it_escape() {
        let text_a = Node::Text(Text {
            value: String::from("![](a.jpg)"),
            position: None,
        });
        let paragraph = Node::Paragraph(Paragraph {
            children: vec![text_a],
            position: None,
        });
        let actual = serialize(&paragraph);
        assert_eq!(actual, "!\\[]\\(a.jpg)");
    }

    #[test]
    fn it_will_strong() {
        let text_a = Node::Text(Text {
            value: String::from("a"),
            position: None,
        });

        let text_b = Node::Text(Text {
            value: String::from("b"),
            position: None,
        });
        let strong = Node::Strong(Strong {
            children: vec![text_a, text_b],
            position: None,
        });
        let actual = serialize(&strong);
        assert_eq!(actual, "**ab**");
    }
}
