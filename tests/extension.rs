use markdown::{
    extension::{
        construct, BoxedConstruct, Construct, ContentType, Ctx, Flow, Reach, Step, Text, Token,
        TokenType,
    },
    mdast::{Custom, Node, Paragraph, Root, Text as TextNode},
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
