use crate::{
    mdast::{List, Node, Paragraph, Root, Strong, Text},
    util::{
        format_code_as_indented::format_code_as_indented,
        format_heading_as_setext::format_heading_as_settext,
    },
};
use alloc::{string::String, vec::Vec};

#[allow(dead_code)]
pub enum ConstructName {
    Autolink,
    Blockquote,
    CodeIndented,
    CodeFenced,
    CodeFencedLangGraveAccent,
    CodeFencedLangTilde,
    CodeFencedMetaGraveAccent,
    CodeFencedMetaTilde,
    Definition,
    DestinationLiteral,
    DestinationRaw,
    Emphasis,
    HeadingAtx,
    HeadingSetext,
    Image,
    ImageReference,
    Label,
    Link,
    LinkReference,
    List,
    ListItem,
    Paragraph,
    Phrasing,
    Reference,
    Strong,
    TitleApostrophe,
    TitleQuote,
}

pub trait PeekNode {
    // TODO make it take a reference to the state options
    fn handle_peek(&self) -> String;
}

impl PeekNode for Strong {
    fn handle_peek(&self) -> String {
        "*".into()
    }
}

pub trait PhrasingParent {
    fn children(&self) -> &Vec<Node>;
}

pub trait FlowParent {
    fn children(&self) -> &Vec<Node>;

    // `parent` has a `spread` field.
    fn spread(&self) -> Option<bool> {
        None
    }
}

impl FlowParent for List {
    fn children(&self) -> &Vec<Node> {
        &self.children
    }

    fn spread(&self) -> Option<bool> {
        Some(self.spread)
    }
}

macro_rules! impl_PhrasingParent {
    (for $($t:ty),+) => {
        $(impl PhrasingParent for $t {
            fn children(&self) -> &Vec<Node> {
                &self.children
            }
        })*
    }
}

macro_rules! impl_FlowParent {
    (for $($t:ty),+) => {
        $(impl FlowParent for $t {
            fn children(&self) -> &Vec<Node> {
                &self.children
            }
        })*
    }
}

impl_PhrasingParent!(for Paragraph);
impl_FlowParent!(for Root);

pub enum Join {
    Number(usize),
    Bool(bool),
}

#[allow(dead_code)]
pub struct State {
    pub stack: Vec<ConstructName>,
    // SAFETY : -1 is used to mark the absense of children.
    // We don't use index_stack values to index into any child.
    pub index_stack: Vec<i64>,
    pub bullet_last_used: Option<String>,
}

#[allow(dead_code)]
pub struct Info<'a> {
    pub before: &'a str,
    pub after: &'a str,
}

impl<'a> Info<'a> {
    pub fn new(before: &'a str, after: &'a str) -> Self {
        Info { before, after }
    }
}

impl State {
    pub fn new() -> Self {
        State {
            stack: Vec::new(),
            index_stack: Vec::new(),
            bullet_last_used: None,
        }
    }

    fn enter(&mut self, name: ConstructName) {
        self.stack.push(name);
    }

    fn exit(&mut self) {
        self.stack.pop();
    }

    pub fn handle(&mut self, node: &Node, info: Info) -> String {
        match node {
            Node::Root(root) => self.handle_root(root, info),
            Node::Paragraph(paragraph) => self.handle_paragraph(paragraph, info),
            Node::Text(text) => self.handle_text(text, info),
            _ => panic!("Not handled yet"),
        }
    }

    fn handle_root(&mut self, node: &Root, info: Info) -> String {
        self.container_flow(node, info)
    }

    fn handle_paragraph(&mut self, node: &Paragraph, info: Info) -> String {
        self.enter(ConstructName::Paragraph);

        self.enter(ConstructName::Phrasing);
        let value = self.container_phrasing(node, info);
        // exit phrasing
        self.exit();
        // exit paragarph
        self.exit();
        value
    }

    fn handle_text(&self, text: &Text, _info: Info) -> String {
        self.safe(text.value.clone())
    }

    fn safe(&self, value: String) -> String {
        value
    }

    fn container_phrasing<T: PhrasingParent>(&mut self, parent: &T, info: Info) -> String {
        let mut results: Vec<String> = Vec::new();
        let mut children_iter = parent.children().into_iter().peekable();
        let mut index = 0;

        self.index_stack.push(-1);

        while let Some(child) = children_iter.next() {
            if let Some(top) = self.index_stack.last_mut() {
                *top = index;
            }

            let mut after: String = "".into();
            if let Some(child) = children_iter.peek() {
                after = match self.determine_first_char(child) {
                    Some(after_char) => after_char,
                    None => self
                        .handle(child, Info::new("", ""))
                        .chars()
                        .next()
                        .map(|c| c.into())
                        .unwrap_or_default(),
                };
            }

            if let Some(result) = results.last() {
                results.push(self.handle(
                    child,
                    Info::new(&result[result.len() - 1..], after.as_ref()),
                ));
            } else {
                results.push(self.handle(child, Info::new(info.before, after.as_ref())));
            }

            index += 1;
        }
        self.index_stack.pop();
        results.into_iter().collect()
    }

    fn determine_first_char(&self, node: &Node) -> Option<String> {
        match node {
            Node::Strong(strong) => Some(strong.handle_peek()),
            _ => None,
        }
    }

    fn container_flow<T: FlowParent>(&mut self, parent: &T, _info: Info) -> String {
        let mut results: Vec<String> = Vec::new();

        let mut children_iter = parent.children().into_iter().peekable();
        let mut index: usize = 0;

        self.index_stack.push(-1);

        while let Some(child) = children_iter.next() {
            if let Some(top) = self.index_stack.last_mut() {
                *top = index as i64;
            }

            if matches!(child, Node::List(_)) {
                self.bullet_last_used = None;
            }

            results.push(self.handle(child, Info::new("\n", "\n")));

            if let Some(next_child) = children_iter.peek() {
                results.push(self.between(&child, next_child, parent));
            }

            index += 1;
        }

        results.into_iter().collect()
    }

    fn between<T: FlowParent>(&self, left: &Node, right: &Node, parent: &T) -> String {
        match self.join_defaults(left, right, parent) {
            Some(Join::Number(num)) => {
                if num == 1 {
                    "\n\n".into()
                } else {
                    "\n".repeat(1 + num)
                }
            }
            Some(Join::Bool(bool)) => {
                if bool {
                    "\n\n".into()
                } else {
                    "\n\n<!---->\n\n".into()
                }
            }
            None => "\n\n".into(),
        }
    }

    fn join_defaults<T: FlowParent>(&self, left: &Node, right: &Node, parent: &T) -> Option<Join> {
        if format_code_as_indented(right, self)
            && (matches!(left, Node::List(_)) || format_code_as_indented(left, self))
        {
            return Some(Join::Bool(false));
        }

        if let Some(spread) = parent.spread() {
            if matches!(left, Node::Paragraph(_)) && Self::matches((left, right))
                || matches!(right, Node::Definition(_))
                || format_heading_as_settext(right, self)
            {
                return None;
            }

            if spread {
                return Some(Join::Number(1));
            } else {
                return Some(Join::Number(0));
            }
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

pub fn serialize(tree: Node) -> String {
    let mut state = State::new();
    let result = state.handle(&tree, Info::new("\n".into(), "\n".into()));
    result
}

#[cfg(test)]
mod init_tests {
    use super::*;
    use alloc::{string::String, vec};

    use crate::mdast::{Node, Paragraph, Text};

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
        let actual = serialize(paragraph);
        assert_eq!(actual, String::from("ab"));
    }
}
