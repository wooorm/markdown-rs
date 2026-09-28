//! Turn an mdast (markdown syntax tree) into a hast (HTML syntax tree).
//!
//! Port of <https://github.com/syntax-tree/mdast-util-to-hast>, by the same
//! author:
//!
//! (The MIT License)
//!
//! Copyright (c) 2016 Titus Wormer <tituswormer@gmail.com>
//!
//! Permission is hereby granted, free of charge, to any person obtaining
//! a copy of this software and associated documentation files (the
//! 'Software'), to deal in the Software without restriction, including
//! without limitation the rights to use, copy, modify, merge, publish,
//! distribute, sublicense, and/or sell copies of the Software, and to
//! permit persons to whom the Software is furnished to do so, subject to
//! the following conditions:
//!
//! The above copyright notice and this permission notice shall be
//! included in all copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED 'AS IS', WITHOUT WARRANTY OF ANY KIND,
//! EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
//! MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
//! IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
//! CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
//! TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
//! SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{
    boxed::Box,
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::fmt;
use markdown::{mdast, normalize_uri, unist::Position};

/// Turn a custom node into hast, given its children already turned into hast.
pub type Handler = dyn Fn(&mdast::Custom, Vec<hast::Node>) -> Vec<hast::Node>;

/// Handlers, by custom node name.
pub type Handlers = BTreeMap<String, Box<Handler>>;

/// Prefix of footnote ids, the default of `markdown::to_html` and `mdast-util-to-hast`.
const CLOBBER_PREFIX: &str = "user-content-";

// To do: support options like these fields of `markdown::CompileOptions`:
// ```
// pub gfm_footnote_label: Option<String>,
// pub gfm_footnote_label_tag_name: Option<String>,
// pub gfm_footnote_label_attributes: Option<String>,
// pub gfm_footnote_back_label: Option<String>,
// pub gfm_footnote_clobber_prefix: Option<String>,
// ```
//
// Maybe also:
// * option to persist `meta`?
// * option to generate a `style` attribute instead of `align`?

/// State needed to turn mdast into hast.
struct State<'a> {
    /// Handlers for custom nodes.
    handlers: &'a Handlers,
    /// List of gathered definitions.
    ///
    /// The field at `0` is the identifier, `1` the URL, and `2` the title.
    definitions: Vec<(String, String, Option<String>)>,
    /// List of gathered GFM footnote definitions.
    ///
    /// The field at `0` is the identifier, `1` the node.
    footnote_definitions: Vec<(String, Vec<hast::Node>)>,
    /// List of gathered GFM footnote calls.
    ///
    /// The field at `0` is the identifier, `1` a counter of how many times
    /// it is used.
    footnote_calls: Vec<(String, usize)>,
}

/// Result of turning something into hast.
#[derive(Debug)]
enum Result {
    /// Multiple nodes.
    Fragment(Vec<hast::Node>),
    /// Single nodes.
    Node(hast::Node),
    /// Nothing.
    None,
}

/// Configuration.
#[derive(Default)]
pub struct Options {
    /// Handlers for custom nodes, by name (default: none).
    ///
    /// A custom node without a handler becomes text of its value, or else a
    /// `div` of its children, like the unknown handler of
    /// `mdast-util-to-hast`.
    /// Its `attributes` are not used, since they can come from the author.
    pub handlers: Handlers,
}

impl fmt::Debug for Options {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Options")
            .field("handlers", &self.handlers.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Turn an mdast tree into hast.
///
/// HTML nodes become `raw` nodes, which `hast_util_to_html` encodes unless
/// dangerous HTML is allowed.
pub fn to_hast(tree: &mdast::Node) -> hast::Node {
    to_hast_with_options(tree, &Options::default())
}

/// Turn an mdast tree, with options, into hast.
pub fn to_hast_with_options(tree: &mdast::Node, options: &Options) -> hast::Node {
    let mut definitions = vec![];

    // Collect definitions.
    // Calls take info from their definition.
    // Calls can come before definitions.
    // Footnote calls can also come before footnote definitions, but those
    // calls *do not* take info from their definitions, so we don’t care
    // about footnotes here.
    visit(tree, |node| {
        if let mdast::Node::Definition(definition) = node {
            definitions.push((
                definition.identifier.clone(),
                definition.url.clone(),
                definition.title.clone(),
            ));
        }
    });

    let mut state = State {
        handlers: &options.handlers,
        definitions,
        footnote_definitions: vec![],
        footnote_calls: vec![],
    };

    let result = one(&mut state, tree, None);

    let mut items = vec![];

    let mut index = 0;
    while index < state.footnote_calls.len() {
        let (id, count) = &state.footnote_calls[index];
        let safe_id = normalize_uri(&id.to_lowercase());

        // Find definition, and skip calls without one, like `mdast-util-to-hast`.
        let mut definition_index = 0;
        while definition_index < state.footnote_definitions.len() {
            if &state.footnote_definitions[definition_index].0 == id {
                break;
            }
            definition_index += 1;
        }
        if definition_index == state.footnote_definitions.len() {
            index += 1;
            continue;
        }

        // We’ll find each used definition once, so we can split off to take the content.
        let mut content = state.footnote_definitions[definition_index].1.split_off(0);

        let mut reference_index = 0;
        let mut backreferences = vec![];
        while reference_index < *count {
            let mut backref_children = vec![hast::Node::Text(hast::Text {
                value: "↩".into(),
                position: None,
            })];

            if reference_index != 0 {
                backreferences.push(hast::Node::Text(hast::Text {
                    value: " ".into(),
                    position: None,
                }));

                backref_children.push(hast::Node::Element(hast::Element {
                    tag_name: "sup".into(),
                    properties: vec![],
                    children: vec![hast::Node::Text(hast::Text {
                        value: (reference_index + 1).to_string().into(),
                        position: None,
                    })],
                    position: None,
                }));
            }

            backreferences.push(hast::Node::Element(hast::Element {
                tag_name: "a".into(),
                properties: vec![
                    (
                        "href".into(),
                        hast::PropertyValue::String(format!(
                            "#{}fnref-{}{}",
                            CLOBBER_PREFIX,
                            safe_id,
                            if reference_index == 0 {
                                String::new()
                            } else {
                                format!("-{}", reference_index + 1)
                            }
                        )),
                    ),
                    (
                        "dataFootnoteBackref".into(),
                        hast::PropertyValue::Boolean(true),
                    ),
                    (
                        "ariaLabel".into(),
                        hast::PropertyValue::String("Back to content".into()),
                    ),
                    (
                        "className".into(),
                        hast::PropertyValue::SpaceSeparated(vec!["data-footnote-backref".into()]),
                    ),
                ],
                children: backref_children,
                position: None,
            }));

            reference_index += 1;
        }

        let mut backreference_opt = Some(backreferences);

        if let Some(hast::Node::Element(tail_element)) = content.last_mut() {
            if tail_element.tag_name == "p" {
                if let Some(hast::Node::Text(text)) = tail_element.children.last_mut() {
                    text.value.to_mut().push(' ');
                } else {
                    tail_element.children.push(hast::Node::Text(hast::Text {
                        value: " ".into(),
                        position: None,
                    }));
                }

                if let Some(mut backreference) = backreference_opt {
                    backreference_opt = None;
                    tail_element.children.append(&mut backreference);
                }
            }
        }

        // No paragraph, just push them.
        if let Some(mut backreference) = backreference_opt {
            content.append(&mut backreference);
        }

        items.push(hast::Node::Element(hast::Element {
            tag_name: "li".into(),
            properties: vec![(
                "id".into(),
                hast::PropertyValue::String(format!("{}fn-{}", CLOBBER_PREFIX, safe_id)),
            )],
            children: wrap(content, true),
            position: None,
        }));
        index += 1;
    }

    // Like `mdast-util-to-hast`, return a single node as is when there is no footer.
    if items.is_empty() {
        if let Result::Node(node) = result {
            return node;
        }
    }

    // We either have to generate a footer, or we don’t have a single node.
    // So we need a root.
    let mut root = hast::Root {
        children: vec![],
        position: None,
    };

    match result {
        Result::Fragment(children) => root.children = children,
        Result::Node(node) => {
            if let hast::Node::Root(existing) = node {
                root = existing;
            } else {
                root.children.push(node);
            }
        }
        Result::None => {}
    }

    if !items.is_empty() {
        root.children.push(hast::Node::Text(hast::Text {
            value: "\n".into(),
            position: None,
        }));
        root.children.push(hast::Node::Element(hast::Element {
            tag_name: "section".into(),
            properties: vec![
                ("dataFootnotes".into(), hast::PropertyValue::Boolean(true)),
                (
                    "className".into(),
                    hast::PropertyValue::SpaceSeparated(vec!["footnotes".into()]),
                ),
            ],
            children: vec![
                hast::Node::Element(hast::Element {
                    tag_name: "h2".into(),
                    properties: vec![
                        (
                            "id".into(),
                            hast::PropertyValue::String("footnote-label".into()),
                        ),
                        (
                            "className".into(),
                            hast::PropertyValue::SpaceSeparated(vec!["sr-only".into()]),
                        ),
                    ],
                    children: vec![hast::Node::Text(hast::Text {
                        value: "Footnotes".into(),
                        position: None,
                    })],
                    position: None,
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None,
                }),
                hast::Node::Element(hast::Element {
                    tag_name: "ol".into(),
                    properties: vec![],
                    children: wrap(items, true),
                    position: None,
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None,
                }),
            ],
            position: None,
        }));
        root.children.push(hast::Node::Text(hast::Text {
            value: "\n".into(),
            position: None,
        }));
    }

    hast::Node::Root(root)
}

/// Turn one mdast node into hast.
fn one(state: &mut State, node: &mdast::Node, parent: Option<&mdast::Node>) -> Result {
    match node {
        mdast::Node::Blockquote(d) => transform_block_quote(state, node, d),
        mdast::Node::Break(d) => transform_break(state, node, d),
        mdast::Node::Code(d) => transform_code(state, node, d),
        mdast::Node::Delete(d) => transform_delete(state, node, d),
        mdast::Node::Emphasis(d) => transform_emphasis(state, node, d),
        mdast::Node::FootnoteDefinition(d) => transform_footnote_definition(state, node, d),
        mdast::Node::FootnoteReference(d) => transform_footnote_reference(state, node, d),
        mdast::Node::Heading(d) => transform_heading(state, node, d),
        mdast::Node::Image(d) => transform_image(state, node, d),
        mdast::Node::ImageReference(d) => transform_image_reference(state, node, d),
        mdast::Node::InlineCode(d) => transform_inline_code(state, node, d),
        mdast::Node::InlineMath(d) => transform_inline_math(state, node, d),
        mdast::Node::Link(d) => transform_link(state, node, d),
        mdast::Node::LinkReference(d) => transform_link_reference(state, node, d),
        mdast::Node::ListItem(d) => transform_list_item(state, node, parent, d),
        mdast::Node::List(d) => transform_list(state, node, d),
        mdast::Node::Math(d) => transform_math(state, node, d),
        mdast::Node::Html(d) => Result::Node(hast::Node::Raw(hast::Raw {
            value: d.value.clone(),
            position: d.position.clone(),
        })),
        mdast::Node::Paragraph(d) => transform_paragraph(state, node, d),
        mdast::Node::Root(d) => transform_root(state, node, d),
        mdast::Node::Strong(d) => transform_strong(state, node, d),
        // Note: this is only called here if there is a single cell passed, not when one is found in a table.
        mdast::Node::TableCell(d) => {
            transform_table_cell(state, node, false, mdast::AlignKind::None, d)
        }
        // Note: this is only called here if there is a single row passed, not when one is found in a table.
        mdast::Node::TableRow(d) => transform_table_row(state, node, false, None, d),
        mdast::Node::Table(d) => transform_table(state, node, d),
        mdast::Node::Text(d) => transform_text(state, node, d),
        mdast::Node::ThematicBreak(d) => transform_thematic_break(state, node, d),
        mdast::Node::Custom(d) => transform_custom(state, node, d),
        // Ignore.
        mdast::Node::Definition(_)
        | mdast::Node::MdxFlowExpression(_)
        | mdast::Node::MdxTextExpression(_)
        | mdast::Node::MdxJsxFlowElement(_)
        | mdast::Node::MdxJsxTextElement(_)
        | mdast::Node::MdxjsEsm(_)
        | mdast::Node::Yaml(_)
        | mdast::Node::Toml(_) => Result::None,
    }
}

/// [`Blockquote`][mdast::Blockquote].
fn transform_block_quote(
    state: &mut State,
    node: &mdast::Node,
    block_quote: &mdast::Blockquote,
) -> Result {
    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "blockquote".into(),
        properties: vec![],
        children: wrap(all(state, node), true),
        position: block_quote.position.clone(),
    }))
}

/// [`Break`][mdast::Break].
fn transform_break(_state: &mut State, _node: &mdast::Node, break_: &mdast::Break) -> Result {
    Result::Fragment(vec![
        hast::Node::Element(hast::Element {
            tag_name: "br".into(),
            properties: vec![],
            children: vec![],
            position: break_.position.clone(),
        }),
        hast::Node::Text(hast::Text {
            value: "\n".into(),
            position: None,
        }),
    ])
}

/// [`Code`][mdast::Code].
fn transform_code(_state: &mut State, _node: &mdast::Node, code: &mdast::Code) -> Result {
    let mut value = code.value.clone();
    // Like `mdast-util-to-hast`: an empty value gets no line ending.
    if !value.is_empty() {
        value.push('\n');
    }
    let mut properties = vec![];

    if let Some(lang) = code.lang.as_ref() {
        properties.push((
            "className".into(),
            hast::PropertyValue::SpaceSeparated(vec![format!("language-{}", lang)]),
        ));
    }

    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "pre".into(),
        properties: vec![],
        children: vec![hast::Node::Element(hast::Element {
            tag_name: "code".into(),
            properties,
            children: vec![hast::Node::Text(hast::Text {
                value: value.into(),
                position: None,
            })],
            position: code.position.clone(),
        })],
        position: code.position.clone(),
    }))
}

/// [`Custom`][mdast::Custom].
///
/// Uses the handler registered for its name.
/// Without one, like the default unknown handler of `mdast-util-to-hast`,
/// makes text of a value, or a `div` of the children.
fn transform_custom(state: &mut State, node: &mdast::Node, custom: &mdast::Custom) -> Result {
    // Children are converted only when used: converting registers footnote calls and definitions.
    if let Some(handler) = state.handlers.get(&custom.name) {
        let children = all(state, node);
        Result::Fragment(handler(custom, children))
    } else if let Some(value) = &custom.value {
        Result::Node(hast::Node::Text(hast::Text {
            value: value.clone().into(),
            position: custom.position.clone(),
        }))
    } else {
        Result::Node(hast::Node::Element(hast::Element {
            tag_name: "div".into(),
            properties: vec![],
            children: all(state, node),
            position: custom.position.clone(),
        }))
    }
}

/// [`Delete`][mdast::Delete].
fn transform_delete(state: &mut State, node: &mdast::Node, delete: &mdast::Delete) -> Result {
    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "del".into(),
        properties: vec![],
        children: all(state, node),
        position: delete.position.clone(),
    }))
}

/// [`Emphasis`][mdast::Emphasis].
fn transform_emphasis(state: &mut State, node: &mdast::Node, emphasis: &mdast::Emphasis) -> Result {
    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "em".into(),
        properties: vec![],
        children: all(state, node),
        position: emphasis.position.clone(),
    }))
}

/// [`FootnoteDefinition`][mdast::FootnoteDefinition].
fn transform_footnote_definition(
    state: &mut State,
    node: &mdast::Node,
    footnote_definition: &mdast::FootnoteDefinition,
) -> Result {
    let children = all(state, node);
    // Set aside.
    state
        .footnote_definitions
        .push((footnote_definition.identifier.clone(), children));
    Result::None
}

/// [`FootnoteReference`][mdast::FootnoteReference].
fn transform_footnote_reference(
    state: &mut State,
    _node: &mdast::Node,
    footnote_reference: &mdast::FootnoteReference,
) -> Result {
    let safe_id = normalize_uri(&footnote_reference.identifier.to_lowercase());
    let mut call_index = 0;

    // See if this has been called before.
    while call_index < state.footnote_calls.len() {
        if state.footnote_calls[call_index].0 == footnote_reference.identifier {
            break;
        }
        call_index += 1;
    }

    // New.
    if call_index == state.footnote_calls.len() {
        state
            .footnote_calls
            .push((footnote_reference.identifier.clone(), 0));
    }

    // Increment.
    state.footnote_calls[call_index].1 += 1;

    let reuse_counter = state.footnote_calls[call_index].1;

    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "sup".into(),
        properties: vec![],
        children: vec![hast::Node::Element(hast::Element {
            tag_name: "a".into(),
            properties: vec![
                (
                    "href".into(),
                    hast::PropertyValue::String(format!("#{}fn-{}", CLOBBER_PREFIX, safe_id)),
                ),
                (
                    "id".into(),
                    hast::PropertyValue::String(format!(
                        "{}fnref-{}{}",
                        CLOBBER_PREFIX,
                        safe_id,
                        if reuse_counter > 1 {
                            format!("-{}", reuse_counter)
                        } else {
                            String::new()
                        }
                    )),
                ),
                ("dataFootnoteRef".into(), hast::PropertyValue::Boolean(true)),
                (
                    "ariaDescribedBy".into(),
                    hast::PropertyValue::String("footnote-label".into()),
                ),
            ],
            children: vec![hast::Node::Text(hast::Text {
                value: (call_index + 1).to_string().into(),
                position: None,
            })],
            position: None,
        })],
        position: footnote_reference.position.clone(),
    }))
}

/// [`Heading`][mdast::Heading].
fn transform_heading(state: &mut State, node: &mdast::Node, heading: &mdast::Heading) -> Result {
    Result::Node(hast::Node::Element(hast::Element {
        tag_name: format!("h{}", heading.depth).into(),
        properties: vec![],
        children: all(state, node),
        position: heading.position.clone(),
    }))
}

/// [`Image`][mdast::Image].
fn transform_image(_state: &mut State, _node: &mdast::Node, image: &mdast::Image) -> Result {
    let mut properties = vec![];

    properties.push((
        "src".into(),
        hast::PropertyValue::String(normalize_uri(&image.url)),
    ));

    properties.push(("alt".into(), hast::PropertyValue::String(image.alt.clone())));

    if let Some(value) = image.title.as_ref() {
        properties.push(("title".into(), hast::PropertyValue::String(value.into())));
    }

    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "img".into(),
        properties,
        children: vec![],
        position: image.position.clone(),
    }))
}

/// [`ImageReference`][mdast::ImageReference].
fn transform_image_reference(
    state: &State,
    _node: &mdast::Node,
    image_reference: &mdast::ImageReference,
) -> Result {
    let mut properties = vec![];

    let definition = state
        .definitions
        .iter()
        .find(|d| d.0 == image_reference.identifier);

    let (_, url, title) = match definition {
        Some(definition) => definition,
        None => {
            return Result::Node(hast::Node::Text(hast::Text {
                value: format!(
                    "![{}{}",
                    image_reference.alt,
                    revert_suffix(
                        &image_reference.reference_kind,
                        image_reference.label.as_deref(),
                        &image_reference.identifier
                    )
                )
                .into(),
                position: None,
            }))
        }
    };

    properties.push((
        "src".into(),
        hast::PropertyValue::String(normalize_uri(url)),
    ));

    properties.push((
        "alt".into(),
        hast::PropertyValue::String(image_reference.alt.clone()),
    ));

    if let Some(value) = title {
        properties.push(("title".into(), hast::PropertyValue::String(value.into())));
    }

    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "img".into(),
        properties,
        children: vec![],
        position: image_reference.position.clone(),
    }))
}

/// [`InlineCode`][mdast::InlineCode].
fn transform_inline_code(
    _state: &mut State,
    _node: &mdast::Node,
    inline_code: &mdast::InlineCode,
) -> Result {
    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "code".into(),
        properties: vec![],
        children: vec![hast::Node::Text(hast::Text {
            value: replace_eols_with_spaces(&inline_code.value).into(),
            position: None,
        })],
        position: inline_code.position.clone(),
    }))
}

/// [`InlineMath`][mdast::InlineMath].
fn transform_inline_math(
    _state: &mut State,
    _node: &mdast::Node,
    inline_math: &mdast::InlineMath,
) -> Result {
    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "code".into(),
        properties: vec![(
            "className".into(),
            hast::PropertyValue::SpaceSeparated(vec!["language-math".into(), "math-inline".into()]),
        )],
        children: vec![hast::Node::Text(hast::Text {
            value: replace_eols_with_spaces(&inline_math.value).into(),
            position: None,
        })],
        position: inline_math.position.clone(),
    }))
}

/// [`Link`][mdast::Link].
fn transform_link(state: &mut State, node: &mdast::Node, link: &mdast::Link) -> Result {
    let mut properties = vec![];

    properties.push((
        "href".into(),
        hast::PropertyValue::String(normalize_uri(&link.url)),
    ));

    if let Some(value) = link.title.as_ref() {
        properties.push(("title".into(), hast::PropertyValue::String(value.into())));
    }

    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "a".into(),
        properties,
        children: all(state, node),
        position: link.position.clone(),
    }))
}

/// [`LinkReference`][mdast::LinkReference].
fn transform_link_reference(
    state: &mut State,
    node: &mdast::Node,
    link_reference: &mdast::LinkReference,
) -> Result {
    let mut properties = vec![];

    let definition = state
        .definitions
        .iter()
        .find(|d| d.0 == link_reference.identifier);

    let (_, url, title) = match definition {
        Some(definition) => definition,
        None => {
            let suffix = revert_suffix(
                &link_reference.reference_kind,
                link_reference.label.as_deref(),
                &link_reference.identifier,
            );
            let mut children = all(state, node);

            if let Some(hast::Node::Text(head)) = children.first_mut() {
                head.value = format!("[{}", head.value).into();
            } else {
                children.insert(
                    0,
                    hast::Node::Text(hast::Text {
                        value: "[".into(),
                        position: None,
                    }),
                );
            }

            if let Some(hast::Node::Text(tail)) = children.last_mut() {
                tail.value.to_mut().push_str(&suffix);
            } else {
                children.push(hast::Node::Text(hast::Text {
                    value: suffix.into(),
                    position: None,
                }));
            }

            return Result::Fragment(children);
        }
    };

    properties.push((
        "href".into(),
        hast::PropertyValue::String(normalize_uri(url)),
    ));

    if let Some(value) = title {
        properties.push(("title".into(), hast::PropertyValue::String(value.into())));
    }

    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "a".into(),
        properties,
        children: all(state, node),
        position: link_reference.position.clone(),
    }))
}

/// End of a reference without a definition, turned back into text, like
/// `revert.js` in `mdast-util-to-hast`.
fn revert_suffix(kind: &mdast::ReferenceKind, label: Option<&str>, identifier: &str) -> String {
    match kind {
        mdast::ReferenceKind::Shortcut => "]".into(),
        mdast::ReferenceKind::Collapsed => "][]".into(),
        mdast::ReferenceKind::Full => format!(
            "][{}]",
            match label {
                Some(label) if !label.is_empty() => label,
                _ => identifier,
            }
        ),
    }
}

/// [`ListItem`][mdast::ListItem].
fn transform_list_item(
    state: &mut State,
    node: &mdast::Node,
    parent: Option<&mdast::Node>,
    list_item: &mdast::ListItem,
) -> Result {
    let mut children = all(state, node);
    let mut loose = list_item_loose(node);

    if let Some(parent) = parent {
        if matches!(parent, mdast::Node::List(_)) {
            loose = list_loose(parent);
        }
    }

    let mut properties = vec![];

    // Inject a checkbox.
    if let Some(checked) = list_item.checked {
        // According to github-markdown-css, this class hides bullet.
        // See: <https://github.com/sindresorhus/github-markdown-css>.
        properties.push((
            "className".into(),
            hast::PropertyValue::SpaceSeparated(vec!["task-list-item".into()]),
        ));

        let mut input = Some(hast::Node::Element(hast::Element {
            tag_name: "input".into(),
            properties: vec![
                (
                    "type".into(),
                    hast::PropertyValue::String("checkbox".into()),
                ),
                ("checked".into(), hast::PropertyValue::Boolean(checked)),
                ("disabled".into(), hast::PropertyValue::Boolean(true)),
            ],
            children: vec![],
            position: None,
        }));

        if let Some(hast::Node::Element(x)) = children.first_mut() {
            if x.tag_name == "p" {
                if !x.children.is_empty() {
                    x.children.insert(
                        0,
                        hast::Node::Text(hast::Text {
                            value: " ".into(),
                            position: None,
                        }),
                    );
                }

                x.children.insert(0, input.take().unwrap());
            }
        }

        // If the input wasn’t injected yet, inject a paragraph.
        if let Some(input) = input {
            children.insert(
                0,
                hast::Node::Element(hast::Element {
                    tag_name: "p".into(),
                    properties: vec![],
                    children: vec![input],
                    position: None,
                }),
            );
        }
    }

    children.reverse();
    let mut result = vec![];
    let mut head = true;
    let empty = children.is_empty();
    let mut tail_p = false;

    while let Some(child) = children.pop() {
        let mut is_p = false;
        if let hast::Node::Element(el) = &child {
            if el.tag_name == "p" {
                is_p = true;
            }
        }

        // Add eols before nodes, except if this is a tight, first paragraph.
        if loose || !head || !is_p {
            result.push(hast::Node::Text(hast::Text {
                value: "\n".into(),
                position: None,
            }));
        }

        if is_p && !loose {
            // Unwrap the paragraph.
            if let hast::Node::Element(mut el) = child {
                result.append(&mut el.children);
            }
        } else {
            result.push(child);
        }

        head = false;
        tail_p = is_p;
    }

    // Add eol after last node, except if it is tight or a paragraph.
    if !empty && (loose || !tail_p) {
        result.push(hast::Node::Text(hast::Text {
            value: "\n".into(),
            position: None,
        }));
    }

    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "li".into(),
        properties,
        children: result,
        position: list_item.position.clone(),
    }))
}

/// [`List`][mdast::List].
fn transform_list(state: &mut State, node: &mdast::Node, list: &mdast::List) -> Result {
    let mut contains_task_list = false;
    let mut index = 0;

    while index < list.children.len() {
        if let mdast::Node::ListItem(item) = &list.children[index] {
            if item.checked.is_some() {
                contains_task_list = true;
            }
        }

        index += 1;
    }

    let mut properties = vec![];

    // Add start.
    if let Some(start) = list.start {
        if list.ordered && start != 1 {
            properties.push((
                "start".into(),
                hast::PropertyValue::String(start.to_string()),
            ));
        }
    }

    // Like GitHub, add a class for custom styling.
    if contains_task_list {
        properties.push((
            "className".into(),
            hast::PropertyValue::SpaceSeparated(vec!["contains-task-list".into()]),
        ));
    }

    Result::Node(hast::Node::Element(hast::Element {
        tag_name: if list.ordered {
            "ol".into()
        } else {
            "ul".into()
        },
        properties,
        children: wrap(all(state, node), true),
        position: list.position.clone(),
    }))
}

/// [`Math`][mdast::Math].
fn transform_math(_state: &mut State, _node: &mdast::Node, math: &mdast::Math) -> Result {
    let mut value = math.value.clone();
    value.push('\n');

    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "pre".into(),
        properties: vec![],
        children: vec![hast::Node::Element(hast::Element {
            tag_name: "code".into(),
            properties: vec![(
                "className".into(),
                hast::PropertyValue::SpaceSeparated(vec![
                    "language-math".into(),
                    "math-display".into(),
                ]),
            )],
            children: vec![hast::Node::Text(hast::Text {
                value: value.into(),
                position: None,
            })],
            position: math.position.clone(),
        })],
        position: math.position.clone(),
    }))
}

/// [`Paragraph`][mdast::Paragraph].
fn transform_paragraph(
    state: &mut State,
    node: &mdast::Node,
    paragraph: &mdast::Paragraph,
) -> Result {
    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "p".into(),
        properties: vec![],
        children: all(state, node),
        position: paragraph.position.clone(),
    }))
}

/// [`Root`][mdast::Root].
fn transform_root(state: &mut State, node: &mdast::Node, root: &mdast::Root) -> Result {
    Result::Node(hast::Node::Root(hast::Root {
        children: wrap(all(state, node), false),
        position: root.position.clone(),
    }))
}

/// [`Strong`][mdast::Strong].
fn transform_strong(state: &mut State, node: &mdast::Node, strong: &mdast::Strong) -> Result {
    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "strong".into(),
        properties: vec![],
        children: all(state, node),
        position: strong.position.clone(),
    }))
}

/// [`TableCell`][mdast::TableCell].
fn transform_table_cell(
    state: &mut State,
    node: &mdast::Node,
    head: bool,
    align: mdast::AlignKind,
    table_cell: &mdast::TableCell,
) -> Result {
    let align_value = match align {
        mdast::AlignKind::None => None,
        mdast::AlignKind::Left => Some("left"),
        mdast::AlignKind::Right => Some("right"),
        mdast::AlignKind::Center => Some("center"),
    };

    let mut properties = vec![];

    if let Some(value) = align_value {
        properties.push(("align".into(), hast::PropertyValue::String(value.into())));
    }

    Result::Node(hast::Node::Element(hast::Element {
        tag_name: if head { "th".into() } else { "td".into() },
        properties,
        children: all(state, node),
        position: table_cell.position.clone(),
    }))
}

/// [`TableRow`][mdast::TableRow].
fn transform_table_row(
    state: &mut State,
    _node: &mdast::Node,
    head: bool,
    align: Option<&[mdast::AlignKind]>,
    table_row: &mdast::TableRow,
) -> Result {
    let mut children = vec![];
    let mut index = 0;
    let len = align.map_or(table_row.children.len(), |d| d.len());
    let empty_cell = mdast::Node::TableCell(mdast::TableCell {
        children: vec![],
        position: None,
    });

    while index < len {
        let align_value = align
            .and_then(|d| d.get(index))
            .unwrap_or(&mdast::AlignKind::None);

        let child = table_row.children.get(index).unwrap_or(&empty_cell);

        let result = if let mdast::Node::TableCell(table_cell) = child {
            transform_table_cell(state, child, head, *align_value, table_cell)
        } else {
            unreachable!("expected table cell in table row")
        };

        append_result(&mut children, result);
        index += 1;
    }

    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "tr".into(),
        properties: vec![],
        children: wrap(children, true),
        position: table_row.position.clone(),
    }))
}

/// [`Table`][mdast::Table].
fn transform_table(state: &mut State, _node: &mdast::Node, table: &mdast::Table) -> Result {
    let mut rows = vec![];
    let mut index = 0;

    while index < table.children.len() {
        let child = &table.children[index];
        let result = if let mdast::Node::TableRow(table_row) = child {
            transform_table_row(
                state,
                &table.children[index],
                index == 0,
                Some(&table.align),
                table_row,
            )
        } else {
            unreachable!("expected table row as child of table")
        };

        append_result(&mut rows, result);
        index += 1;
    }

    // A table without rows has no head row either.
    let body_rows = rows.split_off(rows.len().min(1));
    let head_row = rows.pop();
    let mut children = vec![];

    if let Some(row) = head_row {
        let position = row.position().cloned();
        children.push(hast::Node::Element(hast::Element {
            tag_name: "thead".into(),
            properties: vec![],
            children: wrap(vec![row], true),
            position,
        }));
    }

    if !body_rows.is_empty() {
        let mut position = None;

        if let Some(position_start) = body_rows.first().and_then(hast::Node::position) {
            if let Some(position_end) = body_rows.last().and_then(hast::Node::position) {
                position = Some(Position {
                    start: position_start.start.clone(),
                    end: position_end.end.clone(),
                });
            }
        }

        children.push(hast::Node::Element(hast::Element {
            tag_name: "tbody".into(),
            properties: vec![],
            children: wrap(body_rows, true),
            position,
        }));
    }

    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "table".into(),
        properties: vec![],
        children: wrap(children, true),
        position: table.position.clone(),
    }))
}

/// [`Text`][mdast::Text].
fn transform_text(_state: &mut State, _node: &mdast::Node, text: &mdast::Text) -> Result {
    Result::Node(hast::Node::Text(hast::Text {
        value: text.value.clone().into(),
        position: text.position.clone(),
    }))
}

/// [`ThematicBreak`][mdast::ThematicBreak].
fn transform_thematic_break(
    _state: &mut State,
    _node: &mdast::Node,
    thematic_break: &mdast::ThematicBreak,
) -> Result {
    Result::Node(hast::Node::Element(hast::Element {
        tag_name: "hr".into(),
        properties: vec![],
        children: vec![],
        position: thematic_break.position.clone(),
    }))
}

/// Transform children of `parent`.
fn all(state: &mut State, parent: &mdast::Node) -> Vec<hast::Node> {
    let mut nodes = vec![];
    if let Some(children) = parent.children() {
        let mut index = 0;
        while index < children.len() {
            let child = &children[index];
            let result = one(state, child, Some(parent));
            append_result(&mut nodes, result);
            index += 1;
        }
    }

    nodes
}

/// Wrap `nodes` with line feeds between each entry.
/// Optionally adds line feeds at the start and end.
fn wrap(mut nodes: Vec<hast::Node>, loose: bool) -> Vec<hast::Node> {
    let mut result = vec![];
    let was_empty = nodes.is_empty();
    let mut head = true;

    nodes.reverse();

    if loose {
        result.push(hast::Node::Text(hast::Text {
            value: "\n".into(),
            position: None,
        }));
    }

    while let Some(item) = nodes.pop() {
        // Inject when there’s more:
        if !head {
            result.push(hast::Node::Text(hast::Text {
                value: "\n".into(),
                position: None,
            }));
        }
        head = false;
        result.push(item);
    }

    if loose && !was_empty {
        result.push(hast::Node::Text(hast::Text {
            value: "\n".into(),
            position: None,
        }));
    }

    result
}

/// Visit.
fn visit<Visitor>(node: &mdast::Node, visitor: Visitor)
where
    Visitor: FnMut(&mdast::Node),
{
    visit_impl(node, visitor);
}

/// Internal implementation to visit.
fn visit_impl<Visitor>(node: &mdast::Node, mut visitor: Visitor) -> Visitor
where
    Visitor: FnMut(&mdast::Node),
{
    visitor(node);

    if let Some(children) = node.children() {
        let mut index = 0;
        while index < children.len() {
            let child = &children[index];
            visitor = visit_impl(child, visitor);
            index += 1;
        }
    }

    visitor
}

// To do: trim around breaks: <https://github.com/syntax-tree/mdast-util-to-hast/blob/c393d0a/lib/traverse.js>.
/// Append an optional, variadic result.
fn append_result(list: &mut Vec<hast::Node>, result: Result) {
    match result {
        Result::Fragment(mut fragment) => list.append(&mut fragment),
        Result::Node(node) => list.push(node),
        Result::None => {}
    }
}

/// Replace line endings (CR, LF, CRLF) with spaces.
///
/// Used for inline code and inline math.
fn replace_eols_with_spaces(value: &str) -> String {
    // It’ll grow a bit smaller for each CR+LF.
    let mut result = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    let mut start = 0;

    while index < bytes.len() {
        let byte = bytes[index];

        if byte == b'\r' || byte == b'\n' {
            result.push_str(&value[start..index]);
            result.push(' ');

            if index + 1 < bytes.len() && byte == b'\r' && bytes[index + 1] == b'\n' {
                index += 1;
            }

            start = index + 1;
        }

        index += 1;
    }

    result.push_str(&value[start..]);

    result
}

/// Check if a list is loose.
fn list_loose(node: &mdast::Node) -> bool {
    if let mdast::Node::List(list) = node {
        if list.spread {
            return true;
        }

        if let Some(children) = node.children() {
            let mut index = 0;
            while index < children.len() {
                if list_item_loose(&children[index]) {
                    return true;
                }
                index += 1;
            }
        }
    }

    false
}

/// Check if a list item is loose.
fn list_item_loose(node: &mdast::Node) -> bool {
    if let mdast::Node::ListItem(item) = node {
        item.spread
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;
    use pretty_assertions::assert_eq;

    #[test]
    fn blockquote() {
        assert_eq!(
            to_hast(&mdast::Node::Blockquote(mdast::Blockquote {
                children: vec![],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "blockquote".into(),
                properties: vec![],
                children: vec![hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                })],
                position: None
            }),
            "should support a `Blockquote`",
        );
    }

    #[test]
    fn br() {
        assert_eq!(
            to_hast(&mdast::Node::Break(mdast::Break { position: None })),
            hast::Node::Root(hast::Root {
                children: vec![
                    hast::Node::Element(hast::Element {
                        tag_name: "br".into(),
                        properties: vec![],
                        children: vec![],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    })
                ],
                position: None
            }),
            "should support a `Break`",
        );
    }

    #[test]
    fn code() {
        assert_eq!(
            to_hast(&mdast::Node::Code(mdast::Code {
                lang: Some("b".into()),
                meta: None,
                value: "a".into(),
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "pre".into(),
                properties: vec![],
                children: vec![hast::Node::Element(hast::Element {
                    tag_name: "code".into(),
                    properties: vec![(
                        "className".into(),
                        hast::PropertyValue::SpaceSeparated(vec!["language-b".into()]),
                    ),],
                    children: vec![hast::Node::Text(hast::Text {
                        value: "a\n".into(),
                        position: None
                    })],
                    position: None
                })],
                position: None
            }),
            "should support a `Code`",
        );
    }

    #[test]
    fn custom() {
        assert_eq!(
            to_hast(&mdast::Node::Custom(mdast::Custom {
                value: Some("a".into()),
                children: vec![mdast::Node::FootnoteReference(mdast::FootnoteReference {
                    position: None,
                    identifier: "b".into(),
                    label: None,
                })],
                position: Some(Position::new(1, 1, 0, 1, 2, 1)),
                name: "c".into(),
                ..mdast::Custom::default()
            })),
            hast::Node::Text(hast::Text {
                value: "a".into(),
                position: Some(Position::new(1, 1, 0, 1, 2, 1))
            }),
            "should support a `Custom` with a value, ignoring its children",
        );

        let custom = mdast::Node::Custom(mdast::Custom {
            children: vec![mdast::Node::Text(mdast::Text {
                value: "a".into(),
                position: None,
            })],
            position: Some(Position::new(1, 1, 0, 1, 2, 1)),
            name: "b".into(),
            fields: vec![("c".into(), "d".into())].into_iter().collect(),
            ..mdast::Custom::default()
        });

        assert_eq!(
            to_hast(&custom),
            hast::Node::Element(hast::Element {
                tag_name: "div".into(),
                properties: vec![],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: Some(Position::new(1, 1, 0, 1, 2, 1))
            }),
            "should support a `Custom` with children",
        );

        let mut handlers = Handlers::new();
        handlers.insert(
            "b".into(),
            Box::new(|custom: &mdast::Custom, children: Vec<hast::Node>| {
                vec![hast::Node::Element(hast::Element {
                    tag_name: "span".into(),
                    properties: vec![(
                        "title".into(),
                        hast::PropertyValue::String(custom.fields["c"].clone()),
                    )],
                    children,
                    position: None,
                })]
            }),
        );
        let options = Options { handlers };

        assert_eq!(
            format!("{:?}", options),
            "Options { handlers: [\"b\"] }",
            "should support `Debug` of options with handlers",
        );
        let expected = hast::Node::Root(hast::Root {
            children: vec![hast::Node::Element(hast::Element {
                tag_name: "span".into(),
                properties: vec![("title".into(), hast::PropertyValue::String("d".into()))],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a".into(),
                    position: None,
                })],
                position: None,
            })],
            position: None,
        });

        assert_eq!(
            to_hast_with_options(&custom, &options),
            expected,
            "should support a handler for a `Custom`",
        );

        let mut literal = custom;
        if let mdast::Node::Custom(node) = &mut literal {
            node.value = Some("e".into());
        }
        assert_eq!(
            to_hast_with_options(&literal, &options),
            expected,
            "should prefer a handler to a value",
        );
    }

    #[test]
    fn definition() {
        assert_eq!(
            to_hast(&mdast::Node::Definition(mdast::Definition {
                url: "b".into(),
                title: None,
                identifier: "a".into(),
                label: None,
                position: None
            })),
            hast::Node::Root(hast::Root {
                children: vec![],
                position: None
            }),
            "should support a `Definition`",
        );
    }

    #[test]
    fn delete() {
        assert_eq!(
            to_hast(&mdast::Node::Delete(mdast::Delete {
                children: vec![mdast::Node::Text(mdast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "del".into(),
                properties: vec![],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: None
            }),
            "should support a `Delete`",
        );
    }

    #[test]
    fn emphasis() {
        assert_eq!(
            to_hast(&mdast::Node::Emphasis(mdast::Emphasis {
                children: vec![mdast::Node::Text(mdast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "em".into(),
                properties: vec![],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: None
            }),
            "should support an `Emphasis`",
        );
    }

    #[test]
    fn footnote_definition() {
        assert_eq!(
            to_hast(&mdast::Node::FootnoteDefinition(
                mdast::FootnoteDefinition {
                    identifier: "a".into(),
                    label: None,
                    children: vec![],
                    position: None
                }
            )),
            hast::Node::Root(hast::Root {
                children: vec![],
                position: None
            }),
            "should support a `FootnoteDefinition`",
        );
    }

    #[test]
    fn footnote_reference() {
        assert_eq!(
            to_hast(&mdast::Node::Root(mdast::Root {
                children: vec![
                    mdast::Node::FootnoteDefinition(mdast::FootnoteDefinition {
                        children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                            children: vec![mdast::Node::Text(mdast::Text {
                                value: "b".into(),
                                position: None
                            })],
                            position: None
                        }),],
                        identifier: "a".into(),
                        label: None,
                        position: None
                    }),
                    mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![mdast::Node::FootnoteReference(mdast::FootnoteReference {
                            identifier: "a".into(),
                            label: None,
                            position: None,
                        })],
                        position: None
                    }),
                ],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![
                    // Main.
                    hast::Node::Element(hast::Element {
                        tag_name: "p".into(),
                        properties: vec![],
                        children: vec![hast::Node::Element(hast::Element {
                            tag_name: "sup".into(),
                            properties: vec![],
                            children: vec![hast::Node::Element(hast::Element {
                                tag_name: "a".into(),
                                properties: vec![
                                    (
                                        "href".into(),
                                        hast::PropertyValue::String("#user-content-fn-a".into()),
                                    ),
                                    (
                                        "id".into(),
                                        hast::PropertyValue::String("user-content-fnref-a".into()),
                                    ),
                                    ("dataFootnoteRef".into(), hast::PropertyValue::Boolean(true),),
                                    (
                                        "ariaDescribedBy".into(),
                                        hast::PropertyValue::String("footnote-label".into()),
                                    )
                                ],
                                children: vec![hast::Node::Text(hast::Text {
                                    value: "1".into(),
                                    position: None
                                })],
                                position: None
                            }),],
                            position: None
                        }),],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    // Footer.
                    hast::Node::Element(hast::Element {
                        tag_name: "section".into(),
                        properties: vec![
                            ("dataFootnotes".into(), hast::PropertyValue::Boolean(true),),
                            (
                                "className".into(),
                                hast::PropertyValue::SpaceSeparated(vec!["footnotes".into()]),
                            ),
                        ],
                        children: vec![
                            hast::Node::Element(hast::Element {
                                tag_name: "h2".into(),
                                properties: vec![
                                    (
                                        "id".into(),
                                        hast::PropertyValue::String("footnote-label".into()),
                                    ),
                                    (
                                        "className".into(),
                                        hast::PropertyValue::SpaceSeparated(
                                            vec!["sr-only".into(),]
                                        ),
                                    ),
                                ],
                                children: vec![hast::Node::Text(hast::Text {
                                    value: "Footnotes".into(),
                                    position: None
                                }),],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                            hast::Node::Element(hast::Element {
                                tag_name: "ol".into(),
                                properties: vec![],
                                children: vec![
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "li".into(),
                                        properties: vec![(
                                            "id".into(),
                                            hast::PropertyValue::String("user-content-fn-a".into()),
                                        )],
                                        children: vec![
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                            hast::Node::Element(hast::Element {
                                                tag_name: "p".into(),
                                                properties: vec![],
                                                children: vec![
                                                    hast::Node::Text(hast::Text {
                                                        value: "b ".into(),
                                                        position: None
                                                    }),
                                                    hast::Node::Element(hast::Element {
                                                        tag_name: "a".into(),
                                                        properties: vec![
                                                            (
                                                                "href".into(),
                                                                hast::PropertyValue::String(
                                                                    "#user-content-fnref-a".into()
                                                                ),
                                                            ),
                                                            (
                                                                "dataFootnoteBackref".into(),
                                                                hast::PropertyValue::Boolean(true),
                                                            ),
                                                            (
                                                                "ariaLabel".into(),
                                                                hast::PropertyValue::String(
                                                                    "Back to content".into()
                                                                ),
                                                            ),
                                                            (
                                                                "className".into(),
                                                                hast::PropertyValue::SpaceSeparated(
                                                                    vec!["data-footnote-backref"
                                                                        .into()]
                                                                ),
                                                            )
                                                        ],
                                                        children: vec![hast::Node::Text(
                                                            hast::Text {
                                                                value: "↩".into(),
                                                                position: None
                                                            }
                                                        ),],
                                                        position: None
                                                    })
                                                ],
                                                position: None
                                            }),
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                        ],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                ],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                        ],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                ],
                position: None
            }),
            "should support a `FootnoteReference`",
        );

        assert_eq!(
            to_hast(&mdast::Node::Root(mdast::Root {
                children: vec![
                    mdast::Node::FootnoteDefinition(mdast::FootnoteDefinition {
                        children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                            children: vec![mdast::Node::Text(mdast::Text {
                                value: "b".into(),
                                position: None
                            })],
                            position: None
                        }),],
                        identifier: "a".into(),
                        label: None,
                        position: None
                    }),
                    mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![
                            mdast::Node::FootnoteReference(mdast::FootnoteReference {
                                identifier: "a".into(),
                                label: None,
                                position: None,
                            }),
                            mdast::Node::FootnoteReference(mdast::FootnoteReference {
                                identifier: "a".into(),
                                label: None,
                                position: None,
                            })
                        ],
                        position: None
                    }),
                ],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![
                    // Main.
                    hast::Node::Element(hast::Element {
                        tag_name: "p".into(),
                        properties: vec![],
                        children: vec![
                            hast::Node::Element(hast::Element {
                                tag_name: "sup".into(),
                                properties: vec![],
                                children: vec![hast::Node::Element(hast::Element {
                                    tag_name: "a".into(),
                                    properties: vec![
                                        (
                                            "href".into(),
                                            hast::PropertyValue::String(
                                                "#user-content-fn-a".into()
                                            ),
                                        ),
                                        (
                                            "id".into(),
                                            hast::PropertyValue::String(
                                                "user-content-fnref-a".into()
                                            ),
                                        ),
                                        (
                                            "dataFootnoteRef".into(),
                                            hast::PropertyValue::Boolean(true),
                                        ),
                                        (
                                            "ariaDescribedBy".into(),
                                            hast::PropertyValue::String("footnote-label".into()),
                                        )
                                    ],
                                    children: vec![hast::Node::Text(hast::Text {
                                        value: "1".into(),
                                        position: None
                                    })],
                                    position: None
                                }),],
                                position: None
                            }),
                            hast::Node::Element(hast::Element {
                                tag_name: "sup".into(),
                                properties: vec![],
                                children: vec![hast::Node::Element(hast::Element {
                                    tag_name: "a".into(),
                                    properties: vec![
                                        (
                                            "href".into(),
                                            hast::PropertyValue::String(
                                                "#user-content-fn-a".into()
                                            ),
                                        ),
                                        (
                                            "id".into(),
                                            hast::PropertyValue::String(
                                                "user-content-fnref-a-2".into()
                                            ),
                                        ),
                                        (
                                            "dataFootnoteRef".into(),
                                            hast::PropertyValue::Boolean(true),
                                        ),
                                        (
                                            "ariaDescribedBy".into(),
                                            hast::PropertyValue::String("footnote-label".into()),
                                        )
                                    ],
                                    children: vec![hast::Node::Text(hast::Text {
                                        value: "1".into(),
                                        position: None
                                    })],
                                    position: None
                                }),],
                                position: None
                            }),
                        ],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    // Footer.
                    hast::Node::Element(hast::Element {
                        tag_name: "section".into(),
                        properties: vec![
                            ("dataFootnotes".into(), hast::PropertyValue::Boolean(true),),
                            (
                                "className".into(),
                                hast::PropertyValue::SpaceSeparated(vec!["footnotes".into()]),
                            ),
                        ],
                        children: vec![
                            hast::Node::Element(hast::Element {
                                tag_name: "h2".into(),
                                properties: vec![
                                    (
                                        "id".into(),
                                        hast::PropertyValue::String("footnote-label".into()),
                                    ),
                                    (
                                        "className".into(),
                                        hast::PropertyValue::SpaceSeparated(
                                            vec!["sr-only".into(),]
                                        ),
                                    ),
                                ],
                                children: vec![hast::Node::Text(hast::Text {
                                    value: "Footnotes".into(),
                                    position: None
                                }),],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                            hast::Node::Element(hast::Element {
                                tag_name: "ol".into(),
                                properties: vec![],
                                children: vec![
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "li".into(),
                                        properties: vec![(
                                            "id".into(),
                                            hast::PropertyValue::String("user-content-fn-a".into()),
                                        )],
                                        children: vec![
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                            hast::Node::Element(hast::Element {
                                                tag_name: "p".into(),
                                                properties: vec![],
                                                children: vec![
                                                    hast::Node::Text(hast::Text {
                                                        value: "b ".into(),
                                                        position: None
                                                    }),
                                                    hast::Node::Element(hast::Element {
                                                        tag_name: "a".into(),
                                                        properties: vec![
                                                            (
                                                                "href".into(),
                                                                hast::PropertyValue::String(
                                                                    "#user-content-fnref-a".into()
                                                                ),
                                                            ),
                                                            (
                                                                "dataFootnoteBackref".into(),
                                                                hast::PropertyValue::Boolean(true),
                                                            ),
                                                            (
                                                                "ariaLabel".into(),
                                                                hast::PropertyValue::String(
                                                                    "Back to content".into()
                                                                ),
                                                            ),
                                                            (
                                                                "className".into(),
                                                                hast::PropertyValue::SpaceSeparated(
                                                                    vec!["data-footnote-backref"
                                                                        .into()]
                                                                ),
                                                            )
                                                        ],
                                                        children: vec![hast::Node::Text(
                                                            hast::Text {
                                                                value: "↩".into(),
                                                                position: None
                                                            }
                                                        ),],
                                                        position: None
                                                    }),
                                                    hast::Node::Text(hast::Text {
                                                        value: " ".into(),
                                                        position: None
                                                    }),
                                                    hast::Node::Element(hast::Element {
                                                        tag_name: "a".into(),
                                                        properties: vec![
                                                            (
                                                                "href".into(),
                                                                hast::PropertyValue::String(
                                                                    "#user-content-fnref-a-2"
                                                                        .into()
                                                                ),
                                                            ),
                                                            (
                                                                "dataFootnoteBackref".into(),
                                                                hast::PropertyValue::Boolean(true),
                                                            ),
                                                            (
                                                                "ariaLabel".into(),
                                                                hast::PropertyValue::String(
                                                                    "Back to content".into()
                                                                ),
                                                            ),
                                                            (
                                                                "className".into(),
                                                                hast::PropertyValue::SpaceSeparated(
                                                                    vec!["data-footnote-backref"
                                                                        .into()]
                                                                ),
                                                            )
                                                        ],
                                                        children: vec![
                                                            hast::Node::Text(hast::Text {
                                                                value: "↩".into(),
                                                                position: None
                                                            }),
                                                            hast::Node::Element(hast::Element {
                                                                tag_name: "sup".into(),
                                                                properties: vec![],
                                                                children: vec![hast::Node::Text(
                                                                    hast::Text {
                                                                        value: "2".into(),
                                                                        position: None
                                                                    }
                                                                ),],
                                                                position: None
                                                            })
                                                        ],
                                                        position: None
                                                    })
                                                ],
                                                position: None
                                            }),
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                        ],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                ],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                        ],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                ],
                position: None
            }),
            "should support a `FootnoteReference` (multiple calls to the same definition)",
        );

        assert_eq!(
            to_hast(&mdast::Node::Root(mdast::Root {
                children: vec![
                    mdast::Node::FootnoteDefinition(mdast::FootnoteDefinition {
                        children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                            children: vec![mdast::Node::Text(mdast::Text {
                                value: "b".into(),
                                position: None
                            })],
                            position: None
                        }),],
                        identifier: "a".into(),
                        label: None,
                        position: None
                    }),
                    mdast::Node::FootnoteDefinition(mdast::FootnoteDefinition {
                        children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                            children: vec![mdast::Node::Text(mdast::Text {
                                value: "d".into(),
                                position: None
                            })],
                            position: None
                        }),],
                        identifier: "c".into(),
                        label: None,
                        position: None
                    }),
                    mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![
                            mdast::Node::FootnoteReference(mdast::FootnoteReference {
                                identifier: "a".into(),
                                label: None,
                                position: None,
                            }),
                            mdast::Node::Text(mdast::Text {
                                value: " and ".into(),
                                position: None,
                            }),
                            mdast::Node::FootnoteReference(mdast::FootnoteReference {
                                identifier: "c".into(),
                                label: None,
                                position: None,
                            })
                        ],
                        position: None
                    }),
                ],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![
                    // Main.
                    hast::Node::Element(hast::Element {
                        tag_name: "p".into(),
                        properties: vec![],
                        children: vec![
                            hast::Node::Element(hast::Element {
                                tag_name: "sup".into(),
                                properties: vec![],
                                children: vec![hast::Node::Element(hast::Element {
                                    tag_name: "a".into(),
                                    properties: vec![
                                        (
                                            "href".into(),
                                            hast::PropertyValue::String(
                                                "#user-content-fn-a".into()
                                            ),
                                        ),
                                        (
                                            "id".into(),
                                            hast::PropertyValue::String(
                                                "user-content-fnref-a".into()
                                            ),
                                        ),
                                        (
                                            "dataFootnoteRef".into(),
                                            hast::PropertyValue::Boolean(true),
                                        ),
                                        (
                                            "ariaDescribedBy".into(),
                                            hast::PropertyValue::String("footnote-label".into()),
                                        )
                                    ],
                                    children: vec![hast::Node::Text(hast::Text {
                                        value: "1".into(),
                                        position: None
                                    })],
                                    position: None
                                }),],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: " and ".into(),
                                position: None,
                            }),
                            hast::Node::Element(hast::Element {
                                tag_name: "sup".into(),
                                properties: vec![],
                                children: vec![hast::Node::Element(hast::Element {
                                    tag_name: "a".into(),
                                    properties: vec![
                                        (
                                            "href".into(),
                                            hast::PropertyValue::String(
                                                "#user-content-fn-c".into()
                                            ),
                                        ),
                                        (
                                            "id".into(),
                                            hast::PropertyValue::String(
                                                "user-content-fnref-c".into()
                                            ),
                                        ),
                                        (
                                            "dataFootnoteRef".into(),
                                            hast::PropertyValue::Boolean(true),
                                        ),
                                        (
                                            "ariaDescribedBy".into(),
                                            hast::PropertyValue::String("footnote-label".into()),
                                        )
                                    ],
                                    children: vec![hast::Node::Text(hast::Text {
                                        value: "2".into(),
                                        position: None
                                    })],
                                    position: None
                                }),],
                                position: None
                            }),
                        ],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    // Footer.
                    hast::Node::Element(hast::Element {
                        tag_name: "section".into(),
                        properties: vec![
                            ("dataFootnotes".into(), hast::PropertyValue::Boolean(true),),
                            (
                                "className".into(),
                                hast::PropertyValue::SpaceSeparated(vec!["footnotes".into()]),
                            ),
                        ],
                        children: vec![
                            hast::Node::Element(hast::Element {
                                tag_name: "h2".into(),
                                properties: vec![
                                    (
                                        "id".into(),
                                        hast::PropertyValue::String("footnote-label".into()),
                                    ),
                                    (
                                        "className".into(),
                                        hast::PropertyValue::SpaceSeparated(
                                            vec!["sr-only".into(),]
                                        ),
                                    ),
                                ],
                                children: vec![hast::Node::Text(hast::Text {
                                    value: "Footnotes".into(),
                                    position: None
                                }),],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                            hast::Node::Element(hast::Element {
                                tag_name: "ol".into(),
                                properties: vec![],
                                children: vec![
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "li".into(),
                                        properties: vec![(
                                            "id".into(),
                                            hast::PropertyValue::String("user-content-fn-a".into()),
                                        )],
                                        children: vec![
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                            hast::Node::Element(hast::Element {
                                                tag_name: "p".into(),
                                                properties: vec![],
                                                children: vec![
                                                    hast::Node::Text(hast::Text {
                                                        value: "b ".into(),
                                                        position: None
                                                    }),
                                                    hast::Node::Element(hast::Element {
                                                        tag_name: "a".into(),
                                                        properties: vec![
                                                            (
                                                                "href".into(),
                                                                hast::PropertyValue::String(
                                                                    "#user-content-fnref-a".into()
                                                                ),
                                                            ),
                                                            (
                                                                "dataFootnoteBackref".into(),
                                                                hast::PropertyValue::Boolean(true),
                                                            ),
                                                            (
                                                                "ariaLabel".into(),
                                                                hast::PropertyValue::String(
                                                                    "Back to content".into()
                                                                ),
                                                            ),
                                                            (
                                                                "className".into(),
                                                                hast::PropertyValue::SpaceSeparated(
                                                                    vec!["data-footnote-backref"
                                                                        .into()]
                                                                ),
                                                            )
                                                        ],
                                                        children: vec![hast::Node::Text(
                                                            hast::Text {
                                                                value: "↩".into(),
                                                                position: None
                                                            }
                                                        ),],
                                                        position: None
                                                    })
                                                ],
                                                position: None
                                            }),
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                        ],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "li".into(),
                                        properties: vec![(
                                            "id".into(),
                                            hast::PropertyValue::String("user-content-fn-c".into()),
                                        )],
                                        children: vec![
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                            hast::Node::Element(hast::Element {
                                                tag_name: "p".into(),
                                                properties: vec![],
                                                children: vec![
                                                    hast::Node::Text(hast::Text {
                                                        value: "d ".into(),
                                                        position: None
                                                    }),
                                                    hast::Node::Element(hast::Element {
                                                        tag_name: "a".into(),
                                                        properties: vec![
                                                            (
                                                                "href".into(),
                                                                hast::PropertyValue::String(
                                                                    "#user-content-fnref-c".into()
                                                                ),
                                                            ),
                                                            (
                                                                "dataFootnoteBackref".into(),
                                                                hast::PropertyValue::Boolean(true),
                                                            ),
                                                            (
                                                                "ariaLabel".into(),
                                                                hast::PropertyValue::String(
                                                                    "Back to content".into()
                                                                ),
                                                            ),
                                                            (
                                                                "className".into(),
                                                                hast::PropertyValue::SpaceSeparated(
                                                                    vec!["data-footnote-backref"
                                                                        .into()]
                                                                ),
                                                            )
                                                        ],
                                                        children: vec![hast::Node::Text(
                                                            hast::Text {
                                                                value: "↩".into(),
                                                                position: None
                                                            }
                                                        ),],
                                                        position: None
                                                    })
                                                ],
                                                position: None
                                            }),
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                        ],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                ],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                        ],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                ],
                position: None
            }),
            "should support a `FootnoteReference` (different definitions)",
        );

        assert_eq!(
            to_hast(&mdast::Node::Root(mdast::Root {
                children: vec![
                    mdast::Node::FootnoteDefinition(mdast::FootnoteDefinition {
                        children: vec![mdast::Node::Heading(mdast::Heading {
                            depth: 1,
                            children: vec![mdast::Node::Text(mdast::Text {
                                value: "b".into(),
                                position: None
                            })],
                            position: None
                        }),],
                        identifier: "a".into(),
                        label: None,
                        position: None
                    }),
                    mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![mdast::Node::FootnoteReference(mdast::FootnoteReference {
                            identifier: "a".into(),
                            label: None,
                            position: None,
                        })],
                        position: None
                    }),
                ],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![
                    // Main.
                    hast::Node::Element(hast::Element {
                        tag_name: "p".into(),
                        properties: vec![],
                        children: vec![hast::Node::Element(hast::Element {
                            tag_name: "sup".into(),
                            properties: vec![],
                            children: vec![hast::Node::Element(hast::Element {
                                tag_name: "a".into(),
                                properties: vec![
                                    (
                                        "href".into(),
                                        hast::PropertyValue::String("#user-content-fn-a".into()),
                                    ),
                                    (
                                        "id".into(),
                                        hast::PropertyValue::String("user-content-fnref-a".into()),
                                    ),
                                    ("dataFootnoteRef".into(), hast::PropertyValue::Boolean(true),),
                                    (
                                        "ariaDescribedBy".into(),
                                        hast::PropertyValue::String("footnote-label".into()),
                                    )
                                ],
                                children: vec![hast::Node::Text(hast::Text {
                                    value: "1".into(),
                                    position: None
                                })],
                                position: None
                            }),],
                            position: None
                        }),],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    // Footer.
                    hast::Node::Element(hast::Element {
                        tag_name: "section".into(),
                        properties: vec![
                            ("dataFootnotes".into(), hast::PropertyValue::Boolean(true),),
                            (
                                "className".into(),
                                hast::PropertyValue::SpaceSeparated(vec!["footnotes".into()]),
                            ),
                        ],
                        children: vec![
                            hast::Node::Element(hast::Element {
                                tag_name: "h2".into(),
                                properties: vec![
                                    (
                                        "id".into(),
                                        hast::PropertyValue::String("footnote-label".into()),
                                    ),
                                    (
                                        "className".into(),
                                        hast::PropertyValue::SpaceSeparated(
                                            vec!["sr-only".into(),]
                                        ),
                                    ),
                                ],
                                children: vec![hast::Node::Text(hast::Text {
                                    value: "Footnotes".into(),
                                    position: None
                                }),],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                            hast::Node::Element(hast::Element {
                                tag_name: "ol".into(),
                                properties: vec![],
                                children: vec![
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "li".into(),
                                        properties: vec![(
                                            "id".into(),
                                            hast::PropertyValue::String("user-content-fn-a".into()),
                                        )],
                                        children: vec![
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                            hast::Node::Element(hast::Element {
                                                tag_name: "h1".into(),
                                                properties: vec![],
                                                children: vec![hast::Node::Text(hast::Text {
                                                    value: "b".into(),
                                                    position: None
                                                }),],
                                                position: None,
                                            }),
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                            hast::Node::Element(hast::Element {
                                                tag_name: "a".into(),
                                                properties: vec![
                                                    (
                                                        "href".into(),
                                                        hast::PropertyValue::String(
                                                            "#user-content-fnref-a".into()
                                                        ),
                                                    ),
                                                    (
                                                        "dataFootnoteBackref".into(),
                                                        hast::PropertyValue::Boolean(true),
                                                    ),
                                                    (
                                                        "ariaLabel".into(),
                                                        hast::PropertyValue::String(
                                                            "Back to content".into()
                                                        ),
                                                    ),
                                                    (
                                                        "className".into(),
                                                        hast::PropertyValue::SpaceSeparated(vec![
                                                            "data-footnote-backref".into()
                                                        ]),
                                                    )
                                                ],
                                                children: vec![hast::Node::Text(hast::Text {
                                                    value: "↩".into(),
                                                    position: None
                                                }),],
                                                position: None
                                            }),
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                        ],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                ],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                        ],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                ],
                position: None
            }),
            "should support a `FootnoteReference` (no paragraph in definition)",
        );

        assert_eq!(
            to_hast(&mdast::Node::Root(mdast::Root {
                children: vec![
                    mdast::Node::FootnoteDefinition(mdast::FootnoteDefinition {
                        children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                            children: vec![mdast::Node::InlineCode(mdast::InlineCode {
                                value: "b".into(),
                                position: None
                            })],
                            position: None
                        }),],
                        identifier: "a".into(),
                        label: None,
                        position: None
                    }),
                    mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![mdast::Node::FootnoteReference(mdast::FootnoteReference {
                            identifier: "a".into(),
                            label: None,
                            position: None,
                        })],
                        position: None
                    }),
                ],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![
                    // Main.
                    hast::Node::Element(hast::Element {
                        tag_name: "p".into(),
                        properties: vec![],
                        children: vec![hast::Node::Element(hast::Element {
                            tag_name: "sup".into(),
                            properties: vec![],
                            children: vec![hast::Node::Element(hast::Element {
                                tag_name: "a".into(),
                                properties: vec![
                                    (
                                        "href".into(),
                                        hast::PropertyValue::String("#user-content-fn-a".into()),
                                    ),
                                    (
                                        "id".into(),
                                        hast::PropertyValue::String("user-content-fnref-a".into()),
                                    ),
                                    ("dataFootnoteRef".into(), hast::PropertyValue::Boolean(true),),
                                    (
                                        "ariaDescribedBy".into(),
                                        hast::PropertyValue::String("footnote-label".into()),
                                    )
                                ],
                                children: vec![hast::Node::Text(hast::Text {
                                    value: "1".into(),
                                    position: None
                                })],
                                position: None
                            }),],
                            position: None
                        }),],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    // Footer.
                    hast::Node::Element(hast::Element {
                        tag_name: "section".into(),
                        properties: vec![
                            ("dataFootnotes".into(), hast::PropertyValue::Boolean(true),),
                            (
                                "className".into(),
                                hast::PropertyValue::SpaceSeparated(vec!["footnotes".into()]),
                            ),
                        ],
                        children: vec![
                            hast::Node::Element(hast::Element {
                                tag_name: "h2".into(),
                                properties: vec![
                                    (
                                        "id".into(),
                                        hast::PropertyValue::String("footnote-label".into()),
                                    ),
                                    (
                                        "className".into(),
                                        hast::PropertyValue::SpaceSeparated(
                                            vec!["sr-only".into(),]
                                        ),
                                    ),
                                ],
                                children: vec![hast::Node::Text(hast::Text {
                                    value: "Footnotes".into(),
                                    position: None
                                }),],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                            hast::Node::Element(hast::Element {
                                tag_name: "ol".into(),
                                properties: vec![],
                                children: vec![
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "li".into(),
                                        properties: vec![(
                                            "id".into(),
                                            hast::PropertyValue::String("user-content-fn-a".into()),
                                        )],
                                        children: vec![
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                            hast::Node::Element(hast::Element {
                                                tag_name: "p".into(),
                                                properties: vec![],
                                                children: vec![
                                                    hast::Node::Element(hast::Element {
                                                        tag_name: "code".into(),
                                                        properties: vec![],
                                                        children: vec![hast::Node::Text(
                                                            hast::Text {
                                                                value: "b".into(),
                                                                position: None
                                                            }
                                                        ),],
                                                        position: None
                                                    }),
                                                    hast::Node::Text(hast::Text {
                                                        value: " ".into(),
                                                        position: None
                                                    }),
                                                    hast::Node::Element(hast::Element {
                                                        tag_name: "a".into(),
                                                        properties: vec![
                                                            (
                                                                "href".into(),
                                                                hast::PropertyValue::String(
                                                                    "#user-content-fnref-a".into()
                                                                ),
                                                            ),
                                                            (
                                                                "dataFootnoteBackref".into(),
                                                                hast::PropertyValue::Boolean(true),
                                                            ),
                                                            (
                                                                "ariaLabel".into(),
                                                                hast::PropertyValue::String(
                                                                    "Back to content".into()
                                                                ),
                                                            ),
                                                            (
                                                                "className".into(),
                                                                hast::PropertyValue::SpaceSeparated(
                                                                    vec!["data-footnote-backref"
                                                                        .into()]
                                                                ),
                                                            )
                                                        ],
                                                        children: vec![hast::Node::Text(
                                                            hast::Text {
                                                                value: "↩".into(),
                                                                position: None
                                                            }
                                                        ),],
                                                        position: None
                                                    })
                                                ],
                                                position: None
                                            }),
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                        ],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                ],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                        ],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                ],
                position: None
            }),
            "should support a `FootnoteReference` (no final text in definition content)",
        );

        assert_eq!(
            to_hast(&mdast::Node::Blockquote(mdast::Blockquote {
                children: vec![
                    mdast::Node::FootnoteDefinition(mdast::FootnoteDefinition {
                        children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                            children: vec![mdast::Node::InlineCode(mdast::InlineCode {
                                value: "b".into(),
                                position: None
                            })],
                            position: None
                        }),],
                        identifier: "a".into(),
                        label: None,
                        position: None
                    }),
                    mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![mdast::Node::FootnoteReference(mdast::FootnoteReference {
                            identifier: "a".into(),
                            label: None,
                            position: None,
                        })],
                        position: None
                    }),
                ],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![
                    // Main.
                    hast::Node::Element(hast::Element {
                        tag_name: "blockquote".into(),
                        properties: vec![],
                        children: vec![
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                            hast::Node::Element(hast::Element {
                                tag_name: "p".into(),
                                properties: vec![],
                                children: vec![hast::Node::Element(hast::Element {
                                    tag_name: "sup".into(),
                                    properties: vec![],
                                    children: vec![hast::Node::Element(hast::Element {
                                        tag_name: "a".into(),
                                        properties: vec![
                                            (
                                                "href".into(),
                                                hast::PropertyValue::String(
                                                    "#user-content-fn-a".into()
                                                ),
                                            ),
                                            (
                                                "id".into(),
                                                hast::PropertyValue::String(
                                                    "user-content-fnref-a".into()
                                                ),
                                            ),
                                            (
                                                "dataFootnoteRef".into(),
                                                hast::PropertyValue::Boolean(true),
                                            ),
                                            (
                                                "ariaDescribedBy".into(),
                                                hast::PropertyValue::String(
                                                    "footnote-label".into()
                                                ),
                                            )
                                        ],
                                        children: vec![hast::Node::Text(hast::Text {
                                            value: "1".into(),
                                            position: None
                                        })],
                                        position: None
                                    }),],
                                    position: None
                                }),],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                        ],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    // Footer.
                    hast::Node::Element(hast::Element {
                        tag_name: "section".into(),
                        properties: vec![
                            ("dataFootnotes".into(), hast::PropertyValue::Boolean(true),),
                            (
                                "className".into(),
                                hast::PropertyValue::SpaceSeparated(vec!["footnotes".into()]),
                            ),
                        ],
                        children: vec![
                            hast::Node::Element(hast::Element {
                                tag_name: "h2".into(),
                                properties: vec![
                                    (
                                        "id".into(),
                                        hast::PropertyValue::String("footnote-label".into()),
                                    ),
                                    (
                                        "className".into(),
                                        hast::PropertyValue::SpaceSeparated(
                                            vec!["sr-only".into(),]
                                        ),
                                    ),
                                ],
                                children: vec![hast::Node::Text(hast::Text {
                                    value: "Footnotes".into(),
                                    position: None
                                }),],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                            hast::Node::Element(hast::Element {
                                tag_name: "ol".into(),
                                properties: vec![],
                                children: vec![
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "li".into(),
                                        properties: vec![(
                                            "id".into(),
                                            hast::PropertyValue::String("user-content-fn-a".into()),
                                        )],
                                        children: vec![
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                            hast::Node::Element(hast::Element {
                                                tag_name: "p".into(),
                                                properties: vec![],
                                                children: vec![
                                                    hast::Node::Element(hast::Element {
                                                        tag_name: "code".into(),
                                                        properties: vec![],
                                                        children: vec![hast::Node::Text(
                                                            hast::Text {
                                                                value: "b".into(),
                                                                position: None
                                                            }
                                                        ),],
                                                        position: None
                                                    }),
                                                    hast::Node::Text(hast::Text {
                                                        value: " ".into(),
                                                        position: None
                                                    }),
                                                    hast::Node::Element(hast::Element {
                                                        tag_name: "a".into(),
                                                        properties: vec![
                                                            (
                                                                "href".into(),
                                                                hast::PropertyValue::String(
                                                                    "#user-content-fnref-a".into()
                                                                ),
                                                            ),
                                                            (
                                                                "dataFootnoteBackref".into(),
                                                                hast::PropertyValue::Boolean(true),
                                                            ),
                                                            (
                                                                "ariaLabel".into(),
                                                                hast::PropertyValue::String(
                                                                    "Back to content".into()
                                                                ),
                                                            ),
                                                            (
                                                                "className".into(),
                                                                hast::PropertyValue::SpaceSeparated(
                                                                    vec!["data-footnote-backref"
                                                                        .into()]
                                                                ),
                                                            )
                                                        ],
                                                        children: vec![hast::Node::Text(
                                                            hast::Text {
                                                                value: "↩".into(),
                                                                position: None
                                                            }
                                                        ),],
                                                        position: None
                                                    })
                                                ],
                                                position: None
                                            }),
                                            hast::Node::Text(hast::Text {
                                                value: "\n".into(),
                                                position: None
                                            }),
                                        ],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                ],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                        ],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                ],
                position: None
            }),
            "should support a `FootnoteReference` (not in a root)",
        );
    }

    #[test]
    fn heading() {
        assert_eq!(
            to_hast(&mdast::Node::Heading(mdast::Heading {
                depth: 1,
                children: vec![mdast::Node::Text(mdast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "h1".into(),
                properties: vec![],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: None
            }),
            "should support a `Heading`",
        );
    }

    #[test]
    fn html() {
        assert_eq!(
            to_hast(&mdast::Node::Html(mdast::Html {
                value: "<div>".into(),
                position: None,
            })),
            hast::Node::Raw(hast::Raw {
                value: "<div>".into(),
                position: None
            }),
            "should support an `Html`",
        );
    }

    #[test]
    fn image() {
        assert_eq!(
            to_hast(&mdast::Node::Image(mdast::Image {
                url: "a".into(),
                alt: "b".into(),
                title: Some("c".into()),
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "img".into(),
                properties: vec![
                    ("src".into(), hast::PropertyValue::String("a".into())),
                    ("alt".into(), hast::PropertyValue::String("b".into())),
                    ("title".into(), hast::PropertyValue::String("c".into()))
                ],
                children: vec![],
                position: None
            }),
            "should support an `Image`",
        );
    }

    #[test]
    fn image_reference() {
        assert_eq!(
            to_hast(&mdast::Node::Root(mdast::Root {
                children: vec![
                    mdast::Node::Definition(mdast::Definition {
                        url: "b".into(),
                        title: Some("c".into()),
                        identifier: "a".into(),
                        label: None,
                        position: None
                    }),
                    mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![mdast::Node::ImageReference(mdast::ImageReference {
                            reference_kind: mdast::ReferenceKind::Full,
                            identifier: "a".into(),
                            alt: "d".into(),
                            label: None,
                            position: None,
                        })],
                        position: None
                    }),
                ],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![hast::Node::Element(hast::Element {
                    tag_name: "p".into(),
                    properties: vec![],
                    children: vec![hast::Node::Element(hast::Element {
                        tag_name: "img".into(),
                        properties: vec![
                            ("src".into(), hast::PropertyValue::String("b".into()),),
                            ("alt".into(), hast::PropertyValue::String("d".into()),),
                            ("title".into(), hast::PropertyValue::String("c".into()),)
                        ],
                        children: vec![],
                        position: None
                    }),],
                    position: None
                }),],
                position: None
            }),
            "should support an `ImageReference`",
        );

        assert_eq!(
            to_hast(&mdast::Node::ImageReference(mdast::ImageReference {
                position: None,
                alt: "b".into(),
                reference_kind: mdast::ReferenceKind::Full,
                identifier: "a".into(),
                label: None,
            })),
            hast::Node::Text(hast::Text {
                value: "![b][a]".into(),
                position: None
            }),
            "should turn an `ImageReference` without definition back into text",
        );
    }

    #[test]
    fn inline_code() {
        assert_eq!(
            to_hast(&mdast::Node::InlineCode(mdast::InlineCode {
                value: "a\nb".into(),
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "code".into(),
                properties: vec![],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a b".into(),
                    position: None
                })],
                position: None
            }),
            "should support an `InlineCode`",
        );
    }

    #[test]
    fn inline_math() {
        assert_eq!(
            to_hast(&mdast::Node::InlineMath(mdast::InlineMath {
                value: "a\nb".into(),
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "code".into(),
                properties: vec![(
                    "className".into(),
                    hast::PropertyValue::SpaceSeparated(vec![
                        "language-math".into(),
                        "math-inline".into()
                    ]),
                ),],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a b".into(),
                    position: None
                })],
                position: None
            }),
            "should support an `InlineMath`",
        );
    }

    #[test]
    fn link() {
        assert_eq!(
            to_hast(&mdast::Node::Link(mdast::Link {
                url: "a".into(),
                title: Some("b".into()),
                children: vec![mdast::Node::Text(mdast::Text {
                    value: "c".into(),
                    position: None
                })],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "a".into(),
                properties: vec![
                    ("href".into(), hast::PropertyValue::String("a".into())),
                    ("title".into(), hast::PropertyValue::String("b".into())),
                ],
                children: vec![hast::Node::Text(hast::Text {
                    value: "c".into(),
                    position: None
                })],
                position: None
            }),
            "should support a `Link`",
        );
    }

    #[test]
    fn link_reference() {
        assert_eq!(
            to_hast(&mdast::Node::Root(mdast::Root {
                children: vec![
                    mdast::Node::Definition(mdast::Definition {
                        url: "b".into(),
                        title: Some("c".into()),
                        identifier: "a".into(),
                        label: None,
                        position: None
                    }),
                    mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![mdast::Node::LinkReference(mdast::LinkReference {
                            reference_kind: mdast::ReferenceKind::Full,
                            identifier: "a".into(),
                            label: None,
                            children: vec![mdast::Node::Text(mdast::Text {
                                value: "c".into(),
                                position: None
                            })],
                            position: None,
                        })],
                        position: None
                    }),
                ],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![hast::Node::Element(hast::Element {
                    tag_name: "p".into(),
                    properties: vec![],
                    children: vec![hast::Node::Element(hast::Element {
                        tag_name: "a".into(),
                        properties: vec![
                            ("href".into(), hast::PropertyValue::String("b".into())),
                            ("title".into(), hast::PropertyValue::String("c".into())),
                        ],
                        children: vec![hast::Node::Text(hast::Text {
                            value: "c".into(),
                            position: None
                        })],
                        position: None
                    }),],
                    position: None
                }),],
                position: None
            }),
            "should support a `LinkReference`",
        );

        for (reference_kind, label, expected) in [
            (mdast::ReferenceKind::Shortcut, None, "[c]"),
            (mdast::ReferenceKind::Collapsed, None, "[c][]"),
            (mdast::ReferenceKind::Full, Some("B"), "[c][B]"),
            (mdast::ReferenceKind::Full, None, "[c][a]"),
            (mdast::ReferenceKind::Full, Some(""), "[c][a]"),
        ] {
            assert_eq!(
                to_hast(&mdast::Node::LinkReference(mdast::LinkReference {
                    reference_kind,
                    identifier: "a".into(),
                    label: label.map(Into::into),
                    children: vec![mdast::Node::Text(mdast::Text {
                        value: "c".into(),
                        position: None
                    })],
                    position: None,
                })),
                hast::Node::Root(hast::Root {
                    children: vec![hast::Node::Text(hast::Text {
                        value: expected.into(),
                        position: None
                    })],
                    position: None
                }),
                "should turn a `LinkReference` without definition back into `{}`",
                expected
            );
        }

        assert_eq!(
            to_hast(&mdast::Node::LinkReference(mdast::LinkReference {
                reference_kind: mdast::ReferenceKind::Shortcut,
                identifier: "a".into(),
                label: None,
                children: vec![mdast::Node::Emphasis(mdast::Emphasis {
                    children: vec![],
                    position: None
                })],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![
                    hast::Node::Text(hast::Text {
                        value: "[".into(),
                        position: None
                    }),
                    hast::Node::Element(hast::Element {
                        tag_name: "em".into(),
                        properties: vec![],
                        children: vec![],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "]".into(),
                        position: None
                    }),
                ],
                position: None
            }),
            "should add text around non-text children of a `LinkReference` without definition",
        );
    }

    #[test]
    fn list_item() {
        assert_eq!(
            to_hast(&mdast::Node::Root(mdast::Root {
                children: vec![mdast::Node::ListItem(mdast::ListItem {
                    spread: false,
                    checked: None,
                    children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![mdast::Node::Text(mdast::Text {
                            value: "a".into(),
                            position: None
                        })],
                        position: None
                    }),],
                    position: None
                }),],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![hast::Node::Element(hast::Element {
                    tag_name: "li".into(),
                    properties: vec![],
                    children: vec![hast::Node::Text(hast::Text {
                        value: "a".into(),
                        position: None
                    }),],
                    position: None
                }),],
                position: None
            }),
            "should support a `ListItem`",
        );

        assert_eq!(
            to_hast(&mdast::Node::Root(mdast::Root {
                children: vec![mdast::Node::ListItem(mdast::ListItem {
                    spread: true,
                    checked: None,
                    children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![mdast::Node::Text(mdast::Text {
                            value: "a".into(),
                            position: None
                        })],
                        position: None
                    }),],
                    position: None
                }),],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![hast::Node::Element(hast::Element {
                    tag_name: "li".into(),
                    properties: vec![],
                    children: vec![
                        hast::Node::Text(hast::Text {
                            value: "\n".into(),
                            position: None
                        }),
                        hast::Node::Element(hast::Element {
                            tag_name: "p".into(),
                            properties: vec![],
                            children: vec![hast::Node::Text(hast::Text {
                                value: "a".into(),
                                position: None
                            }),],
                            position: None
                        }),
                        hast::Node::Text(hast::Text {
                            value: "\n".into(),
                            position: None
                        }),
                    ],
                    position: None
                }),],
                position: None
            }),
            "should support a `ListItem` (spread: true)",
        );

        assert_eq!(
            to_hast(&mdast::Node::Root(mdast::Root {
                children: vec![mdast::Node::ListItem(mdast::ListItem {
                    spread: false,
                    checked: Some(true),
                    children: vec![],
                    position: None
                }),],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![hast::Node::Element(hast::Element {
                    tag_name: "li".into(),
                    properties: vec![(
                        "className".into(),
                        hast::PropertyValue::SpaceSeparated(vec!["task-list-item".into()])
                    )],
                    children: vec![hast::Node::Element(hast::Element {
                        tag_name: "input".into(),
                        properties: vec![
                            (
                                "type".into(),
                                hast::PropertyValue::String("checkbox".into()),
                            ),
                            ("checked".into(), hast::PropertyValue::Boolean(true)),
                            ("disabled".into(), hast::PropertyValue::Boolean(true)),
                        ],
                        children: vec![],
                        position: None
                    }),],
                    position: None
                }),],
                position: None
            }),
            "should support a `ListItem` (checked, w/o paragraph)",
        );

        assert_eq!(
            to_hast(&mdast::Node::Root(mdast::Root {
                children: vec![mdast::Node::ListItem(mdast::ListItem {
                    spread: false,
                    checked: Some(false),
                    children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![mdast::Node::Text(mdast::Text {
                            value: "a".into(),
                            position: None
                        })],
                        position: None
                    }),],
                    position: None
                }),],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![hast::Node::Element(hast::Element {
                    tag_name: "li".into(),
                    properties: vec![(
                        "className".into(),
                        hast::PropertyValue::SpaceSeparated(vec!["task-list-item".into()])
                    )],
                    children: vec![
                        hast::Node::Element(hast::Element {
                            tag_name: "input".into(),
                            properties: vec![
                                (
                                    "type".into(),
                                    hast::PropertyValue::String("checkbox".into()),
                                ),
                                ("checked".into(), hast::PropertyValue::Boolean(false)),
                                ("disabled".into(), hast::PropertyValue::Boolean(true)),
                            ],
                            children: vec![],
                            position: None
                        }),
                        hast::Node::Text(hast::Text {
                            value: " ".into(),
                            position: None
                        }),
                        hast::Node::Text(hast::Text {
                            value: "a".into(),
                            position: None
                        }),
                    ],
                    position: None
                }),],
                position: None
            }),
            "should support a `ListItem` (unchecked, w/ paragraph)",
        );
    }

    #[test]
    fn list() {
        assert_eq!(
            to_hast(&mdast::Node::List(mdast::List {
                ordered: true,
                start: Some(1),
                spread: false,
                children: vec![mdast::Node::ListItem(mdast::ListItem {
                    spread: false,
                    checked: None,
                    children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![mdast::Node::Text(mdast::Text {
                            value: "a".into(),
                            position: None
                        })],
                        position: None
                    }),],
                    position: None
                }),],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "ol".into(),
                properties: vec![],
                children: vec![
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    hast::Node::Element(hast::Element {
                        tag_name: "li".into(),
                        properties: vec![],
                        children: vec![hast::Node::Text(hast::Text {
                            value: "a".into(),
                            position: None
                        }),],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                ],
                position: None
            }),
            "should support a `List` (ordered, start: 1)",
        );

        assert_eq!(
            to_hast(&mdast::Node::List(mdast::List {
                ordered: true,
                start: Some(123),
                spread: false,
                children: vec![],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "ol".into(),
                properties: vec![("start".into(), hast::PropertyValue::String("123".into()),),],
                children: vec![hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                })],
                position: None
            }),
            "should support a `List` (ordered, start: 123)",
        );

        assert_eq!(
            to_hast(&mdast::Node::List(mdast::List {
                ordered: false,
                start: None,
                spread: false,
                children: vec![],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "ul".into(),
                properties: vec![],
                children: vec![hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                })],
                position: None
            }),
            "should support a `List` (unordered)",
        );

        assert_eq!(
            to_hast(&mdast::Node::List(mdast::List {
                ordered: false,
                start: None,
                spread: false,
                children: vec![mdast::Node::ListItem(mdast::ListItem {
                    spread: false,
                    checked: Some(true),
                    children: vec![],
                    position: None
                }),],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "ul".into(),
                properties: vec![(
                    "className".into(),
                    hast::PropertyValue::SpaceSeparated(vec!["contains-task-list".into()])
                )],
                children: vec![
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    hast::Node::Element(hast::Element {
                        tag_name: "li".into(),
                        properties: vec![(
                            "className".into(),
                            hast::PropertyValue::SpaceSeparated(vec!["task-list-item".into()])
                        )],
                        children: vec![hast::Node::Element(hast::Element {
                            tag_name: "input".into(),
                            properties: vec![
                                (
                                    "type".into(),
                                    hast::PropertyValue::String("checkbox".into()),
                                ),
                                ("checked".into(), hast::PropertyValue::Boolean(true)),
                                ("disabled".into(), hast::PropertyValue::Boolean(true)),
                            ],
                            children: vec![],
                            position: None
                        }),],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                ],
                position: None
            }),
            "should support a `List` (w/ checked item)",
        );
    }

    #[test]
    fn math() {
        assert_eq!(
            to_hast(&mdast::Node::Math(mdast::Math {
                meta: None,
                value: "a".into(),
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "pre".into(),
                properties: vec![],
                children: vec![hast::Node::Element(hast::Element {
                    tag_name: "code".into(),
                    properties: vec![(
                        "className".into(),
                        hast::PropertyValue::SpaceSeparated(vec![
                            "language-math".into(),
                            "math-display".into()
                        ]),
                    ),],
                    children: vec![hast::Node::Text(hast::Text {
                        value: "a\n".into(),
                        position: None
                    })],
                    position: None
                })],
                position: None
            }),
            "should support a `Math`",
        );
    }

    #[test]
    fn paragraph() {
        assert_eq!(
            to_hast(&mdast::Node::Paragraph(mdast::Paragraph {
                children: vec![mdast::Node::Text(mdast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "p".into(),
                properties: vec![],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: None
            }),
            "should support a `Paragraph`",
        );
    }

    #[test]
    fn root() {
        assert_eq!(
            to_hast(&mdast::Node::Root(mdast::Root {
                children: vec![],
                position: None,
            })),
            hast::Node::Root(hast::Root {
                children: vec![],
                position: None
            }),
            "should support a `Root`",
        );
    }

    #[test]
    fn strong() {
        assert_eq!(
            to_hast(&mdast::Node::Strong(mdast::Strong {
                children: vec![mdast::Node::Text(mdast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "strong".into(),
                properties: vec![],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: None
            }),
            "should support a `Strong`",
        );
    }

    #[test]
    fn table_cell() {
        assert_eq!(
            to_hast(&mdast::Node::TableCell(mdast::TableCell {
                children: vec![mdast::Node::Text(mdast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "td".into(),
                properties: vec![],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a".into(),
                    position: None
                })],
                position: None
            }),
            "should support a `TableCell`",
        );
    }

    #[test]
    fn table_row() {
        assert_eq!(
            to_hast(&mdast::Node::TableRow(mdast::TableRow {
                children: vec![
                    mdast::Node::TableCell(mdast::TableCell {
                        children: vec![mdast::Node::Text(mdast::Text {
                            value: "a".into(),
                            position: None
                        })],
                        position: None,
                    }),
                    mdast::Node::TableCell(mdast::TableCell {
                        children: vec![mdast::Node::Text(mdast::Text {
                            value: "b".into(),
                            position: None
                        })],
                        position: None,
                    })
                ],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "tr".into(),
                properties: vec![],
                children: vec![
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    hast::Node::Element(hast::Element {
                        tag_name: "td".into(),
                        properties: vec![],
                        children: vec![hast::Node::Text(hast::Text {
                            value: "a".into(),
                            position: None
                        })],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    hast::Node::Element(hast::Element {
                        tag_name: "td".into(),
                        properties: vec![],
                        children: vec![hast::Node::Text(hast::Text {
                            value: "b".into(),
                            position: None
                        })],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                ],
                position: None
            }),
            "should support a `TableRow`",
        );
    }

    #[test]
    fn table() {
        assert_eq!(
            to_hast(&mdast::Node::Table(mdast::Table {
                align: vec![],
                children: vec![],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "table".into(),
                properties: vec![],
                children: vec![hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                })],
                position: None
            }),
            "should support a `Table` without rows",
        );

        assert_eq!(
            to_hast(&mdast::Node::Table(mdast::Table {
                align: vec![
                    mdast::AlignKind::None,
                    mdast::AlignKind::Left,
                    mdast::AlignKind::Center,
                    mdast::AlignKind::Right
                ],
                children: vec![
                    mdast::Node::TableRow(mdast::TableRow {
                        children: vec![
                            mdast::Node::TableCell(mdast::TableCell {
                                children: vec![mdast::Node::Text(mdast::Text {
                                    value: "a".into(),
                                    position: None
                                })],
                                position: None,
                            }),
                            mdast::Node::TableCell(mdast::TableCell {
                                children: vec![mdast::Node::Text(mdast::Text {
                                    value: "b".into(),
                                    position: None
                                })],
                                position: None,
                            }),
                            mdast::Node::TableCell(mdast::TableCell {
                                children: vec![mdast::Node::Text(mdast::Text {
                                    value: "c".into(),
                                    position: None
                                })],
                                position: None,
                            }),
                            mdast::Node::TableCell(mdast::TableCell {
                                children: vec![mdast::Node::Text(mdast::Text {
                                    value: "d".into(),
                                    position: None
                                })],
                                position: None,
                            })
                        ],
                        position: None,
                    }),
                    mdast::Node::TableRow(mdast::TableRow {
                        children: vec![
                            mdast::Node::TableCell(mdast::TableCell {
                                children: vec![mdast::Node::Text(mdast::Text {
                                    value: "e".into(),
                                    position: None
                                })],
                                position: None,
                            }),
                            mdast::Node::TableCell(mdast::TableCell {
                                children: vec![mdast::Node::Text(mdast::Text {
                                    value: "f".into(),
                                    position: None
                                })],
                                position: None,
                            })
                        ],
                        position: None,
                    })
                ],
                position: None,
            })),
            hast::Node::Element(hast::Element {
                tag_name: "table".into(),
                properties: vec![],
                children: vec![
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    hast::Node::Element(hast::Element {
                        tag_name: "thead".into(),
                        properties: vec![],
                        children: vec![
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                            hast::Node::Element(hast::Element {
                                tag_name: "tr".into(),
                                properties: vec![],
                                children: vec![
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "th".into(),
                                        properties: vec![],
                                        children: vec![hast::Node::Text(hast::Text {
                                            value: "a".into(),
                                            position: None
                                        })],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "th".into(),
                                        properties: vec![(
                                            "align".into(),
                                            hast::PropertyValue::String("left".into()),
                                        ),],
                                        children: vec![hast::Node::Text(hast::Text {
                                            value: "b".into(),
                                            position: None
                                        })],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "th".into(),
                                        properties: vec![(
                                            "align".into(),
                                            hast::PropertyValue::String("center".into()),
                                        ),],
                                        children: vec![hast::Node::Text(hast::Text {
                                            value: "c".into(),
                                            position: None
                                        })],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "th".into(),
                                        properties: vec![(
                                            "align".into(),
                                            hast::PropertyValue::String("right".into()),
                                        ),],
                                        children: vec![hast::Node::Text(hast::Text {
                                            value: "d".into(),
                                            position: None
                                        })],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                ],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                        ],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    hast::Node::Element(hast::Element {
                        tag_name: "tbody".into(),
                        properties: vec![],
                        children: vec![
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                            hast::Node::Element(hast::Element {
                                tag_name: "tr".into(),
                                properties: vec![],
                                children: vec![
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "td".into(),
                                        properties: vec![],
                                        children: vec![hast::Node::Text(hast::Text {
                                            value: "e".into(),
                                            position: None
                                        })],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "td".into(),
                                        properties: vec![(
                                            "align".into(),
                                            hast::PropertyValue::String("left".into()),
                                        )],
                                        children: vec![hast::Node::Text(hast::Text {
                                            value: "f".into(),
                                            position: None
                                        })],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "td".into(),
                                        properties: vec![(
                                            "align".into(),
                                            hast::PropertyValue::String("center".into()),
                                        )],
                                        children: vec![],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                    hast::Node::Element(hast::Element {
                                        tag_name: "td".into(),
                                        properties: vec![(
                                            "align".into(),
                                            hast::PropertyValue::String("right".into()),
                                        )],
                                        children: vec![],
                                        position: None
                                    }),
                                    hast::Node::Text(hast::Text {
                                        value: "\n".into(),
                                        position: None
                                    }),
                                ],
                                position: None
                            }),
                            hast::Node::Text(hast::Text {
                                value: "\n".into(),
                                position: None
                            }),
                        ],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                ],
                position: None
            }),
            "should support a `Table`",
        );
    }

    #[test]
    fn text() {
        assert_eq!(
            to_hast(&mdast::Node::Text(mdast::Text {
                value: "a".into(),
                position: None,
            })),
            hast::Node::Text(hast::Text {
                value: "a".into(),
                position: None
            }),
            "should support a `Text`",
        );
    }

    #[test]
    fn thematic_break() {
        assert_eq!(
            to_hast(&mdast::Node::ThematicBreak(mdast::ThematicBreak {
                position: None
            })),
            hast::Node::Element(hast::Element {
                tag_name: "hr".into(),
                properties: vec![],
                children: vec![],
                position: None
            }),
            "should support a `Thematicbreak`",
        );
    }

    #[test]
    fn yaml() {
        assert_eq!(
            to_hast(&mdast::Node::Yaml(mdast::Yaml {
                value: "a".into(),
                position: None
            })),
            hast::Node::Root(hast::Root {
                children: vec![],
                position: None
            }),
            "should support a `Yaml`",
        );
    }

    #[test]
    fn toml() {
        assert_eq!(
            to_hast(&mdast::Node::Toml(mdast::Toml {
                value: "a".into(),
                position: None
            })),
            hast::Node::Root(hast::Root {
                children: vec![],
                position: None
            }),
            "should support a `Toml`",
        );
    }

    #[test]
    fn util_replace_eols_with_spaces() {
        assert_eq!(
            replace_eols_with_spaces("a \n b \r c \r\n d"),
            "a   b   c   d",
            "should support CR, LF, and CRLF",
        );
    }

    #[test]
    fn util_list_loose() {
        assert_eq!(
            list_loose(&mdast::Node::Text(mdast::Text {
                value: String::new(),
                position: None
            })),
            false,
            "should mark anything that isn’t a list as not loose",
        );

        assert_eq!(
            list_loose(&mdast::Node::List(mdast::List {
                children: vec![],
                ordered: false,
                start: None,
                spread: false,
                position: None
            })),
            false,
            "should mark lists w/o children and w/o spread as loose",
        );

        assert_eq!(
            list_loose(&mdast::Node::List(mdast::List {
                children: vec![],
                ordered: false,
                start: None,
                spread: true,
                position: None
            })),
            true,
            "should mark lists w/ spread as loose",
        );

        assert_eq!(
            list_loose(&mdast::Node::List(mdast::List {
                children: vec![mdast::Node::ListItem(mdast::ListItem {
                    children: vec![],
                    checked: None,
                    spread: true,
                    position: None
                })],
                ordered: false,
                start: None,
                spread: false,
                position: None
            })),
            true,
            "should mark lists w/o spread as loose if an item is loose",
        );

        assert_eq!(
            list_loose(&mdast::Node::List(mdast::List {
                children: vec![mdast::Node::ListItem(mdast::ListItem {
                    children: vec![],
                    checked: None,
                    spread: false,
                    position: None
                })],
                ordered: false,
                start: None,
                spread: false,
                position: None
            })),
            false,
            "should not mark lists w/o spread as loose if there are no loose items",
        );
    }

    #[test]
    fn util_list_item_loose() {
        assert_eq!(
            list_item_loose(&mdast::Node::Text(mdast::Text {
                value: String::new(),
                position: None
            })),
            false,
            "should mark anything that isn’t a list item as not loose",
        );
    }
}
