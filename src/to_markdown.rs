use crate::mdast::{Node, Paragraph, Root, Strong, Text};
use alloc::{string::String, vec::Vec};

#[allow(dead_code)]
pub enum ConstructName {
    /// Whole autolink.
    /// Example:
    /// > `<https://example.com>` and `<admin@example.com>`
    Autolink,

    /// Whole block quote.
    /// Example:
    /// > `> a`
    /// > `b`
    Blockquote,

    /// Whole code (indented).
    /// Example:
    /// > `    console.log(1)`
    CodeIndented,

    /// Whole code (fenced).
    /// Example:
    /// > ` ```js`
    /// > `console.log(1)`
    /// > ` ````
    CodeFenced,

    /// Code (fenced) language, when fenced with grave accents.
    /// Example:
    /// > ` ```js`
    CodeFencedLangGraveAccent,

    /// Code (fenced) language, when fenced with tildes.
    /// Example:
    /// > ` ~~~js`
    CodeFencedLangTilde,

    /// Code (fenced) meta string, when fenced with grave accents.
    /// Example:
    /// > ` ```js eval`
    CodeFencedMetaGraveAccent,

    /// Code (fenced) meta string, when fenced with tildes.
    /// Example:
    /// > ` ~~~js eval`
    CodeFencedMetaTilde,

    /// Whole definition.
    /// Example:
    /// > `[a]: b "c"`
    Definition,

    /// Destination (literal) (occurs in definition, image, link).
    /// Example:
    /// > `[a]: <b> "c"`
    /// > `a ![b](<c> "d") e`
    DestinationLiteral,

    /// Destination (raw) (occurs in definition, image, link).
    /// Example:
    /// > `[a]: b "c"`
    /// > `a ![b](c "d") e`
    DestinationRaw,

    /// Emphasis.
    /// Example:
    /// > `*a*`
    Emphasis,

    /// Whole heading (atx).
    /// Example:
    /// > `# alpha`
    HeadingAtx,

    /// Whole heading (setext).
    /// Example:
    /// > `alpha`
    /// > `=====`
    HeadingSetext,

    /// Whole image.
    /// Example:
    /// > `![a](b)`
    Image,

    /// Whole image reference.
    /// Example:
    /// > `![a]`
    ImageReference,

    /// Label (occurs in definitions, image reference, image, link reference, link).
    /// Example:
    /// > `[a]: b "c"`
    /// > `a [b] c`
    /// > `a ![b][c] d`
    /// > `a [b](c) d`
    Label,

    /// Whole link.
    /// Example:
    /// > `[a](b)`
    Link,

    /// Whole link reference.
    /// Example:
    /// > `[a]`
    LinkReference,

    /// List.
    /// Example:
    /// > `* a`
    /// > `1. b`
    List,

    /// List item.
    /// Example:
    /// > `* a`
    /// > `1. b`
    ListItem,

    /// Paragraph.
    /// Example:
    /// > `a b`
    /// > `c.`
    Paragraph,

    /// Phrasing (occurs in headings, paragraphs, etc).
    /// Example:
    /// > `a`
    Phrasing,

    /// Reference (occurs in image, link).
    /// Example:
    /// > `[a][]`
    Reference,

    /// Strong.
    /// Example:
    /// > `**a**`
    Strong,

    /// Title using single quotes (occurs in definition, image, link).
    /// Example:
    /// > `[a](b 'c')`
    TitleApostrophe,

    /// Title using double quotes (occurs in definition, image, link).
    /// Example:
    /// > `[a](b "c")`
    TitleQuote,
}

pub trait PeekNode {
    // @todo make it take a reference to the state options
    fn handle_peek(&self) -> String;
}

impl PeekNode for Strong {
    fn handle_peek(&self) -> String {
        "*".into()
    }
}

pub trait PhrasingParent {
    fn children(self) -> Vec<Node>;
}

pub trait FlowParent {
    fn children(self) -> Vec<Node>;
}

impl FlowParent for Root {
    fn children(self) -> Vec<Node> {
        self.children
    }
}

impl PhrasingParent for Paragraph {
    fn children(self) -> Vec<Node> {
        self.children
    }
}

struct State {
    stack: Vec<ConstructName>,
    index_stack: Vec<i64>,
}

struct Info<'a> {
    before: &'a str,
    after: &'a str,
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
        }
    }

    fn enter(&mut self, name: ConstructName) {
        self.stack.push(name);
    }

    fn exit(&mut self) {
        self.stack.pop();
    }

    pub fn handle(&mut self, node: Node, info: Info) -> String {
        match node {
            Node::Root(root) => self.handle_root(root, info),
            Node::Paragraph(paragraph) => self.handle_paragraph(paragraph, info),
            Node::Text(text) => self.handle_text(text, info),
            _ => panic!("Not handled yet"),
        }
    }

    fn handle_root(&mut self, node: Root, info: Info) -> String {
        self.container_flow(node, info)
    }

    fn handle_paragraph(&mut self, node: Paragraph, info: Info) -> String {
        self.enter(ConstructName::Paragraph);

        self.enter(ConstructName::Phrasing);
        let value = self.container_phrasing(node, info);
        // exit phrasing
        self.exit();
        // exit paragarph
        self.exit();
        value
    }

    fn handle_text(&self, text: Text, _info: Info) -> String {
        self.safe(text.value)
    }

    fn container_phrasing<T: PhrasingParent>(&mut self, parent: T, info: Info) -> String {
        let mut results: Vec<String> = Vec::new();

        let mut children_iter = parent.children().into_iter().peekable();
        let mut index = 0;
        // SAFETY : -1 is used to mark the absense of children, we make sure to never use this as
        // an index before checking the presense of a child.
        self.index_stack.push(-1);

        let index_stack_size = self.index_stack.len();
        while let Some(child) = children_iter.next() {
            self.index_stack[index_stack_size - 1] = index;

            let mut after: String = "".into();
            if let Some(child_node) = children_iter.peek() {
                after = match self.determine_first_char(child_node) {
                    Some(after_char) => after_char,
                    None => self
                        .handle(child_node.clone(), Info::new("", ""))
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

    fn container_flow<T: FlowParent>(&mut self, _parent: T, _info: Info) -> String {
        String::new()
    }

    fn safe(&self, value: String) -> String {
        value
    }
}

pub fn serialize(tree: Node) -> String {
    let mut state = State::new();
    let result = state.handle(tree, Info::new("\n".into(), "\n".into()));
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
