//! Constructs found in markdown.
//!
//! There are several *things* found when parsing markdown, such as, say, a
//! thematic break.
//! These things are called constructs here.
//! Sometimes, there are several constructs that result in an equivalent thing.
//! For example, [code (fenced)][code_fenced] and
//! [code (indented)][code_indented] are considered different constructs
//!
//! Content types also have a *rest* thing: after all things are parsed,
//! there’s something left.
//! In flow, that is a [paragraph][].
//! In string and text, that is [data][partial_data].
//!
//! The following constructs are found in markdown:
//!
//! *   [attention (strong, emphasis)][attention]
//! *   [autolink][]
//! *   [blank line][blank_line]
//! *   [block quote][block_quote]
//! *   [character escape][character_escape]
//! *   [character reference][character_reference]
//! *   [code (fenced)][code_fenced]
//! *   [code (indented)][code_indented]
//! *   [code (text)][code_text]
//! *   [definition][]
//! *   [hard break (escape)][hard_break_escape]
//! *   [heading (atx)][heading_atx]
//! *   [heading (setext)][heading_setext]
//! *   [html (flow)][html_flow]
//! *   [html (text)][html_text]
//! *   [label end][label_end]
//! *   [label start (image)][label_start_image]
//! *   [label start (link)][label_start_link]
//! *   [list][]
//! *   [paragraph][]
//! *   [thematic break][thematic_break]
//!
//! > 👉 **Note**: for performance reasons, hard break (trailing) is formed by
//! > [whitespace][partial_whitespace].
//!
//! There are also several routines used in different places:
//!
//! *   [data][partial_data]
//! *   [destination][partial_destination]
//! *   [label][partial_label]
//! *   [non lazy continuation][partial_non_lazy_continuation]
//! *   [space or tab][partial_space_or_tab]
//! *   [title][partial_title]
//! *   [whitespace][partial_whitespace]
//!
//! Each construct maintained here is explained with a BNF diagram.
//! For example, the docs for [character escape][character_escape] contain:
//!
//! ```bnf
//! character_escape ::= '\\' ascii_punctuation
//! ```
//!
//! Such diagrams are considered to be *non-normative*.
//! That is to say, they form illustrative, imperfect, but useful, examples.
//! The code, in Rust, is considered to be normative.
//!
//! They also contain references to character as defined by [char][], so for
//! example `ascii_punctuation` refers to
//! [`char::is_ascii_punctuation`][char::is_ascii_punctuation].

pub mod attention;
pub mod autolink;
pub mod blank_line;
pub mod block_quote;
pub mod character_escape;
pub mod character_reference;
pub mod code_fenced;
pub mod code_indented;
pub mod code_text;
pub mod definition;
pub mod hard_break_escape;
pub mod heading_atx;
pub mod heading_setext;
pub mod html_flow;
pub mod html_text;
pub mod label_end;
pub mod label_start_image;
pub mod label_start_link;
pub mod list;
pub mod paragraph;
pub mod partial_bom;
pub mod partial_data;
pub mod partial_destination;
pub mod partial_label;
pub mod partial_non_lazy_continuation;
pub mod partial_space_or_tab;
pub mod partial_title;
pub mod partial_whitespace;
pub mod thematic_break;
