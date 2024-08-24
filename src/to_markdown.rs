use crate::mdast::{Node, Paragraph, Root, Text};
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
}

impl State {
    pub fn new() -> Self {
        State { stack: Vec::new() }
    }

    fn enter(&mut self, name: ConstructName) {
        self.stack.push(name);
    }

    fn exit(&mut self) {
        self.stack.pop();
    }

    pub fn handle(&mut self, node: Node) -> String {
        match node {
            Node::Root(root) => self.handle_root(root),
            Node::Paragraph(paragraph) => self.handle_paragraph(paragraph),
            Node::Text(text) => self.handle_text(text),
            _ => panic!("Not handled yet"),
        }
    }

    fn handle_root(&mut self, node: Root) -> String {
        self.container_flow(node)
    }

    fn handle_paragraph(&mut self, node: Paragraph) -> String {
        self.enter(ConstructName::Paragraph);

        self.enter(ConstructName::Phrasing);
        let value = self.container_phrasing(node);
        // exit phrasing
        self.exit();
        // exit paragarph
        self.exit();
        value
    }

    fn container_phrasing<T: PhrasingParent>(&mut self, parent: T) -> String {
        let mut results = Vec::new();

        for (_, child) in parent.children().into_iter().enumerate() {
            results.push(self.handle(child));
        }

        results.into_iter().collect()
    }

    fn container_flow<T: FlowParent>(&mut self, _parent: T) -> String {
        String::new()
    }

    fn handle_text(&self, text: Text) -> String {
        self.safe(text.value)
    }

    fn safe(&self, value: String) -> String {
        value
    }
}

pub fn serialize(tree: Node) -> String {
    let mut state = State::new();
    let result = state.handle(tree);
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
