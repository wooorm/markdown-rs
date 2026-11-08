//! Recipe 5: a delimiter run, such as highlighted text in `==mark==`.
//!
//! A run pairs like emphasis, so it needs no states:
//! [`Reach::Attention`][crate::extension::Reach::Attention] hands it to the
//! attention resolver, which pairs runs of the construct's marker as GFM
//! pairs `~~`, and `step` is never called.
//! The sizes are those a run can have, here only 2, and both sides of a
//! pair have the same size, so `===a===` is text.
//! A run opens when it is not followed by whitespace, and closes when it is
//! not preceded by it, so `a == b` is text.
//! Emphasis can go around a pair and inside it.
//!
//! The tokens are [`RunToken`][crate::extension::RunToken]: the pair, its
//! two sequences, and the text between them, which is markdown.
//! The construct has no tokens of its own.
//!
//! ```
//! use markdown::{
//!     extension::{construct, Construct, Ctx, HtmlEvent, HtmlWriter, Reach, RunToken, Step, Text, Token, TokenSlice},
//!     mdast::{Custom, Node},
//!     to_html_with_options, Options, ParseOptions,
//! };
//!
//! struct Mark;
//!
//! impl Construct for Mark {
//!     type Place = Text;
//!     type State = ();
//!     type Token = RunToken;
//!
//!     fn markers(&self) -> &[u8] {
//!         b"="
//!     }
//!
//!     fn start(&self) {}
//!
//!     fn reach(&self) -> Reach {
//!         Reach::Attention(&[2])
//!     }
//!
//!     // Not called for a run.
//!     fn step(&self, (): (), t: Ctx<'_, '_, Self>) -> Step<Self> {
//!         t.nok()
//!     }
//!
//!     fn to_mdast(&self, mut tokens: Vec<Token<'_, RunToken>>) -> Node {
//!         let children = tokens.take_children(RunToken::Text);
//!         Node::Custom(Custom {
//!             name: "mark".into(),
//!             h_name: Some("mark".into()),
//!             children,
//!             ..Custom::default()
//!         })
//!     }
//!
//!     fn html(&self, event: HtmlEvent<'_, RunToken>, _: &[Token<'_, RunToken>], w: &mut HtmlWriter<'_>) {
//!         match event {
//!             HtmlEvent::Enter(token) if token.name == RunToken::Run => w.tag("<mark>"),
//!             HtmlEvent::Exit(token) if token.name == RunToken::Run => w.tag("</mark>"),
//!             // The sequences are not written, and the text is markdown.
//!             _ => {}
//!         }
//!     }
//! }
//!
//! let options = Options {
//!     parse: ParseOptions {
//!         text_constructs: vec![construct(Mark)],
//!         ..ParseOptions::default()
//!     },
//!     ..Options::default()
//! };
//! let html = |input: &str| to_html_with_options(input, &options).unwrap();
//!
//! assert_eq!(html("a ==b== c"), "<p>a <mark>b</mark> c</p>");
//! assert_eq!(html("*==a==*"), "<p><em><mark>a</mark></em></p>");
//! assert_eq!(html("==*a*=="), "<p><mark><em>a</em></mark></p>");
//! assert_eq!(html("===a==="), "<p>===a===</p>");
//! assert_eq!(html("a == b == c"), "<p>a == b == c</p>");
//! assert_eq!(html("==a"), "<p>==a</p>");
//! ```
