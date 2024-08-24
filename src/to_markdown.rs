use crate::mdast::{Node, Paragraph, Root};
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
    fn children(&self) -> &Vec<Node>;
}

pub trait FlowParent {
    fn children(&self) -> &Vec<Node>;
}

impl FlowParent for Root {
    fn children(&self) -> &Vec<Node> {
        &self.children
    }
}

impl PhrasingParent for Paragraph {
    fn children(&self) -> &Vec<Node> {
        &self.children
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

    pub fn handle(mut self, node: Node) -> String {
        match node {
            Node::Root(root) => self.handle_root(root),
            Node::Paragraph(paragarph) => self.handle_paragraph(paragarph),
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

    fn container_phrasing<T: PhrasingParent>(&self, _parent: T) -> String {
        String::new()
    }

    fn container_flow<T: FlowParent>(&self, _parent: T) -> String {
        String::new()
    }
}

pub fn serialize(tree: Node) -> String {
    let state = State::new();
    let result = state.handle(tree);
    result
}
