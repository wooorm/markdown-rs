//! Constructs found in markdown.
//!
//! Constructs are grouped by content type.
//! Which content type is allowed somewhere, defines which constructs are
//! allowed there.
//!
//! ## Content type
//!
//! The following content types are found in markdown:
//!
//! *   [document][]
//! *   [flow][]
//! *   [string][]
//! *   [text][]
//!
//! Content types also have a *rest* thing: after all things are parsed,
//! there’s something left.
//! In document, that is [flow][].
//! In flow, that is a [paragraph][].
//! In string and text, that is [data][partial_data].
//!
//! ## Construct
//!
//! There are several *things* found when parsing markdown, such as, say, a
//! thematic break.
//! These things are called constructs here.
//! Sometimes, there are several constructs that result in an equivalent thing.
//! For example, [code (fenced)][code_fenced] and
//! [code (indented)][code_indented] are considered different constructs.
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
//! *   [list item][list_item]
//! *   [paragraph][]
//! *   [thematic break][thematic_break]
//!
//! > 👉 **Note**: for performance reasons, hard break (trailing) is formed by
//! > [whitespace][partial_whitespace].
//!
//! There are also several small subroutines typically used in different places:
//!
//! *   [bom][partial_bom]
//! *   [data][partial_data]
//! *   [destination][partial_destination]
//! *   [label][partial_label]
//! *   [non lazy continuation][partial_non_lazy_continuation]
//! *   [space or tab][partial_space_or_tab]
//! *   [space or tab, eol][partial_space_or_tab_eol]
//! *   [title][partial_title]
//! *   [whitespace][partial_whitespace]
//!
//! ## Grammar
//!
//! Each construct maintained here is explained with a BNF diagram.
//!
//! Such diagrams are considered to be *non-normative*.
//! That is to say, they form illustrative, imperfect, but useful, examples.
//! The code, in Rust, is considered to be normative.
//!
//! The actual syntax of markdown can be described in Backus–Naur form (BNF) as:
//!
//! ```bnf
//! markdown = .*
//! ```
//!
//! No, that’s [not a typo][bnf]: markdown has no syntax errors; anything
//! thrown at it renders *something*.
//!
//! These diagrams contain references to character group as defined by Rust on
//! for example [char][], but also often on [u8][], which is what `micromark-rs`
//! typically works on.
//! So, for example, `ascii_punctuation` refers to
//! [`u8::is_ascii_punctuation`][u8::is_ascii_punctuation].
//!
//! For clarity, the productions used throughout are:
//!
//! ```bnf
//! ; Rust / ASCII groups:
//! ; 'a'..='z'
//! ascii_lowercase ::= 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l' | 'm' | 'n' | 'o' | 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'v' | 'w' | 'x' | 'y' | 'z'
//! ; 'A'..='Z'
//! ascii_uppercase ::= 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z'
//! ; 'A'..='Z', 'a'..='z'
//! ascii_alphabetic ::= ascii_lowercase | ascii_uppercase
//! ; '0'..='9'
//! ascii_digit ::= '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
//! ; '0'..='9', 'A'..='F', 'a'..='f'
//! ascii_hexdigit ::= ascii_digit | 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'A' | 'B' | 'C' | 'D' | 'E' | 'F'
//! ; '0'..='9', 'A'..='Z', 'a'..='z'
//! ascii_alphanumeric ::= ascii_digit | ascii_alphabetic
//! ; '!'..='/', ':'..='@', '['..='`', '{'..='~'
//! ascii_punctuation ::= '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@' | '[' | '\' | ']' | '^' | '_' | '`' | '{' | '|' | '}' | '~'
//! ; 0x00..=0x1F, 0x7F
//! ascii_control ::= 0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 | 0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x0F | 0x10 | 0x11 | 0x12 | 0x13 | 0x14 | 0x15 | 0x16 | 0x17 | 0x18 | 0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E | 0x1F | 0x7F
//!
//! ; Markdown groups:
//! ; Any byte (u8)
//! byte ::= 0x00..=0xFFFF
//! space_or_tab ::= '\t' | ' '
//! eol ::= '\n' | '\r' | '\r\n'
//! line ::= byte - eol
//! text ::= line - space_or_tab
//! space_or_tab_eol ::= 1*space_or_tab | *space_or_tab eol *space_or_tab
//!
//! ; Unicode groups:
//! unicode_whitespace ::= ? ; See `char::is_whitespace`.
//! unicode_punctuation ::= ? ; See `src/unicode.rs`.
//! ```
//!
//! [bnf]: http://trevorjim.com/a-specification-for-markdown/

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
pub mod document;
pub mod flow;
pub mod hard_break_escape;
pub mod heading_atx;
pub mod heading_setext;
pub mod html_flow;
pub mod html_text;
pub mod label_end;
pub mod label_start_image;
pub mod label_start_link;
pub mod list_item;
pub mod paragraph;
pub mod partial_bom;
pub mod partial_data;
pub mod partial_destination;
pub mod partial_label;
pub mod partial_non_lazy_continuation;
pub mod partial_space_or_tab;
pub mod partial_space_or_tab_eol;
pub mod partial_title;
pub mod partial_whitespace;
pub mod string;
pub mod text;
pub mod thematic_break;
