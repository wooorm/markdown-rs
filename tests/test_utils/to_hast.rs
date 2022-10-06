use crate::test_utils::hast;
use micromark::{mdast, sanitize_, unist::Position};

// To do: support these compile options:
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
// * support `Raw` nodes for HTML?
//
// To do:
// * revert references when undefined?
//   <https://github.com/syntax-tree/mdast-util-to-hast/blob/c393d0a/lib/revert.js>

#[derive(Debug)]
struct State {
    definitions: Vec<(String, String, Option<String>)>,
    footnote_definitions: Vec<(String, Vec<hast::Node>)>,
    footnote_calls: Vec<(String, usize)>,
}

#[derive(Debug)]
enum Result {
    Fragment(Vec<hast::Node>),
    Node(hast::Node),
    None,
}

#[allow(dead_code)]
pub fn to_hast(mdast: &mdast::Node) -> hast::Node {
    let mut definitions = vec![];

    // Collect definitions.
    // Calls take info from their definition.
    // Calls can come come before definitions.
    // Footnote calls can also come before footnote definitions, but those
    // calls *do not* take info from their definitions, so we don’t care
    // about footnotes here.
    visit(mdast, |node| {
        if let mdast::Node::Definition(definition) = node {
            definitions.push((
                definition.identifier.clone(),
                definition.url.clone(),
                definition.title.clone(),
            ));
        }
    });

    let mut state = State {
        definitions,
        footnote_definitions: vec![],
        footnote_calls: vec![],
    };

    let result = one(&mut state, mdast, None);

    if state.footnote_calls.is_empty() {
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

    if !state.footnote_calls.is_empty() {
        let mut items = vec![];

        let mut index = 0;
        while index < state.footnote_calls.len() {
            let (id, count) = &state.footnote_calls[index];
            let safe_id = sanitize_(&id.to_lowercase());

            // Find definition: we’ll always find it.
            let mut definition_index = 0;
            while definition_index < state.footnote_definitions.len() {
                if &state.footnote_definitions[definition_index].0 == id {
                    break;
                }
                definition_index += 1;
            }
            debug_assert_ne!(
                definition_index,
                state.footnote_definitions.len(),
                "expected definition"
            );

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
                            value: (reference_index + 1).to_string(),
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
                                "#fnref-{}{}",
                                safe_id,
                                if reference_index == 0 {
                                    "".into()
                                } else {
                                    format!("-{}", &(reference_index + 1).to_string())
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
                            hast::PropertyValue::SpaceSeparated(vec![
                                "data-footnote-backref".into()
                            ]),
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
                        text.value.push(' ');
                    } else {
                        tail_element.children.push(hast::Node::Text(hast::Text {
                            value: " ".into(),
                            position: None,
                        }));
                    }

                    tail_element
                        .children
                        .append(&mut backreference_opt.take().unwrap());
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
                    hast::PropertyValue::String(format!("#fn-{}", safe_id)),
                )],
                children: wrap(content, true),
                position: None,
            }));
            index += 1;
        }

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

fn one(state: &mut State, node: &mdast::Node, parent: Option<&mdast::Node>) -> Result {
    match node {
        mdast::Node::BlockQuote(d) => transform_block_quote(state, node, d),
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
        mdast::Node::MdxFlowExpression(_) | mdast::Node::MdxTextExpression(_) => {
            transform_mdx_expression(state, node)
        }
        mdast::Node::MdxJsxFlowElement(_) | mdast::Node::MdxJsxTextElement(_) => {
            transform_mdx_jsx_element(state, node)
        }
        mdast::Node::MdxjsEsm(d) => transform_mdxjs_esm(state, node, d),
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
        // Ignore.
        mdast::Node::Definition(_)
        | mdast::Node::Html(_)
        | mdast::Node::Yaml(_)
        | mdast::Node::Toml(_) => Result::None,
    }
}

/// [`BlockQuote`][mdast::BlockQuote].
fn transform_block_quote(
    state: &mut State,
    node: &mdast::Node,
    block_quote: &mdast::BlockQuote,
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
    value.push('\n');
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
                value,
                position: None,
            })],
            position: code.position.clone(),
        })],
        position: code.position.clone(),
    }))
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
    let safe_id = sanitize_(&footnote_reference.identifier.to_lowercase());
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
                    hast::PropertyValue::String(format!("#fn-{}", safe_id)),
                ),
                (
                    "id".into(),
                    hast::PropertyValue::String(format!(
                        "fnref-{}{}",
                        safe_id,
                        if reuse_counter > 1 {
                            format!("-{}", reuse_counter)
                        } else {
                            "".into()
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
                value: (call_index + 1).to_string(),
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
        tag_name: format!("h{}", heading.depth),
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
        hast::PropertyValue::String(sanitize_(&image.url)),
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
    state: &mut State,
    _node: &mdast::Node,
    image_reference: &mdast::ImageReference,
) -> Result {
    let mut properties = vec![];

    let definition = state
        .definitions
        .iter()
        .find(|d| d.0 == image_reference.identifier);

    let (_, url, title) =
        definition.expect("expected reference to have a corresponding definition");

    properties.push(("src".into(), hast::PropertyValue::String(sanitize_(url))));

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
            value: replace_eols_with_spaces(&inline_code.value),
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
            value: replace_eols_with_spaces(&inline_math.value),
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
        hast::PropertyValue::String(sanitize_(&link.url)),
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

    let (_, url, title) =
        definition.expect("expected reference to have a corresponding definition");

    properties.push(("href".into(), hast::PropertyValue::String(sanitize_(url))));

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
    };

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

        // If the input wasn‘t injected yet, inject a paragraph.
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
                value,
                position: None,
            })],
            position: math.position.clone(),
        })],
        position: math.position.clone(),
    }))
}

/// [`MdxFlowExpression`][mdast::MdxFlowExpression],[`MdxTextExpression`][mdast::MdxTextExpression].
fn transform_mdx_expression(_state: &mut State, node: &mdast::Node) -> Result {
    Result::Node(hast::Node::MdxExpression(hast::MdxExpression {
        value: node.to_string(),
        position: node.position().cloned(),
    }))
}

/// [`MdxJsxFlowElement`][mdast::MdxJsxFlowElement],[`MdxJsxTextElement`][mdast::MdxJsxTextElement].
fn transform_mdx_jsx_element(state: &mut State, node: &mdast::Node) -> Result {
    let (name, attributes) = match node {
        mdast::Node::MdxJsxFlowElement(n) => (&n.name, &n.attributes),
        mdast::Node::MdxJsxTextElement(n) => (&n.name, &n.attributes),
        _ => unreachable!("expected mdx jsx element"),
    };

    Result::Node(hast::Node::MdxJsxElement(hast::MdxJsxElement {
        name: name.clone(),
        attributes: attributes.clone(),
        children: all(state, node),
        position: node.position().cloned(),
    }))
}

/// [`MdxjsEsm`][mdast::MdxjsEsm].
fn transform_mdxjs_esm(
    _state: &mut State,
    _node: &mdast::Node,
    mdxjs_esm: &mdast::MdxjsEsm,
) -> Result {
    Result::Node(hast::Node::MdxjsEsm(hast::MdxjsEsm {
        value: mdxjs_esm.value.clone(),
        position: mdxjs_esm.position.clone(),
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
    #[allow(clippy::redundant_closure_for_method_calls)]
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
            unreachable!("expected tale cell in table row")
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

    let body_rows = rows.split_off(1);
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
        value: text.value.clone(),
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

// Transform children of `parent`.
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

/// Visit, mutably.
// Probably useful later:
#[allow(dead_code)]
fn visit_mut<Visitor>(node: &mut mdast::Node, visitor: Visitor)
where
    Visitor: FnMut(&mut mdast::Node),
{
    visit_mut_impl(node, visitor);
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

/// Internal implementation to visit, mutably.
fn visit_mut_impl<Visitor>(node: &mut mdast::Node, mut visitor: Visitor) -> Visitor
where
    Visitor: FnMut(&mut mdast::Node),
{
    visitor(node);

    if let Some(children) = node.children_mut() {
        let mut index = 0;
        while let Some(child) = children.get_mut(index) {
            visitor = visit_mut_impl(child, visitor);
            index += 1;
        }
    }

    visitor
}

// To do: trim arounds breaks: <https://github.com/syntax-tree/mdast-util-to-hast/blob/c393d0a/lib/traverse.js>.
/// Append an (optional, variadic) result.
fn append_result(list: &mut Vec<hast::Node>, result: Result) {
    match result {
        Result::Fragment(mut fragment) => list.append(&mut fragment),
        Result::Node(node) => list.push(node),
        Result::None => {}
    };
}

/// Replace line endings (CR, LF, CRLF) with spaces.
///
/// Used for inline code and inline math.
fn replace_eols_with_spaces(value: &str) -> String {
    // It’ll grow a bit small for each CR+LF.
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
