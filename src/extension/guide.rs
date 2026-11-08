//! Guide to syntax extensions: how to write a construct, and which of its
//! options to choose.
//!
//! Start with the [tutorial], which builds a mention, `@alice`, step by step.
//! The [recipes] each show one complete construct for a common pattern.
//! The reference is the rest of [`extension`][super]: [`Construct`][super::Construct]
//! lists its rules, and [`Ctx`][super::Ctx] its steps.
//!
//! ## Words
//!
//! These words mean the same thing everywhere in these docs:
//!
//! * **marker**: a byte that a construct can start at, such as `@`
//! * **match**: the bytes a construct took, from its marker, and the tokens
//!   it made of them
//! * **place**: where a construct runs: [`Text`][super::Text], such as a
//!   paragraph; [`Flow`][super::Flow], the start of a line; or
//!   [`Container`][super::Container], the prefix of each line of a block,
//!   such as `>`
//! * **token**: a named span of a match; tokens nest, and the first one holds
//!   the others
//! * **content**: a token whose bytes are parsed again as markdown, as text
//!   (phrasing, such as a label) or as a document (blocks, such as a body)
//! * **state**: where a construct is in a match, which its step gets and
//!   returns; usually an enum, whose variants carry what the construct has
//!   seen
//! * **reach**: how far a match can read, which decides whether parsing
//!   stays linear
//!
//! ## Which place?
//!
//! | Place | Runs | Like | Options |
//! | - | - | - | - |
//! | [`Text`][super::Text] | at its markers in text | a link, emphasis | `text_constructs` |
//! | [`Flow`][super::Flow] | at the start of a line, over one line or more | a heading, fenced code | `flow_constructs` |
//! | [`Container`][super::Container] | at the start of each line of a block | a block quote, a list item | `document_constructs` |
//!
//! ## Which reach?
//!
//! A text construct that fails is tried again at its next marker, so one
//! that reads far before it fails makes parsing quadratic: on `[[a [[a [[a`
//! without `]]`, each `[[` would read to the end.
//! [`Construct::reach`][super::Construct::reach] keeps it linear:
//!
//! | Reach | Use it when | Linear |
//! | - | - | - |
//! | [`Bytes(n)`][super::Reach::Bytes] | a match is short, such as a mention or a label of at most 999 bytes | yes |
//! | [`Pair`][super::Reach::Pair] | a match stops at its first closer, and cannot start inside the bytes a failed match read | yes |
//! | [`Attention`][super::Reach::Attention] | runs of a marker pair, like `==mark==` | yes |
//! | [`Unbounded`][super::Reach::Unbounded] | a match stops at each of its markers, or you checked it with `assert_linear` | only if so |
//!
//! Flow constructs and containers are tried once per line, so they are
//! linear whatever their reach.
//! With the `test-util` feature, `extension::assert_linear` checks a set of
//! constructs; [recipe 11][recipes::testing] shows it.
//!
//! ## Which HTML route?
//!
//! | Route | For | How |
//! | - | - | - |
//! | [`Construct::html`][super::Construct::html] | [`to_html()`][crate::to_html()] | write the markup around content, event by event |
//! | `h_name` and `attributes` | `markdown_processor`, an element | set them on the [`Custom`][crate::mdast::Custom] node |
//! | a hast handler | `markdown_processor`, anything else | turn the node into hast yourself |
//!
//! A construct used both ways writes its HTML twice, once for each path, as
//! with micromark and mdast-util-to-hast.
//! `markdown_processor::guide` shows all three for one construct.
//!
//! ## From micromark
//!
//! | micromark | here |
//! | - | - |
//! | a `tokenize` function and its states | [`Construct::step`][super::Construct::step] and [`Construct::State`][super::Construct::State] |
//! | `code` | [`Ctx::current`][super::Ctx::current], or [`current_char`][super::Ctx::current_char] |
//! | `effects.enter`, `effects.exit` | [`enter`][super::Ctx::enter], [`exit`][super::Ctx::exit] |
//! | `effects.consume` | [`consume`][super::Ctx::consume], then [`next`][super::Consumed::next] |
//! | `ok`, `nok` | [`ok`][super::Ctx::ok], [`nok`][super::Ctx::nok] |
//! | `effects.attempt` | [`attempt`][super::Ctx::attempt] |
//! | a token's `contentType` | [`TokenType::content`][super::TokenType::content] |
//! | `continuation` and `containerState` | a [`Container`][super::Container]'s state, given with [`container`][super::Consumed::container] |
//! | `previous` | [`Construct::previous`][super::Construct::previous] |
//! | `classifyCharacter` | [`classify`][super::classify] |
//! | `attentionMarkers` | [`Reach::Attention`][super::Reach::Attention] |
//! | an `htmlExtension` | [`Construct::html`][super::Construct::html] |
//! | `this.tag`, `this.encode`, `this.buffer`, `this.resume` | [`HtmlWriter`][super::HtmlWriter]'s `tag`, `text`, `buffer`, `resume` |
//! | `sanitizeUri` | [`HtmlWriter::url`][super::HtmlWriter::url] |
//!
//! There is no `effects.check`.
//! An [`attempt`][super::Ctx::attempt] keeps what it read when it ends in
//! `ok`, and undoes it when it ends in `nok`, so to look ahead, make the
//! outcome to undo the `nok` one: [recipe 3][recipes::flanking_span] does.
//!
//! ## Security
//!
//! The authors of constructs are trusted: a construct is code that runs in
//! the parser.
//! The input is not trusted, and constructs keep it safe in these ways:
//!
//! * a construct that breaks a rule does not match, so input cannot make it
//!   panic; with `test-util`, it panics instead, so its tests find the
//!   mistake
//! * limits cap each match: 256 steps for each byte it reaches, attempts 256
//!   deep, and 32 passes over content
//! * a reach other than `Unbounded` keeps parsing linear on hostile input;
//!   an unbounded construct can make it quadratic
//! * [`HtmlWriter::text`][super::HtmlWriter::text] escapes, and
//!   [`HtmlWriter::url`][super::HtmlWriter::url] checks protocols unless
//!   `allow_dangerous_protocol`; [`HtmlWriter::tag`][super::HtmlWriter::tag]
//!   takes only a `&'static str`, so input cannot become markup;
//!   [`HtmlWriter::attr`][super::HtmlWriter::attr] leaves out attributes
//!   that run script, and [`HtmlWriter::raw`][super::HtmlWriter::raw]
//!   writes HTML from input only with `allow_dangerous_html`
//! * a flow construct that fails after its first line, such as at a limit
//!   that hostile input reaches, ends before that line, which is then
//!   parsed as other markdown: constructs never make parsing return an
//!   error

pub mod recipes;
pub mod tutorial;
