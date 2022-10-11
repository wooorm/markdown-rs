extern crate micromark;
use micromark::{
    mdast::{List, ListItem, Node, Paragraph, Root, Text},
    micromark, micromark_to_mdast, micromark_with_options,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn gfm_task_list_item() -> Result<(), String> {
    let gfm = Options {
        parse: ParseOptions {
            constructs: Constructs::gfm(),
            ..ParseOptions::default()
        },
        ..Options::default()
    };

    assert_eq!(
        micromark("* [x] y."),
        "<ul>\n<li>[x] y.</li>\n</ul>",
        "should ignore task list item checks by default"
    );

    assert_eq!(
        micromark_with_options("* [x] y.", &gfm)?,
        "<ul>\n<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> y.</li>\n</ul>",
        "should support task list item checks"
    );

    assert_eq!(
        micromark_with_options("* [ ] z.", &gfm)?,
        "<ul>\n<li><input type=\"checkbox\" disabled=\"\" /> z.</li>\n</ul>",
        "should support unchecked task list item checks"
    );

    assert_eq!(
        micromark_with_options("*\n    [x]", &gfm)?,
        "<ul>\n<li>[x]</li>\n</ul>",
        "should not support laziness (1)"
    );

    assert_eq!(
        micromark_with_options("*\n[x]", &gfm)?,
        "<ul>\n<li></li>\n</ul>\n<p>[x]</p>",
        "should not support laziness (2)"
    );

    assert_eq!(
        micromark_with_options(
            &r###"
* [ ] foo
* [x] bar

- [x] foo
  - [ ] bar
  - [x] baz
- [ ] bim

+ [ ] Unchecked?

* [x] Checked?

+ [y] What is this even?

- [n]: #
  [ ] After a definition

+ [ ] In a setext heading
  =======================

* In the…

  [ ] Second paragraph

- [	] With a tab

+ [X] With an upper case `x`

* [
] In a lazy line

- [  ] With two spaces

+  [x] Two spaces indent

*   [x] Three spaces indent

-    [x] Four spaces indent

+     [x] Five spaces indent

[ ] here?

* > [ ] here?

- [ ]No space?

Empty?

+ [ ]

Space after:

+ [ ]␠

* [ ]␠Text.

Tab after:

+ [ ]␉

* [ ]␉Text.

EOL after:

+ [ ]

* [ ]
  Text.

-
  [ ] after blank?

+ # [ ] ATX Heading

> * [x] In a list in a block quote
"###
            .replace('␠', " ")
            .replace('␉', "\t"),
            &gfm
        )?,
        r###"<ul>
<li><input type="checkbox" disabled="" /> foo</li>
<li><input type="checkbox" disabled="" checked="" /> bar</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" checked="" /> foo
<ul>
<li><input type="checkbox" disabled="" /> bar</li>
<li><input type="checkbox" disabled="" checked="" /> baz</li>
</ul>
</li>
<li><input type="checkbox" disabled="" /> bim</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" /> Unchecked?</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" checked="" /> Checked?</li>
</ul>
<ul>
<li>[y] What is this even?</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" /> After a definition</li>
</ul>
<ul>
<li>
<h1>[ ] In a setext heading</h1>
</li>
</ul>
<ul>
<li>
<p>In the…</p>
<p>[ ] Second paragraph</p>
</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" /> With a tab</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" checked="" /> With an upper case <code>x</code></li>
</ul>
<ul>
<li><input type="checkbox" disabled="" /> In a lazy line</li>
</ul>
<ul>
<li>[  ] With two spaces</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" checked="" /> Two spaces indent</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" checked="" /> Three spaces indent</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" checked="" /> Four spaces indent</li>
</ul>
<ul>
<li>
<pre><code>[x] Five spaces indent
</code></pre>
</li>
</ul>
<p>[ ] here?</p>
<ul>
<li>
<blockquote>
<p>[ ] here?</p>
</blockquote>
</li>
</ul>
<ul>
<li>[ ]No space?</li>
</ul>
<p>Empty?</p>
<ul>
<li>[ ]</li>
</ul>
<p>Space after:</p>
<ul>
<li>[ ]</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" /> Text.</li>
</ul>
<p>Tab after:</p>
<ul>
<li>[ ]</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" />	Text.</li>
</ul>
<p>EOL after:</p>
<ul>
<li>[ ]</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" />
Text.</li>
</ul>
<ul>
<li><input type="checkbox" disabled="" /> after blank?</li>
</ul>
<ul>
<li>
<h1>[ ] ATX Heading</h1>
</li>
</ul>
<blockquote>
<ul>
<li><input type="checkbox" disabled="" checked="" /> In a list in a block quote</li>
</ul>
</blockquote>
"###,
        "should handle things like GitHub"
    );

    assert_eq!(
        micromark_to_mdast("* [x] a\n* [ ] b\n* c", &gfm.parse)?,
        Node::Root(Root {
            children: vec![Node::List(List {
                ordered: false,
                spread: false,
                start: None,
                children: vec![
                    Node::ListItem(ListItem {
                        checked: Some(true),
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "a".into(),
                                position: Some(Position::new(1, 7, 6, 1, 8, 7))
                            }),],
                            position: Some(Position::new(1, 7, 6, 1, 8, 7))
                        })],
                        position: Some(Position::new(1, 1, 0, 1, 8, 7))
                    }),
                    Node::ListItem(ListItem {
                        checked: Some(false),
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "b".into(),
                                position: Some(Position::new(2, 7, 14, 2, 8, 15))
                            }),],
                            position: Some(Position::new(2, 7, 14, 2, 8, 15))
                        })],
                        position: Some(Position::new(2, 1, 8, 2, 8, 15))
                    }),
                    Node::ListItem(ListItem {
                        checked: None,
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "c".into(),
                                position: Some(Position::new(3, 3, 18, 3, 4, 19))
                            }),],
                            position: Some(Position::new(3, 3, 18, 3, 4, 19))
                        })],
                        position: Some(Position::new(3, 1, 16, 3, 4, 19))
                    }),
                ],
                position: Some(Position::new(1, 1, 0, 3, 4, 19))
            })],
            position: Some(Position::new(1, 1, 0, 3, 4, 19))
        }),
        "should support task list items as `checked` fields on `ListItem`s in mdast"
    );

    Ok(())
}
