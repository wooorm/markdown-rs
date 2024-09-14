use markdown::mdast::Node;
use markdown::message::Message;

#[allow(unused)]
#[derive(Debug)]
enum Error {
    Mdast(Message),
    Serde(serde_json::Error),
}

#[cfg_attr(feature = "serde", test)]
fn serde() -> Result<(), Error> {
    let source = markdown::to_mdast(
        r#"
---
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

"#,
        &markdown::ParseOptions::mdx(),
    )
    .map_err(Error::Mdast)?;

    let value: String = serde_json::to_string(&source).map_err(Error::Serde)?;

    let target: Node = serde_json::from_slice(value.as_bytes()).map_err(Error::Serde)?;

    pretty_assertions::assert_eq!(source, target);

    Ok(())
}
