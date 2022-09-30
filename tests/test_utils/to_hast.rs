use crate::test_utils::hast;
use micromark::{mdast, sanitize_, unist::Position};

// Options?
// - dangerous: raw? No
// - clobberPrefix / footnoteLabel / footnoteLabelTagName / footnoteLabelProperties / footnoteBacklabel? Later

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

    let (result, mut state) = one(
        mdast,
        None,
        State {
            definitions,
            footnote_definitions: vec![],
            footnote_calls: vec![],
        },
    );

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
                // To do: support clobber prefix.
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

fn one(node: &mdast::Node, parent: Option<&mdast::Node>, state: State) -> (Result, State) {
    match node {
        mdast::Node::BlockQuote(_) => transform_block_quote(node, state),
        mdast::Node::Break(_) => transform_break(node, state),
        mdast::Node::Code(_) => transform_code(node, state),
        mdast::Node::Delete(_) => transform_delete(node, state),
        mdast::Node::Emphasis(_) => transform_emphasis(node, state),
        mdast::Node::FootnoteDefinition(_) => transform_footnote_definition(node, state),
        mdast::Node::FootnoteReference(_) => transform_footnote_reference(node, state),
        mdast::Node::Heading(_) => transform_heading(node, state),
        mdast::Node::Image(_) => transform_image(node, state),
        mdast::Node::ImageReference(_) => transform_image_reference(node, state),
        mdast::Node::InlineCode(_) => transform_inline_code(node, state),
        mdast::Node::InlineMath(_) => transform_inline_math(node, state),
        mdast::Node::Link(_) => transform_link(node, state),
        mdast::Node::LinkReference(_) => transform_link_reference(node, state),
        mdast::Node::ListItem(_) => transform_list_item(node, parent, state),
        mdast::Node::List(_) => transform_list(node, state),
        mdast::Node::Math(_) => transform_math(node, state),
        mdast::Node::MdxFlowExpression(_) | mdast::Node::MdxTextExpression(_) => {
            transform_mdx_expression(node, state)
        }
        mdast::Node::MdxJsxFlowElement(_) | mdast::Node::MdxJsxTextElement(_) => {
            transform_mdx_jsx_element(node, state)
        }
        mdast::Node::MdxjsEsm(_) => transform_mdxjs_esm(node, state),
        mdast::Node::Paragraph(_) => transform_paragraph(node, state),
        mdast::Node::Root(_) => transform_root(node, state),
        mdast::Node::Strong(_) => transform_strong(node, state),
        // Note: this is only called here if there is a single cell passed, not when one is found in a table.
        mdast::Node::TableCell(_) => {
            transform_table_cell(node, false, mdast::AlignKind::None, state)
        }
        // Note: this is only called here if there is a single row passed, not when one is found in a table.
        mdast::Node::TableRow(_) => transform_table_row(node, false, None, state),
        mdast::Node::Table(_) => transform_table(node, state),
        mdast::Node::Text(_) => transform_text(node, state),
        mdast::Node::ThematicBreak(_) => transform_thematic_break(node, state),
        // Ignore.
        // Idea: support `Raw` nodes for HTML, optionally?
        mdast::Node::Definition(_)
        | mdast::Node::Html(_)
        | mdast::Node::Yaml(_)
        | mdast::Node::Toml(_) => (Result::None, state),
    }
}

/// [`BlockQuote`][mdast::BlockQuote].
fn transform_block_quote(node: &mdast::Node, state: State) -> (Result, State) {
    let (children, state) = all(node, state);
    (
        Result::Node(augment_node(
            node,
            hast::Node::Element(hast::Element {
                tag_name: "blockquote".into(),
                properties: vec![],
                children: wrap(children, true),
                position: None,
            }),
        )),
        state,
    )
}

/// [`Break`][mdast::Break].
fn transform_break(node: &mdast::Node, state: State) -> (Result, State) {
    (
        Result::Fragment(vec![
            augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "br".into(),
                    properties: vec![],
                    children: vec![],
                    position: None,
                }),
            ),
            hast::Node::Text(hast::Text {
                value: "\n".into(),
                position: None,
            }),
        ]),
        state,
    )
}

/// [`Code`][mdast::Code].
fn transform_code(node: &mdast::Node, state: State) -> (Result, State) {
    if let mdast::Node::Code(code) = node {
        let mut value = code.value.clone();
        value.push('\n');
        let mut properties = vec![];

        if let Some(lang) = code.lang.as_ref() {
            let mut value = "language-".to_string();
            value.push_str(lang);
            properties.push((
                "className".into(),
                hast::PropertyValue::SpaceSeparated(vec![value]),
            ));
        }

        // To do: option to persist `meta`?

        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "pre".into(),
                    properties: vec![],
                    children: vec![augment_node(
                        node,
                        hast::Node::Element(hast::Element {
                            tag_name: "code".into(),
                            properties,
                            children: vec![hast::Node::Text(hast::Text {
                                value,
                                position: None,
                            })],
                            position: None,
                        }),
                    )],
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `Code`")
    }
}

/// [`Delete`][mdast::Delete].
fn transform_delete(node: &mdast::Node, state: State) -> (Result, State) {
    let (children, state) = all(node, state);
    (
        Result::Node(augment_node(
            node,
            hast::Node::Element(hast::Element {
                tag_name: "del".into(),
                properties: vec![],
                children,
                position: None,
            }),
        )),
        state,
    )
}

/// [`Emphasis`][mdast::Emphasis].
fn transform_emphasis(node: &mdast::Node, state: State) -> (Result, State) {
    let (children, state) = all(node, state);
    (
        Result::Node(augment_node(
            node,
            hast::Node::Element(hast::Element {
                tag_name: "em".into(),
                properties: vec![],
                children,
                position: None,
            }),
        )),
        state,
    )
}

/// [`FootnoteDefinition`][mdast::FootnoteDefinition].
fn transform_footnote_definition(node: &mdast::Node, mut state: State) -> (Result, State) {
    if let mdast::Node::FootnoteDefinition(definition) = node {
        let result = all(node, state);
        let children = result.0;
        state = result.1;
        // Set aside.
        state
            .footnote_definitions
            .push((definition.identifier.clone(), children));
        (Result::None, state)
    } else {
        unreachable!("expected `FootnoteDefinition`")
    }
}

/// [`FootnoteReference`][mdast::FootnoteReference].
fn transform_footnote_reference(node: &mdast::Node, mut state: State) -> (Result, State) {
    if let mdast::Node::FootnoteReference(reference) = node {
        let safe_id = sanitize_(&reference.identifier.to_lowercase());
        let mut call_index = 0;

        // See if this has been called before.
        while call_index < state.footnote_calls.len() {
            if state.footnote_calls[call_index].0 == reference.identifier {
                break;
            }
            call_index += 1;
        }

        // New.
        if call_index == state.footnote_calls.len() {
            state.footnote_calls.push((reference.identifier.clone(), 0));
        }

        // Increment.
        state.footnote_calls[call_index].1 += 1;

        let reuse_counter = state.footnote_calls[call_index].1;

        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "sup".into(),
                    properties: vec![],
                    children: vec![hast::Node::Element(hast::Element {
                        tag_name: "a".into(),
                        // To do: support clobber prefix.
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
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `FootnoteReference`")
    }
}

/// [`Heading`][mdast::Heading].
fn transform_heading(node: &mdast::Node, state: State) -> (Result, State) {
    if let mdast::Node::Heading(heading) = node {
        let (children, state) = all(node, state);
        let tag_name = format!("h{}", heading.depth);
        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name,
                    properties: vec![],
                    children,
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `Heading`")
    }
}

/// [`Image`][mdast::Image].
fn transform_image(node: &mdast::Node, state: State) -> (Result, State) {
    if let mdast::Node::Image(image) = node {
        let mut properties = vec![];

        properties.push((
            "src".into(),
            hast::PropertyValue::String(sanitize_(&image.url)),
        ));

        properties.push(("alt".into(), hast::PropertyValue::String(image.alt.clone())));

        if let Some(value) = image.title.as_ref() {
            properties.push(("title".into(), hast::PropertyValue::String(value.into())));
        }

        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "img".into(),
                    properties,
                    children: vec![],
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `Image`")
    }
}

/// [`ImageReference`][mdast::ImageReference].
fn transform_image_reference(node: &mdast::Node, state: State) -> (Result, State) {
    if let mdast::Node::ImageReference(reference) = node {
        let mut properties = vec![];

        let definition = state
            .definitions
            .iter()
            .find(|d| d.0 == reference.identifier);

        // To do: revert when undefined? <https://github.com/syntax-tree/mdast-util-to-hast/blob/c393d0a60941d8936135e05a5cc78734d87578ba/lib/revert.js>
        let (_, url, title) =
            definition.expect("expected reference to have a corresponding definition");

        properties.push(("src".into(), hast::PropertyValue::String(sanitize_(url))));

        properties.push((
            "alt".into(),
            hast::PropertyValue::String(reference.alt.clone()),
        ));

        if let Some(value) = title {
            properties.push(("title".into(), hast::PropertyValue::String(value.into())));
        }

        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "img".into(),
                    properties,
                    children: vec![],
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `ImageReference`")
    }
}

/// [`InlineCode`][mdast::InlineCode].
fn transform_inline_code(node: &mdast::Node, state: State) -> (Result, State) {
    if let mdast::Node::InlineCode(code) = node {
        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "code".into(),
                    properties: vec![],
                    children: vec![hast::Node::Text(hast::Text {
                        value: replace_eols_with_spaces(&code.value),
                        position: None,
                    })],
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `InlineCode`")
    }
}

/// [`InlineMath`][mdast::InlineMath].
fn transform_inline_math(node: &mdast::Node, state: State) -> (Result, State) {
    if let mdast::Node::InlineMath(math) = node {
        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "code".into(),
                    properties: vec![(
                        "className".into(),
                        hast::PropertyValue::SpaceSeparated(vec![
                            "language-math".into(),
                            "math-inline".into(),
                        ]),
                    )],
                    children: vec![hast::Node::Text(hast::Text {
                        value: replace_eols_with_spaces(&math.value),
                        position: None,
                    })],
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `InlineMath`")
    }
}

/// [`Link`][mdast::Link].
fn transform_link(node: &mdast::Node, state: State) -> (Result, State) {
    if let mdast::Node::Link(link) = node {
        let mut properties = vec![];

        properties.push((
            "href".into(),
            hast::PropertyValue::String(sanitize_(&link.url)),
        ));

        if let Some(value) = link.title.as_ref() {
            properties.push(("title".into(), hast::PropertyValue::String(value.into())));
        }

        let (children, state) = all(node, state);
        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "a".into(),
                    properties,
                    children,
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `Link`")
    }
}

/// [`LinkReference`][mdast::LinkReference].
fn transform_link_reference(node: &mdast::Node, state: State) -> (Result, State) {
    if let mdast::Node::LinkReference(reference) = node {
        let mut properties = vec![];

        let definition = state
            .definitions
            .iter()
            .find(|d| d.0 == reference.identifier);

        // To do: revert when undefined? <https://github.com/syntax-tree/mdast-util-to-hast/blob/c393d0a60941d8936135e05a5cc78734d87578ba/lib/revert.js>
        let (_, url, title) =
            definition.expect("expected reference to have a corresponding definition");

        properties.push(("href".into(), hast::PropertyValue::String(sanitize_(url))));

        if let Some(value) = title {
            properties.push(("title".into(), hast::PropertyValue::String(value.into())));
        }

        let (children, state) = all(node, state);
        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "a".into(),
                    properties,
                    children,
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `LinkReference`")
    }
}

/// [`ListItem`][mdast::ListItem].
fn transform_list_item(
    node: &mdast::Node,
    parent: Option<&mdast::Node>,
    state: State,
) -> (Result, State) {
    if let mdast::Node::ListItem(item) = node {
        let (mut children, state) = all(node, state);
        let mut loose = list_item_loose(node);

        if let Some(parent) = parent {
            if matches!(parent, mdast::Node::List(_)) {
                loose = list_loose(parent);
            }
        };

        let mut properties = vec![];

        // Inject a checkbox.
        if let Some(checked) = item.checked {
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

        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "li".into(),
                    properties,
                    children: result,
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `ListItem`")
    }
}

/// [`List`][mdast::List].
fn transform_list(node: &mdast::Node, state: State) -> (Result, State) {
    if let mdast::Node::List(list) = node {
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

        let (children, state) = all(node, state);
        let mut properties = vec![];
        let tag_name = if list.ordered {
            "ol".into()
        } else {
            "ul".into()
        };

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

        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name,
                    properties,
                    children: wrap(children, true),
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `List`")
    }
}

/// [`Math`][mdast::Math].
fn transform_math(node: &mdast::Node, state: State) -> (Result, State) {
    if let mdast::Node::Math(math) = node {
        let mut value = math.value.clone();
        value.push('\n');

        // To do: option to persist `meta`?

        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "pre".into(),
                    properties: vec![],
                    children: vec![augment_node(
                        node,
                        hast::Node::Element(hast::Element {
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
                            position: None,
                        }),
                    )],
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `Math`")
    }
}

/// [`MdxFlowExpression`][mdast::MdxFlowExpression],[`MdxTextExpression`][mdast::MdxTextExpression].
fn transform_mdx_expression(node: &mdast::Node, state: State) -> (Result, State) {
    (
        Result::Node(augment_node(
            node,
            hast::Node::MdxExpression(hast::MdxExpression {
                value: node.to_string(),
                position: None,
            }),
        )),
        state,
    )
}

/// [`MdxJsxFlowElement`][mdast::MdxJsxFlowElement],[`MdxJsxTextElement`][mdast::MdxJsxTextElement].
fn transform_mdx_jsx_element(node: &mdast::Node, state: State) -> (Result, State) {
    let (children, state) = all(node, state);

    let (name, attributes) = match node {
        mdast::Node::MdxJsxFlowElement(n) => (&n.name, &n.attributes),
        mdast::Node::MdxJsxTextElement(n) => (&n.name, &n.attributes),
        _ => unreachable!("expected mdx jsx element"),
    };

    (
        Result::Node(augment_node(
            node,
            hast::Node::MdxJsxElement(hast::MdxJsxElement {
                name: name.clone(),
                attributes: attributes.clone(),
                children,
                position: None,
            }),
        )),
        state,
    )
}

/// [`MdxjsEsm`][mdast::MdxjsEsm].
fn transform_mdxjs_esm(node: &mdast::Node, state: State) -> (Result, State) {
    (
        Result::Node(augment_node(
            node,
            hast::Node::MdxjsEsm(hast::MdxjsEsm {
                value: node.to_string(),
                position: None,
            }),
        )),
        state,
    )
}

/// [`Paragraph`][mdast::Paragraph].
fn transform_paragraph(node: &mdast::Node, state: State) -> (Result, State) {
    let (children, state) = all(node, state);
    (
        Result::Node(augment_node(
            node,
            hast::Node::Element(hast::Element {
                tag_name: "p".into(),
                properties: vec![],
                children,
                position: None,
            }),
        )),
        state,
    )
}

/// [`Root`][mdast::Root].
fn transform_root(node: &mdast::Node, state: State) -> (Result, State) {
    let (children, state) = all(node, state);
    (
        Result::Node(augment_node(
            node,
            hast::Node::Root(hast::Root {
                children: wrap(children, false),
                position: None,
            }),
        )),
        state,
    )
}

/// [`Strong`][mdast::Strong].
fn transform_strong(node: &mdast::Node, state: State) -> (Result, State) {
    let (children, state) = all(node, state);
    (
        Result::Node(augment_node(
            node,
            hast::Node::Element(hast::Element {
                tag_name: "strong".into(),
                properties: vec![],
                children,
                position: None,
            }),
        )),
        state,
    )
}

/// [`TableCell`][mdast::TableCell].
fn transform_table_cell(
    node: &mdast::Node,
    head: bool,
    align: mdast::AlignKind,
    state: State,
) -> (Result, State) {
    let (children, state) = all(node, state);
    // To do: option to generate a `style` instead?
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

    (
        Result::Node(augment_node(
            node,
            hast::Node::Element(hast::Element {
                tag_name: if head { "th".into() } else { "td".into() },
                properties,
                children,
                position: None,
            }),
        )),
        state,
    )
}

/// [`TableRow`][mdast::TableRow].
fn transform_table_row(
    node: &mdast::Node,
    head: bool,
    align: Option<&[mdast::AlignKind]>,
    mut state: State,
) -> (Result, State) {
    if let mdast::Node::TableRow(row) = node {
        let mut children = vec![];
        let mut index = 0;
        #[allow(clippy::redundant_closure_for_method_calls)]
        let len = align.map_or(row.children.len(), |d| d.len());
        let empty_cell = mdast::Node::TableCell(mdast::TableCell {
            children: vec![],
            position: None,
        });

        while index < len {
            let align_value = align
                .and_then(|d| d.get(index))
                .unwrap_or(&mdast::AlignKind::None);

            let child = row.children.get(index).unwrap_or(&empty_cell);
            let tuple = transform_table_cell(child, head, *align_value, state);
            append_result(&mut children, tuple.0);
            state = tuple.1;
            index += 1;
        }

        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "tr".into(),
                    properties: vec![],
                    children: wrap(children, true),
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `TableRow`")
    }
}

/// [`Table`][mdast::Table].
fn transform_table(node: &mdast::Node, mut state: State) -> (Result, State) {
    if let mdast::Node::Table(table) = node {
        let mut rows = vec![];
        let mut index = 0;

        while index < table.children.len() {
            let tuple = transform_table_row(
                &table.children[index],
                index == 0,
                Some(&table.align),
                state,
            );
            append_result(&mut rows, tuple.0);
            state = tuple.1;
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

        (
            Result::Node(augment_node(
                node,
                hast::Node::Element(hast::Element {
                    tag_name: "table".into(),
                    properties: vec![],
                    children: wrap(children, true),
                    position: None,
                }),
            )),
            state,
        )
    } else {
        unreachable!("expected `Table`")
    }
}

/// [`Text`][mdast::Text].
fn transform_text(node: &mdast::Node, state: State) -> (Result, State) {
    (
        Result::Node(augment_node(
            node,
            hast::Node::Text(hast::Text {
                value: node.to_string(),
                position: None,
            }),
        )),
        state,
    )
}

/// [`ThematicBreak`][mdast::ThematicBreak].
fn transform_thematic_break(node: &mdast::Node, state: State) -> (Result, State) {
    (
        Result::Node(augment_node(
            node,
            hast::Node::Element(hast::Element {
                tag_name: "hr".into(),
                properties: vec![],
                children: vec![],
                position: None,
            }),
        )),
        state,
    )
}

// Transform children of `parent`.
fn all(parent: &mdast::Node, mut state: State) -> (Vec<hast::Node>, State) {
    let mut result = vec![];
    if let Some(children) = parent.children() {
        let mut index = 0;
        while index < children.len() {
            let child = &children[index];
            let tuple = one(child, Some(parent), state);
            append_result(&mut result, tuple.0);
            state = tuple.1;
            index += 1;
        }
    }

    (result, state)
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

/// Patch a position from the node `left` onto `right`.
fn augment_node(left: &mdast::Node, right: hast::Node) -> hast::Node {
    if let Some(position) = left.position() {
        augment_position(position, right)
    } else {
        right
    }
}

/// Patch a position from `left` onto `right`.
fn augment_position(left: &Position, mut right: hast::Node) -> hast::Node {
    right.position_set(Some(left.clone()));
    right
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
