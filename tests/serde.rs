use markdown::{mdast::Node, message::Message, Constructs, ParseOptions};
use test_utils::swc::{parse_esm, parse_expression};
mod test_utils;

#[allow(unused)]
#[derive(Debug)]
enum Error {
    Mdast(Message),
    Serde(serde_json::Error),
}

#[test]
#[cfg(feature = "serde")]
fn serde_constructs() -> Result<(), Error> {
    use pretty_assertions::assert_eq;

    assert_eq!(
        serde_json::to_string(&Constructs::default()).unwrap(),
        r#"{"attention":true,"autolink":true,"blockQuote":true,"characterEscape":true,"characterReference":true,"codeIndented":true,"codeFenced":true,"codeText":true,"definition":true,"frontmatter":false,"gfmAutolinkLiteral":false,"gfmFootnoteDefinition":false,"gfmLabelStartFootnote":false,"gfmStrikethrough":false,"gfmTable":false,"gfmTaskListItem":false,"hardBreakEscape":true,"hardBreakTrailing":true,"headingAtx":true,"headingSetext":true,"htmlFlow":true,"htmlText":true,"labelStartImage":true,"labelStartLink":true,"labelEnd":true,"listItem":true,"mathFlow":false,"mathText":false,"mdxEsm":false,"mdxExpressionFlow":false,"mdxExpressionText":false,"mdxJsxFlow":false,"mdxJsxText":false,"thematicBreak":true}"#
    );

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn serde_compile_options() -> Result<(), Error> {
    use pretty_assertions::assert_eq;

    assert_eq!(
        serde_json::to_string(&markdown::CompileOptions::gfm()).unwrap(),
        r#"{"allowAnyImgSrc":false,"allowDangerousHtml":false,"allowDangerousProtocol":false,"defaultLineEnding":"\n","gfmFootnoteBackLabel":null,"gfmFootnoteClobberPrefix":null,"gfmFootnoteLabelAttributes":null,"gfmFootnoteLabelTagName":null,"gfmFootnoteLabel":null,"gfmTaskListItemCheckable":false,"gfmTagfilter":true}"#
    );

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn serde_parse_options() -> Result<(), Error> {
    use pretty_assertions::assert_eq;

    assert_eq!(
        serde_json::to_string(&ParseOptions::gfm()).unwrap(),
        r#"{"constructs":{"attention":true,"autolink":true,"blockQuote":true,"characterEscape":true,"characterReference":true,"codeIndented":true,"codeFenced":true,"codeText":true,"definition":true,"frontmatter":false,"gfmAutolinkLiteral":true,"gfmFootnoteDefinition":true,"gfmLabelStartFootnote":true,"gfmStrikethrough":true,"gfmTable":true,"gfmTaskListItem":true,"hardBreakEscape":true,"hardBreakTrailing":true,"headingAtx":true,"headingSetext":true,"htmlFlow":true,"htmlText":true,"labelStartImage":true,"labelStartLink":true,"labelEnd":true,"listItem":true,"mathFlow":false,"mathText":false,"mdxEsm":false,"mdxExpressionFlow":false,"mdxExpressionText":false,"mdxJsxFlow":false,"mdxJsxText":false,"thematicBreak":true},"gfmStrikethroughSingleTilde":true,"mathTextSingleDollar":true}"#
    );

    Ok(())
}

#[test]
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
          "children": [{"type": "text", "value": "a"}]
        }
      ]
    }
  ]
}
"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_footnote_definition() -> Result<(), Error> {
    assert_serde(
        "[^a]: b",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "footnoteDefinition",
      "identifier": "a",
      "label": "a",
      "children": [
        {
          "type": "paragraph",
          "children": [{"type": "text", "value": "b"}]
        }
      ]
    }
  ]
}"#,
        ParseOptions::gfm(),
    )
}

#[test]
fn serde_mdx_jsx_flow_element() -> Result<(), Error> {
    let options = ParseOptions {
        mdx_esm_parse: Some(Box::new(parse_esm)),
        mdx_expression_parse: Some(Box::new(parse_expression)),
        ..ParseOptions::mdx()
    };

    assert_serde(
        "<a b c={d} e=\"f\" {...g} />",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "mdxJsxFlowElement",
      "name": "a",
      "attributes": [
        {"type": "mdxJsxAttribute", "name": "b"},
        {
          "type": "mdxJsxAttribute",
          "name": "c",
          "value": {
            "_markdownRsStops": [[0, 8]],
            "type": "mdxJsxAttributeValueExpression",
            "value": "d"
          }
        },
        {"type": "mdxJsxAttribute", "name": "e", "value": "f"},
        {
          "_markdownRsStops": [[0, 18]],
          "type": "mdxJsxExpressionAttribute",
          "value": "...g"
        }
      ],
      "children": []
    }
  ]
}"#,
        options,
    )
}

#[test]
fn serde_list() -> Result<(), Error> {
    assert_serde(
        "* a",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "list",
      "ordered": false,
      "spread": false,
      "children": [
        {
          "type": "listItem",
          "spread": false,
          "children": [
            {
              "type": "paragraph",
              "children": [{"type": "text", "value": "a"}]
            }
          ]
        }
      ]
    }
  ]
}"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_mdxjs_esm() -> Result<(), Error> {
    let options = ParseOptions {
        mdx_esm_parse: Some(Box::new(parse_esm)),
        mdx_expression_parse: Some(Box::new(parse_expression)),
        ..ParseOptions::mdx()
    };

    assert_serde(
        "import a, {b} from 'c'",
        r#"{
  "type": "root",
  "children": [
    {
      "_markdownRsStops": [[0, 0]],
      "type": "mdxjsEsm",
      "value": "import a, {b} from 'c'"
    }
  ]
}"#,
        options,
    )
}

#[test]
fn serde_toml() -> Result<(), Error> {
    assert_serde(
        "+++\na: b\n+++",
        r#"{
  "type": "root",
  "children": [{"type": "toml", "value": "a: b"}]
}"#,
        ParseOptions {
            constructs: Constructs {
                frontmatter: true,
                ..Constructs::default()
            },
            ..ParseOptions::default()
        },
    )
}

#[test]
fn serde_yaml() -> Result<(), Error> {
    assert_serde(
        "---\na: b\n---",
        r#"{
  "type": "root",
  "children": [{"type": "yaml", "value": "a: b"}]
}"#,
        ParseOptions {
            constructs: Constructs {
                frontmatter: true,
                ..Constructs::default()
            },
            ..ParseOptions::default()
        },
    )
}

#[test]
fn serde_break() -> Result<(), Error> {
    assert_serde(
        "a\\\nb",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {"type": "text", "value": "a"},
        {"type": "break"},
        {"type": "text", "value": "b"}
      ]
    }
  ]
}"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_inline_code() -> Result<(), Error> {
    assert_serde(
        "`a`",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [{"type": "inlineCode", "value": "a"}]
    }
  ]
}"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_inline_math() -> Result<(), Error> {
    assert_serde(
        "$a$",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [{"type": "inlineMath","value": "a"}]
    }
  ]
}"#,
        ParseOptions {
            constructs: Constructs {
                math_text: true,
                ..Constructs::default()
            },
            ..ParseOptions::default()
        },
    )
}

#[test]
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
          "children": [{"type": "text","value": "a"}]
        }
      ]
    }
  ]
}"#,
        ParseOptions::gfm(),
    )
}

#[test]
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
          "children": [{"type": "text","value": "a"}]
        }
      ]
    }
  ]
}"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_mdx_text_expression() -> Result<(), Error> {
    let options = ParseOptions {
        mdx_esm_parse: Some(Box::new(parse_esm)),
        mdx_expression_parse: Some(Box::new(parse_expression)),
        ..ParseOptions::mdx()
    };

    assert_serde(
        "a {b}",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {"type": "text","value": "a "},
        {
          "_markdownRsStops": [[0,3]],
          "type": "mdxTextExpression",
          "value": "b"
        }
      ]
    }
  ]
}"#,
        options,
    )
}

#[test]
fn serde_footnote_reference() -> Result<(), Error> {
    assert_serde(
        "[^a]\n\n[^a]: b",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {"type": "footnoteReference", "identifier": "a", "label": "a"}
      ]
    },
    {
      "type": "footnoteDefinition",
      "identifier": "a",
      "label": "a",
      "children": [
        {
          "type": "paragraph",
          "children": [{"type": "text", "value": "b"}]
        }
      ]
    }
  ]
}"#,
        ParseOptions::gfm(),
    )
}

#[test]
fn serde_html() -> Result<(), Error> {
    assert_serde(
        "<a>",
        r#"{
  "type": "root",
  "children": [{"type": "html", "value": "<a>"}]
}"#,
        ParseOptions::gfm(),
    )
}

#[test]
fn serde_image() -> Result<(), Error> {
    assert_serde(
        "![a](b)",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [{"type": "image", "url": "b", "alt": "a"}]
    }
  ]
}"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_image_reference() -> Result<(), Error> {
    assert_serde(
        "![a]\n\n[a]: b",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "imageReference",
          "alt": "a",
          "label": "a",
          "identifier": "a",
          "referenceType": "shortcut"
        }
      ]
    },
    {"type": "definition", "url": "b", "identifier": "a", "label": "a"}
  ]
}"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_mdx_jsx_text_element() -> Result<(), Error> {
    assert_serde(
        "a <b c />",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {"type": "text", "value": "a "},
        {
          "type": "mdxJsxTextElement",
          "name": "b",
          "attributes": [{"type": "mdxJsxAttribute", "name": "c"}],
          "children": []
        }
      ]
    }
  ]
}"#,
        ParseOptions::mdx(),
    )
}

#[test]
fn serde_link() -> Result<(), Error> {
    assert_serde(
        "[a](b)",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "link",
          "url": "b",
          "children": [{"type": "text", "value": "a"}]
        }
      ]
    }
  ]
}"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_link_reference() -> Result<(), Error> {
    assert_serde(
        "[a]\n\n[a]: b",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "linkReference",
          "identifier": "a",
          "label": "a",
          "referenceType": "shortcut",
          "children": [{"type": "text", "value": "a"}]
        }
      ]
    },
    {
      "type": "definition",
      "url": "b",
      "identifier": "a",
      "label": "a"
    }
  ]
}"#,
        ParseOptions::default(),
    )
}

#[test]
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
          "children": [{"type": "text", "value": "a"}]
        }
      ]
    }
  ]
}"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_text() -> Result<(), Error> {
    assert_serde(
        "a",
        r#"{
  "type": "root",
  "children": [
    {"type": "paragraph", "children": [{"type": "text", "value": "a"}]}
  ]
}"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_code() -> Result<(), Error> {
    assert_serde(
        "~~~js eval\nconsole.log(1)\n~~~",
        r#"{
  "type": "root",
  "children": [
    {"type": "code", "lang": "js", "meta": "eval", "value": "console.log(1)"}
  ]
}"#,
        ParseOptions::default(),
    )?;

    assert_serde(
        "```\nconsole.log(1)\n```",
        r#"{
  "type": "root",
  "children": [{"type": "code", "value": "console.log(1)"}]
}"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_math() -> Result<(), Error> {
    assert_serde(
        "$$\n1 + 1 = 2\n$$",
        r#"{
  "type": "root",
  "children": [{"type": "math", "value": "1 + 1 = 2"}]
}"#,
        ParseOptions {
            constructs: Constructs {
                math_flow: true,
                ..Constructs::default()
            },
            ..ParseOptions::default()
        },
    )
}

#[test]
fn serde_mdx_flow_expression() -> Result<(), Error> {
    let options = ParseOptions {
        mdx_esm_parse: Some(Box::new(parse_esm)),
        mdx_expression_parse: Some(Box::new(parse_expression)),
        ..ParseOptions::mdx()
    };

    assert_serde(
        "{a}",
        r#"{
  "type": "root",
  "children": [
    {"_markdownRsStops": [[0, 1]], "type": "mdxFlowExpression", "value": "a"}
  ]
}"#,
        options,
    )
}

#[test]
fn serde_heading() -> Result<(), Error> {
    assert_serde(
        "# a",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "heading",
      "depth": 1,
      "children": [{"type": "text", "value": "a"}]
    }
  ]
}"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_table() -> Result<(), Error> {
    // To do: `"none"` should serialize in serde as `null`.
    assert_serde(
        "| a | b | c | d |\n| - | :- | -: | :-: |\n| 1 | 2 | 3 | 4 |",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "table",
      "align": [null, "left", "right", "center"],
      "children": [
        {
          "type": "tableRow",
          "children": [
            {"type": "tableCell", "children": [{"type": "text", "value": "a"}]},
            {"type": "tableCell", "children": [{"type": "text", "value": "b"}]},
            {"type": "tableCell", "children": [{"type": "text", "value": "c"}]},
            {"type": "tableCell", "children": [{"type": "text", "value": "d"}]}
          ]
        },
        {
          "type": "tableRow",
          "children": [
            {"type": "tableCell","children": [{"type": "text", "value": "1"}]},
            {"type": "tableCell","children": [{"type": "text", "value": "2"}]},
            {"type": "tableCell","children": [{"type": "text", "value": "3"}]},
            {"type": "tableCell","children": [{"type": "text", "value": "4"}]}
          ]
        }
      ]
    }
  ]
}"#,
        ParseOptions::gfm(),
    )
}

#[test]
fn serde_thematic_break() -> Result<(), Error> {
    assert_serde(
        "***",
        r#"{"type": "root", "children": [{"type": "thematicBreak"}]}"#,
        ParseOptions::default(),
    )
}

#[test]
fn serde_definition() -> Result<(), Error> {
    assert_serde(
        "[a]: b",
        r###"{
  "type": "root",
  "children": [
    {"type": "definition", "url": "b", "identifier": "a", "label": "a"}
  ]
}"###,
        ParseOptions::default(),
    )
}

#[test]
fn serde_paragraph() -> Result<(), Error> {
    assert_serde(
        "a",
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [{"type": "text", "value": "a"}]
    }
  ]
}"#,
        ParseOptions::default(),
    )
}

/// Assert serde of mdast constructs.
///
/// Refer below links for the mdast JSON construct types.
/// * <https://github.com/syntax-tree/mdast#nodes>
/// * <https://github.com/syntax-tree/mdast-util-mdx#syntax-tree>
/// * <https://github.com/syntax-tree/mdast-util-frontmatter#syntax-tree>
#[cfg(feature = "serde")]
fn assert_serde(input: &str, expected: &str, options: ParseOptions) -> Result<(), Error> {
    use pretty_assertions::assert_eq;

    let mut source = markdown::to_mdast(input, &options).map_err(Error::Mdast)?;

    remove_position(&mut source);
    // Serialize to JSON
    let actual_value: serde_json::Value = serde_json::to_value(&source).map_err(Error::Serde)?;
    let expected_value: serde_json::Value = serde_json::from_str(expected).map_err(Error::Serde)?;

    // Assert serialization.
    assert_eq!(actual_value, expected_value);

    // Assert deserialization.
    assert_eq!(
        source,
        serde_json::from_value(actual_value).map_err(Error::Serde)?
    );

    Ok(())
}

#[cfg(not(feature = "serde"))]
#[allow(unused_variables)]
fn assert_serde(input: &str, expected: &str, options: ParseOptions) -> Result<(), Error> {
    Ok(())
}

#[allow(dead_code)]
fn remove_position(node: &mut Node) {
    if let Some(children) = node.children_mut() {
        for child in children {
            remove_position(child);
        }
    }

    node.position_set(None);
}
