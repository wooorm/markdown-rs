use markdown::mdast::Node;

mod test_utils;

#[allow(unused)]
#[derive(Debug)]
enum Error {
    Mdast(markdown::message::Message),
    Serde(serde_json::Error),
}

#[cfg_attr(feature = "serde", test)]
fn serde_blockquote() -> Result<(), Error> {
    assert_serde(
        "> a",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "blockquote",
      "children": [
        {
          "type": "paragraph",
          "children": [
            {
              "type": "text",
              "value": "a"
            }
          ]
        }
      ]
    }
  ]
}
"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_footnote_definition() -> Result<(), Error> {
    assert_serde(
        "[^a]: b",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "footnoteDefinition",
      "children": [
        {
          "type": "paragraph",
          "children": [
            {
              "type": "text",
              "value": "b"
            }
          ]
        }
      ],
      "identifier": "a",
      "label": "a"
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_mdx_jsx_flow_element() -> Result<(), Error> {
    let source = r#"<Test id={id} class="test" {...b} />"#;
    assert_serde(
        source,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "mdxJsxFlowElement",
      "children": [],
      "name": "Test",
      "attributes": [
        {
          "type": "mdxJsxAttribute",
          "name": "id",
          "value": {
            "type": "mdxJsxAttributeValueExpression",
            "value": "id",
            "stops": [
              [
                0,
                10
              ]
            ]
          }
        },
        {
          "type": "mdxJsxAttribute",
          "name": "class",
          "value": "test"
        },
        {
          "type": "mdxJsxExpressionAttribute",
          "value": "...b",
          "stops": [
            [
              0,
              28
            ]
          ]
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_list() -> Result<(), Error> {
    assert_serde(
        "* a",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "list",
      "children": [
        {
          "type": "listItem",
          "children": [
            {
              "type": "paragraph",
              "children": [
                {
                  "type": "text",
                  "value": "a"
                }
              ]
            }
          ],
          "spread": false
        }
      ],
      "ordered": false,
      "spread": false
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_mdxjs_esm() -> Result<(), Error> {
    assert_serde(
        r#"
import Test from 'test';
"#,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "mdxjsEsm",
      "value": "import Test from 'test';",
      "stops": [
        [
          0,
          1
        ]
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_toml() -> Result<(), Error> {
    assert_serde(
        r#"+++
a: b
+++
"#,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "toml",
      "value": "a: b"
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_yaml() -> Result<(), Error> {
    assert_serde(
        r#"---
a: b
---
"#,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "yaml",
      "value": "a: b"
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_break() -> Result<(), Error> {
    let source = r#"a\
b
"#;
    assert_serde(
        source,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "text",
          "value": "a"
        },
        {
          "type": "break"
        },
        {
          "type": "text",
          "value": "b"
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_inline_code() -> Result<(), Error> {
    assert_serde(
        "`a`",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "inlineCode",
          "value": "a"
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_inline_math() -> Result<(), Error> {
    assert_serde(
        "$a$",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "inlineMath",
          "value": "a"
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_delete() -> Result<(), Error> {
    assert_serde(
        "~~a~~",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "delete",
          "children": [
            {
              "type": "text",
              "value": "a"
            }
          ]
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_emphasis() -> Result<(), Error> {
    assert_serde(
        "*a*",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "emphasis",
          "children": [
            {
              "type": "text",
              "value": "a"
            }
          ]
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_mdx_text_expression() -> Result<(), Error> {
    assert_serde(
        "a {b}",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "text",
          "value": "a "
        },
        {
          "type": "mdxTextExpression",
          "value": "b",
          "stops": [
            [
              0,
              3
            ]
          ]
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_footnote_reference() -> Result<(), Error> {
    let source = r#"Refer to [^a]
[^a]: b
"#;
    assert_serde(
        source,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "text",
          "value": "Refer to "
        },
        {
          "type": "footnoteReference",
          "identifier": "a",
          "label": "a"
        }
      ]
    },
    {
      "type": "footnoteDefinition",
      "children": [
        {
          "type": "paragraph",
          "children": [
            {
              "type": "text",
              "value": "b"
            }
          ]
        }
      ],
      "identifier": "a",
      "label": "a"
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_html() -> Result<(), Error> {
    assert_serde(
        "<a>",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "html",
      "value": "<a>"
    }
  ]
}"#,
        Some(markdown::ParseOptions {
            constructs: markdown::Constructs::gfm(),
            ..markdown::ParseOptions::gfm()
        }),
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_image() -> Result<(), Error> {
    assert_serde(
        "![a](b)",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "image",
          "alt": "a",
          "url": "b"
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_image_reference() -> Result<(), Error> {
    let source = r#"[x]: y
a ![x] b
"#;
    assert_serde(
        source,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "definition",
      "url": "y",
      "identifier": "x",
      "label": "x"
    },
    {
      "type": "paragraph",
      "children": [
        {
          "type": "text",
          "value": "a "
        },
        {
          "type": "imageReference",
          "alt": "x",
          "referenceType": "shortcut",
          "identifier": "x",
          "label": "x"
        },
        {
          "type": "text",
          "value": " b"
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_mdx_jsx_text_element() -> Result<(), Error> {
    let source = r#"text <Test id={id} class="test" {...b} />"#;
    assert_serde(
        source,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "text",
          "value": "text "
        },
        {
          "type": "mdxJsxTextElement",
          "children": [],
          "name": "Test",
          "attributes": [
            {
              "type": "mdxJsxAttribute",
              "name": "id",
              "value": {
                "type": "mdxJsxAttributeValueExpression",
                "value": "id",
                "stops": [
                  [
                    0,
                    15
                  ]
                ]
              }
            },
            {
              "type": "mdxJsxAttribute",
              "name": "class",
              "value": "test"
            },
            {
              "type": "mdxJsxExpressionAttribute",
              "value": "...b",
              "stops": [
                [
                  0,
                  33
                ]
              ]
            }
          ]
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_link() -> Result<(), Error> {
    assert_serde(
        "link [a](b)",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "text",
          "value": "link "
        },
        {
          "type": "link",
          "children": [
            {
              "type": "text",
              "value": "a"
            }
          ],
          "url": "b"
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_link_reference() -> Result<(), Error> {
    let source = r#"[x]: y
a [x] b
"#;
    assert_serde(
        source,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "definition",
      "url": "y",
      "identifier": "x",
      "label": "x"
    },
    {
      "type": "paragraph",
      "children": [
        {
          "type": "text",
          "value": "a "
        },
        {
          "type": "linkReference",
          "children": [
            {
              "type": "text",
              "value": "x"
            }
          ],
          "referenceType": "shortcut",
          "identifier": "x",
          "label": "x"
        },
        {
          "type": "text",
          "value": " b"
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_strong() -> Result<(), Error> {
    assert_serde(
        "**a**",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "strong",
          "children": [
            {
              "type": "text",
              "value": "a"
            }
          ]
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_text() -> Result<(), Error> {
    assert_serde(
        "a",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "text",
          "value": "a"
        }
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_code() -> Result<(), Error> {
    let source = r#"~~~
let a = b;
~~~
"#;
    assert_serde(
        source,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "code",
      "value": "let a = b;"
    }
  ]
}"#,
        None,
    )?;
    let source = r#"```
let a = b;
```
"#;
    assert_serde(
        source,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "code",
      "value": "let a = b;"
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_math() -> Result<(), Error> {
    let source = r#"$$
1 + 1 = 2
$$
"#;
    assert_serde(
        source,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "math",
      "value": "1 + 1 = 2"
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_mdx_flow_expression() -> Result<(), Error> {
    assert_serde(
        "{a}",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "mdxFlowExpression",
      "value": "a",
      "stops": [
        [
          0,
          1
        ]
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_heading() -> Result<(), Error> {
    assert_serde(
        "# a",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "heading",
      "children": [
        {
          "type": "text",
          "value": "a"
        }
      ],
      "depth": 1
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_table() -> Result<(), Error> {
    let source = r#"| a   | b   |
|-----|-----|
| c11 | c12 |
| c21 | c22 |
"#;
    assert_serde(
        source,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "table",
      "children": [
        {
          "type": "tableRow",
          "children": [
            {
              "type": "tableCell",
              "children": [
                {
                  "type": "text",
                  "value": "a"
                }
              ]
            },
            {
              "type": "tableCell",
              "children": [
                {
                  "type": "text",
                  "value": "b"
                }
              ]
            }
          ]
        },
        {
          "type": "tableRow",
          "children": [
            {
              "type": "tableCell",
              "children": [
                {
                  "type": "text",
                  "value": "c11"
                }
              ]
            },
            {
              "type": "tableCell",
              "children": [
                {
                  "type": "text",
                  "value": "c12"
                }
              ]
            }
          ]
        },
        {
          "type": "tableRow",
          "children": [
            {
              "type": "tableCell",
              "children": [
                {
                  "type": "text",
                  "value": "c21"
                }
              ]
            },
            {
              "type": "tableCell",
              "children": [
                {
                  "type": "text",
                  "value": "c22"
                }
              ]
            }
          ]
        }
      ],
      "align": [
        "none",
        "none"
      ]
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_thematic_break() -> Result<(), Error> {
    assert_serde(
        "***",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "thematicBreak"
    }
  ]
}"#,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_definition() -> Result<(), Error> {
    assert_serde(
        "[a]: # (b)",
        r###"{
  "type": "root",
  "children": [
    {
      "type": "definition",
      "url": "#",
      "title": "b",
      "identifier": "a",
      "label": "a"
    }
  ]
}"###,
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_paragraph() -> Result<(), Error> {
    assert_serde(
        "a",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "text",
          "value": "a"
        }
      ]
    }
  ]
}"#,
        None,
    )
}

/// Assert serde of Mdast constructs.
///
/// Refer below links for the MDAST JSON construct types.
/// * https://github.com/syntax-tree/mdast#nodes
/// * https://github.com/syntax-tree/mdast-util-mdx-jsx?tab=readme-ov-file#returns-1
#[cfg(feature = "serde")]
fn assert_serde(
    input: &str,
    expected: &str,
    options: Option<markdown::ParseOptions>,
) -> Result<(), Error> {
    // Parse Mdast with default options of MDX and GFM
    use test_utils::swc::{parse_esm, parse_expression};
    let mut source = markdown::to_mdast(
        input,
        &options.unwrap_or(markdown::ParseOptions {
            constructs: markdown::Constructs {
                frontmatter: true,
                gfm_autolink_literal: true,
                gfm_footnote_definition: true,
                gfm_label_start_footnote: true,
                gfm_strikethrough: true,
                gfm_table: true,
                gfm_task_list_item: true,
                math_flow: true,
                math_text: true,
                ..markdown::Constructs::mdx()
            },
            mdx_esm_parse: Some(Box::new(parse_esm)),
            mdx_expression_parse: Some(Box::new(parse_expression)),
            ..markdown::ParseOptions::gfm()
        }),
    )
    .map_err(Error::Mdast)?;

    remove_position(&mut source);
    // Serialize to JSON
    let actual_value: serde_json::Value = serde_json::to_value(&source).map_err(Error::Serde)?;
    let expected_value: serde_json::Value = serde_json::from_str(expected).map_err(Error::Serde)?;
    // Assert serialization
    pretty_assertions::assert_eq!(actual_value, expected_value);
    // Assert deserialization
    pretty_assertions::assert_eq!(
        source,
        serde_json::from_value(actual_value).map_err(Error::Serde)?
    );
    Ok(())
}

fn remove_position(node: &mut Node) {
    if let Some(children) = node.children_mut() {
        for child in children {
            remove_position(child);
        }
    }
    node.position_set(None);
}
