use markdown::{
    extension::{
        construct, BoxedConstruct, Construct, Container, ContentType, Ctx, Flow, HtmlEvent,
        HtmlWriter, Reach, RunToken, Step, Text, Token, TokenType, Url,
    },
    mdast::{Blockquote, Custom, Node, Paragraph, Root, Text as TextNode},
    to_html_with_options, to_mdast,
    unist::Position,
    CompileOptions, Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;
use std::convert::TryFrom;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Tokens of a mention.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MentionToken {
    Mention,
    Marker,
    Name,
}

impl TokenType for MentionToken {
    const ALL: &'static [Self] = &[Self::Mention, Self::Marker, Self::Name];

    fn name(self) -> &'static str {
        match self {
            Self::Mention => "mention",
            Self::Marker => "mentionMarker",
            Self::Name => "mentionName",
        }
    }
}

/// States of a mention.
#[derive(Clone, Copy, Debug)]
enum MentionState {
    Start,
    Name,
}

/// `@` followed by one byte that is not whitespace, turned into a `mention`
/// node that keeps the token values it was given.
struct Mention {
    marker: u8,
    /// Second bytes this construct accepts.
    accept: &'static [u8],
}

impl Construct for Mention {
    type Place = Text;
    type State = MentionState;
    type Token = MentionToken;

    fn markers(&self) -> &[u8] {
        core::slice::from_ref(&self.marker)
    }

    fn start(&self) -> MentionState {
        MentionState::Start
    }

    fn step(&self, state: MentionState, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (MentionState::Start, Some(byte)) if byte == self.marker => t
                .enter(MentionToken::Mention)
                .enter(MentionToken::Marker)
                .consume()
                .exit()
                .next(MentionState::Name),
            (MentionState::Name, Some(byte)) if self.accept.contains(&byte) => {
                t.enter(MentionToken::Name).consume().exit().exit().ok()
            }
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, MentionToken>>) -> Node {
        Node::Custom(Custom {
            name: "mention".into(),
            attributes: tokens
                .iter()
                .map(|token| (token.name.name().into(), token.value.clone().into_owned()))
                .collect(),
            ..Custom::default()
        })
    }
}

fn options(constructs: Vec<Mention>) -> ParseOptions {
    ParseOptions {
        text_constructs: constructs.into_iter().map(construct).collect(),
        ..ParseOptions::default()
    }
}

fn mention() -> Mention {
    Mention {
        marker: b'@',
        accept: b"abc<",
    }
}

/// Children of the first paragraph.
fn phrasing(tree: Node) -> Vec<Node> {
    match tree {
        Node::Root(Root { mut children, .. }) => match children.remove(0) {
            Node::Paragraph(Paragraph { children, .. }) => children,
            node => panic!("expected paragraph, got {:?}", node),
        },
        node => panic!("expected root, got {:?}", node),
    }
}

fn custom(node: &Node) -> &Custom {
    match node {
        Node::Custom(custom) => custom,
        node => panic!("expected custom node, got {:?}", node),
    }
}

/// Run `parse` with a construct that breaks `rule`: with the `test-util`
/// feature, check that it panics, naming the rule; without, return what it
/// returns.
fn breaking<T>(rule: &str, parse: impl FnOnce() -> T) -> Option<T> {
    let result = catch_unwind(AssertUnwindSafe(parse));
    if cfg!(feature = "test-util") {
        let error = result.err().expect("expected a panic with `test-util`");
        let message = error
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| error.downcast_ref::<&str>().copied())
            .unwrap_or_default();
        assert!(
            message.contains(&format!("`{rule}`")),
            "should name the rule `{}` in the panic, got {:?}",
            rule,
            message
        );
        None
    } else {
        Some(result.expect("expected no panic without `test-util`"))
    }
}

#[test]
fn construct_makes_node_from_its_tokens() {
    let children = phrasing(to_mdast("x @a y", &options(vec![mention()])).unwrap());
    let node = custom(&children[1]);

    assert_eq!(
        node.attributes
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("mention", "@a"),
            ("mentionMarker", "@"),
            ("mentionName", "a")
        ],
        "should pass each token's name and value"
    );
    assert_eq!(
        node.position,
        Some(Position::new(1, 3, 2, 1, 5, 4)),
        "should give the node the position of the outer token"
    );
    assert_eq!(children.len(), 3, "should keep the text around it");
}

#[test]
fn falls_back_to_text_when_construct_does_not_match() {
    assert_eq!(
        phrasing(to_mdast("x @z y", &options(vec![mention()])).unwrap()),
        vec![Node::Text(TextNode {
            value: "x @z y".into(),
            position: Some(Position::new(1, 1, 0, 1, 7, 6)),
        })],
        "should discard the construct's events and keep the bytes as text"
    );
}

#[test]
fn works_inside_containers() {
    let tree = to_mdast("> x\n> @a", &options(vec![mention()])).unwrap();
    let quote = tree.children().unwrap()[0].children().unwrap();
    let children = quote[0].children().unwrap();

    let node = children
        .iter()
        .find(|node| matches!(node, Node::Custom(_)))
        .expect("expected a mention");

    assert_eq!(
        custom(node).attributes.get("mention").map(String::as_str),
        Some("@a"),
        "should see the stream without the `> ` prefix"
    );
}

#[test]
fn runs_before_builtin_constructs() {
    let star = Mention {
        marker: b'*',
        accept: b"a",
    };
    let children = phrasing(to_mdast("*a*", &options(vec![star])).unwrap());

    assert!(
        matches!(children[0], Node::Custom(_)),
        "should try the construct before emphasis at `*`"
    );
}

#[test]
fn tries_next_construct_after_no_match() {
    let only_a = Mention {
        marker: b'@',
        accept: b"a",
    };
    let only_b = Mention {
        marker: b'@',
        accept: b"b",
    };
    let children = phrasing(to_mdast("@a @b", &options(vec![only_a, only_b])).unwrap());

    assert_eq!(
        custom(&children[2])
            .attributes
            .get("mentionName")
            .map(String::as_str),
        Some("b"),
        "should try the second construct when the first does not match"
    );
}

#[test]
fn to_html_writes_construct_source_as_text() {
    let options = Options {
        parse: options(vec![mention()]),
        ..Options::default()
    };

    assert_eq!(
        to_html_with_options("x @<http://a> y", &options).unwrap(),
        "<p>x @&lt;http://a&gt; y</p>",
        "should keep and encode the source of constructs it cannot render"
    );
}

/// Tokens of scripted constructs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tok {
    A,
    B,
    Braces,
    BracesData,
    Line,
    Percent,
    Letter,
    Mid,
    /// `b`, as text content.
    BText,
    /// `c`, as text content.
    CText,
    BracesContent,
    BracesPrefix,
    Pair,
    Left,
    Right,
    PairPrefix,
    Raw,
    Leaf,
    LeafName,
    LeafLabel,
    Fenced,
    Fence,
}

impl TokenType for Tok {
    const ALL: &'static [Self] = &[
        Self::A,
        Self::B,
        Self::Braces,
        Self::BracesData,
        Self::Line,
        Self::Percent,
        Self::Letter,
        Self::Mid,
        Self::BText,
        Self::CText,
        Self::BracesContent,
        Self::BracesPrefix,
        Self::Pair,
        Self::Left,
        Self::Right,
        Self::PairPrefix,
        Self::Raw,
        Self::Leaf,
        Self::LeafName,
        Self::LeafLabel,
        Self::Fenced,
        Self::Fence,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B | Self::BText => "b",
            Self::Braces => "braces",
            Self::BracesData => "bracesData",
            Self::Line => "line",
            Self::Percent => "percent",
            Self::Letter => "letter",
            Self::Mid => "mid",
            Self::CText => "c",
            Self::BracesContent => "bracesContent",
            Self::BracesPrefix => "bracesPrefix",
            Self::Pair => "pair",
            Self::Left => "left",
            Self::Right => "right",
            Self::PairPrefix => "pairPrefix",
            Self::Raw => "raw",
            Self::Leaf => "leaf",
            Self::LeafName => "leafName",
            Self::LeafLabel => "leafLabel",
            Self::Fenced => "fenced",
            Self::Fence => "fence",
        }
    }

    fn content(self) -> Option<ContentType> {
        match self {
            Self::BText
            | Self::CText
            | Self::BracesContent
            | Self::Left
            | Self::Right
            | Self::LeafLabel => Some(ContentType::Text),
            _ => None,
        }
    }
}

/// A step of a scripted construct, with numbered states.
type StepFn = fn(u16, Ctx<'_, '_, Scripted>) -> Step<Scripted>;

/// A construct scripted by a function, with `tokens[0].value` as its value.
struct Scripted {
    marker: u8,
    step: StepFn,
}

impl Construct for Scripted {
    type Place = Text;
    type State = u16;
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        core::slice::from_ref(&self.marker)
    }

    fn start(&self) -> u16 {
        0
    }

    fn step(&self, state: u16, t: Ctx<'_, '_, Self>) -> Step<Self> {
        (self.step)(state, t)
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, Tok>>) -> Node {
        scripted_node(tokens)
    }
}

/// The node of a scripted construct: its first value, the names of its
/// tokens, the values of its other tokens, and its content.
fn scripted_node<T: TokenType>(tokens: Vec<Token<'_, T>>) -> Node {
    Node::Custom(Custom {
        name: "scripted".into(),
        value: Some(tokens[0].value.clone().into_owned()),
        fields: vec![(
            "tokens".into(),
            tokens
                .iter()
                .map(|token| token.name.name())
                .collect::<Vec<_>>()
                .join(","),
        )]
        .into_iter()
        .collect(),
        attributes: tokens[1..]
            .iter()
            .map(|token| (token.name.name().into(), token.value.clone().into_owned()))
            .collect(),
        children: tokens
            .into_iter()
            .flat_map(|token| token.children)
            .collect(),
        ..Custom::default()
    })
}

/// `{{`, anything including line endings (as `bracesData`), `}}`.
fn braces(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
    match (state, t.current()) {
        (0, Some(b'{')) => t.enter(Tok::Braces).consume().next(1),
        (1, Some(b'{')) => t.consume().enter(Tok::BracesData).next(2),
        (2, Some(b'}')) => t.exit().consume().next(3),
        (2, Some(_)) => t.consume().next(2),
        (3, Some(b'}')) => t.consume().exit().ok(),
        _ => t.nok(),
    }
}

fn scripted_at(marker: u8, step: StepFn) -> ParseOptions {
    ParseOptions {
        text_constructs: vec![construct(Scripted { marker, step })],
        ..ParseOptions::default()
    }
}

fn scripted(step: StepFn) -> ParseOptions {
    scripted_at(b'{', step)
}

fn find_scripted(node: &Node) -> Option<&Custom> {
    match node {
        Node::Custom(custom) if custom.name == "scripted" => Some(custom),
        _ => node.children()?.iter().find_map(find_scripted),
    }
}

#[test]
fn joins_tokens_across_lines_without_container_prefixes() {
    for input in ["> {{a\n> b}}", "- {{a\n  b}}", "{{a\nb}}"] {
        let tree = to_mdast(input, &scripted(braces)).unwrap();
        let node = find_scripted(&tree).expect("expected a match");

        assert_eq!(
            node.value.as_deref(),
            Some("{{a\nb}}"),
            "should join the outer token without prefixes in {input:?}"
        );
        assert_eq!(
            node.attributes.get("bracesData").map(String::as_str),
            Some("a\nb"),
            "should join inner tokens without prefixes in {input:?}"
        );
    }
}

#[test]
fn to_html_writes_multiline_source_without_prefixes() {
    let options = Options {
        parse: scripted(braces),
        ..Options::default()
    };

    assert_eq!(
        to_html_with_options("> {{*a*\n> b}}", &options).unwrap(),
        "<blockquote>\n<p>{{*a*\nb}}</p>\n</blockquote>"
    );
}

#[test]
fn broken_constructs_leave_text() {
    let cases: Vec<(&str, StepFn)> = vec![
        ("ok-without-bytes", |_, t| t.ok()),
        ("consume-outside-token", |_, t| t.consume().ok()),
        ("consume-outside-token", |state, t| match state {
            0 => t.enter(Tok::A).consume().exit().next(1),
            1 => t.consume().next(2),
            _ => t.ok(),
        }),
        ("exit-without-token", |_, t| {
            t.exit().enter(Tok::A).consume().exit().ok()
        }),
        ("ok-with-open-tokens", |_, t| t.enter(Tok::A).consume().ok()),
        ("ok-with-open-tokens", |_, t| {
            t.enter(Tok::A).enter(Tok::B).consume().exit().ok()
        }),
        ("token-after-match", |state, t| match state {
            0 => t.enter(Tok::A).consume().exit().enter(Tok::B).next(1),
            _ => t.consume().exit().ok(),
        }),
        ("empty-token", |_, t| {
            t.enter(Tok::A).exit().enter(Tok::B).consume().exit().ok()
        }),
        ("token-after-content", |state, t| match state {
            0 => t.enter(Tok::A).consume().enter(Tok::BText).next(1),
            1 => t.consume().next(2),
            _ => t.enter(Tok::Mid).consume().exit().exit().exit().ok(),
        }),
        ("content-in-content", |state, t| match state {
            0 => t.enter(Tok::A).consume().enter(Tok::BText).next(1),
            1 => t.consume().next(2),
            _ => t.enter(Tok::CText).consume().exit().exit().exit().ok(),
        }),
        // Its content, `{y`, does not match again: without the rule, the
        // construct matches.
        ("content-at-start", |state, t| match (state, t.current()) {
            (0, _) => t.enter(Tok::A).enter(Tok::BText).consume().next(1),
            (1, _) => t.consume().next(2),
            (_, Some(b'z')) => t.exit().consume().exit().ok(),
            _ => t.nok(),
        }),
    ];

    for (rule, step) in cases {
        if let Some(tree) = breaking(rule, || to_mdast("x {yz", &scripted(step)).unwrap()) {
            assert!(
                find_scripted(&tree).is_none(),
                "should not match a construct that breaks `{}`",
                rule
            );
            assert_eq!(
                tree.to_string(),
                "x {yz",
                "should keep text for a construct that breaks `{rule}`"
            );
        }
    }
}

/// Tokens whose `ALL` leaves out `Missing`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Partial {
    Listed,
    Missing,
}

impl TokenType for Partial {
    const ALL: &'static [Self] = &[Self::Listed];

    fn name(self) -> &'static str {
        match self {
            Self::Listed => "listed",
            Self::Missing => "missing",
        }
    }
}

/// `{` and a byte, in a token missing from `ALL` when `missing`.
struct Unlisted {
    missing: bool,
}

impl Construct for Unlisted {
    type Place = Text;
    type State = ();
    type Token = Partial;

    fn markers(&self) -> &[u8] {
        b"{"
    }

    fn start(&self) {}

    fn step(&self, (): (), t: Ctx<'_, '_, Self>) -> Step<Self> {
        let token = if self.missing {
            Partial::Missing
        } else {
            Partial::Listed
        };
        t.enter(token).consume().exit().ok()
    }

    fn to_mdast(&self, _: Vec<Token<'_, Partial>>) -> Node {
        Node::Custom(Custom {
            name: "scripted".into(),
            ..Custom::default()
        })
    }
}

#[test]
fn a_token_missing_from_all_breaks_a_rule() {
    let parse = |missing| ParseOptions {
        text_constructs: vec![construct(Unlisted { missing })],
        ..ParseOptions::default()
    };

    assert!(
        find_scripted(&to_mdast("x {y", &parse(false)).unwrap()).is_some(),
        "should match with a token in `ALL`"
    );

    if let Some(tree) = breaking("token-not-in-all", || {
        to_mdast("x {y", &parse(true)).unwrap()
    }) {
        assert_eq!(
            tree.to_string(),
            "x {y",
            "should keep text for a construct that enters a token missing from `ALL`"
        );
    }
}

#[test]
fn debugs_steps() {
    /// A construct that checks the debug output of what a step gets.
    fn debugged(_: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        assert_eq!(format!("{:?}", t), "Ctx { current: Some(123), .. }");
        let consumed = t.enter(Tok::A).consume();
        assert_eq!(format!("{:?}", consumed), "Consumed { .. }");
        let step = consumed.exit().ok();
        assert_eq!(format!("{:?}", step), "Step { .. }");
        step
    }

    assert!(find_scripted(&to_mdast("{", &scripted(debugged)).unwrap()).is_some());
}

#[test]
fn retrying_forever_leaves_text() {
    let tree = to_mdast("x {y", &scripted(|_, t| t.retry(0))).unwrap();

    assert_eq!(
        tree.to_string(),
        "x {y",
        "should stop a construct that retries 256 times in a row, without a panic"
    );
}

#[test]
fn tokens_inside_a_character_leave_text() {
    let cases: Vec<(&str, u8, StepFn)> = vec![
        // Consume one byte of the two in `é`.
        ("an outer token ending", b'{', |state, t| match state {
            0 => t.enter(Tok::A).consume().next(1),
            _ => t.consume().exit().ok(),
        }),
        ("an inner token ending", b'{', |state, t| match state {
            0 => t.enter(Tok::A).consume().next(1),
            1 => t.enter(Tok::B).consume().exit().next(2),
            _ => t.consume().exit().ok(),
        }),
        // The second byte of `é`.
        ("a token starting", 0xA9, |_, t| {
            t.enter(Tok::A).consume().exit().ok()
        }),
    ];

    for (label, marker, step) in cases {
        let options = Options {
            parse: scripted_at(marker, step),
            ..Options::default()
        };

        if let Some(html) = breaking("token-inside-character", || {
            to_html_with_options("{é", &options).unwrap()
        }) {
            assert_eq!(html, "<p>{é</p>", "should keep text for {label}");
        }
    }
}

#[test]
fn allows_a_retry_before_each_byte() {
    /// `{`, then `a`s, each after a retry.
    fn retry_each(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current()) {
            (0, Some(b'{')) => t.enter(Tok::A).consume().next(1),
            (1, Some(b'a')) => t.retry(2),
            (2, Some(b'a')) => t.consume().next(1),
            (1, _) => t.exit().ok(),
            _ => t.nok(),
        }
    }

    let input = format!("{{{}", "a".repeat(300));
    let mut found = vec![];
    values(
        &to_mdast(&input, &scripted(retry_each)).unwrap(),
        &mut found,
    );

    assert_eq!(
        found,
        vec![input],
        "should allow a retry before each byte of a long match"
    );
}

#[test]
fn line_endings_are_never_markers() {
    let options = Options {
        parse: scripted_at(b'\n', |_, _| {
            unreachable!("should not start at a line ending")
        }),
        ..Options::default()
    };

    assert_eq!(
        to_html_with_options("a  \nb", &options).unwrap(),
        "<p>a<br />\nb</p>",
        "should keep hard breaks working"
    );
}

/// `%`, then bytes, as one token, ending before a line ending or at the end.
fn percent_to_line_end(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
    match (state, t.current()) {
        (0, Some(b'%')) => t.enter(Tok::Percent).consume().next(1),
        (1, Some(b'\n') | None) => t.exit().ok(),
        (1, Some(_)) => t.consume().next(1),
        _ => t.nok(),
    }
}

/// Like `percent_to_line_end`, but wrongly consumes the line ending before
/// ending.
fn percent_through_line_end(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
    match (state, t.current()) {
        (1, Some(b'\n')) => t.consume().exit().ok(),
        _ => percent_to_line_end(state, t),
    }
}

fn percent(step: StepFn) -> ParseOptions {
    scripted_at(b'%', step)
}

fn values(node: &Node, values: &mut Vec<String>) {
    match node {
        Node::Custom(custom) => values.push(custom.value.clone().unwrap_or_default()),
        _ => {
            for child in node.children().into_iter().flatten() {
                self::values(child, values);
            }
        }
    }
}

#[test]
fn ends_before_a_line_ending_without_consuming_it() {
    let mut found = vec![];
    values(
        &to_mdast("> %a\n> %b", &percent(percent_to_line_end)).unwrap(),
        &mut found,
    );

    assert_eq!(found, vec!["%a", "%b"], "should end at the line ending");
}

#[test]
fn does_not_end_right_after_consuming_a_line_ending() {
    let parse = percent(percent_through_line_end);
    if let Some(tree) = breaking("ok-after-line-ending", || {
        to_mdast("> %a\n> %b", &parse).unwrap()
    }) {
        let mut found = vec![];
        values(&tree, &mut found);

        assert_eq!(
            found,
            vec!["%b"],
            "should reject the first match, and keep the later one"
        );
    }
}

#[test]
fn does_not_start_a_token_right_after_consuming_a_line_ending() {
    /// `braces`, with a `line` token from each line ending to the next `}`.
    fn line_after_line_ending(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current()) {
            (2, Some(b'\n')) => t.consume().enter(Tok::Line).next(5),
            (5, Some(b'}')) => t.exit().retry(2),
            (5, Some(_)) => t.consume().next(5),
            _ => braces(state, t),
        }
    }

    let parse = scripted(line_after_line_ending);
    if let Some(tree) = breaking("enter-after-line-ending", || {
        to_mdast("> {{a\n> b}}", &parse).unwrap()
    }) {
        assert!(
            find_scripted(&tree).is_none(),
            "should reject a token started before the next prefix is skipped"
        );
    }
}

#[test]
fn ends_a_token_after_a_line_ending() {
    fn line_ending_then_ok(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current()) {
            (1, Some(b'\n')) => t.consume().exit().next(2),
            (2, _) => t.ok(),
            _ => percent_to_line_end(state, t),
        }
    }

    let mut found = vec![];
    values(
        &to_mdast("> %a\n> %b", &percent(line_ending_then_ok)).unwrap(),
        &mut found,
    );

    assert_eq!(
        found,
        vec!["%a\n", "%b"],
        "should keep the line ending in the token, without the next prefix"
    );
}

#[test]
fn keeps_line_endings_in_tokens() {
    for (input, expected) in [
        ("> {{\n> }}", "\n"),
        ("> {{\n> a}}", "\na"),
        ("> {{\r\n> a}}", "\r\na"),
    ] {
        let tree = to_mdast(input, &scripted(braces)).unwrap();
        let node = find_scripted(&tree).expect("expected a match");

        assert_eq!(
            node.attributes.get("bracesData").map(String::as_str),
            Some(expected),
            "should keep the line ending, without the next prefix, in {input:?}"
        );
    }
}

#[test]
fn leaves_out_tabs_used_by_prefixes() {
    let tree = to_mdast("-\t{{a\n\tb}}", &scripted(braces)).unwrap();

    assert_eq!(
        find_scripted(&tree).and_then(|node| node.value.as_deref()),
        Some("{{a\nb}}"),
        "should match `to_html` and code (text), which leave the tab out"
    );
}

#[test]
fn consuming_at_the_end_does_nothing() {
    fn to_end(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current()) {
            (0, _) => t.enter(Tok::A).consume().next(1),
            (_, Some(_)) => t.consume().next(1),
            (_, None) => t.consume().exit().ok(),
        }
    }

    let tree = to_mdast("x {y", &scripted(to_end)).unwrap();
    assert_eq!(
        find_scripted(&tree).and_then(|node| node.value.as_deref()),
        Some("{y"),
        "should match up to the end, without panicking"
    );
}

/// `mention`, after characters that `allow` accepts.
struct After(fn(Option<char>) -> bool);

impl Construct for After {
    type Place = Text;
    type State = MentionState;
    type Token = MentionToken;

    fn markers(&self) -> &[u8] {
        b"@"
    }

    fn start(&self) -> MentionState {
        MentionState::Start
    }

    fn previous(&self, previous: Option<char>) -> bool {
        (self.0)(previous)
    }

    fn step(&self, state: MentionState, t: Ctx<'_, '_, Self>) -> Step<Self> {
        // The same steps as a mention, for a construct of another type.
        match (state, t.current()) {
            (MentionState::Start, Some(b'@')) => t
                .enter(MentionToken::Mention)
                .consume()
                .next(MentionState::Name),
            (MentionState::Name, Some(b'a'..=b'z')) => {
                t.enter(MentionToken::Name).consume().exit().exit().ok()
            }
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, MentionToken>>) -> Node {
        mention().to_mdast(tokens)
    }
}

fn has_custom(node: &Node) -> bool {
    matches!(node, Node::Custom(_))
        || node
            .children()
            .map_or(false, |children| children.iter().any(has_custom))
}

fn after(allow: fn(Option<char>) -> bool) -> ParseOptions {
    ParseOptions {
        text_constructs: vec![construct(After(allow))],
        ..ParseOptions::default()
    }
}

#[test]
fn checks_the_character_before() {
    let not_after_star = after(|previous| previous != Some('*'));

    for (input, expected) in [
        ("@a", true),
        ("x @a", true),
        ("*@a", false),
        ("\\*@a", true),
    ] {
        assert_eq!(
            has_custom(&to_mdast(input, &not_after_star).unwrap()),
            expected,
            "should start only after allowed characters, with `None` after an escape, in {input:?}"
        );
    }

    let at_line_start = after(|previous| matches!(previous, None | Some('\n')));

    for (input, expected) in [
        ("x\n@a", true),
        ("> x\n> @a", true),
        ("> x\n>@a", true),
        ("> x\n@a", true),
        ("- x\n  @a", true),
        ("x\r\n@a", true),
        ("x @a", false),
    ] {
        assert_eq!(
            has_custom(&to_mdast(input, &at_line_start).unwrap()),
            expected,
            "should see a line ending before a line, without container prefixes, in {input:?}"
        );
    }

    let start = after(|previous| previous.is_none());

    for (input, expected) in [
        ("@a", true),
        ("x\n\n@a", true),
        ("# @a", true),
        ("> @a", true),
        ("- @a", true),
        ("x\n@a", false),
    ] {
        assert_eq!(
            has_custom(&to_mdast(input, &start).unwrap()),
            expected,
            "should see `None` at the start of text, as micromark does, in {input:?}"
        );
    }

    let after_punctuation = after(|previous| {
        previous.map_or(true, |char| {
            char.is_whitespace() || (char.is_ascii_punctuation() || matches!(char, '—' | '«'))
        })
    });

    for (input, expected) in [
        ("—@a", true),
        ("«@a", true),
        ("é@a", false),
        ("x—@a", true),
        ("a\u{1F600}@a", false),
    ] {
        assert_eq!(
            has_custom(&to_mdast(input, &after_punctuation).unwrap()),
            expected,
            "should get the whole character before, not its last byte, in {input:?}"
        );
    }
}

/// States of a run of `{`, other bytes, and a run of `}` of the same size.
#[derive(Clone, Copy, Debug)]
enum Balanced {
    Start,
    Open(usize),
    Inside(usize),
    Close { open: usize, closed: usize },
}

/// A construct whose states carry what it has seen.
struct BalancedBraces;

impl Construct for BalancedBraces {
    type Place = Text;
    type State = Balanced;
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        b"{"
    }

    fn start(&self) -> Balanced {
        Balanced::Start
    }

    fn step(&self, state: Balanced, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (Balanced::Start, Some(b'{')) => t.enter(Tok::Braces).consume().next(Balanced::Open(1)),
            (Balanced::Open(open), Some(b'{')) => t.consume().next(Balanced::Open(open + 1)),
            (
                Balanced::Open(open) | Balanced::Inside(open) | Balanced::Close { open, .. },
                Some(b'}'),
            ) => {
                let closed = match state {
                    Balanced::Close { closed, .. } => closed + 1,
                    _ => 1,
                };
                let t = t.consume();
                if closed == open {
                    t.exit().ok()
                } else {
                    t.next(Balanced::Close { open, closed })
                }
            }
            (Balanced::Open(open) | Balanced::Inside(open), Some(_)) => {
                t.consume().next(Balanced::Inside(open))
            }
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, Tok>>) -> Node {
        Node::Custom(Custom {
            name: "balanced".into(),
            value: Some(tokens[0].value.clone().into_owned()),
            ..Custom::default()
        })
    }
}

#[test]
fn keeps_data_in_states_for_one_match() {
    let parse = ParseOptions {
        text_constructs: vec![construct(BalancedBraces)],
        ..ParseOptions::default()
    };

    for (input, expected) in [
        ("{{a}}", vec!["{{a}}"]),
        // Tried again at the next `{`.
        ("{{a}", vec!["{a}"]),
        ("{a}}", vec!["{a}"]),
        ("{a} {{b}}", vec!["{a}", "{{b}}"]),
    ] {
        let mut found = vec![];
        values(&to_mdast(input, &parse).unwrap(), &mut found);

        assert_eq!(
            found, expected,
            "should keep what a match has seen in its states, from its start state, in {input:?}"
        );
    }
}

#[test]
fn reads_characters() {
    /// `:` and one alphabetic character.
    fn letter(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current_char()) {
            (0, Some(':')) => t.enter(Tok::Letter).consume().next(1),
            (1, Some(char)) if char.is_alphabetic() => t.consume().next(2),
            // The rest of the character.
            (2, None) if t.current().is_some() => t.consume().next(2),
            (2, _) => t.exit().ok(),
            _ => t.nok(),
        }
    }

    let parse = scripted_at(b':', letter);

    for (input, expected) in [
        (":é", vec![":é"]),
        (":a b", vec![":a"]),
        (":\u{1F600}", vec![]),
        (":1", vec![]),
    ] {
        let mut found = vec![];
        values(&to_mdast(input, &parse).unwrap(), &mut found);

        assert_eq!(
            found, expected,
            "should give whole characters, and `None` inside one, in {input:?}"
        );
    }
}

#[test]
fn sees_parse_options() {
    /// `{` and a byte, if GFM autolink literals are on.
    fn with_gfm(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current()) {
            (0, Some(b'{')) if t.options().constructs.gfm_autolink_literal => {
                t.enter(Tok::A).consume().next(1)
            }
            (1, Some(_)) => t.consume().exit().ok(),
            _ => t.nok(),
        }
    }

    let mut found = vec![];
    values(&to_mdast("{x", &scripted(with_gfm)).unwrap(), &mut found);
    assert_eq!(found, Vec::<String>::new(), "should see options (off)");

    let parse = ParseOptions {
        constructs: Constructs::gfm(),
        ..scripted(with_gfm)
    };
    let mut found = vec![];
    values(&to_mdast("{x", &parse).unwrap(), &mut found);
    assert_eq!(found, vec!["{x"], "should see options (on)");
}

/// `{`, then any bytes, to the end of the paragraph.
struct ToEnd(Reach);

impl Construct for ToEnd {
    type Place = Text;
    type State = u16;
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        b"{"
    }

    fn start(&self) -> u16 {
        0
    }

    fn reach(&self) -> Reach {
        self.0
    }

    fn step(&self, state: u16, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'{')) => t.enter(Tok::A).consume().next(1),
            (_, None) => t.exit().ok(),
            (_, Some(_)) => t.consume().next(1),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, Tok>>) -> Node {
        Node::Custom(Custom {
            name: "toEnd".into(),
            value: Some(tokens[0].value.clone().into_owned()),
            ..Custom::default()
        })
    }
}

#[test]
fn stops_a_match_at_its_reach() {
    let reach = |reach: Reach, input: &str| {
        let mut found = vec![];
        let parse = ParseOptions {
            text_constructs: vec![construct(ToEnd(reach))],
            ..ParseOptions::default()
        };
        values(&to_mdast(input, &parse).unwrap(), &mut found);
        found
    };

    assert_eq!(
        reach(Reach::Unbounded, "{abc"),
        vec!["{abc"],
        "should read as far as it needs without a bound"
    );
    assert_eq!(
        reach(Reach::Bytes(4), "{abc"),
        vec!["{abc"],
        "should match within its reach"
    );
    assert_eq!(
        reach(Reach::Bytes(3), "{abc"),
        Vec::<String>::new(),
        "should not match a construct that reads past its reach"
    );
    assert_eq!(
        reach(Reach::Bytes(2), "{a {b"),
        vec!["{b"],
        "should try again at the next marker, with a new reach"
    );
    assert_eq!(
        reach(Reach::Bytes(2), "{a"),
        vec!["{a"],
        "should let a match end at the end of its reach"
    );
}

#[test]
fn errors_with_more_than_255_constructs() {
    let parse = options((0..256).map(|_| mention()).collect());
    let error = to_mdast("@a", &parse).unwrap_err();

    assert_eq!(
        error.to_string(),
        "Unexpected 256 constructs, expected at most 255 (markdown-rs:too-many-constructs)",
        "should error with 256 constructs"
    );
    assert!(
        to_mdast("@a", &options((0..255).map(|_| mention()).collect())).is_ok(),
        "should work with 255 constructs"
    );

    let parse = ParseOptions {
        flow_constructs: vec![construct(FlowAfter(|_| true))],
        ..options((0..255).map(|_| mention()).collect())
    };
    assert!(
        to_mdast("@a", &parse).is_err(),
        "should count text and flow constructs together"
    );

    let parse = ParseOptions {
        document_constructs: vec![construct(Contained::new(line_block, Some(10)))],
        ..options((0..255).map(|_| mention()).collect())
    };
    assert!(
        to_mdast("@a", &parse).is_err(),
        "should count containers too"
    );
}

/// A token type with 300 names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Many(u16);

impl TokenType for Many {
    const ALL: &'static [Self] = &{
        let mut all = [Many(0); 300];
        let mut index = 0;
        while index < all.len() {
            all[index] = Many(index as u16);
            index += 1;
        }
        all
    };

    fn name(self) -> &'static str {
        "many"
    }
}

/// A construct that never matches, with many token names.
struct ManyNames;

impl Construct for ManyNames {
    type Place = Text;
    type State = ();
    type Token = Many;

    fn markers(&self) -> &[u8] {
        b"@"
    }

    fn start(&self) {}

    fn step(&self, (): (), t: Ctx<'_, '_, Self>) -> Step<Self> {
        t.nok()
    }

    fn to_mdast(&self, _: Vec<Token<'_, Many>>) -> Node {
        unreachable!("never matches")
    }
}

#[test]
fn errors_with_more_than_65535_token_names() {
    let many = |count: usize| -> Vec<BoxedConstruct<Text>> {
        (0..count).map(|_| construct(ManyNames)).collect()
    };
    let parse = ParseOptions {
        text_constructs: many(110),
        ..ParseOptions::default()
    };

    assert_eq!(
        to_mdast("@a", &parse).unwrap_err().to_string(),
        "Unexpected 66000 token names in constructs, expected at most 65,535 (markdown-rs:too-many-construct-tokens)",
        "should error before parsing when token names do not fit"
    );

    let parse = ParseOptions {
        text_constructs: many(109),
        ..ParseOptions::default()
    };
    assert!(
        to_mdast("@a", &parse).is_ok(),
        "should work with 65,400 token names, each counted as a token and as a token in content"
    );
}

/// At its marker, a construct that consumes `len` bytes, line endings
/// included, then fails.
struct FailAfter {
    marker: u8,
    len: u16,
}

impl Construct for FailAfter {
    type Place = Text;
    type State = u16;
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        core::slice::from_ref(&self.marker)
    }

    fn start(&self) -> u16 {
        0
    }

    fn step(&self, state: u16, t: Ctx<'_, '_, Self>) -> Step<Self> {
        // Start at a character, as a token cannot start inside one.
        if state == 0 && t.current_char().is_none() {
            return t.nok();
        }
        let t = if state == 0 { t.enter(Tok::A) } else { t };
        if state >= self.len || t.current().is_none() {
            t.nok()
        } else {
            t.consume().next(state + 1)
        }
    }

    fn to_mdast(&self, _: Vec<Token<'_, Tok>>) -> Node {
        unreachable!("never matches")
    }
}

#[test]
fn a_failed_match_changes_nothing() {
    let constructs = |base: &ParseOptions, extra: Option<FailAfter>| ParseOptions {
        constructs: base.constructs.clone(),
        text_constructs: extra.into_iter().map(construct).collect(),
        ..ParseOptions::default()
    };

    // The case from review.
    let gfm = ParseOptions::gfm();
    let input = "> =\n>www.b.c\n";
    assert_eq!(
        format!(
            "{:?}",
            to_mdast(
                input,
                &constructs(
                    &gfm,
                    Some(FailAfter {
                        marker: b'=',
                        len: 2
                    })
                )
            )
        ),
        format!("{:?}", to_mdast(input, &constructs(&gfm, None))),
        "should parse the line after a failed match that consumed its line ending as without it"
    );

    // Random documents of lines that start with container prefixes, against
    // a construct that fails after a random number of bytes.
    let prefixes = [
        "", "", "> ", ">", ">>", "- ", "  ", "1. ", "   ", "\t", "> - ",
    ];
    let pieces = [
        "a",
        "b",
        " ",
        "  ",
        "\t",
        "\n",
        "\n\n",
        "> ",
        ">",
        "- ",
        "1. ",
        "*",
        "_",
        "`",
        "~",
        "=",
        "[",
        "]",
        "(",
        ")",
        "<",
        "!",
        "\\",
        "&amp;",
        "www.",
        "x.y",
        "@",
        ":",
        "|",
        "$",
        "#",
        "---",
        "```",
        "    ",
        "http://a.b",
        "c@d.e",
        "[^1]",
        "[x]: /y",
        "<a>",
        "<!--",
        "-->",
    ];
    let markers = *b"=a>*w[\\$ ";
    let sets = [
        ParseOptions::default(),
        ParseOptions::gfm(),
        ParseOptions {
            constructs: Constructs {
                frontmatter: true,
                math_flow: true,
                math_text: true,
                ..Constructs::gfm()
            },
            ..ParseOptions::default()
        },
    ];
    let compile = CompileOptions {
        allow_dangerous_html: true,
        ..CompileOptions::default()
    };
    // A fixed xorshift, so the documents are the same on every run.
    let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut random = move |below: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        // The remainder is below `below`, a `usize`.
        usize::try_from(seed % below as u64).unwrap()
    };

    for _ in 0..1500 {
        let mut input = String::new();
        for _ in 0..1 + random(4) {
            input.push_str(prefixes[random(prefixes.len())]);
            for _ in 0..random(6) {
                input.push_str(pieces[random(pieces.len())]);
            }
            input.push('\n');
        }
        let marker = markers[random(markers.len())];
        let len = u16::try_from(1 + random(12)).unwrap();

        for base in &sets {
            let html = |parse: ParseOptions| {
                catch_unwind(AssertUnwindSafe(|| {
                    format!(
                        "{:?}",
                        to_html_with_options(
                            &input,
                            &Options {
                                parse,
                                compile: compile.clone()
                            }
                        )
                    )
                }))
                .unwrap_or_else(|_| "panic".into())
            };
            let tree = |parse: ParseOptions| {
                catch_unwind(AssertUnwindSafe(|| {
                    format!("{:?}", to_mdast(&input, &parse))
                }))
                .unwrap_or_else(|_| "panic".into())
            };
            let fail = || Some(FailAfter { marker, len });

            assert_eq!(
                html(constructs(base, fail())),
                html(constructs(base, None)),
                "should give the same HTML after a failed match, in {input:?} at {:?} after {len} bytes",
                char::from(marker)
            );
            assert_eq!(
                tree(constructs(base, fail())),
                tree(constructs(base, None)),
                "should give the same tree after a failed match, in {input:?} at {:?} after {len} bytes",
                char::from(marker)
            );
        }
    }
}

#[test]
fn classifies_characters() {
    use markdown::extension::{classify, CharacterKind};

    for (char, kind) in [
        (' ', CharacterKind::Whitespace),
        ('\u{a0}', CharacterKind::Whitespace),
        ('*', CharacterKind::Punctuation),
        ('\u{2014}', CharacterKind::Punctuation),
        ('\u{a673}', CharacterKind::Punctuation),
        ('a', CharacterKind::Other),
        ('\u{e9}', CharacterKind::Other),
    ] {
        assert_eq!(classify(char), kind, "should classify {:?}", char);
    }
}
/// `{{`, content parsed as text, `}}`.
fn braces_content(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
    match (state, t.current()) {
        (0, Some(b'{')) => t.enter(Tok::Braces).consume().next(1),
        (1, Some(b'{')) => t.consume().enter(Tok::BracesContent).next(2),
        (2, Some(b'}')) => t.exit().consume().next(3),
        (2, Some(_)) => t.consume().next(2),
        (3, Some(b'}')) => t.consume().exit().ok(),
        _ => t.nok(),
    }
}

fn html(input: &str, parse: ParseOptions) -> String {
    to_html_with_options(
        input,
        &Options {
            parse,
            ..Options::default()
        },
    )
    .unwrap()
}

#[test]
fn parses_content_as_markdown() {
    let children = phrasing(to_mdast("a {{*b*}} c", &scripted(braces_content)).unwrap());
    let node = custom(&children[1]);

    assert!(
        matches!(node.children[..], [Node::Emphasis(_)]),
        "should give the content as children, got {:?}",
        node.children
    );
    assert_eq!(
        node.value.as_deref(),
        Some("{{}}"),
        "should leave content out of the values of other tokens"
    );
    assert_eq!(
        node.attributes.get("bracesContent").map(String::as_str),
        Some(""),
        "should give content tokens an empty value"
    );
}

#[test]
fn parses_content_across_lines_without_prefixes() {
    let tree = to_mdast("> a {{*b\n> c*}} d", &scripted(braces_content)).unwrap();
    let node = find_scripted(&tree).expect("expected a match");

    assert_eq!(
        node.children[0].to_string(),
        "b\nc",
        "should parse content across lines, without the `> ` prefix"
    );
    assert_eq!(
        node.children[0].position(),
        Some(&Position::new(1, 7, 6, 2, 5, 13)),
        "should give nested nodes positions in the source"
    );
}

#[test]
fn keeps_whitespace_around_content() {
    let tree = to_mdast("{{ *a* }}", &scripted(braces_content)).unwrap();

    assert_eq!(
        find_scripted(&tree).unwrap().children.len(),
        3,
        "should keep initial and final whitespace, like micromark labels"
    );
}

#[test]
fn parses_constructs_in_content() {
    let parse = ParseOptions {
        text_constructs: vec![
            construct(Scripted {
                marker: b'{',
                step: braces_content,
            }),
            construct(mention()),
        ],
        ..ParseOptions::default()
    };
    let tree = to_mdast("{{x @a y}}", &parse).unwrap();
    let node = find_scripted(&tree).expect("expected a match");

    assert!(
        matches!(&node.children[1], Node::Custom(Custom { name, .. }) if name == "mention"),
        "should parse other constructs in content, got {:?}",
        node.children
    );
}

#[test]
fn drops_empty_content() {
    let empty: StepFn = |state, t| match state {
        0 => t.enter(Tok::A).consume().enter(Tok::BText).next(1),
        _ => t.exit().exit().ok(),
    };
    let tree = to_mdast("{", &scripted(empty)).unwrap();

    assert_eq!(
        find_scripted(&tree).and_then(|node| node.fields.get("tokens").cloned()),
        Some("a".into()),
        "should drop a content token without content"
    );
}

#[test]
fn to_html_renders_content_and_writes_own_text() {
    assert_eq!(
        html("a {{*b*}} c", scripted(braces_content)),
        "<p>a {{<em>b</em>}} c</p>",
        "should render content once, with the construct text around it"
    );
    assert_eq!(
        html("> a {{*b\n> c*}} d", scripted(braces_content)),
        "<blockquote>\n<p>a {{<em>b\nc</em>}} d</p>\n</blockquote>",
        "should write construct text without prefixes"
    );
}

/// `braces_content`, with a `|` token at the start of later lines.
fn braces_prefixed(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
    match (state, t.current()) {
        (2, Some(b'\n')) => t.consume().next(4),
        (4, Some(b'|')) => t.enter(Tok::BracesPrefix).consume().exit().next(2),
        (4, _) => t.retry(2),
        _ => braces_content(state, t),
    }
}

#[test]
fn keeps_tokens_at_line_starts_out_of_content() {
    let tree = to_mdast("{{*a\n|b*}}", &scripted(braces_prefixed)).unwrap();
    let node = find_scripted(&tree).expect("expected a match");

    assert_eq!(
        node.children[0].to_string(),
        "a\nb",
        "should parse content across the token, without it"
    );
    assert_eq!(
        node.fields.get("tokens").map(String::as_str),
        Some("braces,bracesContent"),
        "should not give tokens in content to `to_mdast`"
    );
    assert_eq!(
        html("{{*a\n|b*}}", scripted(braces_prefixed)),
        "<p>{{<em>a\nb</em>}}</p>",
        "should leave tokens in content out of HTML"
    );
}

#[test]
fn content_has_no_blank_lines() {
    // Built-in constructs in text expect no blank lines, as in a paragraph.
    for input in [
        "{{a\n|\n|b}}",
        "{{a\n| \n|b}}",
        "{{[a](b\n|\n|c)}}",
        "{{<a\n|\n|b>}}",
    ] {
        let parse = scripted(braces_prefixed);
        if let Some(tree) = breaking("blank-line-in-content", || to_mdast(input, &parse).unwrap()) {
            assert!(
                find_scripted(&tree).is_none(),
                "should not match content with a blank line in {:?}",
                input
            );
            assert_eq!(
                html(input, scripted(braces_prefixed)),
                to_html_with_options(input, &Options::default()).unwrap(),
                "should keep text for content with a blank line in {:?}",
                input
            );
        }
    }
}

#[test]
fn content_can_start_and_end_with_blank_lines() {
    for (input, value, message) in [
        ("{{ }}", " ", "should keep whitespace-only content"),
        ("{{\na}}", "\na", "should keep a line ending at the start"),
        ("{{a\n}}", "a", "should leave a line ending at the end out"),
        (
            "{{a\n|\n|}}",
            "a",
            "should leave blank lines at the end out, with their prefixes",
        ),
        (
            "{{[x](\n}}",
            "[x](",
            "should leave a line ending at the end out, where text never ends",
        ),
        ("> {{a\n> }}", "a", "should leave it out in a container"),
    ] {
        let tree = to_mdast(input, &scripted(braces_prefixed)).unwrap();
        let node = find_scripted(&tree).expect("expected a match");
        assert_eq!(
            node.children
                .iter()
                .map(ToString::to_string)
                .collect::<String>(),
            value,
            "{} in {:?}",
            message,
            input
        );
    }

    assert_eq!(
        html("{{*a*\n|  \n}}", scripted(braces_prefixed)),
        "<p>{{<em>a</em>}}</p>",
        "should write content without the line endings and blank lines at its end"
    );
}

#[test]
fn tokens_do_not_end_inside_a_tab() {
    /// `>` and the next three bytes, which can end inside a tab.
    fn four_bytes(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match state {
            0 => t.enter(Tok::A).consume().next(1),
            1..=3 => t.consume().next(state + 1),
            _ => t.exit().ok(),
        }
    }

    /// `>` and the whitespace after it, character by character.
    fn whitespace(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current_char()) {
            (0, _) => t.enter(Tok::A).consume().next(1),
            // Columns of a tab after its first.
            (1, None) if t.current().is_some() => t.consume().next(1),
            (1, Some(' ' | '\t')) => t.consume().next(1),
            _ => t.exit().ok(),
        }
    }

    /// `>` and one tab, read as a character.
    fn tab(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current_char()) {
            (0, _) => t.enter(Tok::A).consume().next(1),
            (1, Some('\t')) => t.consume().next(2),
            (2, None) if t.current().is_some() => t.consume().next(2),
            (2, _) => t.exit().ok(),
            _ => t.nok(),
        }
    }

    let gfm = |step| ParseOptions {
        text_constructs: vec![construct(Scripted { marker: b'>', step })],
        ..ParseOptions::gfm()
    };

    let tree = to_mdast("#>\t z", &gfm(tab)).unwrap();
    assert_eq!(
        custom(&tree.children().unwrap()[0].children().unwrap()[1])
            .value
            .as_deref(),
        Some(">\t"),
        "should give `None` as the character inside a tab"
    );

    /// The rest of a tab that a line starts inside of.
    fn rest_of_tab(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current_char()) {
            (0, Some(' ')) => t.enter(Tok::A).consume().next(1),
            (1, None) if t.current().is_some() => t.consume().next(1),
            (1, _) => t.exit().ok(),
            _ => t.nok(),
        }
    }

    assert!(
        find_scripted(&to_mdast("- a\n\t@b", &scripted_at(b' ', rest_of_tab)).unwrap()).is_some(),
        "should start a token where a line starts inside a tab"
    );

    let input = "#>  \tza@b.c";
    let parse = gfm(four_bytes);
    if let Some(tree) = breaking("token-inside-character", || {
        to_mdast(input, &parse).unwrap()
    }) {
        assert_eq!(
            tree,
            to_mdast(input, &ParseOptions::gfm()).unwrap(),
            "should not match a construct that ends inside a tab"
        );
    }

    let tree = to_mdast(input, &gfm(whitespace)).unwrap();
    let paragraph = tree.children().unwrap()[0].children().unwrap();

    assert_eq!(
        custom(&paragraph[1]).value.as_deref(),
        Some(">  \t"),
        "should match a construct that takes the whole tab"
    );
    assert_eq!(
        paragraph[2].position(),
        Some(&Position::new(1, 9, 5, 1, 15, 11)),
        "should keep positions after the tab"
    );
}

#[test]
fn ends_gfm_autolink_literals_at_the_end_of_content() {
    /// `{`, content up to `X`, `X`, `}`.
    fn up_to_x(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current()) {
            (0, Some(b'{')) => t.enter(Tok::A).consume().enter(Tok::BText).next(1),
            (1, Some(b'X')) => t.exit().consume().next(2),
            (1, Some(_)) => t.consume().next(1),
            (2, Some(b'}')) => t.consume().exit().ok(),
            _ => t.nok(),
        }
    }

    let parse = ParseOptions {
        constructs: Constructs::gfm(),
        ..scripted(up_to_x)
    };
    let tree = to_mdast("{www.aX}", &parse).unwrap();
    let node = find_scripted(&tree).expect("expected a match");

    assert!(
        matches!(&node.children[..], [Node::Link(link)] if link.children[0].to_string() == "www.a"),
        "should end a literal where the content ends, got {:?}",
        node.children
    );
}

#[test]
fn errors_for_mdx_jsx_left_open_in_content() {
    let parse = ParseOptions {
        constructs: Constructs::mdx(),
        ..scripted(braces_content)
    };

    assert_eq!(
        to_mdast("a {{<b>}} c", &parse)
            .unwrap_err()
            .rule_id
            .as_str(),
        "end-tag-mismatch",
        "should error like a paragraph does"
    );
}

#[test]
fn errors_for_mdx_jsx_closed_in_other_content() {
    let parse = ParseOptions {
        constructs: Constructs::mdx(),
        ..scripted(braces_content)
    };

    assert_eq!(
        to_mdast("x <a> {{b</a>}} c", &parse)
            .unwrap_err()
            .rule_id
            .as_str(),
        "end-tag-mismatch",
        "should error for a closing tag whose opening tag is outside the content"
    );
}

/// Levels of matches of `scripted` in `node`.
fn depth(node: &Node) -> usize {
    let own = usize::from(matches!(node, Node::Custom(custom) if custom.name == "scripted"));
    own + node
        .children()
        .map_or(0, |children| children.iter().map(depth).max().unwrap_or(0))
}

/// `(`, content with balanced parentheses, `)`: the state is the depth.
fn parens(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
    match (state, t.current()) {
        (0, Some(b'(')) => t.enter(Tok::A).consume().enter(Tok::BText).next(1),
        (1, Some(b')')) => t.exit().consume().exit().ok(),
        (depth, Some(b'(')) if depth > 0 => t.consume().next(depth + 1),
        (depth, Some(b')')) if depth > 0 => t.consume().next(depth - 1),
        (depth, Some(_)) if depth > 0 => t.consume().next(depth),
        _ => t.nok(),
    }
}

#[test]
fn limits_the_depth_of_content() {
    let parse = scripted_at(b'(', parens);
    let depth_of = |levels: usize| {
        let input = format!("{}a{}", "(".repeat(levels), ")".repeat(levels));
        depth(&to_mdast(&input, &parse).unwrap())
    };
    // With the text of the paragraph, 31 levels are 32 levels of content.
    assert_eq!(depth_of(31), 31, "should nest content 32 deep");
    assert_eq!(
        depth_of(40),
        31,
        "should not match content deeper than that, without a panic"
    );
}

/// `{`, content, `,`, content, `}`, with a `|` token allowed at the start of
/// the second content: state 2 is its start, 3 is after it.
fn pair(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
    match (state, t.current()) {
        (0, Some(b'{')) => t.enter(Tok::Pair).consume().enter(Tok::Left).next(1),
        (1, Some(b',')) => t.exit().consume().enter(Tok::Right).next(2),
        (2, Some(b'|')) => t.enter(Tok::PairPrefix).consume().exit().next(3),
        (2 | 3, Some(b'}')) => t.exit().consume().exit().ok(),
        (1, Some(_)) => t.consume().next(1),
        (2 | 3, Some(_)) => t.consume().next(3),
        _ => t.nok(),
    }
}

#[test]
fn parses_each_content_token_alone() {
    let tree = to_mdast("{*a*,*b*}", &scripted(pair)).unwrap();
    let node = find_scripted(&tree).expect("expected a match");

    assert!(
        matches!(node.children[..], [Node::Emphasis(_), Node::Emphasis(_)]),
        "should parse each content token on its own, got {:?}",
        node.children
    );
    assert_eq!(
        html("{*a*,*b*}", scripted(pair)),
        "<p>{<em>a</em>,<em>b</em>}</p>",
        "should render each content token"
    );

    let tree = to_mdast("{a,|b}", &scripted(pair)).unwrap();
    let node = find_scripted(&tree).expect("expected a match");

    assert_eq!(
        node.fields.get("tokens").map(String::as_str),
        Some("pair,left,right"),
        "should allow a token before the content of a later content token"
    );
}

#[test]
fn skips_prefix_tokens_of_an_outer_match() {
    /// `braces_prefixed`, or `{[`, raw text as a token, `]]`.
    fn outer_or_raw(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current()) {
            (1, Some(b'[')) => t.consume().enter(Tok::Raw).next(20),
            (20, Some(b']')) => t.exit().consume().next(21),
            (20, Some(_)) => t.consume().next(20),
            (21, Some(b']')) => t.consume().exit().ok(),
            _ => braces_prefixed(state, t),
        }
    }

    let tree = to_mdast("{{a {[b\n|c]] d}}", &scripted(outer_or_raw)).unwrap();
    let outer = find_scripted(&tree).expect("expected a match");
    let inner = outer
        .children
        .iter()
        .find_map(find_scripted)
        .expect("expected an inner match");

    assert_eq!(
        inner.attributes.get("raw").map(String::as_str),
        Some("b\nc"),
        "should leave the prefix of the outer match out of the inner one"
    );
    assert_eq!(
        inner.fields.get("tokens").map(String::as_str),
        Some("braces,raw"),
        "should not give the prefix of the outer match to the inner one"
    );
}

#[test]
fn limits_leave_text() {
    let cases: Vec<(&str, StepFn)> = vec![
        (
            "an attempt that succeeds without progress, forever",
            |state, t| match state {
                0 => t.enter(Tok::A).consume().next(1),
                1 => t.attempt(10, 1, 2),
                10 => t.ok(),
                _ => t.nok(),
            },
        ),
        (
            "an attempt that fails after progress, forever",
            |state, t| match state {
                0 => t.enter(Tok::A).consume().next(1),
                1 => t.attempt(10, 2, 1),
                10 => t.enter(Tok::B).consume().exit().next(11),
                _ => t.nok(),
            },
        ),
        (
            "nested attempts that fail and are tried again",
            |state, t| {
                if state == 0 {
                    t.enter(Tok::A).consume().next(1)
                } else {
                    t.attempt(1, 1, 1)
                }
            },
        ),
    ];

    for (label, step) in cases {
        let tree = to_mdast("x {yz", &scripted(step)).unwrap();
        assert!(
            find_scripted(&tree).is_none(),
            "should not match past a limit: {}",
            label
        );
        assert_eq!(
            tree.to_string(),
            "x {yz",
            "should keep text past a limit, without a panic: {}",
            label
        );
    }
}

#[test]
fn breaking_a_rule_in_an_attempt_leaves_text() {
    let cases: Vec<(&str, StepFn)> = vec![
        ("exit-before-attempt", |state, t| match state {
            0 => t.enter(Tok::A).enter(Tok::B).consume().exit().next(1),
            1 => t.attempt(10, 2, 2),
            10 => t.exit().nok(),
            _ => t.ok(),
        }),
        ("exit-before-attempt", |state, t| match state {
            0 => t.enter(Tok::A).consume().enter(Tok::BText).next(1),
            1 => t.consume().next(2),
            2 => t.attempt(10, 3, 4),
            3 => t.exit().exit().ok(),
            // Closes the content, which the attempt did not open.
            10 => t.exit().ok(),
            _ => t.nok(),
        }),
        ("ok-with-open-tokens", |state, t| match state {
            0 => t.enter(Tok::A).consume().next(1),
            1 => t.attempt(10, 2, 3),
            10 => t.enter(Tok::B).consume().ok(),
            2 => t.exit().exit().ok(),
            _ => t.nok(),
        }),
    ];

    for (rule, step) in cases {
        if let Some(tree) = breaking(rule, || to_mdast("x {yz", &scripted(step)).unwrap()) {
            assert!(
                find_scripted(&tree).is_none(),
                "should not match a construct that breaks `{}` in an attempt",
                rule
            );
            assert_eq!(
                tree.to_string(),
                "x {yz",
                "should keep text for a construct that breaks `{}` in an attempt",
                rule
            );
        }
    }
}

#[test]
fn undoes_a_failed_attempt() {
    let optional: StepFn = |state, t| match (state, t.current()) {
        (0, Some(b'{')) => t.enter(Tok::A).consume().next(1),
        (1, _) => t.attempt(10, 2, 2),
        (10, Some(b'x')) => t.enter(Tok::B).consume().next(11),
        (11, Some(b'y')) => t.consume().exit().ok(),
        (2, Some(b'}')) => t.consume().exit().ok(),
        (2, Some(_)) => t.consume().next(2),
        _ => t.nok(),
    };
    let tokens = |input| {
        let tree = to_mdast(input, &scripted(optional)).unwrap();
        find_scripted(&tree).and_then(|node| node.fields.get("tokens").cloned())
    };

    assert_eq!(tokens("{xy}"), Some("a,b".into()), "should keep a match");
    assert_eq!(
        tokens("{xz}"),
        Some("a".into()),
        "should undo a failure, and continue at `nok`"
    );
}

#[test]
fn undoes_a_failed_attempt_that_ends_a_line_of_content() {
    let step: StepFn = |state, t| match (state, t.current()) {
        (0, Some(b'{')) => t.enter(Tok::A).consume().enter(Tok::BText).next(1),
        (1, Some(b'x')) => t.consume().next(2),
        (2, _) => t.attempt(10, 3, 3),
        (10, Some(b'\n')) => t.consume().next(11),
        (3, Some(b'}')) => t.exit().consume().exit().ok(),
        (3, Some(_)) => t.consume().next(3),
        _ => t.nok(),
    };
    let tree = to_mdast("{x\ny}", &scripted(step)).unwrap();

    assert_eq!(
        find_scripted(&tree).map(|node| Node::Paragraph(Paragraph {
            children: node.children.clone(),
            position: None
        })
        .to_string()),
        Some("x\ny".into()),
        "should keep content before an attempt that consumed a line ending"
    );
}

#[test]
fn keeps_content_across_attempts_in_it() {
    // `{`, content, and an attempt at `y`, then `}`.
    let step: StepFn = |state, t| match (state, t.current()) {
        (0, Some(b'{')) => t.enter(Tok::A).consume().enter(Tok::BText).next(1),
        (1, Some(b'x')) => t.consume().next(2),
        (2, _) => t.attempt(10, 3, 3),
        // Content up to `y`, which succeeds, or `}`, which fails.
        (10, Some(b'y')) => t.consume().next(11),
        (10, Some(byte)) if byte != b'}' => t.consume().next(10),
        (11, _) => t.ok(),
        (3, _) => t.exit().retry(4),
        (4, Some(b'}')) => t.consume().exit().ok(),
        (4, Some(_)) => t.consume().next(4),
        _ => t.nok(),
    };
    let content = |input| {
        let tree = to_mdast(input, &scripted(step)).unwrap();
        find_scripted(&tree).map(|node| {
            Node::Paragraph(Paragraph {
                children: node.children.clone(),
                position: None,
            })
            .to_string()
        })
    };

    assert_eq!(
        content("{xy}"),
        Some("xy".into()),
        "should keep content that an attempt added"
    );
    assert_eq!(
        content("{x\nz}"),
        Some("x".into()),
        "should undo a line of content that a failed attempt added"
    );
}

#[test]
fn allows_an_attempt_before_each_byte() {
    /// `{`, then `a`s, each consumed in an attempt.
    fn attempt_each(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current()) {
            (0, Some(b'{')) => t.enter(Tok::A).consume().next(1),
            (1, Some(b'a')) => t.attempt(10, 1, 2),
            (10, Some(b'a')) => t.enter(Tok::B).consume().exit().next(11),
            (11, _) => t.ok(),
            (1 | 2, _) => t.exit().ok(),
            _ => t.nok(),
        }
    }

    let input = format!("{{{}", "a".repeat(300));
    let mut found = vec![];
    values(
        &to_mdast(&input, &scripted(attempt_each)).unwrap(),
        &mut found,
    );

    assert_eq!(
        found,
        vec![input],
        "should allow an attempt before each byte of a long match"
    );
}

#[test]
fn restores_content_after_a_failed_attempt() {
    /// `{`, content of `x` and a line ending, where an attempt tries to
    /// take everything up to `}` and fails there, `}`; a `|` token can
    /// start a line of content.
    fn step(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current()) {
            (0, Some(b'{')) => t.enter(Tok::A).consume().enter(Tok::BText).next(1),
            (1, Some(b'x')) => t.consume().next(5),
            (5, Some(b'\n')) => t.consume().next(2),
            (2, _) => t.attempt(10, 4, 4),
            (10, Some(byte)) if byte != b'}' => t.consume().next(10),
            (3, Some(b'\n')) => t.consume().next(4),
            (4, Some(b'|')) => t.enter(Tok::BracesPrefix).consume().exit().next(3),
            (3 | 4, Some(b'}')) => t.exit().consume().exit().ok(),
            (3 | 4, Some(_)) => t.consume().next(3),
            _ => t.nok(),
        }
    }

    for (input, expected) in [
        // New chunks after the undo link to the chunk before the attempt.
        ("{x\nz\nw}", "x\nz\nw"),
        // The line of the last content is that before the attempt.
        ("{x\n|z}", "x\nz"),
    ] {
        let tree = to_mdast(input, &scripted(step)).unwrap();
        let node = find_scripted(&tree).expect("expected a match");

        assert_eq!(
            Node::Paragraph(Paragraph {
                children: node.children.clone(),
                position: None
            })
            .to_string(),
            expected,
            "should continue content after a failed attempt in {:?}",
            input
        );
    }
}

/// A state of a counting construct: a number, and two counters.
type Counted = (u16, usize, usize);

/// A step of a counting construct.
type CountedFn = fn(Counted, Ctx<'_, '_, Counting>) -> Step<Counting>;

/// A construct at `{` whose states carry counts, scripted by a function.
struct Counting(CountedFn);

impl Construct for Counting {
    type Place = Text;
    type State = Counted;
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        b"{"
    }

    fn start(&self) -> Counted {
        (0, 0, 0)
    }

    fn step(&self, state: Counted, t: Ctx<'_, '_, Self>) -> Step<Self> {
        (self.0)(state, t)
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, Tok>>) -> Node {
        Node::Custom(Custom {
            name: "scripted".into(),
            value: Some(tokens[0].value.clone().into_owned()),
            ..Custom::default()
        })
    }
}

fn counting(step: CountedFn) -> ParseOptions {
    ParseOptions {
        text_constructs: vec![construct(Counting(step))],
        ..ParseOptions::default()
    }
}

/// `{` and `a`s, then `N` retries.
fn spend<const N: usize>(state: Counted, t: Ctx<'_, '_, Counting>) -> Step<Counting> {
    match (state, t.current()) {
        ((0, ..), _) => t.enter(Tok::A).consume().next((1, 0, 0)),
        ((1, ..), Some(b'a')) => t.consume().next((1, 0, 0)),
        ((_, retries, _), _) if retries < N => t.retry((2, retries + 1, 0)),
        _ => t.exit().ok(),
    }
}

#[test]
fn limits_steps_per_byte_reached() {
    // 10 bytes and the end allow 2,816 steps: 11 to consume and end, and
    // 2,805 retries.
    let mut found = vec![];
    values(
        &to_mdast("{aaaaaaaaa", &counting(spend::<2805>)).unwrap(),
        &mut found,
    );
    assert_eq!(
        found,
        vec!["{aaaaaaaaa"],
        "should allow steps for each byte reached, anywhere in the match"
    );

    let mut found = vec![];
    values(
        &to_mdast(
            &format!("{{aaaaaaaaa{}{{aaaaaaaaa", " b".repeat(500)),
            &counting(spend::<2000>),
        )
        .unwrap(),
        &mut found,
    );
    assert_eq!(
        found,
        vec!["{aaaaaaaaa", "{aaaaaaaaa"],
        "should count steps for each match, from its start"
    );

    let mut found = vec![];
    values(
        &to_mdast("{aaaaaaaaa", &counting(spend::<2806>)).unwrap(),
        &mut found,
    );
    assert_eq!(
        found,
        Vec::<String>::new(),
        "should not match past 256 steps for each byte reached"
    );

    // Before `{`, an attempt looks at every byte and fails; elsewhere, 1,000
    // retries, which one byte does not allow.
    let look_or_spend: CountedFn = |state, t| match (state, t.current()) {
        ((0, ..), _) => t.enter(Tok::A).consume().next((1, 0, 0)),
        ((1, ..), Some(b'{')) => t.attempt((10, 0, 0), (2, 0, 0), (2, 0, 0)),
        ((10, ..), Some(_)) => t.enter(Tok::B).consume().exit().next((10, 0, 0)),
        ((1 | 2, retries, _), _) if retries < 1000 => t.retry((2, retries + 1, 0)),
        ((1 | 2, ..), _) => t.exit().ok(),
        _ => t.nok(),
    };
    let input = format!("{{{{{}", "a".repeat(20));

    let mut found = vec![];
    values(
        &to_mdast(&input, &counting(look_or_spend)).unwrap(),
        &mut found,
    );
    assert_eq!(
        found,
        vec!["{"],
        "should count bytes a failed attempt reached, but not in a later match"
    );
}

#[test]
fn restores_the_line_start_after_a_failed_attempt() {
    /// `{a`, a line ending, and an attempt that takes the next line and
    /// fails.
    fn step(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current()) {
            (0, _) => t.enter(Tok::A).consume().next(1),
            (1 | 2 | 10 | 11, Some(_)) => t.consume().next(state + 1),
            (3, _) => t.attempt(10, 4, 4),
            (4, _) => t.exit().ok(),
            _ => t.nok(),
        }
    }

    let tree = to_mdast("{a\nb\nc", &scripted(step)).unwrap();
    assert_eq!(
        find_scripted(&tree).and_then(|node| node.value.as_deref()),
        Some("{a\n"),
        "should end the token where the attempt started"
    );
    assert_eq!(
        html("{a\nb\nc", scripted(step)),
        "<p>{a\nb\nc</p>",
        "should continue text where the attempt started"
    );
}

#[test]
fn forgets_the_content_of_an_earlier_match() {
    /// `{`, then content of `c` and failure, or a failed attempt and `ok`.
    fn step(state: u16, t: Ctx<'_, '_, Scripted>) -> Step<Scripted> {
        match (state, t.current()) {
            (0, _) => t.enter(Tok::A).consume().next(1),
            (1, Some(b'c')) => t.enter(Tok::BText).consume().next(2),
            (1, _) => t.attempt(10, 3, 3),
            (3, _) => t.exit().ok(),
            _ => t.nok(),
        }
    }

    let mut found = vec![];
    values(&to_mdast("{c {x", &scripted(step)).unwrap(), &mut found);
    assert_eq!(found, vec!["{"], "should match after an earlier failure");
}

#[test]
fn fails_only_the_attempt_that_breaks_a_rule() {
    let step: StepFn = |state, t| match state {
        0 => t.enter(Tok::A).consume().next(1),
        1 => t.attempt(10, 2, 3),
        // Closes `a`, which the attempt did not open.
        10 => t.exit().ok(),
        3 => t.exit().ok(),
        _ => t.nok(),
    };
    let parse = scripted(step);

    if let Some(tree) = breaking("exit-before-attempt", || to_mdast("{x", &parse).unwrap()) {
        let mut found = vec![];
        values(&tree, &mut found);
        assert_eq!(found, vec!["{"], "should continue at `nok`");
    }
}

#[test]
fn nests_attempts() {
    // The inner attempt fails, the outer one succeeds.
    let inner_fails: StepFn = |state, t| match state {
        0 => t.enter(Tok::A).consume().next(1),
        1 => t.attempt(10, 2, 3),
        10 => t.enter(Tok::B).consume().next(11),
        11 => t.attempt(20, 3, 12),
        20 => t.consume().next(21),
        12 => t.exit().ok(),
        2 => t.exit().ok(),
        _ => t.nok(),
    };
    let tree = to_mdast("{xy", &scripted(inner_fails)).unwrap();
    let node = find_scripted(&tree).expect("expected a match");

    assert_eq!(
        (
            node.value.as_deref(),
            node.fields.get("tokens").map(String::as_str)
        ),
        (Some("{x"), Some("a,b")),
        "should undo an inner attempt and keep the outer one"
    );

    // The inner attempt adds content, the outer one fails.
    let outer_fails: StepFn = |state, t| match state {
        0 => t.enter(Tok::A).consume().enter(Tok::BText).next(1),
        1 => t.attempt(10, 2, 2),
        10 => t.attempt(20, 11, 11),
        20 => t.consume().next(21),
        21 => t.ok(),
        11 => t.consume().next(12),
        2 => t.consume().next(3),
        3 => t.exit().exit().ok(),
        _ => t.nok(),
    };
    let tree = to_mdast("{xy}", &scripted(outer_fails)).unwrap();
    let node = find_scripted(&tree).expect("expected a match");

    assert_eq!(
        (node.value.as_deref(), node.children[0].to_string()),
        (Some("{"), "x".into()),
        "should undo content of an inner attempt with the outer one"
    );
}

#[test]
fn goes_on_with_the_states_given_to_an_attempt() {
    // Counts the attempts in the first counter: the one in `nok` goes on,
    // even though the attempt that changed nothing failed.
    let step: CountedFn = |state, t| match (state, t.current()) {
        ((0, ..), _) => t.enter(Tok::A).consume().next((1, 0, 0)),
        ((1, tries, _), _) if tries < 3 => {
            t.attempt((10, tries, 0), (2, tries, 0), (1, tries + 1, 0))
        }
        ((10, ..), _) => t.nok(),
        ((1, 3, _), Some(b'x')) => t.consume().exit().ok(),
        _ => t.nok(),
    };
    let mut found = vec![];
    values(&to_mdast("{x", &counting(step)).unwrap(), &mut found);

    assert_eq!(
        found,
        vec!["{x"],
        "should go on in the `nok` state with the data it was given"
    );
}

/// `{x`, `N` attempts that cross the line ending and fail, then the line
/// ending and `y`.
fn recross<const N: usize>(state: Counted, t: Ctx<'_, '_, Counting>) -> Step<Counting> {
    match (state, t.current()) {
        ((0, ..), _) => t.enter(Tok::A).consume().next((1, 0, 0)),
        ((1, ..), Some(b'x')) => t.consume().next((2, 0, 0)),
        ((2, tries, _), Some(b'\n')) if tries < N => {
            t.attempt((10, tries, 0), (3, 0, 0), (2, tries + 1, 0))
        }
        ((step @ (2 | 10), ..), Some(b'\n')) => t.consume().next((step + 1, 0, 0)),
        ((3, ..), Some(b'y')) => t.consume().exit().ok(),
        _ => t.nok(),
    }
}

#[test]
fn counts_container_prefixes_it_moves_past() {
    let prefix = ">".repeat(1000);

    assert!(
        find_scripted(&to_mdast("{x\ny", &counting(recross::<300>)).unwrap()).is_some(),
        "should allow 300 attempts across a line ending"
    );
    assert!(
        find_scripted(
            &to_mdast(
                &format!("{} {{x\n{} y", prefix, prefix),
                &counting(recross::<300>)
            )
            .unwrap()
        )
        .is_none(),
        "should count the bytes of a container prefix that each attempt moves past"
    );
}

/// `{` and `a`s, then `N` attempts, each in the one before it: the first
/// counter is the attempts started, the second those still open.
fn nest<const N: usize>(state: Counted, t: Ctx<'_, '_, Counting>) -> Step<Counting> {
    match (state, t.current()) {
        ((0, ..), _) => t.enter(Tok::A).consume().next((1, 0, 0)),
        ((1, ..), Some(b'a')) => t.consume().next((1, 0, 0)),
        ((1 | 2, started, open), _) if started < N => t.attempt(
            (2, started + 1, open + 1),
            (3, started + 1, open),
            (4, 0, 0),
        ),
        // The innermost attempt, and each one around it, succeeds.
        ((2, _, open), _) if open > 0 => t.ok(),
        ((3, _, open), _) if open > 0 => t.ok(),
        ((1..=3, ..), _) => t.exit().ok(),
        _ => t.nok(),
    }
}

#[test]
fn limits_attempts_in_attempts() {
    let input = format!("{{{}", "a".repeat(20));

    assert!(
        find_scripted(&to_mdast(&input, &counting(nest::<256>)).unwrap()).is_some(),
        "should allow 256 attempts in each other"
    );
    assert!(
        find_scripted(&to_mdast(&input, &counting(nest::<257>)).unwrap()).is_none(),
        "should fail an attempt nested more than 256 deep"
    );
}

/// A step of a scripted flow construct, with numbered states.
type FlowFn = fn(u16, Ctx<'_, '_, ScriptedFlow>) -> Step<ScriptedFlow>;

/// A flow construct scripted by a function, with the node of [`Scripted`].
struct ScriptedFlow {
    marker: u8,
    step: FlowFn,
}

impl Construct for ScriptedFlow {
    type Place = Flow;
    type State = u16;
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        core::slice::from_ref(&self.marker)
    }

    fn start(&self) -> u16 {
        0
    }

    fn step(&self, state: u16, t: Ctx<'_, '_, Self>) -> Step<Self> {
        (self.step)(state, t)
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, Tok>>) -> Node {
        scripted_node(tokens)
    }
}

fn scripted_flow(marker: u8, step: FlowFn) -> ParseOptions {
    ParseOptions {
        flow_constructs: vec![construct(ScriptedFlow { marker, step })],
        ..ParseOptions::default()
    }
}

/// `::name[label]`, with the label parsed as text.
fn leaf(state: u16, t: Ctx<'_, '_, ScriptedFlow>) -> Step<ScriptedFlow> {
    match (state, t.current()) {
        (0 | 1, Some(b':')) => {
            let t = if state == 0 { t.enter(Tok::Leaf) } else { t };
            t.consume().next(state + 1)
        }
        (2 | 3, Some(b'a'..=b'z')) => {
            let t = if state == 2 {
                t.enter(Tok::LeafName)
            } else {
                t
            };
            t.consume().next(3)
        }
        (3, Some(b'[')) => t.exit().consume().next(4),
        (4, Some(b']')) => t.consume().next(6),
        (4, Some(_)) => t.enter(Tok::LeafLabel).retry(5),
        (5, Some(b']')) => t.exit().consume().next(6),
        (5, Some(byte)) if byte != b'\n' => t.consume().next(5),
        (6, None | Some(b'\n')) => t.exit().ok(),
        _ => t.nok(),
    }
}

#[test]
fn flow_construct_takes_a_line() {
    let tree = to_mdast("::a[*b*]", &scripted_flow(b':', leaf)).unwrap();
    assert!(
        matches!(&tree.children().unwrap()[0], Node::Custom(node) if matches!(node.children[..], [Node::Emphasis(_)])),
        "should make a block with a parsed label, got {:?}",
        tree
    );

    let tree = to_mdast("x\n::a[b]\ny", &scripted_flow(b':', leaf)).unwrap();
    assert_eq!(
        tree.children().unwrap().len(),
        3,
        "should interrupt a paragraph"
    );

    let tree = to_mdast("x\n::a[b]\n[c]: d", &scripted_flow(b':', leaf)).unwrap();
    assert!(
        matches!(
            &tree.children().unwrap()[..],
            [Node::Paragraph(_), Node::Custom(_), Node::Definition(_)]
        ),
        "should let a definition follow, got {:?}",
        tree
    );

    let parse = ParseOptions {
        constructs: Constructs::mdx(),
        mdx_esm_parse: Some(Box::new(|_| markdown::MdxSignal::Ok)),
        ..scripted_flow(b':', leaf)
    };
    let tree = to_mdast("x\n::a[b]\nimport a from 'b'", &parse).unwrap();
    assert!(
        matches!(
            &tree.children().unwrap()[..],
            [Node::Paragraph(_), Node::Custom(_), Node::MdxjsEsm(_)]
        ),
        "should let flow that cannot interrupt a paragraph follow, got {:?}",
        tree
    );

    let tree = to_mdast("> ::a[b]\n> c", &scripted_flow(b':', leaf)).unwrap();
    assert!(
        matches!(
            &tree.children().unwrap()[0].children().unwrap()[0],
            Node::Custom(_)
        ),
        "should work in containers"
    );

    for input in ["::a[b] x", "::a[b\nc]"] {
        let tree = to_mdast(input, &scripted_flow(b':', leaf)).unwrap();
        assert!(
            find_scripted(&tree).is_none(),
            "should not match a construct that does not end at a line ending: {:?}",
            input
        );
    }
}

#[test]
fn flow_construct_skips_indentation() {
    let tree = to_mdast("   ::a[b]", &scripted_flow(b':', leaf)).unwrap();
    assert!(
        find_scripted(&tree).is_some(),
        "should match after up to 3 columns"
    );

    let tree = to_mdast("    ::a[b]", &scripted_flow(b':', leaf)).unwrap();
    assert!(
        matches!(&tree.children().unwrap()[0], Node::Code(_)),
        "should leave 4 columns to indented code"
    );

    let any: FlowFn = |state, t| match (state, t.current()) {
        (0, _) => t.enter(Tok::A).consume().next(1),
        (1, None | Some(b'\n')) => t.exit().ok(),
        _ => t.consume().next(1),
    };
    assert!(
        find_scripted(&to_mdast("  b", &scripted_flow(b':', any)).unwrap()).is_none(),
        "should not start after indentation without a marker"
    );

    for input in ["  b", "    b"] {
        let tree = to_mdast(input, &scripted_flow(b' ', any)).unwrap();
        assert!(
            find_scripted(&tree).is_none(),
            "should not take a space as a marker, which is indentation, in {:?}",
            input
        );
    }
}

#[test]
fn ends_a_flow_construct_that_fails_after_its_first_line() {
    let late: FlowFn = |state, t| match (state, t.current()) {
        (0, Some(b'%')) => t.enter(Tok::A).consume().next(1),
        (1, Some(b'\n')) => t.consume().next(2),
        (1, Some(_)) => t.consume().next(1),
        _ => t.nok(),
    };
    let parse = scripted_flow(b'%', late);

    if let Some(tree) = breaking("nok-after-first-line", || to_mdast("%a\n*b*", &parse)) {
        let tree = tree.expect("should not error for a failure after the first line");
        let children = tree.children().unwrap();
        assert!(
            matches!(&children[..], [Node::Custom(_), Node::Paragraph(_)]),
            "should end the construct before the line it fails on, which is then a paragraph, got {:?}",
            tree
        );
        assert_eq!(
            to_html_with_options(
                "%a\n*b*",
                &Options {
                    parse: scripted_flow(b'%', late),
                    ..Options::default()
                }
            )
            .unwrap(),
            "%a\n<p><em>b</em></p>",
            "should write the line it fails on as other markdown"
        );
    }

    let spend: FlowFn = |state, t| match (state, t.current()) {
        (0, Some(b'%')) => t.enter(Tok::A).consume().next(1),
        (1, Some(b'\n')) => t.consume().next(2),
        (1, Some(_)) => t.consume().next(1),
        _ => t.retry(2),
    };
    let tree = to_mdast("%a\nb", &scripted_flow(b'%', spend))
        .expect("should not error for a construct past its step budget after its first line");
    assert!(
        matches!(
            &tree.children().unwrap()[..],
            [Node::Custom(_), Node::Paragraph(_)]
        ),
        "should end a construct past its step budget before that line, got {:?}",
        tree
    );
    assert!(
        to_mdast("%a", &parse).is_ok(),
        "should not error for a failure on the first line"
    );
}

#[test]
fn attempts_in_flow_stay_on_one_line() {
    let across: FlowFn = |state, t| match (state, t.current()) {
        (0, Some(b'%')) => t.enter(Tok::A).consume().next(1),
        (1, _) => t.attempt(10, 2, 2),
        (10, Some(b'\n')) => t.enter(Tok::B).consume().next(11),
        (11, _) => t.exit().ok(),
        (2, None | Some(b'\n')) => t.exit().ok(),
        _ => t.nok(),
    };
    let parse = scripted_flow(b'%', across);

    if let Some(tree) = breaking("line-ending-in-attempt", || {
        to_mdast("%\nb", &parse).unwrap()
    }) {
        assert!(
            matches!(
                &tree.children().unwrap()[..],
                [Node::Custom(_), Node::Paragraph(_)]
            ),
            "should fail an attempt that consumes a line ending, got {:?}",
            tree
        );
    }

    let across_then_fail: FlowFn = |state, t| match (state, t.current()) {
        (0, Some(b'%')) => t.enter(Tok::A).consume().next(1),
        (1, _) => t.attempt(10, 2, 2),
        (10, Some(b'\n')) => t.enter(Tok::B).consume().next(11),
        _ => t.nok(),
    };
    let parse = scripted_flow(b'%', across_then_fail);

    if let Some(tree) = breaking("line-ending-in-attempt", || {
        to_mdast("%\nb", &parse).unwrap()
    }) {
        assert!(
            find_scripted(&tree).is_none(),
            "should fail on the first line after an attempt that consumes a line ending"
        );
    }
}

/// `%%%`, raw lines as tokens, and `%%%` or the end: a fenced raw block.
fn fenced(state: u16, t: Ctx<'_, '_, ScriptedFlow>) -> Step<ScriptedFlow> {
    match (state, t.current()) {
        (0, Some(b'%')) => t.enter(Tok::Fenced).enter(Tok::Fence).consume().next(1),
        (1 | 2, Some(b'%')) => t.consume().next(state + 1),
        (3, Some(b'\n')) => t.exit().consume().next(4),
        // At the start of a line, a closing fence, which is tried, or a
        // line.
        (4, Some(b'%')) => t.attempt(10, 7, 5),
        // A blank line.
        (4, Some(b'\n')) => t.consume().next(4),
        (4 | 5, Some(_)) => t.enter(Tok::Line).consume().next(6),
        (6, Some(b'\n')) => t.exit().consume().next(4),
        (6, Some(_)) => t.consume().next(6),
        (6, None) => t.exit().exit().ok(),
        (3 | 4, None) => {
            let t = if state == 3 { t.exit() } else { t };
            t.exit().ok()
        }
        (10, Some(b'%')) => t.enter(Tok::Fence).consume().next(11),
        (11 | 12, Some(b'%')) => t.consume().next(state + 1),
        (13, None | Some(b'\n')) => t.exit().ok(),
        (7, None | Some(b'\n')) => t.exit().ok(),
        _ => t.nok(),
    }
}

#[test]
fn flow_construct_takes_lines() {
    let flow = || scripted_flow(b'%', fenced);
    let tokens = |input: &str| {
        find_scripted(&to_mdast(input, &flow()).unwrap()).map(|node| {
            (
                node.value.clone().unwrap_or_default(),
                node.fields["tokens"].clone(),
            )
        })
    };

    assert_eq!(
        tokens("%%%\na\n\nb\n%%%\nc"),
        Some((
            "%%%\na\n\nb\n%%%".into(),
            "fenced,fence,line,line,fence".into()
        )),
        "should take lines up to a closing fence, blank ones too"
    );
    assert_eq!(
        tokens("> %%%\n> a\n>\n> %%%"),
        Some(("%%%\na\n\n%%%".into(), "fenced,fence,line,fence".into())),
        "should leave container prefixes out of lines"
    );
    assert_eq!(
        tokens("%%%\na"),
        Some(("%%%\na".into(), "fenced,fence,line".into())),
        "should end at the end"
    );

    let tree = to_mdast("> %%%\n> a\nb", &flow()).unwrap();
    assert!(
        matches!(
            &tree.children().unwrap()[..],
            [Node::Blockquote(_), Node::Paragraph(_)]
        ),
        "should end before a lazy line, got {:?}",
        tree
    );
    assert_eq!(
        find_scripted(&tree).and_then(|node| node.value.as_deref()),
        Some("%%%\na"),
        "should end the construct before the lazy line"
    );
    assert_eq!(
        tokens("> %%%\n> - a\n> %%%"),
        Some(("%%%\n- a\n%%%".into(), "fenced,fence,line,fence".into())),
        "should not start containers while the construct runs"
    );
    assert!(
        matches!(
            &to_mdast("%%%\n%%%\n> b", &flow())
                .unwrap()
                .children()
                .unwrap()[..],
            [Node::Custom(_), Node::Blockquote(_)]
        ),
        "should allow containers after the construct"
    );

    // `%` and a line, then another line, which a lazy line is not.
    let two_lines: FlowFn = |state, t| match (state, t.current()) {
        (0, Some(b'%')) => t.enter(Tok::A).consume().next(1),
        (1, Some(b'\n')) => t.consume().next(2),
        (1..=3, Some(byte)) if byte != b'\n' => t.consume().next(if state == 1 { 1 } else { 3 }),
        (3, None | Some(b'\n')) => t.exit().ok(),
        _ => t.nok(),
    };
    let parse = scripted_flow(b'%', two_lines);
    if let Some(tree) = breaking("nok-after-first-line", || to_mdast("> %a\nb", &parse)) {
        assert!(
            matches!(
                &tree.expect("should not error").children().unwrap()[..],
                [Node::Blockquote(_), Node::Paragraph(_)]
            ),
            "should end a construct that fails at a lazy line before it"
        );
    }

    let mut found = vec![];
    values(
        &to_mdast("> %%%\n> a\nb\n\n%%%\nc\n%%%\n%d", &flow()).unwrap(),
        &mut found,
    );
    assert_eq!(
        found,
        vec!["%%%\na", "%%%\nc\n%%%"],
        "should start each construct afresh, after one that ended at a lazy line"
    );

    assert_eq!(
        html("%%%\n<a>\n%%%", flow()),
        "%%%\n&lt;a&gt;\n%%%",
        "should write the source of a flow construct as text"
    );
}

/// `@` and a letter on a line of its own, after characters that `allow`
/// accepts.
struct FlowAfter(fn(Option<char>) -> bool);

impl Construct for FlowAfter {
    type Place = Flow;
    type State = MentionState;
    type Token = MentionToken;

    fn markers(&self) -> &[u8] {
        b"@"
    }

    fn start(&self) -> MentionState {
        MentionState::Start
    }

    fn previous(&self, previous: Option<char>) -> bool {
        (self.0)(previous)
    }

    fn step(&self, state: MentionState, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (MentionState::Start, Some(b'@')) => t
                .enter(MentionToken::Mention)
                .consume()
                .next(MentionState::Name),
            (MentionState::Name, Some(b'a'..=b'z')) => t
                .enter(MentionToken::Name)
                .consume()
                .exit()
                .next(MentionState::Name),
            (MentionState::Name, None | Some(b'\n')) => t.exit().ok(),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, MentionToken>>) -> Node {
        mention().to_mdast(tokens)
    }
}

#[test]
fn flow_constructs_see_a_line_ending_before_them() {
    let after = |allow: fn(Option<char>) -> bool| ParseOptions {
        flow_constructs: vec![construct(FlowAfter(allow))],
        ..ParseOptions::default()
    };
    let line_ending = after(|previous| previous == Some('\n'));
    let start = after(|previous| previous.is_none());

    for (input, expected) in [
        ("@a", false),
        ("x\n\n@a", true),
        ("x\n\n  @a", true),
        ("> x\n>\n> @a", true),
        ("- x\n\n  @a", true),
    ] {
        assert_eq!(
            has_custom(&to_mdast(input, &line_ending).unwrap()),
            expected,
            "should see a line ending before a line, after indentation and prefixes, in {:?}",
            input
        );
    }

    for (input, expected) in [("@a", true), ("  @a", true), ("x\n\n@a", false)] {
        assert_eq!(
            has_custom(&to_mdast(input, &start).unwrap()),
            expected,
            "should see `None` before the first line in {:?}",
            input
        );
    }
}

#[test]
fn broken_flow_constructs_leave_text() {
    let cases: Vec<(&str, FlowFn)> = vec![
        ("ok-before-line-end", |state, t| match state {
            0 => t.enter(Tok::A).consume().next(1),
            _ => t.exit().ok(),
        }),
        ("exit-after-line-ending", |state, t| match state {
            0 => t.enter(Tok::A).consume().next(1),
            1 => t.consume().next(2),
            2 => t.consume().exit().next(3),
            _ => t.nok(),
        }),
    ];

    for (rule, step) in cases {
        let parse = scripted_flow(b'%', step);
        if let Some(tree) = breaking(rule, || to_mdast("%b\nc", &parse).unwrap()) {
            assert!(
                find_scripted(&tree).is_none(),
                "should not match a flow construct that breaks `{}`",
                rule
            );
        }
    }
}

#[test]
fn flow_constructs_see_their_indent() {
    let two: FlowFn = |state, t| {
        if state == 0 && t.indent() != 2 {
            t.nok()
        } else {
            leaf(state, t)
        }
    };

    for (input, expected) in [("  ::a[b]", true), ("::a[b]", false), (" ::a[b]", false)] {
        assert_eq!(
            find_scripted(&to_mdast(input, &scripted_flow(b':', two)).unwrap()).is_some(),
            expected,
            "should give the columns of indentation in {:?}",
            input
        );
    }
}

#[test]
fn parses_flow_content_across_lines() {
    // `%`, then content up to the end or a lazy line.
    let rest: FlowFn = |state, t| match (state, t.current()) {
        (0, Some(b'%')) => t.enter(Tok::A).consume().enter(Tok::BText).next(1),
        (1, Some(_)) => t.consume().next(1),
        (1, None) => t.exit().exit().ok(),
        _ => t.nok(),
    };

    let parse = scripted_flow(b'%', rest);
    if let Some(tree) = breaking("blank-line-in-content", || to_mdast("%a\n\nb", &parse)) {
        assert!(
            matches!(
                &tree.expect("should not error").children().unwrap()[..],
                [Node::Custom(_), Node::Paragraph(_)]
            ),
            "should break content with a blank line after the first line, ending the construct before it"
        );
    }

    for (input, expected) in [
        ("%`a\nb\nc", "`a\nb\nc"),
        ("> %`a\n> b\n> c", "`a\nb\nc"),
        ("%b\n", "b"),
        ("%b\r\n", "b"),
        ("> %a\n> b\n", "a\nb"),
        ("- %a\n", "a"),
    ] {
        let tree = to_mdast(input, &scripted_flow(b'%', rest)).unwrap();
        let node = find_scripted(&tree).expect("expected a match");

        assert_eq!(
            Node::Paragraph(Paragraph {
                children: node.children.clone(),
                position: None
            })
            .to_string(),
            expected,
            "should keep every line of content, and end before a final line ending, in {:?}",
            input
        );
    }

    for input in ["%*a\nb*", "> %*a\n> b*", "> %*a\n> b*\nc"] {
        let tree = to_mdast(input, &scripted_flow(b'%', rest)).unwrap();
        let node = find_scripted(&tree).expect("expected a match");

        assert!(
            matches!(&node.children[..], [Node::Emphasis(emphasis)] if Node::Emphasis(emphasis.clone()).to_string() == "a\nb"),
            "should parse content across lines, without prefixes, in {:?}, got {:?}",
            input,
            node.children
        );
    }
}

/// `%`, text content across lines, `%`, then the end of its line.
fn percent_across_lines(state: u16, t: Ctx<'_, '_, ScriptedFlow>) -> Step<ScriptedFlow> {
    match (state, t.current()) {
        (0, Some(b'%')) => t.enter(Tok::Leaf).consume().next(1),
        (1, Some(b'%') | None) => t.nok(),
        (1, _) => t.enter(Tok::BText).retry(2),
        (2, Some(b'%')) => t.exit().consume().next(3),
        (2, Some(_)) => t.consume().next(2),
        (3, None | Some(b'\n')) => t.exit().ok(),
        _ => t.nok(),
    }
}

#[test]
fn flow_content_leaves_its_last_line_ending_out() {
    for (input, value) in [
        ("%a\n%", "a"),
        ("%a\nb\n%", "a\nb"),
        ("%a\n\n%", "a"),
        ("> %a\n> %", "a"),
        ("%\n%", ""),
    ] {
        let tree = to_mdast(input, &scripted_flow(b'%', percent_across_lines)).unwrap();
        let node = find_scripted(&tree).expect("expected a match");
        assert_eq!(
            node.children
                .iter()
                .map(ToString::to_string)
                .collect::<String>(),
            value,
            "should leave the line endings at the end of {:?} out of its content",
            input
        );
    }
}

/// Tokens of scripted flow constructs with bodies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DocTok {
    A,
    /// `b`, as document content.
    B,
    Container,
    ContainerFence,
    ContainerContent,
    F,
    Open,
    Body,
    Prefix,
    Close,
    Bars,
    BarsBody,
    BarsPrefix,
}

impl TokenType for DocTok {
    const ALL: &'static [Self] = &[
        Self::A,
        Self::B,
        Self::Container,
        Self::ContainerFence,
        Self::ContainerContent,
        Self::F,
        Self::Open,
        Self::Body,
        Self::Prefix,
        Self::Close,
        Self::Bars,
        Self::BarsBody,
        Self::BarsPrefix,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
            Self::Container => "container",
            Self::ContainerFence => "containerFence",
            Self::ContainerContent => "containerContent",
            Self::F => "f",
            Self::Open => "open",
            Self::Body => "body",
            Self::Prefix => "prefix",
            Self::Close => "close",
            Self::Bars => "bars",
            Self::BarsBody => "barsBody",
            Self::BarsPrefix => "barsPrefix",
        }
    }

    fn content(self) -> Option<ContentType> {
        match self {
            Self::B | Self::ContainerContent | Self::Body | Self::BarsBody => {
                Some(ContentType::Document)
            }
            _ => None,
        }
    }
}

/// A step of a scripted flow construct with a body.
type DocFn = fn(u16, Ctx<'_, '_, ScriptedDoc>) -> Step<ScriptedDoc>;

/// A flow construct with a body, scripted by a function.
struct ScriptedDoc {
    marker: u8,
    step: DocFn,
}

impl Construct for ScriptedDoc {
    type Place = Flow;
    type State = u16;
    type Token = DocTok;

    fn markers(&self) -> &[u8] {
        core::slice::from_ref(&self.marker)
    }

    fn start(&self) -> u16 {
        0
    }

    fn step(&self, state: u16, t: Ctx<'_, '_, Self>) -> Step<Self> {
        (self.step)(state, t)
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, DocTok>>) -> Node {
        scripted_node(tokens)
    }
}

fn scripted_doc(marker: u8, step: DocFn) -> ParseOptions {
    ParseOptions {
        flow_constructs: vec![construct(ScriptedDoc { marker, step })],
        ..ParseOptions::default()
    }
}

/// A text construct with a token of document content.
struct TextWithDocument;

impl Construct for TextWithDocument {
    type Place = Text;
    type State = u16;
    type Token = DocTok;

    fn markers(&self) -> &[u8] {
        b"{"
    }

    fn start(&self) -> u16 {
        0
    }

    fn step(&self, state: u16, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match state {
            0 => t.enter(DocTok::A).consume().enter(DocTok::B).next(1),
            _ => t.consume().exit().exit().ok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, DocTok>>) -> Node {
        scripted_node(tokens)
    }
}

#[test]
fn document_content_in_text_leaves_text() {
    let parse = ParseOptions {
        text_constructs: vec![construct(TextWithDocument)],
        ..ParseOptions::default()
    };

    if let Some(tree) = breaking("document-content-in-text", || {
        to_mdast("x {yz", &parse).unwrap()
    }) {
        assert_eq!(
            tree.to_string(),
            "x {yz",
            "should not match a text construct that enters document content"
        );
    }
}

/// `:::`, a body parsed as a document, and `:::`, which is tried as an
/// attempt at each line.
fn container(state: u16, t: Ctx<'_, '_, ScriptedDoc>) -> Step<ScriptedDoc> {
    match (state, t.current()) {
        (0, Some(b':')) => t
            .enter(DocTok::Container)
            .enter(DocTok::ContainerFence)
            .consume()
            .next(1),
        (1 | 2, Some(b':')) => t.consume().next(state + 1),
        (3, Some(b':')) => t.consume().next(3),
        (3, None | Some(b'\n')) => t.exit().retry(4),
        (4, Some(b'\n')) => t.consume().next(5),
        (4 | 5, None) => t.exit().ok(),
        (5, Some(_)) => t.enter(DocTok::ContainerContent).retry(10),
        (10, Some(_)) => t.attempt(20, 30, 11),
        (10 | 11, None) => t.exit().exit().ok(),
        (11, Some(b'\n')) => t.consume().next(10),
        (11, Some(_)) => t.consume().next(11),
        (20, Some(b':')) => t.enter(DocTok::ContainerFence).consume().next(21),
        (21 | 22, Some(b':')) => t.consume().next(state + 1),
        (23, Some(b':')) => t.consume().next(23),
        (23, None | Some(b'\n')) => t.exit().ok(),
        (30, _) => t.exit().exit().ok(),
        _ => t.nok(),
    }
}

fn container_body(input: &str) -> Vec<Node> {
    let tree = to_mdast(input, &scripted_doc(b':', container)).unwrap();
    find_scripted(&tree)
        .expect("expected a match")
        .children
        .clone()
}

#[test]
fn parses_a_body_as_a_document() {
    assert!(
        matches!(
            &container_body(":::\n> *a*\n:::")[..],
            [Node::Blockquote(_)]
        ),
        "should parse containers in the body"
    );

    let body = container_body("> :::\n> - a\n>\n>   b\n> :::");
    assert!(
        matches!(&body[..], [Node::List(list)] if matches!(&list.children[..], [Node::ListItem(item)] if item.spread && item.children.len() == 2)),
        "should parse a spread list item in a body in a block quote, got {:?}",
        body
    );
    assert_eq!(
        body[0].children().unwrap()[0].children().unwrap()[0].position(),
        Some(&Position::new(2, 5, 10, 2, 6, 11)),
        "should start the first line of a body after its prefixes"
    );

    let tree = to_mdast(":::\na\n:::\nb", &scripted_doc(b':', container)).unwrap();
    assert_eq!(
        find_scripted(&tree).and_then(|node| node.fields.get("tokens").cloned()),
        Some("container,containerFence,containerContent".into()),
        "should not give tokens in content, such as the closing fence, to `to_mdast`"
    );
    assert!(
        matches!(tree.children().unwrap().last(), Some(Node::Paragraph(_))),
        "should end at the closing fence"
    );
}

#[test]
fn resolves_definitions_across_bodies() {
    for input in [":::\n[x]: /u\n:::\n\n[x]", "[x]\n\n:::\n[x]: /u\n:::"] {
        let tree = to_mdast(input, &scripted_doc(b':', container)).unwrap();
        assert!(
            format!("{:?}", tree).contains("LinkReference"),
            "should resolve a reference with a definition in a body: {:?}",
            input
        );
    }
}

#[test]
fn ends_a_flow_construct_at_a_lazy_line_or_the_end() {
    let tree = to_mdast("> :::\n> a\nb", &scripted_doc(b':', container)).unwrap();
    assert!(
        matches!(
            &tree.children().unwrap()[..],
            [Node::Blockquote(_), Node::Paragraph(_)]
        ),
        "should end before a lazy line, got {:?}",
        tree
    );

    assert!(
        matches!(&container_body(":::\na\n\n")[..], [Node::Paragraph(_)]),
        "should run to the end without a closing fence"
    );
}

/// `{` and `}` fences around a document whose lines can start with a two
/// space prefix, a token in content.
fn braced(state: u16, t: Ctx<'_, '_, ScriptedDoc>) -> Step<ScriptedDoc> {
    match (state, t.current()) {
        (0, Some(b'{')) => t
            .enter(DocTok::F)
            .enter(DocTok::Open)
            .consume()
            .exit()
            .next(1),
        (1, Some(b'\n')) => t.consume().next(2),
        (2, _) => t.enter(DocTok::Body).retry(3),
        (3, None) => t.exit().exit().ok(),
        (3, _) => t.attempt(10, 20, 4),
        (4, Some(b' ')) => t.enter(DocTok::Prefix).consume().next(5),
        (4, _) => t.retry(6),
        (5, Some(b' ')) => t.consume().exit().next(6),
        (5, _) => t.exit().retry(6),
        (6, Some(b'\n')) => t.consume().next(3),
        (6, None) => t.retry(3),
        (6, Some(_)) => t.consume().next(6),
        (10, Some(b'}')) => t.enter(DocTok::Close).consume().exit().next(11),
        (11, None | Some(b'\n')) => t.ok(),
        (20, _) => t.exit().exit().ok(),
        _ => t.nok(),
    }
}

#[test]
fn leaves_prefixes_of_an_outer_match_out_of_an_inner_one() {
    let tree = to_mdast("{\n  {\n  a\n  }\n}", &scripted_doc(b'{', braced)).unwrap();
    let outer = find_scripted(&tree).expect("expected a match");
    let inner = outer
        .children
        .iter()
        .find_map(find_scripted)
        .expect("expected a nested match");

    assert_eq!(
        (
            inner.fields.get("tokens").map(String::as_str),
            inner.value.as_deref()
        ),
        (Some("f,open,body"), Some("{\n")),
        "should leave the prefix of the outer match out of the inner one"
    );
}

#[test]
fn allows_blank_lines_in_a_body() {
    assert!(
        matches!(
            &container_body(":::\na\n\nb\n:::")[..],
            [Node::Paragraph(_), Node::Paragraph(_)]
        ),
        "should parse blank lines in a document"
    );
}

#[test]
fn parses_an_indented_body_of_lines() {
    let body = container_body("  :::\n  a\n  b\n:::");

    assert!(
        matches!(&body[..], [Node::Paragraph(_)]),
        "should parse the lines of an indented body, got {:?}",
        body
    );
    assert_eq!(body[0].to_string(), "a\nb");
}

/// `!` and a line ending, then lines with a `|` prefix, whose rest is a
/// document; with `TAKE_EOL`, a prefix takes the line ending of a line that
/// has nothing else. State 2 is a line before the body, 6 a line in it.
fn bars<const TAKE_EOL: bool>(state: u16, t: Ctx<'_, '_, ScriptedDoc>) -> Step<ScriptedDoc> {
    match (state, t.current()) {
        (0, Some(b'!')) => t.enter(DocTok::Bars).consume().next(1),
        (1, Some(b'\n')) => t.consume().next(2),
        (2 | 6, Some(b'|')) => {
            let t = if state == 2 {
                t.enter(DocTok::BarsBody)
            } else {
                t
            };
            t.enter(DocTok::BarsPrefix).consume().next(3)
        }
        (3, Some(b'\n')) if TAKE_EOL => t.consume().next(5),
        (5, _) => t.exit().retry(6),
        (3, _) => t.exit().retry(4),
        (4, Some(b'\n')) => t.consume().next(6),
        (4, Some(_)) => t.consume().next(4),
        (4 | 6, None) => t.exit().exit().ok(),
        _ => t.nok(),
    }
}

#[test]
fn keeps_line_endings_of_a_body_in_the_body() {
    let tree = to_mdast("!\n|a\n|\n|b", &scripted_doc(b'!', bars::<false>)).unwrap();
    assert!(
        matches!(
            &find_scripted(&tree).expect("expected a match").children[..],
            [Node::Paragraph(_), Node::Paragraph(_)]
        ),
        "should parse a line with only a prefix as a blank line"
    );

    let parse = scripted_doc(b'!', bars::<true>);
    if let Some(result) = breaking("line-ending-in-document", || {
        to_mdast("!\n|a\n|\n|b", &parse)
    }) {
        assert!(
            result.is_ok(),
            "should end a construct whose token in a body takes a line ending, without an error"
        );
    }
}

#[test]
fn writes_bodies_as_html() {
    assert_eq!(
        html("{\n[a]:\n b", scripted_doc(b'{', braced)),
        "{\n",
        "should skip prefixes in definitions in a body"
    );
    assert!(
        html("- :::\n  a\n  :::", scripted_doc(b':', container)).contains("<p>a</p>"),
        "should not make a body in a tight list tight"
    );
}

#[test]
fn ends_a_construct_at_the_end_of_a_body() {
    let parse = ParseOptions {
        flow_constructs: vec![
            construct(ScriptedDoc {
                marker: b':',
                step: container,
            }),
            construct(ScriptedFlow {
                marker: b'%',
                step: fenced,
            }),
        ],
        ..ParseOptions::default()
    };
    let tree = to_mdast(":::\n%%%\nx\n:::", &parse).unwrap();
    let body = &find_scripted(&tree).expect("expected a match").children;

    assert_eq!(
        body.iter()
            .find_map(find_scripted)
            .and_then(|node| node.value.as_deref()),
        Some("%%%\nx"),
        "should end before the last line ending of a body, as at the end"
    );
}

#[test]
fn parses_more_in_bodies() {
    assert!(
        matches!(&container_body(":::\r\na\r\n:::")[..], [Node::Paragraph(paragraph)] if paragraph.children[0].to_string() == "a"),
        "should support CR+LF"
    );

    let parse = ParseOptions {
        constructs: Constructs::gfm(),
        ..scripted_doc(b':', container)
    };
    let tree = to_mdast(":::\n[^a]: b\n:::\n\nc[^a]", &parse).unwrap();
    assert!(
        format!("{:?}", tree).contains("FootnoteReference"),
        "should resolve a footnote with a definition in a body"
    );

    let late: FlowFn = |state, t| match (state, t.current()) {
        (0, Some(b'%')) => t.enter(Tok::A).consume().next(1),
        (1, Some(b'\n')) => t.consume().next(2),
        (1, Some(_)) => t.consume().next(1),
        _ => t.nok(),
    };
    let parse = ParseOptions {
        flow_constructs: vec![
            construct(ScriptedDoc {
                marker: b':',
                step: container,
            }),
            construct(ScriptedFlow {
                marker: b'%',
                step: late,
            }),
        ],
        ..ParseOptions::default()
    };
    if let Some(result) = breaking("nok-after-first-line", || {
        to_mdast(":::\n%a\nb\n:::", &parse)
    }) {
        assert!(
            result.is_ok(),
            "should end a construct that fails late in a body, without an error"
        );
    }
}

#[test]
fn limits_the_depth_of_bodies() {
    let parse = |levels: usize| {
        let mut input = String::new();
        for level in 0..levels {
            input.push_str(&format!("{}{{\n", "  ".repeat(level)));
        }
        input.push_str(&format!("{}a\n", "  ".repeat(levels)));
        for level in (0..levels).rev() {
            input.push_str(&format!("{}}}\n", "  ".repeat(level)));
        }
        to_mdast(&input, &scripted_doc(b'{', braced))
    };
    assert_eq!(depth(&parse(32).unwrap()), 32, "should nest bodies 32 deep");
    assert_eq!(
        depth(&parse(33).expect("should not error for bodies deeper than that")),
        33,
        "should end the deepest construct before a body that is too deep, which is then other markdown"
    );
}

/// Like [`braced`], with a prefix of two columns of spaces or tabs, which can
/// take part of a tab.
fn braced_columns(state: u16, t: Ctx<'_, '_, ScriptedDoc>) -> Step<ScriptedDoc> {
    match (state, t.current()) {
        (4, Some(b'\t' | b' ')) => t.enter(DocTok::Prefix).consume().next(5),
        (5, Some(b'\t' | b' ')) => t.consume().exit().next(6),
        _ => braced(state, t),
    }
}

#[test]
fn takes_part_of_a_tab_in_a_line_prefix() {
    let tree = to_mdast("{\n\ta\n\t\tb\n}", &scripted_doc(b'{', braced_columns)).unwrap();
    let node = find_scripted(&tree).expect("expected a match");
    assert!(
        matches!(&node.children[..], [Node::Paragraph(_)]),
        "should take two columns of a tab as a prefix, got {:?}",
        node.children
    );
    // As in `>\ta`, the rest of a tab stays a tab in the text.
    assert_eq!(
        node.children[0].to_string(),
        "\ta\n\tb",
        "should leave the rest of the tab to the content"
    );
}

/// Tokens of scripted containers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContTok {
    LineBlock,
    LineBlockPrefix,
    LineBlockContent,
    LineBlockInner,
    LineBlockSecond,
    Quote,
    QuoteContent,
    QuotePrefix,
    Spaced,
    SpacedContent,
    Look,
}

impl TokenType for ContTok {
    const ALL: &'static [Self] = &[
        Self::LineBlock,
        Self::LineBlockPrefix,
        Self::LineBlockContent,
        Self::LineBlockInner,
        Self::LineBlockSecond,
        Self::Quote,
        Self::QuoteContent,
        Self::QuotePrefix,
        Self::Spaced,
        Self::SpacedContent,
        Self::Look,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::LineBlock => "lineBlock",
            Self::LineBlockPrefix => "lineBlockPrefix",
            Self::LineBlockContent => "lineBlockContent",
            Self::LineBlockInner => "lineBlockInner",
            Self::LineBlockSecond => "lineBlockSecond",
            Self::Quote => "quote",
            Self::QuoteContent => "quoteContent",
            Self::QuotePrefix => "quotePrefix",
            Self::Spaced => "spaced",
            Self::SpacedContent => "spacedContent",
            Self::Look => "look",
        }
    }

    fn content(self) -> Option<ContentType> {
        match self {
            Self::LineBlockContent | Self::QuoteContent | Self::SpacedContent => {
                Some(ContentType::Document)
            }
            _ => None,
        }
    }
}

/// The state a scripted container gives for its next line, which it turns
/// into its continuation, if any.
const NEXT_LINE: u16 = 1000;

/// A state of a scripted container: a number, and two counters that carry
/// across lines.
type ContState = (u16, usize, usize);

/// A step of a scripted container.
type ContFn = fn(ContState, Ctx<'_, '_, Contained>) -> Step<Contained>;

/// A container, scripted by a function, with `tokens[0].value` as its value
/// and the names of its tokens as `tokens`.
struct Contained {
    markers: &'static [u8],
    step: ContFn,
    /// The state that checks a later line, which takes the counters given
    /// at `NEXT_LINE`; `None` ends the container there.
    continuation: Option<u16>,
    allow: fn(Option<char>) -> bool,
}

impl Contained {
    fn new(step: ContFn, continuation: Option<u16>) -> Self {
        Contained {
            markers: b"|",
            step,
            continuation,
            allow: |_| true,
        }
    }
}

impl Construct for Contained {
    type Place = Container;
    type State = ContState;
    type Token = ContTok;

    fn markers(&self) -> &[u8] {
        self.markers
    }

    fn start(&self) -> ContState {
        (0, 0, 0)
    }

    fn previous(&self, previous: Option<char>) -> bool {
        (self.allow)(previous)
    }

    fn step(&self, state: ContState, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state.0, self.continuation) {
            (NEXT_LINE, Some(continuation)) => (self.step)((continuation, state.1, state.2), t),
            (NEXT_LINE, None) => t.nok(),
            _ => (self.step)(state, t),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, ContTok>>) -> Node {
        let mut node = scripted_node(tokens);
        if let Node::Custom(custom) = &mut node {
            custom.name = "container".into();
        }
        node
    }
}

fn s(state: u16) -> ContState {
    (state, 0, 0)
}

/// `|` and an optional space or column of a tab as the prefix of the lines
/// of a container, checked from state 10 on later lines.
fn line_block(state: ContState, t: Ctx<'_, '_, Contained>) -> Step<Contained> {
    let (number, a, b) = state;
    match (number, t.current()) {
        (0, Some(b'|')) => t
            .enter(ContTok::LineBlock)
            .enter(ContTok::LineBlockPrefix)
            .consume()
            .next(s(1)),
        (1 | 11, Some(b'\t' | b' ')) => t.consume().exit().next((number + 1, a, b)),
        (1 | 11, _) => t.exit().retry((number + 1, a, b)),
        (2, _) => t.enter(ContTok::LineBlockContent).container(s(NEXT_LINE)),
        (10, Some(b'|')) => t.enter(ContTok::LineBlockPrefix).consume().next((11, a, b)),
        (12, _) => t.container((NEXT_LINE, a, 0)),
        _ => t.nok(),
    }
}

fn contained(step: ContFn, continuation: Option<u16>) -> ParseOptions {
    ParseOptions {
        document_constructs: vec![construct(Contained::new(step, continuation))],
        ..ParseOptions::default()
    }
}

fn line_blocks() -> ParseOptions {
    contained(line_block, Some(10))
}

/// Children of the root, with each container as its name and its children.
fn outline(node: &Node) -> String {
    node.children()
        .unwrap()
        .iter()
        .map(|child| match child {
            Node::Custom(_) => format!("container({})", outline(child)),
            Node::Paragraph(_) => "paragraph".into(),
            Node::Heading(_) => "heading".into(),
            Node::List(list) => format!("list{}", list.children.len()),
            Node::Code(_) => "code".into(),
            Node::Blockquote(_) => format!("blockquote({})", outline(child)),
            node => format!("{:?}", node),
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn outline_of(input: &str, parse: &ParseOptions) -> String {
    outline(&to_mdast(input, parse).unwrap())
}

#[test]
fn containers_hold_flow() {
    let parse = line_blocks();

    for (input, expected, message) in [
        ("| a", "container(paragraph)", "should start a container"),
        (
            "| a\n| b",
            "container(paragraph)",
            "should keep one paragraph across prefixes",
        ),
        (
            "| a\n| ===",
            "container(heading)",
            "should find a setext heading across prefixes",
        ),
        (
            "| - a\n| - b",
            "container(list2)",
            "should keep one list across prefixes",
        ),
        (
            "| a\nb",
            "container(paragraph)",
            "should continue a paragraph on a lazy line",
        ),
        ("| | a", "container(container(paragraph))", "should nest"),
        (
            "| a\n\nb",
            "container(paragraph),paragraph",
            "should end at a line without its prefix",
        ),
        (
            "|\ta",
            "container(paragraph)",
            "should take a column of a tab",
        ),
        (
            "|\t\ta",
            "container(code)",
            "should leave the rest of a tab, like `>`",
        ),
        (
            "> | a\n> | b",
            "blockquote(container(paragraph))",
            "should be in block quotes",
        ),
        (
            "| > a\n| > b",
            "container(blockquote(paragraph))",
            "should hold block quotes",
        ),
        ("- | a\n  | b", "list1", "should be in list items"),
        (
            "   | a",
            "container(paragraph)",
            "should start after indentation",
        ),
        ("    | a", "code", "should not start in indented code"),
        (
            "a\n| b",
            "paragraph,container(paragraph)",
            "should interrupt a paragraph",
        ),
    ] {
        assert_eq!(
            outline_of(input, &parse),
            expected,
            "{}: {:?}",
            message,
            input
        );
    }

    let tree = to_mdast("| a\n| b", &parse).unwrap();
    let node = custom(&tree.children().unwrap()[0]);
    assert_eq!(
        (
            node.value.as_deref(),
            node.fields.get("tokens").map(String::as_str)
        ),
        (
            Some("| "),
            Some("lineBlock,lineBlockPrefix,lineBlockContent")
        ),
        "should give the tokens of its first line, not the prefixes of later ones"
    );
    assert_eq!(
        node.position
            .as_ref()
            .map(|position| (position.start.offset, position.end.offset)),
        Some((0, 7)),
        "should span its lines"
    );
}

#[test]
fn separates_lists_across_an_inner_container() {
    assert_eq!(
        outline_of("| - a\n| | x\n| - b", &line_blocks()),
        "container(list1,container(paragraph),list1)",
        "should not skip an inner container when looking past prefixes"
    );
    assert_eq!(
        outline_of("> - a\n> > x\n> - b", &ParseOptions::default()),
        "blockquote(list1,blockquote(paragraph),list1)",
        "should match block quotes"
    );
}

#[test]
fn keeps_list_items_tight_in_containers() {
    assert!(
        html("| - a\n|", line_blocks()).contains("<li>a</li>"),
        "should see past a prefix before the end of a list item"
    );
}

#[test]
fn writes_containers_as_html() {
    assert_eq!(
        html("| *a*\n| b", line_blocks()),
        "| \n<p><em>a</em>\nb</p>",
        "should write the source of its first line and render its content"
    );
}

#[test]
fn leaves_container_prefixes_out_of_values() {
    let parse = ParseOptions {
        text_constructs: vec![construct(Scripted {
            marker: b'{',
            step: braces,
        })],
        ..line_blocks()
    };
    let tree = to_mdast("| {{a\n| b}}", &parse).unwrap();

    assert_eq!(
        find_scripted(&tree).and_then(|node| node.attributes.get("bracesData").cloned()),
        Some("a\nb".into()),
        "should leave out the prefix of a container, like `> `"
    );
}

#[test]
fn ends_a_container_without_a_continuation() {
    let parse = contained(line_block, None);
    assert_eq!(
        outline_of("| a\n| b", &parse),
        "container(paragraph),container(paragraph)",
        "should end at the next line"
    );
    assert_eq!(
        outline_of("| a\nb", &parse),
        "container(paragraph)",
        "should still continue a paragraph on a lazy line"
    );
}

#[test]
fn tries_the_next_container_after_no_match() {
    let parse = ParseOptions {
        document_constructs: vec![
            construct(Contained::new(|_, t| t.nok(), None)),
            construct(Contained::new(line_block, Some(10))),
        ],
        ..ParseOptions::default()
    };
    assert_eq!(outline_of("| a\n| b", &parse), "container(paragraph)");
}

#[test]
fn runs_before_builtin_containers() {
    /// `>` as the prefix of the lines of a container.
    fn quote(state: ContState, t: Ctx<'_, '_, Contained>) -> Step<Contained> {
        match (state.0, t.current()) {
            (0, Some(b'>')) => t.enter(ContTok::Quote).consume().next(s(1)),
            (1, _) => t.enter(ContTok::QuoteContent).container(s(NEXT_LINE)),
            (10, Some(b'>')) => t.enter(ContTok::QuotePrefix).consume().exit().next(s(11)),
            (11, _) => t.container(s(NEXT_LINE)),
            _ => t.nok(),
        }
    }

    let parse = ParseOptions {
        document_constructs: vec![construct(Contained {
            markers: b">",
            ..Contained::new(quote, Some(10))
        })],
        ..ParseOptions::default()
    };
    assert_eq!(outline_of(">a\n>b", &parse), "container(paragraph)");
}

#[test]
fn parses_flow_like_a_block_quote() {
    let parse = line_blocks();

    for quote in [
        "> # a\n> b\n> c",
        "># a\n>b\n> c",
        "> a\n> - b\n>   c\n>\n> d",
        "> a\nb\n> c",
        "> ```\n> a\n\nb",
        ">     a\n>     b",
        ">\t\ta",
        "> - a\n>\n>   b",
        "> 1. a\n>\n> 2. b",
        "> [a]: b\n>\n> [a]",
        "> a\n> ***\n> b",
        "> <div>\n> a\n\nb",
        ">\n> a\n>",
    ] {
        // Markers at the start of lines.
        let line_block = quote
            .split('\n')
            .map(|line| {
                let markers = line.len() - line.trim_start_matches('>').len();
                format!("{}{}", "|".repeat(markers), &line[markers..])
            })
            .collect::<Vec<_>>()
            .join("\n");
        let quoted = to_mdast(quote, &ParseOptions::default()).unwrap();
        let contained = to_mdast(&line_block, &parse).unwrap();
        let first = |tree: &Node| tree.children().unwrap()[0].children().unwrap().to_vec();
        assert_eq!(
            first(&contained),
            first(&quoted),
            "should parse the content of {:?} like a block quote",
            line_block
        );
    }
}

#[test]
fn broken_containers_leave_text() {
    let cases: [(&str, ContFn, &str); 6] = [
        (
            "consume-in-container-content",
            |state, t| match state.0 {
                2 => t
                    .enter(ContTok::LineBlockContent)
                    .consume()
                    .container(s(NEXT_LINE)),
                _ => line_block(state, t),
            },
            "| a",
        ),
        (
            "container-tokens",
            |state, t| match (state.0, t.current()) {
                (1, Some(b' ')) => t
                    .consume()
                    .enter(ContTok::LineBlockContent)
                    .container(s(NEXT_LINE)),
                _ => line_block(state, t),
            },
            "| a",
        ),
        (
            "line-ending-in-container",
            |state, t| match (state.0, t.current()) {
                (1, Some(b'\n')) => t.consume().exit().next(s(2)),
                _ => line_block(state, t),
            },
            "|\na",
        ),
        (
            "container-tokens",
            |state, t| match state.0 {
                2 => t.container(s(NEXT_LINE)),
                _ => line_block(state, t),
            },
            "| a",
        ),
        (
            "container-tokens",
            |state, t| match state.0 {
                2 => t
                    .enter(ContTok::LineBlockInner)
                    .consume()
                    .container(s(NEXT_LINE)),
                _ => line_block(state, t),
            },
            "| a",
        ),
        (
            "container-tokens",
            |state, t| match state.0 {
                2 => t
                    .enter(ContTok::LineBlockContent)
                    .enter(ContTok::LineBlockInner)
                    .container(s(NEXT_LINE)),
                _ => line_block(state, t),
            },
            "| a",
        ),
    ];

    for (rule, step, input) in cases {
        let parse = contained(step, Some(10));
        if let Some(outline) = breaking(rule, || outline_of(input, &parse)) {
            assert_eq!(
                outline,
                outline_of(input, &ParseOptions::default()),
                "should not match a container that breaks `{}`",
                rule
            );
        }
    }
}

#[test]
fn ok_is_not_how_a_container_ends() {
    let parse = contained(
        |state, t| match state.0 {
            2 => t.enter(ContTok::LineBlockContent).ok(),
            _ => line_block(state, t),
        },
        Some(10),
    );
    if let Some(outline) = breaking("ok-in-container", || outline_of("| a", &parse)) {
        assert_eq!(
            outline, "paragraph",
            "should not match a container that ends in `ok`"
        );
    }

    let parse = contained(
        |state, t| match state.0 {
            2 => t
                .enter(ContTok::LineBlockContent)
                .attempt(s(20), s(3), s(3)),
            20 => t.container(s(NEXT_LINE)),
            3 => t.container(s(NEXT_LINE)),
            _ => line_block(state, t),
        },
        Some(10),
    );
    if let Some(outline) = breaking("container-in-attempt", || outline_of("| a", &parse)) {
        assert_eq!(
            outline, "container(paragraph)",
            "should fail an attempt that ends in `container`, and continue at `nok`"
        );
    }
}

/// A container that enters text content.
struct TextInContainer;

impl Construct for TextInContainer {
    type Place = Container;
    type State = u16;
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        b"|"
    }

    fn start(&self) -> u16 {
        0
    }

    fn step(&self, _: u16, t: Ctx<'_, '_, Self>) -> Step<Self> {
        t.enter(Tok::A).consume().enter(Tok::BText).container(1)
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, Tok>>) -> Node {
        scripted_node(tokens)
    }
}

#[test]
fn text_content_left_open_breaks_a_container() {
    let parse = ParseOptions {
        document_constructs: vec![construct(TextInContainer)],
        ..ParseOptions::default()
    };

    if let Some(outline) = breaking("container-tokens", || outline_of("| a", &parse)) {
        assert_eq!(
            outline, "paragraph",
            "should not match a container that ends its first line in text content"
        );
    }
}

/// Tokens of a container with a title.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TitleTok {
    Block,
    Prefix,
    Title,
    Body,
}

impl TokenType for TitleTok {
    const ALL: &'static [Self] = &[Self::Block, Self::Prefix, Self::Title, Self::Body];

    fn name(self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::Prefix => "blockPrefix",
            Self::Title => "blockTitle",
            Self::Body => "blockBody",
        }
    }

    fn content(self) -> Option<ContentType> {
        match self {
            Self::Title => Some(ContentType::Text),
            Self::Body => Some(ContentType::Document),
            _ => None,
        }
    }
}

/// `|`, the rest of its first line as a title, and a body of lines that
/// start with `|`; with `title_later`, a later line enters text content.
struct Titled {
    title_later: bool,
}

impl Construct for Titled {
    type Place = Container;
    type State = u8;
    type Token = TitleTok;

    fn markers(&self) -> &[u8] {
        b"|"
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'|')) => t
                .enter(TitleTok::Block)
                .enter(TitleTok::Prefix)
                .consume()
                .exit()
                .next(1),
            (1, Some(byte)) if byte != b'\n' => t.enter(TitleTok::Title).consume().next(2),
            (2, Some(byte)) if byte != b'\n' => t.consume().next(2),
            (2, _) => t.exit().retry(3),
            (1 | 3, _) => t.enter(TitleTok::Body).container(10),
            (10, Some(b'|')) if self.title_later => t.enter(TitleTok::Title).consume().next(11),
            (10, Some(b'|')) => t.enter(TitleTok::Prefix).consume().exit().next(12),
            (11, _) => t.exit().container(10),
            (12, _) => t.container(10),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, TitleTok>>) -> Node {
        let mut node = scripted_node(tokens);
        if let Node::Custom(custom) = &mut node {
            custom.name = "titled".into();
        }
        node
    }
}

#[test]
fn takes_text_content_on_the_first_line_of_a_container() {
    let parse = ParseOptions {
        document_constructs: vec![construct(Titled { title_later: false })],
        ..ParseOptions::default()
    };
    let tree = to_mdast("|*a* b\n|c\nd", &parse).unwrap();
    let node = match &tree.children().unwrap()[0] {
        Node::Custom(node) => node,
        node => panic!("expected a container, got {:?}", node),
    };
    assert!(
        matches!(
            &node.children[..],
            [Node::Emphasis(_), Node::Text(_), Node::Paragraph(_)]
        ),
        "should parse the title as text, and the body as a document, got {:?}",
        node.children
    );
    assert_eq!(
        node.children[2].to_string(),
        "c\nd",
        "should keep a lazy line in the body"
    );
    let outline = outline_of("> |a\n> |b", &parse);
    assert!(
        outline.starts_with("blockquote(container(Text { value: \"a\"")
            && outline.ends_with("paragraph))"),
        "should take a title inside a block quote, got {:?}",
        outline
    );

    let later = ParseOptions {
        document_constructs: vec![construct(Titled { title_later: true })],
        ..ParseOptions::default()
    };
    if let Some(outline) = breaking("text-content-in-container", || outline_of("|a\n|b", &later)) {
        // The later line starts a container of its own.
        assert_eq!(
            outline.matches("container(").count(),
            2,
            "should end a container at a later line that enters text content, got {:?}",
            outline
        );
    }
}

#[test]
fn broken_continuations_end_containers() {
    let cases: [(&str, ContFn, &str); 4] = [
        (
            "container-tokens",
            |state, t| match state.0 {
                11 => t.container(s(NEXT_LINE)),
                _ => line_block(state, t),
            },
            "| a\n| b",
        ),
        (
            "line-ending-in-container",
            |state, t| match (state.0, t.current()) {
                (10, Some(b'\n')) => t
                    .enter(ContTok::LineBlockPrefix)
                    .consume()
                    .exit()
                    .container(s(NEXT_LINE)),
                _ => line_block(state, t),
            },
            "| a\n\nb",
        ),
        (
            "content-in-content",
            |state, t| match state.0 {
                12 => t.enter(ContTok::LineBlockContent).container(s(NEXT_LINE)),
                _ => line_block(state, t),
            },
            "| a\n| b",
        ),
        (
            "token-after-match",
            |state, t| match state.0 {
                12 => t
                    .enter(ContTok::LineBlockSecond)
                    .consume()
                    .exit()
                    .container(s(NEXT_LINE)),
                _ => line_block(state, t),
            },
            "| a\n| b",
        ),
    ];

    for (rule, step, input) in cases {
        let parse = contained(step, Some(10));
        if let Some(outline) = breaking(rule, || outline_of(input, &parse)) {
            assert_eq!(
                outline,
                outline_of(input, &contained(line_block, None)),
                "should end a container whose continuation breaks `{}`",
                rule
            );
        }
    }

    // The line ending check depends on the container continuing otherwise.
    assert_eq!(
        outline_of(
            "| a\n\nb",
            &contained(
                |state, t| match state.0 {
                    20 => t.container(s(NEXT_LINE)),
                    _ => line_block(state, t),
                },
                Some(20)
            )
        ),
        "container(paragraph,paragraph)",
        "should continue with a continuation that takes no bytes"
    );
}

#[test]
fn keeps_a_state_for_a_container() {
    // Lines of at most 3, from state 20: the first counter, which the state
    // for each next line carries, counts the lines.
    let parse = contained(
        |state, t| match state {
            (20, lines, _) if lines < 2 => t.retry((10, lines + 1, 0)),
            (20, ..) => t.nok(),
            _ => line_block(state, t),
        },
        Some(20),
    );

    assert_eq!(
        outline_of("| a\n| b\n| c\n| d", &parse),
        "container(paragraph),container(paragraph)",
        "should keep the state a container gives for its next line"
    );
}

#[test]
fn containers_see_their_indent_on_their_first_line() {
    let first = contained(
        |state, t| match state.0 {
            0 if t.indent() != 2 => t.nok(),
            _ => line_block(state, t),
        },
        Some(10),
    );
    assert_eq!(outline_of("  | a", &first), "container(paragraph)");
    assert_eq!(outline_of("| a", &first), "paragraph");

    let later = contained(
        |state, t| match state.0 {
            10 if t.indent() != 0 => t.nok(),
            _ => line_block(state, t),
        },
        Some(10),
    );
    assert_eq!(
        outline_of("  | a\n| b", &later),
        "container(paragraph)",
        "should see `0` on later lines"
    );
}

#[test]
fn spaces_and_tabs_are_never_container_markers() {
    /// A space as the prefix of a container.
    fn spaced(state: ContState, t: Ctx<'_, '_, Contained>) -> Step<Contained> {
        match (state.0, t.current()) {
            (0, Some(b' ' | b'\t')) => t.enter(ContTok::Spaced).consume().next(s(1)),
            (1, _) => t.enter(ContTok::SpacedContent).container(s(NEXT_LINE)),
            _ => t.nok(),
        }
    }

    let parse = ParseOptions {
        document_constructs: vec![construct(Contained {
            markers: b" \t",
            ..Contained::new(spaced, None)
        })],
        ..ParseOptions::default()
    };
    assert_eq!(outline_of("    a", &parse), "code");
    assert_eq!(outline_of("\ta", &parse), "code");
}

/// From state 20, `N` retries, counted in the second counter, then the
/// prefix of a line block.
fn spend_continuation<const N: usize>(
    state: ContState,
    t: Ctx<'_, '_, Contained>,
) -> Step<Contained> {
    match state {
        (20, a, retries) if retries < N => t.retry((20, a, retries + 1)),
        (20, a, _) => t.retry((10, a, 0)),
        _ => line_block(state, t),
    }
}

#[test]
fn limits_steps_of_a_continuation() {
    // Retries at its first byte: with the step that consumes `|`, 254 fit
    // in the 256 steps one byte allows.
    let lines = |step: ContFn| outline_of("| a\n| b", &contained(step, Some(20)));

    assert_eq!(lines(spend_continuation::<254>), "container(paragraph)");
    assert_eq!(
        lines(spend_continuation::<255>),
        "container(paragraph),container(paragraph)",
        "should count steps from the start of the continuation"
    );
}

#[test]
fn containers_see_a_line_ending_before_them() {
    let after = |allow: fn(Option<char>) -> bool| ParseOptions {
        document_constructs: vec![construct(Contained {
            allow,
            ..Contained::new(line_block, Some(10))
        })],
        ..ParseOptions::default()
    };
    let line_ending = after(|previous| previous == Some('\n'));
    let start = after(|previous| previous.is_none());

    for (input, expected) in [
        ("| a", false),
        ("x\n\n| a", true),
        ("x\n\n  | a", true),
        ("> x\n> | a", true),
        ("- x\n- | a", true),
    ] {
        assert_eq!(
            has_custom(&to_mdast(input, &line_ending).unwrap()),
            expected,
            "should see a line ending before a line, after indentation and prefixes, in {:?}",
            input
        );
    }

    for (input, expected) in [
        ("| a", true),
        ("  | a", true),
        ("> | a", true),
        ("x\n\n| a", false),
    ] {
        assert_eq!(
            has_custom(&to_mdast(input, &start).unwrap()),
            expected,
            "should see `None` before the first line in {:?}",
            input
        );
    }
}

#[test]
fn nests_containers_in_the_deepest_bodies() {
    let parse = ParseOptions {
        flow_constructs: vec![construct(ScriptedDoc {
            marker: b'{',
            step: braced,
        })],
        ..line_blocks()
    };
    let mut input = String::new();
    for level in 0..32 {
        input.push_str(&format!("{}{{\n", "  ".repeat(level)));
    }
    input.push_str(&format!("{}| a\n", "  ".repeat(32)));
    for level in (0..32).rev() {
        input.push_str(&format!("{}}}\n", "  ".repeat(level)));
    }
    let tree = to_mdast(&input, &parse).unwrap();
    let mut node = &tree;
    while let Some(child) = node.children().and_then(|children| children.first()) {
        node = child;
        if matches!(node, Node::Custom(custom) if custom.name == "container") {
            break;
        }
    }
    assert!(
        matches!(node, Node::Custom(custom) if custom.name == "container"),
        "should not count containers toward the depth of content"
    );
}

/// Turn containers into block quotes, to compare trees.
fn as_block_quotes(node: &mut Node) {
    if let Node::Custom(custom) = node {
        *node = Node::Blockquote(Blockquote {
            children: std::mem::take(&mut custom.children),
            position: custom.position.take(),
        });
    }
    if let Some(children) = node.children_mut() {
        children.iter_mut().for_each(as_block_quotes);
    }
}

/// Like [`line_block`], with up to 3 columns of indentation before `|` on a
/// later line, as a block quote takes, counted in `state.1`.
fn indented_line_block(state: ContState, t: Ctx<'_, '_, Contained>) -> Step<Contained> {
    let (number, columns, b) = state;
    match (number, t.current()) {
        // A tab is `\t` at its first column and a space after.
        (10, Some(b'\t' | b' ')) => t.enter(ContTok::LineBlockPrefix).consume().next((20, 1, b)),
        (20, Some(b'\t' | b' ')) if columns < 3 => t.consume().next((20, columns + 1, b)),
        (20, Some(b'|')) => t.consume().next((11, 0, b)),
        (20, _) => t.nok(),
        _ => line_block(state, t),
    }
}

#[test]
fn takes_indentation_on_later_lines_like_a_block_quote() {
    let parse = contained(indented_line_block, Some(10));

    for quote in [
        "> a\n   > b",
        "> a\n    > b",
        "> a\n\t> b",
        "> a\n \t> b",
        "> >\n> \t> a",
        "> >\n>  \t> a",
        "> >\n>   \t> a",
        "> >\n> \t > a",
        "> >\n>\t\t> a",
        "> >\n>\t \t> a",
        "> - a\n>  \t> b",
    ] {
        let quoted = to_mdast(quote, &ParseOptions::default()).unwrap();
        let mut contained = to_mdast(&quote.replace('>', "|"), &parse).unwrap();
        as_block_quotes(&mut contained);
        // A marker that does not continue is text: `|` where `>` would be.
        assert_eq!(
            format!("{:?}", contained).replace('|', ">"),
            format!("{:?}", quoted),
            "should take indentation on later lines of {:?} like a block quote",
            quote
        );
    }
}

#[test]
fn nests_flow_like_a_block_quote() {
    let parse = line_blocks();

    for quote in [
        "* >\n* a",
        "- >\n\n- a",
        "1. >\n2. a",
        "- > a\n  > b\n- c",
        "- a\n\n  > b\n- c",
        "> - >\n>   a",
        "> > a\n> b",
        "- > - a\n  >\n  > - b",
        "* > a\n  >\n* b",
        "- >\n  >\n- b",
        ">>- a\n>>\n>>-",
        "* > > a\n  > >\n* b",
    ] {
        let quoted = to_mdast(quote, &ParseOptions::default()).unwrap();
        let mut contained = to_mdast(&quote.replace('>', "|"), &parse).unwrap();
        as_block_quotes(&mut contained);
        assert_eq!(
            contained, quoted,
            "should parse {:?} like a block quote",
            quote
        );
    }
}

#[test]
fn keeps_the_budget_of_a_continuation_after_an_outer_one() {
    /// `&` as the prefix of the lines of a container, whose continuation
    /// first tries a token to the end of the line and fails it.
    fn looking(state: ContState, t: Ctx<'_, '_, Contained>) -> Step<Contained> {
        match (state.0, t.current()) {
            (0 | 10, Some(b'&')) => {
                let t = if state.0 == 0 {
                    t.enter(ContTok::LineBlock)
                } else {
                    t
                };
                t.enter(ContTok::LineBlockPrefix)
                    .consume()
                    .next(s(state.0 + 1))
            }
            (20, _) => t.attempt(s(30), s(10), s(10)),
            (30, _) => t.enter(ContTok::Look).retry(s(31)),
            (31, Some(byte)) if byte != b'\n' => t.consume().next(s(31)),
            (0 | 10 | 31, _) => t.nok(),
            _ => line_block(state, t),
        }
    }

    let nested = |step: ContFn| ParseOptions {
        document_constructs: vec![
            construct(Contained {
                markers: b"&",
                ..Contained::new(looking, Some(20))
            }),
            construct(Contained::new(step, Some(20))),
        ],
        ..ParseOptions::default()
    };
    let input = "& | a\n& | b";

    assert_eq!(
        outline_of(input, &nested(spend_continuation::<254>)),
        "container(container(paragraph))"
    );
    assert_eq!(
        outline_of(input, &nested(spend_continuation::<255>)),
        "container(container(paragraph),container(paragraph))",
        "should not give a continuation the bytes an earlier one reached"
    );
}

/// `=` delimiter runs of the given sizes, such as `==a==`, as `mark` nodes
/// with the names of their tokens as `tokens`.
struct Mark {
    markers: &'static [u8],
    sizes: &'static [usize],
    previous: fn(Option<char>) -> bool,
}

impl Construct for Mark {
    type Place = Text;
    type State = ();
    type Token = RunToken;

    fn markers(&self) -> &[u8] {
        self.markers
    }

    fn start(&self) {}

    fn reach(&self) -> Reach {
        Reach::Attention(self.sizes)
    }

    fn previous(&self, previous: Option<char>) -> bool {
        (self.previous)(previous)
    }

    fn step(&self, (): (), _: Ctx<'_, '_, Self>) -> Step<Self> {
        unreachable!("expected a delimiter run not to step")
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, RunToken>>) -> Node {
        let value = tokens[1].value.clone().into_owned();
        let mut node = scripted_node(tokens);
        if let Node::Custom(custom) = &mut node {
            custom.name = "mark".into();
            custom.value = Some(value);
        }
        node
    }
}

/// A run construct with `markers`, `sizes`, and a `previous` check.
fn run(
    markers: &'static [u8],
    sizes: &'static [usize],
    previous: fn(Option<char>) -> bool,
) -> BoxedConstruct<Text> {
    construct(Mark {
        markers,
        sizes,
        previous,
    })
}

fn marks(sizes: &'static [usize]) -> ParseOptions {
    ParseOptions {
        text_constructs: vec![run(b"=", sizes, |_| true)],
        ..ParseOptions::default()
    }
}

/// Minimal HTML of paragraphs, emphasis, marks, and text.
fn render_marks(node: &Node) -> String {
    let inner = |children: &[Node]| children.iter().map(render_marks).collect::<String>();
    match node {
        Node::Root(root) => inner(&root.children),
        Node::Blockquote(quote) => inner(&quote.children),
        Node::Paragraph(paragraph) => format!("<p>{}</p>", inner(&paragraph.children)),
        Node::Emphasis(emphasis) => format!("<em>{}</em>", inner(&emphasis.children)),
        Node::Delete(delete) => format!("<del>{}</del>", inner(&delete.children)),
        Node::Custom(custom) => format!("<mark>{}</mark>", inner(&custom.children)),
        node => node.to_string(),
    }
}

fn marked(input: &str, parse: &ParseOptions) -> String {
    render_marks(&to_mdast(input, parse).unwrap())
}

#[test]
fn pairs_delimiter_runs() {
    let parse = marks(&[2]);

    for (input, expected, message) in [
        ("==a==", "<p><mark>a</mark></p>", "should pair runs"),
        (
            "==a *b*==",
            "<p><mark>a <em>b</em></mark></p>",
            "should parse text in them",
        ),
        (
            "*==a==*",
            "<p><em><mark>a</mark></em></p>",
            "should allow emphasis around them",
        ),
        (
            "==*a*==",
            "<p><mark><em>a</em></mark></p>",
            "should allow emphasis in them",
        ),
        (
            "==a *b== c*",
            "<p><mark>a *b</mark> c*</p>",
            "should not misnest",
        ),
        (
            "==a ==b== c==",
            "<p><mark>a <mark>b</mark> c</mark></p>",
            "should nest",
        ),
        (
            "===a===",
            "<p>===a===</p>",
            "should not pair sizes it does not allow",
        ),
        (
            "=a=",
            "<p>=a=</p>",
            "should not pair a size it does not allow",
        ),
        (
            "==a===",
            "<p>==a===</p>",
            "should not pair runs of other sizes",
        ),
        (
            "===a==",
            "<p>===a==</p>",
            "should not pair a shorter closing run",
        ),
        (
            "a*==b==*c",
            "<p>a<em><mark>b</mark></em>c</p>",
            "should count as markers around emphasis",
        ),
        (
            "a==*b*==c",
            "<p>a==<em>b</em>==c</p>",
            "should not see markers around itself, like `~`",
        ),
        (
            "== a ==",
            "<p>== a ==</p>",
            "should not open before or close after whitespace",
        ),
        (
            "a==b==c",
            "<p>a<mark>b</mark>c</p>",
            "should pair inside words, like `~`",
        ),
        (
            "> ==a\n> b==",
            "<p><mark>a\nb</mark></p>",
            "should pair across lines",
        ),
    ] {
        assert_eq!(marked(input, &parse), expected, "{}: {:?}", message, input);
    }

    assert_eq!(
        marked("=a= ==b== ===c===", &marks(&[1, 3])),
        "<p><mark>a</mark> ==b== <mark>c</mark></p>",
        "should pair each size it allows"
    );
}

#[test]
fn gives_delimiter_runs_their_tokens() {
    let tree = to_mdast("x ==*a*== y", &marks(&[2])).unwrap();
    let node = custom(&phrasing(tree)[1]).clone();

    assert_eq!(
        (
            node.value.as_deref(),
            node.fields.get("tokens").map(String::as_str)
        ),
        (
            Some("=="),
            Some("attention,attentionSequence,attentionText,attentionSequence")
        ),
        "should give a run, its sequences, and its text"
    );
    assert_eq!(
        node.position
            .as_ref()
            .map(|position| (position.start.offset, position.end.offset)),
        Some((2, 9)),
        "should span the run"
    );
    assert_eq!(
        html("x ==*a*== y", marks(&[2])),
        "<p>x ==<em>a</em>== y</p>",
        "should write the sequences as text in HTML"
    );
}

#[test]
fn checks_the_character_before_a_delimiter_run() {
    let parse = ParseOptions {
        text_constructs: vec![run(b"=", &[2], |previous| previous != Some('a'))],
        ..ParseOptions::default()
    };

    assert_eq!(marked("b==c== d", &parse), "<p>b<mark>c</mark> d</p>");
    assert_eq!(marked("a==c== d", &parse), "<p>a==c== d</p>");
}

#[test]
fn delimiter_runs_come_before_builtin_markers() {
    let parse = ParseOptions {
        constructs: Constructs::gfm(),
        text_constructs: vec![run(b"~", &[3], |_| true)],
        ..ParseOptions::gfm()
    };

    assert_eq!(
        marked("x ~~~a~~~ ~~b~~", &parse),
        "<p>x <mark>a</mark> ~~b~~</p>",
        "should pair by the sizes of the construct, not strikethrough"
    );
    assert_eq!(
        marked("x ~~~a~~~ ~~b~~", &ParseOptions::gfm()),
        "<p>x ~~~a~~~ <del>b</del></p>",
        "should otherwise be strikethrough"
    );
}

#[test]
fn leaves_attention_to_constructs_that_are_not_runs() {
    let parse = ParseOptions {
        text_constructs: vec![
            construct(Scripted {
                marker: b'*',
                step: |_, t| t.nok(),
            }),
            run(b"=", &[2], |_| true),
        ],
        ..ParseOptions::default()
    };

    assert_eq!(
        marked("*a* ==b==", &parse),
        "<p><em>a</em> <mark>b</mark></p>"
    );
}

#[test]
fn delimiter_runs_are_ascii() {
    for markers in [b"\xC2", b"\xA7"] {
        let parse = ParseOptions {
            text_constructs: vec![run(markers, &[1], |_| true)],
            ..ParseOptions::default()
        };
        assert_eq!(
            marked("\u{a7}a\u{a7}", &parse),
            "<p>\u{a7}a\u{a7}</p>",
            "should not start a run inside a character"
        );
    }
}

#[test]
fn pairs_runs_of_the_same_construct() {
    let gfm = ParseOptions {
        text_constructs: vec![run(b"~", &[2], |previous| previous != Some('a'))],
        ..ParseOptions::gfm()
    };
    assert_eq!(
        marked("a~~ba~~ c", &gfm),
        "<p>a<del>ba</del> c</p>",
        "should leave what the construct does not take to the built-ins"
    );
    assert_eq!(
        marked("a~~b~~ c", &gfm),
        "<p>a~~b~~ c</p>",
        "should not pair a run with a built-in sequence"
    );
    assert_eq!(marked("b~~c~~ d", &gfm), "<p>b<mark>c</mark> d</p>");

    let shared = ParseOptions {
        text_constructs: vec![
            run(b"=", &[2], |previous| previous != Some('a')),
            run(b"=", &[1], |_| true),
        ],
        ..ParseOptions::default()
    };
    assert_eq!(
        marked("a=ba= c ==d==", &shared),
        "<p>a<mark>ba</mark> c <mark>d</mark></p>",
        "should pair runs of each construct at a marker"
    );
}

#[test]
fn delimiter_runs_flank_like_strikethrough() {
    let parse = |markers: &'static [u8]| ParseOptions {
        text_constructs: vec![run(markers, &[1], |_| true)],
        ..ParseOptions::default()
    };
    assert_eq!(
        marked("a*_b_*c", &parse(b"*")),
        "<p>a*<em>b</em>*c</p>",
        "should not see markers around a run at `*`"
    );
    assert_eq!(
        marked("a_b_c", &parse(b"_")),
        "<p>a<mark>b</mark>c</p>",
        "should pair inside words at `_`"
    );
    assert_eq!(
        marked("a *\nb*", &parse(b"=\n")),
        "<p>a *\nb*</p>",
        "should not take a line ending for a marker"
    );
}

/// A delimiter run with its own tokens, which a run cannot have.
struct RunWithOwnTokens;

impl Construct for RunWithOwnTokens {
    type Place = Text;
    type State = ();
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        b"="
    }

    fn start(&self) {}

    fn reach(&self) -> Reach {
        Reach::Attention(&[2])
    }

    fn step(&self, (): (), t: Ctx<'_, '_, Self>) -> Step<Self> {
        t.nok()
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, Tok>>) -> Node {
        scripted_node(tokens)
    }
}

/// A delimiter run in flow, which a run cannot be.
struct RunInFlow;

impl Construct for RunInFlow {
    type Place = Flow;
    type State = ();
    type Token = RunToken;

    fn markers(&self) -> &[u8] {
        b"="
    }

    fn start(&self) {}

    fn reach(&self) -> Reach {
        Reach::Attention(&[2])
    }

    fn step(&self, (): (), t: Ctx<'_, '_, Self>) -> Step<Self> {
        t.nok()
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, RunToken>>) -> Node {
        scripted_node(tokens)
    }
}

#[test]
fn errors_for_delimiter_runs_without_run_tokens_or_outside_text() {
    let message = "Unexpected delimiter run with other tokens than `RunToken` or outside text, expected `RunToken` in text (markdown-rs:attention-construct)";

    let parse = ParseOptions {
        text_constructs: vec![construct(RunWithOwnTokens)],
        ..ParseOptions::default()
    };
    assert_eq!(
        to_mdast("==a==", &parse).unwrap_err().to_string(),
        message,
        "should error for a run whose tokens are not `RunToken`"
    );

    let parse = ParseOptions {
        flow_constructs: vec![construct(RunInFlow)],
        ..ParseOptions::default()
    };
    assert_eq!(
        to_mdast("==a==", &parse).unwrap_err().to_string(),
        message,
        "should error for a run outside text"
    );
}

/// Tokens of a wiki link.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WikiToken {
    Wiki,
    Target,
}

impl TokenType for WikiToken {
    const ALL: &'static [Self] = &[Self::Wiki, Self::Target];

    fn name(self) -> &'static str {
        match self {
            Self::Wiki => "wiki",
            Self::Target => "wikiTarget",
        }
    }
}

/// `[[`, a target without `|`, and `]]`; with `nests`, a `[[` in the target
/// needs its own `]]`, and `state.1` counts how deep it is.
struct Wiki {
    reach: Reach,
    nests: bool,
}

impl Construct for Wiki {
    type Place = Text;
    type State = (u8, usize);
    type Token = WikiToken;

    fn markers(&self) -> &[u8] {
        b"["
    }

    fn start(&self) -> (u8, usize) {
        (0, 0)
    }

    fn reach(&self) -> Reach {
        self.reach
    }

    fn step(&self, (state, depth): (u8, usize), t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'[')) => t.enter(WikiToken::Wiki).consume().next((1, 0)),
            (1, Some(b'[')) => t.consume().next((2, 0)),
            (2, None | Some(b']' | b'|')) => t.nok(),
            (2, Some(_)) => t.enter(WikiToken::Target).retry((3, 0)),
            // In the target.
            (3, Some(b']')) if depth == 0 => t.exit().consume().next((4, 0)),
            (3, Some(b']')) => t.consume().next((5, depth)),
            (3, Some(b'[')) if self.nests => t.consume().next((6, depth)),
            (3, Some(byte)) if byte != b'|' => t.consume().next((3, depth)),
            // After the first `]` of the closer.
            (4, Some(b']')) => t.consume().exit().ok(),
            // After a `]` in a nested pair, or a `[`.
            (5, Some(b']')) => t.consume().next((3, depth - 1)),
            (6, Some(b'[')) => t.consume().next((3, depth + 1)),
            (5 | 6, _) => t.retry((3, depth)),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, WikiToken>>) -> Node {
        Node::Custom(Custom {
            name: "wiki".into(),
            value: Some(tokens[1].value.to_string()),
            ..Custom::default()
        })
    }
}

fn wikis(reach: Reach, nests: bool) -> ParseOptions {
    ParseOptions {
        text_constructs: vec![construct(Wiki { reach, nests })],
        ..ParseOptions::default()
    }
}

/// Targets of the wiki links in `node`.
fn targets(node: &Node) -> Vec<String> {
    match node {
        Node::Custom(custom) if custom.name == "wiki" => vec![custom.value.clone().unwrap()],
        _ => node
            .children()
            .map(|children| children.iter().flat_map(targets).collect())
            .unwrap_or_default(),
    }
}

#[test]
fn pairs_skip_markers_a_failed_match_read() {
    for (input, expected) in [
        ("[[a]] [[b]]", vec!["a", "b"]),
        ("[[a [[b]]", vec!["a [[b"]),
        ("[[a | [[b]]", vec!["b"]),
        ("[[a [[b | ]]", vec![]),
        ("> [[a\n> [[b]]", vec!["a\n[[b"]),
        ("[[a\n\n[[b]]", vec!["b"]),
        ("[[a | *[[b]]*", vec!["b"]),
    ] {
        for reach in [Reach::Unbounded, Reach::Pair] {
            assert_eq!(
                targets(&to_mdast(input, &wikis(reach, false)).unwrap()),
                expected,
                "should find the same links in {:?} with {:?}",
                input,
                reach
            );
        }
    }
}

#[test]
fn pairs_match_like_unbounded_constructs_that_stop_at_the_first_closer() {
    // A construct that stops at its first closer keeps the promise of
    // `Reach::Pair`, so skipping changes nothing.
    let pieces = [
        "[", "[[", "]", "]]", "a", " ", "|", "\n", "\n\n", "> ", "*", "`", "\\",
    ];
    let mut seed: u64 = 7;
    let mut random = move |below: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        usize::try_from(seed % u64::try_from(below).unwrap()).unwrap()
    };

    for _ in 0..2000 {
        let input = (0..random(24))
            .map(|_| pieces[random(pieces.len())])
            .collect::<String>();
        assert_eq!(
            to_mdast(&input, &wikis(Reach::Pair, false)).unwrap(),
            to_mdast(&input, &wikis(Reach::Unbounded, false)).unwrap(),
            "should parse {:?} the same with `Reach::Pair`",
            input
        );
    }
}

#[test]
fn pairs_lose_matches_of_constructs_that_nest() {
    assert_eq!(
        targets(&to_mdast("[[a [[b]] c]]", &wikis(Reach::Unbounded, true)).unwrap()),
        vec!["a [[b]] c"],
        "should nest"
    );
    assert_eq!(
        targets(&to_mdast("[[a [[b]]", &wikis(Reach::Unbounded, true)).unwrap()),
        vec!["b"],
        "should find an inner link after the outer one fails"
    );
    assert_eq!(
        targets(&to_mdast("[[a [[b]]", &wikis(Reach::Pair, true)).unwrap()),
        Vec::<String>::new(),
        "should lose an inner link with `Reach::Pair`, which a construct that nests breaks"
    );
}

/// `[[`, a target without `[`, `]`, or `|`, and `]]`.
struct PlainWiki(Reach);

impl Construct for PlainWiki {
    type Place = Text;
    type State = u8;
    type Token = WikiToken;

    fn markers(&self) -> &[u8] {
        b"["
    }

    fn start(&self) -> u8 {
        0
    }

    fn reach(&self) -> Reach {
        self.0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'[')) => t.enter(WikiToken::Wiki).consume().next(1),
            (1, Some(b'[')) => t.consume().next(2),
            (2 | 3, Some(b']')) if state == 3 => t.exit().consume().next(4),
            (2 | 3, Some(byte)) if !matches!(byte, b'[' | b']' | b'|') => {
                let t = if state == 2 {
                    t.enter(WikiToken::Target)
                } else {
                    t
                };
                t.consume().next(3)
            }
            (4, Some(b']')) => t.consume().exit().ok(),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, WikiToken>>) -> Node {
        Node::Custom(Custom {
            name: "wiki".into(),
            value: Some(tokens[1].value.to_string()),
            ..Custom::default()
        })
    }
}

#[test]
fn pairs_lose_matches_that_start_inside_an_opener() {
    let parse = |reach| ParseOptions {
        text_constructs: vec![construct(PlainWiki(reach))],
        ..ParseOptions::default()
    };
    assert_eq!(
        targets(&to_mdast("[[[a]]", &parse(Reach::Unbounded)).unwrap()),
        vec!["a"],
        "should find a link that starts inside the opener of a failed one"
    );
    assert_eq!(
        targets(&to_mdast("[[[a]]", &parse(Reach::Pair)).unwrap()),
        Vec::<String>::new(),
        "should lose it with `Reach::Pair`, which such a construct breaks"
    );
}

#[cfg(feature = "test-util")]
#[test]
fn constructs_that_stop_at_each_marker_are_linear() {
    markdown::extension::assert_linear(&ParseOptions {
        text_constructs: vec![construct(PlainWiki(Reach::Unbounded))],
        ..ParseOptions::default()
    });
}

#[test]
fn pairs_are_unbounded_in_flow() {
    let parse = ParseOptions {
        flow_constructs: vec![construct(ScriptedFlow {
            marker: b':',
            step: leaf,
        })],
        ..ParseOptions::default()
    };
    assert_eq!(
        to_mdast("::a[b]\n::c[d]", &parse)
            .unwrap()
            .children()
            .unwrap()
            .len(),
        2,
        "should match each line"
    );
}

#[cfg(feature = "test-util")]
#[test]
fn asserts_linear_steps() {
    use markdown::extension::assert_linear;

    assert_linear(&wikis(Reach::Pair, false));
    assert_linear(&wikis(Reach::Bytes(64), false));
    assert_linear(&wikis(Reach::Bytes(64), true));
    assert_linear(&marks(&[1, 2]));
    assert_linear(&scripted_flow(b':', leaf));
    assert_linear(&line_blocks());
    assert_linear(&ParseOptions {
        constructs: Constructs::gfm(),
        ..wikis(Reach::Pair, false)
    });
    assert_linear(&ParseOptions {
        text_constructs: vec![
            construct(Wiki {
                reach: Reach::Pair,
                nests: false,
            }),
            run(b"=", &[2], |_| true),
        ],
        flow_constructs: vec![construct(ScriptedFlow {
            marker: b':',
            step: leaf,
        })],
        document_constructs: vec![construct(Contained::new(line_block, Some(10)))],
        ..ParseOptions::default()
    });

    for (name, options) in [
        ("unbounded", wikis(Reach::Unbounded, false)),
        ("nesting", wikis(Reach::Unbounded, true)),
        (
            "with others",
            ParseOptions {
                text_constructs: vec![
                    run(b"=", &[2], |_| true),
                    construct(Wiki {
                        reach: Reach::Unbounded,
                        nests: false,
                    }),
                ],
                ..ParseOptions::default()
            },
        ),
    ] {
        let message = catch_unwind(AssertUnwindSafe(|| assert_linear(&options)))
            .expect_err("expected a panic")
            .downcast::<String>()
            .map(|message| *message)
            .unwrap_or_default();
        assert!(
            message.starts_with("expected steps of constructs linear in the size of the document"),
            "should find a quadratic construct ({}), got {:?}",
            name,
            message
        );
    }
}

/// Tokens of the constructs that write their own HTML.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HtmlTok {
    Span,
    SpanMarker,
    SpanName,
    Link,
    LinkTarget,
    LinkAlias,
    Line,
    LineMarker,
    LineText,
    Abbr,
    AbbrLabel,
}

impl TokenType for HtmlTok {
    const ALL: &'static [Self] = &[
        Self::Span,
        Self::SpanMarker,
        Self::SpanName,
        Self::Link,
        Self::LinkTarget,
        Self::LinkAlias,
        Self::Line,
        Self::LineMarker,
        Self::LineText,
        Self::Abbr,
        Self::AbbrLabel,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Span => "span",
            Self::SpanMarker => "spanMarker",
            Self::SpanName => "spanName",
            Self::Link => "link",
            Self::LinkTarget => "linkTarget",
            Self::LinkAlias => "linkAlias",
            Self::Line => "line",
            Self::LineMarker => "lineMarker",
            Self::LineText => "lineText",
            Self::Abbr => "abbr",
            Self::AbbrLabel => "abbrLabel",
        }
    }

    fn content(self) -> Option<ContentType> {
        match self {
            Self::LineText | Self::AbbrLabel | Self::LinkAlias => Some(ContentType::Text),
            _ => None,
        }
    }
}

fn empty_node(_: Vec<Token<'_, HtmlTok>>) -> Node {
    Node::Custom(Custom::default())
}

/// `@` and lowercase letters, as `<span class="mention">` with the letters.
struct SpanMention;

impl Construct for SpanMention {
    type Place = Text;
    type State = u8;
    type Token = HtmlTok;

    fn markers(&self) -> &[u8] {
        b"@"
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'@')) => t
                .enter(HtmlTok::Span)
                .enter(HtmlTok::SpanMarker)
                .consume()
                .exit()
                .next(1),
            (1, Some(b'a'..=b'z')) => t.enter(HtmlTok::SpanName).consume().next(2),
            (2, Some(b'a'..=b'z')) => t.consume().next(2),
            (2, _) => t.exit().exit().ok(),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, HtmlTok>>) -> Node {
        empty_node(tokens)
    }

    fn html(
        &self,
        event: HtmlEvent<'_, HtmlTok>,
        _: &[Token<'_, HtmlTok>],
        w: &mut HtmlWriter<'_>,
    ) {
        match event {
            HtmlEvent::Enter(token) if token.name == HtmlTok::Span => {
                w.tag("<span class=\"mention\">");
            }
            HtmlEvent::Exit(token) if token.name == HtmlTok::Span => w.tag("</span>"),
            HtmlEvent::Text(HtmlTok::SpanName, value) => w.text(value),
            _ => {}
        }
    }
}

/// `[[`, a target without `]`, and `]]`, as a link to the target.
struct TargetLink;

impl Construct for TargetLink {
    type Place = Text;
    type State = u8;
    type Token = HtmlTok;

    fn markers(&self) -> &[u8] {
        b"["
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'[')) => t.enter(HtmlTok::Link).consume().next(1),
            (1, Some(b'[')) => t.consume().next(2),
            (2, Some(b']') | None) => t.nok(),
            (2, Some(_)) => t.enter(HtmlTok::LinkTarget).consume().next(3),
            (3, Some(b']')) => t.exit().consume().next(4),
            (3, Some(_)) => t.consume().next(3),
            (4, Some(b']')) => t.consume().exit().ok(),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, HtmlTok>>) -> Node {
        empty_node(tokens)
    }

    fn html(
        &self,
        event: HtmlEvent<'_, HtmlTok>,
        _: &[Token<'_, HtmlTok>],
        w: &mut HtmlWriter<'_>,
    ) {
        if let HtmlEvent::Text(HtmlTok::LinkTarget, value) = event {
            w.tag("<a href=\"");
            w.url(value, Url::Href);
            w.tag("\">");
            w.text(value);
            w.tag("</a>");
        }
    }
}

/// `%` and the rest of the line as text, as a paragraph.
struct ParagraphLine;

impl Construct for ParagraphLine {
    type Place = Flow;
    type State = u8;
    type Token = HtmlTok;

    fn markers(&self) -> &[u8] {
        b"%"
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'%')) => t
                .enter(HtmlTok::Line)
                .enter(HtmlTok::LineMarker)
                .consume()
                .exit()
                .next(1),
            (1, Some(byte)) if byte != b'\n' => t.enter(HtmlTok::LineText).consume().next(2),
            (2, None | Some(b'\n')) => t.exit().exit().ok(),
            (2, Some(_)) => t.consume().next(2),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, HtmlTok>>) -> Node {
        empty_node(tokens)
    }

    fn html(
        &self,
        event: HtmlEvent<'_, HtmlTok>,
        _: &[Token<'_, HtmlTok>],
        w: &mut HtmlWriter<'_>,
    ) {
        match event {
            HtmlEvent::Enter(token) if token.name == HtmlTok::Line => w.tag("<p>"),
            HtmlEvent::Exit(token) if token.name == HtmlTok::Line => w.tag("</p>"),
            _ => {}
        }
    }
}

/// `{{`, a label as text, and `}}`, as `<abbr>` with the HTML of the label
/// also in its `title`.
struct TitledAbbr;

impl Construct for TitledAbbr {
    type Place = Text;
    type State = u8;
    type Token = HtmlTok;

    fn markers(&self) -> &[u8] {
        b"{"
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'{')) => t.enter(HtmlTok::Abbr).consume().next(1),
            (1, Some(b'{')) => t.consume().next(2),
            (2, Some(b'}') | None) => t.nok(),
            (2, Some(_)) => t.enter(HtmlTok::AbbrLabel).consume().next(3),
            (3, Some(b'}')) => t.exit().consume().next(4),
            (3, Some(_)) => t.consume().next(3),
            (4, Some(b'}')) => t.consume().exit().ok(),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, HtmlTok>>) -> Node {
        empty_node(tokens)
    }

    fn html(
        &self,
        event: HtmlEvent<'_, HtmlTok>,
        _: &[Token<'_, HtmlTok>],
        w: &mut HtmlWriter<'_>,
    ) {
        match event {
            HtmlEvent::Enter(token) if token.name == HtmlTok::AbbrLabel => w.buffer(),
            HtmlEvent::Exit(token) if token.name == HtmlTok::AbbrLabel => {
                let label = w.resume();
                w.tag("<abbr");
                w.attr("title", label.as_str());
                w.tag(">");
                w.html(&label);
                w.tag("</abbr>");
            }
            _ => {}
        }
    }
}

fn html_with(input: &str, parse: ParseOptions, compile: CompileOptions) -> String {
    to_html_with_options(input, &Options { parse, compile }).unwrap()
}

fn texts(constructs: Vec<BoxedConstruct<Text>>) -> ParseOptions {
    ParseOptions {
        text_constructs: constructs,
        ..ParseOptions::default()
    }
}

#[test]
fn writes_html_with_handlers() {
    assert_eq!(
        html("a @bc d", texts(vec![construct(SpanMention)])),
        "<p>a <span class=\"mention\">bc</span> d</p>",
        "should write markup around text, and drop the text it does not write"
    );
    assert_eq!(
        html("> a @b\n> @c", texts(vec![construct(SpanMention)])),
        "<blockquote>\n<p>a <span class=\"mention\">b</span>\n<span class=\"mention\">c</span></p>\n</blockquote>",
        "should write matches in containers"
    );
    assert_eq!(
        html("![a @b](c)", texts(vec![construct(SpanMention)])),
        "<p><img src=\"c\" alt=\"a b\" /></p>",
        "should leave markup out of the `alt` of an image, as for built-ins"
    );
    assert_eq!(
        html("a {{*b* c}} d", texts(vec![construct(TitledAbbr)])),
        "<p>a <abbr title=\"&lt;em&gt;b&lt;/em&gt; c\"><em>b</em> c</abbr> d</p>",
        "should give content written to a buffer"
    );
    assert_eq!(
        html("{{@b}}", texts(vec![construct(TitledAbbr), construct(SpanMention)])),
        "<p><abbr title=\"&lt;span class=&quot;mention&quot;&gt;b&lt;/span&gt;\"><span class=\"mention\">b</span></abbr></p>",
        "should write matches in content with their own handlers"
    );
}

#[test]
fn checks_urls_written_by_handlers() {
    let parse = || texts(vec![construct(TargetLink)]);
    assert_eq!(
        html("[[https://a.b]] [[c]]", parse()),
        "<p><a href=\"https://a.b\">https://a.b</a> <a href=\"c\">c</a></p>",
        "should write safe URLs"
    );
    assert_eq!(
        html("[[javascript:alert(1)]]", parse()),
        "<p><a href=\"\">javascript:alert(1)</a></p>",
        "should empty a URL with a dangerous protocol"
    );
    assert_eq!(
        html_with(
            "[[javascript:alert(1)]]",
            parse(),
            CompileOptions {
                allow_dangerous_protocol: true,
                ..CompileOptions::default()
            }
        ),
        "<p><a href=\"javascript:alert(1)\">javascript:alert(1)</a></p>",
        "should keep it with `allow_dangerous_protocol`"
    );
    assert_eq!(
        html("[[a\"<b>]]", parse()),
        "<p><a href=\"a%22%3Cb%3E\">a&quot;&lt;b&gt;</a></p>",
        "should encode URLs and escape text"
    );
}

/// `[[`, a target without `]` or `|`, optionally `|` and an alias as text,
/// and `]]`, as a link to the target with the alias, or else the target.
struct AliasLink;

impl Construct for AliasLink {
    type Place = Text;
    type State = u8;
    type Token = HtmlTok;

    fn markers(&self) -> &[u8] {
        b"["
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'[')) => t.enter(HtmlTok::Link).consume().next(1),
            (1, Some(b'[')) => t.consume().next(2),
            (2, None | Some(b']' | b'|')) => t.nok(),
            (2, Some(_)) => t.enter(HtmlTok::LinkTarget).consume().next(3),
            (3, Some(b'|')) => t.exit().consume().next(4),
            (3, Some(b']')) => t.exit().consume().next(6),
            (3, Some(_)) => t.consume().next(3),
            (4, None | Some(b']')) => t.nok(),
            (4, Some(_)) => t.enter(HtmlTok::LinkAlias).consume().next(5),
            (5, Some(b']')) => t.exit().consume().next(6),
            (5, Some(_)) => t.consume().next(5),
            (6, Some(b']')) => t.consume().exit().ok(),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, HtmlTok>>) -> Node {
        empty_node(tokens)
    }

    fn html(
        &self,
        event: HtmlEvent<'_, HtmlTok>,
        tokens: &[Token<'_, HtmlTok>],
        w: &mut HtmlWriter<'_>,
    ) {
        match event {
            // The target and whether there is an alias are known at the start.
            HtmlEvent::Enter(token) if token.name == HtmlTok::Link => {
                let target = tokens
                    .iter()
                    .find(|token| token.name == HtmlTok::LinkTarget)
                    .expect("expected a target");
                w.tag("<a href=\"");
                w.url(&target.value, Url::Href);
                w.tag("\">");
                if !tokens.iter().any(|token| token.name == HtmlTok::LinkAlias) {
                    w.text(&target.value);
                }
            }
            HtmlEvent::Exit(token) if token.name == HtmlTok::Link => w.tag("</a>"),
            _ => {}
        }
    }
}

#[test]
fn gives_handlers_the_tokens_of_the_match() {
    assert_eq!(
        html("[[a]] [[b|*c*]]", texts(vec![construct(AliasLink)])),
        "<p><a href=\"a\">a</a> <a href=\"b\"><em>c</em></a></p>",
        "should use a later token at the start of a match"
    );
}

#[test]
fn writes_flow_matches_as_blocks() {
    let parse = || ParseOptions {
        flow_constructs: vec![construct(ParagraphLine)],
        ..ParseOptions::default()
    };
    // Where no line ending comes before it, a block starts on its own line.
    for (input, expected, message) in [
        (
            "a\n%*b*\nc",
            "<p>a</p>\n<p><em>b</em></p>\n<p>c</p>",
            "between paragraphs",
        ),
        (
            "> %a\n> %b",
            "<blockquote>\n<p>a</p>\n<p>b</p>\n</blockquote>",
            "at the start of a block quote",
        ),
        (
            "1. %a\n2. %b",
            "<ol>\n<li>\n<p>a</p>\n</li>\n<li>\n<p>b</p>\n</li>\n</ol>",
            "in list items",
        ),
        (
            "- a\n\n%b",
            "<ul>\n<li>a</li>\n</ul>\n<p>b</p>",
            "after a list",
        ),
    ] {
        assert_eq!(
            html(input, parse()),
            expected,
            "should write a flow match {}",
            message
        );
    }

    let with_containers = || ParseOptions {
        document_constructs: vec![construct(Contained::new(line_block, Some(10)))],
        ..parse()
    };
    assert_eq!(
        html("| %a\n| %b", with_containers()),
        "| \n<p>a</p>\n<p>b</p>",
        "should write a flow match in a container"
    );
    assert_eq!(
        html("- a\n\n| b", with_containers()),
        "<ul>\n<li>a</li>\n</ul>\n| \n<p>b</p>",
        "should write a container on its own line"
    );
    assert_eq!(
        html("> ::b[c]", scripted_flow(b':', leaf)),
        "<blockquote>\n::b[c]\n</blockquote>",
        "should write the source of a flow match without a handler on its own line"
    );
}

#[test]
fn resume_without_buffer_panics() {
    /// `{` as a construct that resumes without a buffer.
    struct Resuming;

    impl Construct for Resuming {
        type Place = Text;
        type State = ();
        type Token = HtmlTok;

        fn markers(&self) -> &[u8] {
            b"{"
        }

        fn start(&self) {}

        fn step(&self, (): (), t: Ctx<'_, '_, Self>) -> Step<Self> {
            t.enter(HtmlTok::Abbr).consume().exit().ok()
        }

        fn to_mdast(&self, tokens: Vec<Token<'_, HtmlTok>>) -> Node {
            empty_node(tokens)
        }

        fn html(
            &self,
            _: HtmlEvent<'_, HtmlTok>,
            _: &[Token<'_, HtmlTok>],
            w: &mut HtmlWriter<'_>,
        ) {
            w.resume();
        }
    }

    let message = catch_unwind(AssertUnwindSafe(|| {
        html("{", texts(vec![construct(Resuming)]))
    }))
    .expect_err("expected a panic")
    .downcast::<&str>()
    .map(|message| *message)
    .unwrap_or_default();
    assert_eq!(
        message, "expected a buffer from `HtmlWriter::buffer` to resume",
        "should panic, naming the mistake"
    );
}

/// `@`, written as an element with the attributes in `ATTRIBUTES`, and the
/// raw HTML `<b>`.
struct Attrs;

const ATTRIBUTES: &[(&str, &str)] = &[
    ("id", "a\"b<c"),
    ("onclick", "alert(1)"),
    ("OnLoad", "alert(1)"),
    ("style", "color: red"),
    ("srcdoc", "<script>"),
    ("srcset", "a.png 1x"),
    ("href", "javascript:alert(1)"),
    ("src", "https://example.com/a.png"),
    ("data-x", "1"),
    ("class", "c"),
    ("aria-label", "l"),
    ("x-on:click", "alert(1)"),
    ("hx-on:click", "alert(1)"),
    ("data-hx-on:click", "alert(1)"),
    ("data-ng-click", "alert(1)"),
    ("data-bind", "text:alert(1)"),
    ("ng-click", "alert(1)"),
    ("to", "javascript:alert(1)"),
    ("a b", "1"),
    ("1a", "1"),
    ("x\"y", "1"),
];

impl Construct for Attrs {
    type Place = Text;
    type State = u8;
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        b"@"
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'@')) => t.enter(Tok::A).consume().exit().ok(),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, _: Vec<Token<'_, Tok>>) -> Node {
        Node::Custom(Custom::default())
    }

    fn html(&self, event: HtmlEvent<'_, Tok>, _: &[Token<'_, Tok>], w: &mut HtmlWriter<'_>) {
        if let HtmlEvent::Enter(_) = event {
            w.tag("<x");
            for (name, value) in ATTRIBUTES {
                w.attr(name, value);
            }
            w.tag(">");
            w.raw("<b>");
            w.tag("</x>");
        }
    }
}

#[test]
fn html_writer_keeps_input_out_of_markup() {
    let parse = || texts(vec![construct(Attrs)]);

    assert_eq!(
        html_with("@", parse(), CompileOptions::default()),
        "<p><x id=\"a&quot;b&lt;c\" href=\"\" src=\"https://example.com/a.png\" data-x=\"1\" class=\"c\" aria-label=\"l\">&lt;b&gt;</x></p>",
        "should write only attributes that cannot run script, escape values, check URLs, leave out names that are not names, and escape raw HTML"
    );

    assert_eq!(
        html_with(
            "@",
            parse(),
            CompileOptions {
                allow_dangerous_html: true,
                allow_dangerous_protocol: true,
                ..CompileOptions::default()
            }
        ),
        "<p><x id=\"a&quot;b&lt;c\" onclick=\"alert(1)\" OnLoad=\"alert(1)\" style=\"color: red\" srcdoc=\"&lt;script&gt;\" srcset=\"a.png 1x\" href=\"javascript:alert(1)\" src=\"https://example.com/a.png\" data-x=\"1\" class=\"c\" aria-label=\"l\" x-on:click=\"alert(1)\" hx-on:click=\"alert(1)\" data-hx-on:click=\"alert(1)\" data-ng-click=\"alert(1)\" data-bind=\"text:alert(1)\" ng-click=\"alert(1)\" to=\"javascript:alert(1)\"><b></x></p>",
        "should write all attributes with names, and raw HTML, when dangerous HTML and protocols are allowed"
    );

    assert_eq!(
        html_with("![@](u)", parse(), CompileOptions::default()),
        "<p><img src=\"u\" alt=\"&lt;b&gt;\" /></p>",
        "should leave out markup and attributes in the `alt` of an image"
    );
}

/// `@`, written with a URL of each kind, buffered HTML, and raw HTML.
struct Kinds;

impl Construct for Kinds {
    type Place = Text;
    type State = u8;
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        b"@"
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'@')) => t.enter(Tok::A).consume().exit().ok(),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, _: Vec<Token<'_, Tok>>) -> Node {
        Node::Custom(Custom::default())
    }

    fn html(&self, event: HtmlEvent<'_, Tok>, _: &[Token<'_, Tok>], w: &mut HtmlWriter<'_>) {
        if let HtmlEvent::Enter(_) = event {
            w.tag("<iframe");
            w.attr("src", "javascript:alert(1)");
            w.tag("></iframe><img src=\"");
            w.url("javascript:alert(2)", Url::Image);
            w.tag("\" />");
            w.raw("<script>alert(3)</script>");
        }
    }
}

#[test]
fn html_writer_keeps_script_out_of_urls_and_raw_html() {
    let parse = || texts(vec![construct(Kinds)]);
    let compile = |any_img_src, dangerous_html| CompileOptions {
        allow_any_img_src: any_img_src,
        allow_dangerous_html: dangerous_html,
        gfm_tagfilter: true,
        ..CompileOptions::default()
    };

    assert_eq!(
        html_with("@", parse(), compile(true, false)),
        "<p><iframe src=\"\"></iframe><img src=\"javascript:alert(2)\" />&lt;script&gt;alert(3)&lt;/script&gt;</p>",
        "should allow any protocol with `allow_any_img_src` only for `Url::Image`, not for `src` on any element"
    );

    assert_eq!(
        html_with("@", parse(), compile(false, true)),
        "<p><iframe src=\"\"></iframe><img src=\"\" />&lt;script>alert(3)&lt;/script></p>",
        "should filter raw HTML with `gfm_tagfilter`, as `to_html` filters raw HTML in markdown"
    );
}

/// `{{`, text content, and `}}`, written back from a buffer; it keeps the
/// last buffer, which a construct must not do, to check what `html` does
/// with it in the `alt` of an image.
struct Kept(std::cell::RefCell<Option<markdown::extension::Html>>);

impl Construct for Kept {
    type Place = Text;
    type State = u8;
    type Token = HtmlTok;

    fn markers(&self) -> &[u8] {
        b"{"
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0 | 1, Some(b'{')) => {
                let t = if state == 0 {
                    t.enter(HtmlTok::Abbr)
                } else {
                    t
                };
                t.consume().next(state + 1)
            }
            (2, Some(byte)) if byte != b'}' => t.enter(HtmlTok::AbbrLabel).retry(3),
            (3, Some(b'}')) => t.exit().consume().next(4),
            (3, Some(_)) => t.consume().next(3),
            (4, Some(b'}')) => t.consume().exit().ok(),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, HtmlTok>>) -> Node {
        empty_node(tokens)
    }

    fn html(
        &self,
        event: HtmlEvent<'_, HtmlTok>,
        _: &[Token<'_, HtmlTok>],
        w: &mut HtmlWriter<'_>,
    ) {
        match event {
            HtmlEvent::Enter(token) if token.name == HtmlTok::AbbrLabel => w.buffer(),
            HtmlEvent::Exit(token) if token.name == HtmlTok::AbbrLabel => {
                let label = w.resume();
                let kept = self.0.borrow_mut().replace(label.clone());
                w.html(&kept.unwrap_or(label));
            }
            _ => {}
        }
    }
}

#[test]
fn html_writer_keeps_markup_out_of_alt() {
    let parse = texts(vec![construct(Kept(std::cell::RefCell::new(None)))]);
    assert_eq!(
        html_with(
            "{{![q](/onerror=alert(1)//)}} ![b {{c}}](u)",
            parse,
            CompileOptions::default()
        ),
        "<p><img src=\"/onerror=alert(1)//\" alt=\"q\" /> <img src=\"u\" alt=\"b \" /></p>",
        "should leave out HTML written with markup in the `alt` of an image"
    );
}

#[test]
fn html_writer_resumes_only_its_own_buffers() {
    struct Resume;

    impl Construct for Resume {
        type Place = Text;
        type State = u8;
        type Token = Tok;

        fn markers(&self) -> &[u8] {
            b"@"
        }

        fn start(&self) -> u8 {
            0
        }

        fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
            match (state, t.current()) {
                (0, Some(b'@')) => t.enter(Tok::A).consume().exit().ok(),
                _ => t.nok(),
            }
        }

        fn to_mdast(&self, _: Vec<Token<'_, Tok>>) -> Node {
            Node::Custom(Custom::default())
        }

        fn html(&self, event: HtmlEvent<'_, Tok>, _: &[Token<'_, Tok>], w: &mut HtmlWriter<'_>) {
            if let HtmlEvent::Enter(_) = event {
                w.resume();
            }
        }
    }

    let parse = texts(vec![construct(Resume)]);
    assert!(
        catch_unwind(AssertUnwindSafe(|| html_with(
            "![@](u)",
            parse,
            CompileOptions::default()
        )))
        .is_err(),
        "should panic when a handler resumes a buffer it did not start, such as that of an `alt`"
    );
}
