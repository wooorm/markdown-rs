mod test_utils;

#[allow(unused)]
#[derive(Debug)]
enum Error {
    Assert(String),
    Mdast(markdown::message::Message),
    Serde(serde_json::Error),
}

#[cfg_attr(feature = "serde", test)]
fn serde_root() -> Result<(), Error> {
    let source = r#"---
title: Serde
---

import Test from 'test';
import Inner from 'test';
import {Another, YetAnother} from 'another';

# <HelloMessage />, {username}!

> Blockquote
Add test constructs below!

## Test serialization and deserialization of mdast

<Test id={id} name="test">
  <Inner name="inner" id={id}>
    ## Inner
    [Link](./link.md)
  </Inner>
</Test>

<Another id={id} class="test" />

{test} this is text expression

<YetAnother id={id} class="test" />

# Text

~~The world is flat.~~ We now know that the world is round.

*Emphasis*

*Strong Emphasis*

$This is math$

Let's break\
yes!

***

## List

* item1
* item2
* item3

## Code block

```shell
cargo test --features json
```

## Inline

`Inline code` with backticks

## Image

![Image](http://url/a.png)

## Table

| Syntax    | Description |
|-----------|-------------|
| Header    | Title       |
| Paragraph | Text        |

## Task lists

- [x] Write the press release
- [ ] Update the website
- [ ] Contact the media

## Footnotes

Here's a simple footnote,[^1] and here's a longer one.[^bignote]

[^1]: This is the first footnote.

[^bignote]: Here's one with multiple paragraphs and code.

    Indent paragraphs to include them in the footnote.

    `{ my code }`

    Add as many paragraphs as you like.

"#;

    assert_jq(source, ".type", "root", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_blockquote() -> Result<(), Error> {
    let source = r#"> a
"#;
    assert_jq(source, ".children[0].type", "blockquote", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_footnote_definition() -> Result<(), Error> {
    assert_jq("[^a]: b", ".children[0].type", "footnoteDefinition", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_mdx_jsx_flow_element() -> Result<(), Error> {
    assert_jq("<a />", ".children[0].type", "mdxJsxFlowElement", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_mdx_jsx_flow_element_attributes() -> Result<(), Error> {
    assert_jq(
        "<a {...b}/>",
        ".children[0].attributes[0].value",
        "...b",
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_mdx_jsx_flow_element_attributes_expressions() -> Result<(), Error> {
    let source = r#"<Test id={id} class="test" />"#;
    assert_jq(
        source,
        ".children[0].attributes[0].type",
        "mdxJsxAttribute",
        None,
    )?;
    assert_jq(source, ".children[0].attributes[0].name", "id", None)?;
    assert_jq(
        source,
        ".children[0].attributes[0].value.type",
        "mdxJsxAttributeValueExpression",
        None,
    )?;
    assert_jq(source, ".children[0].attributes[1].name", "class", None)?;
    assert_jq(source, ".children[0].attributes[1].value", "test", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_list() -> Result<(), Error> {
    assert_jq("* a", ".children[0].type", "list", None)?;
    assert_jq("* a", ".children[0].children[0].type", "listItem", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_mdxjs_esm() -> Result<(), Error> {
    let source = r#"
import Test from 'test';
"#;
    assert_jq(source, ".children[0].type", "mdxjsEsm", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_toml() -> Result<(), Error> {
    let source = r#"+++
a: b
+++
"#;
    assert_jq(source, ".children[0].type", "toml", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_yaml() -> Result<(), Error> {
    let source = r#"---
a: b
---
"#;
    assert_jq(source, ".children[0].type", "yaml", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_break() -> Result<(), Error> {
    let source = r#"a\
b
"#;
    assert_jq(source, ".children[0].children[1].type", "break", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_inline_code() -> Result<(), Error> {
    assert_jq("`a`", ".children[0].children[0].type", "inlineCode", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_inline_math() -> Result<(), Error> {
    assert_jq("$a$", ".children[0].children[0].type", "inlineMath", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_delete() -> Result<(), Error> {
    assert_jq("~~a~~", ".children[0].children[0].type", "delete", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_emphasis() -> Result<(), Error> {
    assert_jq("*a*", ".children[0].children[0].type", "emphasis", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_mdx_text_expression() -> Result<(), Error> {
    assert_jq(
        "a {b}",
        ".children[0].children[1].type",
        "mdxTextExpression",
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_footnote_reference() -> Result<(), Error> {
    let source = r#"Refer to [^a]
[^a]: b
"#;
    assert_jq(
        source,
        ".children[0].children[1].type",
        "footnoteReference",
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_html() -> Result<(), Error> {
    assert_jq(
        "<a>",
        ".children[0].type",
        "html",
        Some(markdown::ParseOptions {
            constructs: markdown::Constructs::gfm(),
            ..markdown::ParseOptions::gfm()
        }),
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_image() -> Result<(), Error> {
    assert_jq("![a](b)", ".children[0].children[0].type", "image", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_image_reference() -> Result<(), Error> {
    let source = r#"[x]: y
a ![x] b
"#;
    assert_jq(
        source,
        ".children[1].children[1].type",
        "imageReference",
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_mdx_jsx_text_element() -> Result<(), Error> {
    assert_jq(
        "text <b />",
        ".children[0].children[1].type",
        "mdxJsxTextElement",
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_link() -> Result<(), Error> {
    assert_jq("link [a](b)", ".children[0].children[1].type", "link", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_link_reference() -> Result<(), Error> {
    let source = r#"[x]: y
a [x] b
"#;
    assert_jq(
        source,
        ".children[1].children[1].type",
        "linkReference",
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_strong() -> Result<(), Error> {
    assert_jq("**a**", ".children[0].children[0].type", "strong", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_text() -> Result<(), Error> {
    assert_jq("a", ".children[0].children[0].type", "text", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_code() -> Result<(), Error> {
    let source = r#"~~~
let a = b;
~~~
"#;
    assert_jq(source, ".children[0].type", "code", None)?;
    let source = r#"```
let a = b;
```
"#;
    assert_jq(source, ".children[0].type", "code", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_math() -> Result<(), Error> {
    let source = r#"$$
1 + 1 = 2
$$
"#;
    assert_jq(source, ".children[0].type", "math", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_mdx_flow_expression() -> Result<(), Error> {
    assert_jq("{a}", ".children[0].type", "mdxFlowExpression", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_heading() -> Result<(), Error> {
    assert_jq("# a", ".children[0].type", "heading", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_table() -> Result<(), Error> {
    let source = r#"| a   | b   |
|-----|-----|
| c11 | c12 |
| c21 | c22 |
"#;
    assert_jq(source, ".children[0].type", "table", None)?;
    assert_jq(source, ".children[0].children[0].type", "tableRow", None)?;
    assert_jq(
        source,
        ".children[0].children[0].children[0].type",
        "tableCell",
        None,
    )
}

#[cfg_attr(feature = "serde", test)]
fn serde_thematic_break() -> Result<(), Error> {
    assert_jq("***", ".children[0].type", "thematicBreak", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_definition() -> Result<(), Error> {
    assert_jq("[a]: b", ".children[0].type", "definition", None)
}

#[cfg_attr(feature = "serde", test)]
fn serde_paragraph() -> Result<(), Error> {
    assert_jq("a", ".children[0].type", "paragraph", None)
}

/// Assert serde of Mdast constructs
#[cfg(feature = "serde")]
fn assert_jq(
    input: &str,
    query: &str,
    expected: &str,
    options: Option<markdown::ParseOptions>,
) -> Result<(), Error> {
    use jaq_core::{load, Ctx, Native, RcIter};
    use jaq_json::Val;
    use load::{Arena, File, Loader};
    use serde_json::{json, Value};
    // Parse Mdast with default options of MDX and GFM
    use test_utils::swc::{parse_esm, parse_expression};
    let source = markdown::to_mdast(
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
    // Serialize to JSON
    let source_value: Value = serde_json::to_value(&source).map_err(Error::Serde)?;
    /*
    println!(
        "{}",
        serde_json::to_string_pretty(&source).map_err(Error::Serde)?
    );
     */
    // Parse JSON with JQL using https://github.com/01mf02/jaq
    let program = File {
        path: "".into(),
        code: query,
    };
    let loader = Loader::new([]);
    let arena = Arena::default();
    let modules = loader.load(&arena, program).unwrap();
    let filter = jaq_core::Compiler::<_, Native<_>>::default()
        .compile(modules)
        .unwrap();
    let inputs = RcIter::new(core::iter::empty());
    let mut output = filter.run((Ctx::new([], &inputs), source_value.clone().into()));
    // Assert serialization of construct
    pretty_assertions::assert_eq!(output.next(), Some(Ok(Val::from(json!(expected)))));
    pretty_assertions::assert_eq!(output.next(), None);
    // Assert deserialization
    pretty_assertions::assert_eq!(
        source,
        serde_json::from_value(source_value).map_err(Error::Serde)?
    );
    Ok(())
}
