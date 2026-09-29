use markdown::{
    extension::{construct, BoxedConstruct, Construct, Ctx, Reach, Step, Text, Token, TokenType},
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
    ];

    fn name(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
            Self::Braces => "braces",
            Self::BracesData => "bracesData",
            Self::Line => "line",
            Self::Percent => "percent",
            Self::Letter => "letter",
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
            ..Custom::default()
        })
    }
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
    ];

    for (rule, step) in cases {
        if let Some(tree) = breaking(rule, || to_mdast("x {y", &scripted(step)).unwrap()) {
            assert!(
                find_scripted(&tree).is_none(),
                "should not match a construct that breaks `{}`",
                rule
            );
            assert_eq!(
                tree.to_string(),
                "x {y",
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
fn resets_retries_after_each_byte() {
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
        text_constructs: many(219),
        ..ParseOptions::default()
    };

    assert_eq!(
        to_mdast("@a", &parse).unwrap_err().to_string(),
        "Unexpected 65700 token names in constructs, expected at most 65,535 (markdown-rs:too-many-construct-tokens)",
        "should error before parsing when token names do not fit"
    );

    let parse = ParseOptions {
        text_constructs: many(218),
        ..ParseOptions::default()
    };
    assert!(
        to_mdast("@a", &parse).is_ok(),
        "should work with 65,400 token names"
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
