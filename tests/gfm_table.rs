use markdown::{
    mdast::{AlignKind, InlineCode, Node, Root, Table, TableCell, TableRow, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    CompileOptions, Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn gfm_table() -> Result<(), message::Message> {
    assert_eq!(
        to_html("| a |\n| - |\n| b |"),
        "<p>| a |\n| - |\n| b |</p>",
        "should ignore tables by default"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\n| b |", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>b</td>\n</tr>\n</tbody>\n</table>",
        "should support tables"
    );

    assert_eq!(
        to_html_with_options("| a |", &Options::gfm())?,
        "<p>| a |</p>",
        "should not support a table w/ the head row ending in an eof (1)"
    );

    assert_eq!(
        to_html_with_options("| a", &Options::gfm())?,
        "<p>| a</p>",
        "should not support a table w/ the head row ending in an eof (2)"
    );

    assert_eq!(
        to_html_with_options("a |", &Options::gfm())?,
        "<p>a |</p>",
        "should not support a table w/ the head row ending in an eof (3)"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>",
        "should support a table w/ a delimiter row ending in an eof (1)"
    );

    assert_eq!(
        to_html_with_options("| a\n| -", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>",
        "should support a table w/ a delimiter row ending in an eof (2)"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\n| b |", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>b</td>\n</tr>\n</tbody>\n</table>",
        "should support a table w/ a body row ending in an eof (1)"
    );

    assert_eq!(
        to_html_with_options("| a\n| -\n| b", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>b</td>\n</tr>\n</tbody>\n</table>",
        "should support a table w/ a body row ending in an eof (2)"
    );

    assert_eq!(
        to_html_with_options("a|b\n-|-\nc|d", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>c</td>\n<td>d</td>\n</tr>\n</tbody>\n</table>",
        "should support a table w/ a body row ending in an eof (3)"
    );

    assert_eq!(
        to_html_with_options("| a  \n| -\t\n| b |     ", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>b</td>\n</tr>\n</tbody>\n</table>",
        "should support rows w/ trailing whitespace (1)"
    );

    assert_eq!(
        to_html_with_options("| a | \n| - |", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>",
        "should support rows w/ trailing whitespace (2)"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - | ", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>",
        "should support rows w/ trailing whitespace (3)"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\n| b | ", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>b</td>\n</tr>\n</tbody>\n</table>",
        "should support rows w/ trailing whitespace (4)"
    );

    assert_eq!(
        to_html_with_options("||a|\n|-|-|", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th></th>\n<th>a</th>\n</tr>\n</thead>\n</table>",
        "should support empty first header cells"
    );

    assert_eq!(
        to_html_with_options("|a||\n|-|-|", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n<th></th>\n</tr>\n</thead>\n</table>",
        "should support empty last header cells"
    );

    assert_eq!(
        to_html_with_options("a||b\n-|-|-", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n<th></th>\n<th>b</th>\n</tr>\n</thead>\n</table>",
        "should support empty header cells"
    );

    assert_eq!(
        to_html_with_options("|a|b|\n|-|-|\n||c|", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td></td>\n<td>c</td>\n</tr>\n</tbody>\n</table>",
        "should support empty first body cells"
    );

    assert_eq!(
        to_html_with_options("|a|b|\n|-|-|\n|c||", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>c</td>\n<td></td>\n</tr>\n</tbody>\n</table>",
        "should support empty last body cells"
    );

    assert_eq!(
        to_html_with_options("a|b|c\n-|-|-\nd||e", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n<th>c</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>d</td>\n<td></td>\n<td>e</td>\n</tr>\n</tbody>\n</table>",
        "should support empty body cells"
    );

    assert_eq!(
        to_html_with_options(":\n|-|\n|a|\n\nb\n|-|\n|c|\n\n|\n|-|\n|d|\n\n|\n|-|\n|e|\n\n|:\n|-|\n|f|\n\n||\n|-|\n|g|\n\n| |\n|-|\n|h|\n", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>:</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>a</td>\n</tr>\n</tbody>\n</table>\n<table>\n<thead>\n<tr>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>c</td>\n</tr>\n</tbody>\n</table>\n<p>|\n|-|\n|d|</p>\n<p>|\n|-|\n|e|</p>\n<table>\n<thead>\n<tr>\n<th>:</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>f</td>\n</tr>\n</tbody>\n</table>\n<table>\n<thead>\n<tr>\n<th></th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>g</td>\n</tr>\n</tbody>\n</table>\n<table>\n<thead>\n<tr>\n<th></th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>h</td>\n</tr>\n</tbody>\n</table>\n",
        "should need any character other than a single pipe in the header row"
    );

    assert_eq!(
        to_html_with_options("a\n|-\n\nb\n||\n\nc\n|-|\n\nd\n|:|\n\ne\n| |\n\nf\n| -|\n\ng\n|- |\n", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n<p>b\n||</p>\n<table>\n<thead>\n<tr>\n<th>c</th>\n</tr>\n</thead>\n</table>\n<p>d\n|:|</p>\n<p>e\n| |</p>\n<table>\n<thead>\n<tr>\n<th>f</th>\n</tr>\n</thead>\n</table>\n<table>\n<thead>\n<tr>\n<th>g</th>\n</tr>\n</thead>\n</table>\n",
        "should need a dash in the delimimter row"
    );

    assert_eq!(
        to_html_with_options("|\n|", &Options::gfm())?,
        "<p>|\n|</p>",
        "should need something"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\n- b", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n<ul>\n<li>b</li>\n</ul>",
        "should support a list after a table"
    );

    assert_eq!(
        to_html_with_options("> | a |\n| - |", &Options::gfm())?,
        "<blockquote>\n<p>| a |\n| - |</p>\n</blockquote>",
        "should not support a lazy delimiter row (1)"
    );

    assert_eq!(
        to_html_with_options("> a\n> | b |\n| - |", &Options::gfm())?,
        "<blockquote>\n<p>a\n| b |\n| - |</p>\n</blockquote>",
        "should not support a lazy delimiter row (2)"
    );

    assert_eq!(
        to_html_with_options("| a |\n> | - |", &Options::gfm())?,
        "<p>| a |</p>\n<blockquote>\n<p>| - |</p>\n</blockquote>",
        "should not support a piercing delimiter row"
    );

    assert_eq!(
        to_html_with_options("> a\n> | b |\n|-", &Options::gfm())?,
        "<blockquote>\n<p>a\n| b |\n|-</p>\n</blockquote>",
        "should not support a lazy body row (2)"
    );

    assert_eq!(
        to_html_with_options("> | a |\n> | - |\n| b |", &Options::gfm())?,
        "<blockquote>\n<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n</blockquote>\n<p>| b |</p>",
        "should not support a lazy body row (1)"
    );

    assert_eq!(
        to_html_with_options("> a\n> | b |\n> | - |\n| c |", &Options::gfm())?,
        "<blockquote>\n<p>a</p>\n<table>\n<thead>\n<tr>\n<th>b</th>\n</tr>\n</thead>\n</table>\n</blockquote>\n<p>| c |</p>",
        "should not support a lazy body row (2)"
    );

    assert_eq!(
        to_html_with_options("> | A |\n> | - |\n> | 1 |\n| 2 |", &Options::gfm())?,
        "<blockquote>\n<table>\n<thead>\n<tr>\n<th>A</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>1</td>\n</tr>\n</tbody>\n</table>\n</blockquote>\n<p>| 2 |</p>",
        "should not support a lazy body row (3)"
    );

    assert_eq!(
        to_html_with_options("   - d\n    - e", &Options::gfm())?,
        to_html("   - d\n    - e"),
        "should not change how lists and lazyness work"
    );

    assert_eq!(
        to_html_with_options("| a |\n   | - |", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>",
        "should form a table if the delimiter row is indented w/ 3 spaces"
    );

    assert_eq!(
        to_html_with_options("| a |\n    | - |", &Options::gfm())?,
        "<p>| a |\n| - |</p>",
        "should not form a table if the delimiter row is indented w/ 4 spaces"
    );

    assert_eq!(
        to_html_with_options("| a |\n    | - |", &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        code_indented: false,
                        ..Constructs::gfm()
                    },
                    ..ParseOptions::gfm()
                },
                ..Options::gfm()
            })?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>",
        "should form a table if the delimiter row is indented w/ 4 spaces and indented code is turned off"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\n> block quote?", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n<blockquote>\n<p>block quote?</p>\n</blockquote>",
        "should be interrupted by a block quote"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\n>", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n<blockquote>\n</blockquote>",
        "should be interrupted by a block quote (empty)"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\n- list?", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n<ul>\n<li>list?</li>\n</ul>",
        "should be interrupted by a list"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\n-", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n<ul>\n<li></li>\n</ul>",
        "should be interrupted by a list (empty)"
    );

    assert_eq!(
        to_html_with_options(
            "| a |\n| - |\n<!-- HTML? -->",
            &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..CompileOptions::gfm()
                },
                ..Options::gfm()
            }
        )?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n<!-- HTML? -->",
        "should be interrupted by HTML (flow)"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\n\tcode?", &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..CompileOptions::gfm()
                },
                ..Options::gfm()
            })?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n<pre><code>code?\n</code></pre>",
        "should be interrupted by code (indented)"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\n```js\ncode?", &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..CompileOptions::gfm()
                },
                ..Options::gfm()
            })?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n<pre><code class=\"language-js\">code?\n</code></pre>\n",
        "should be interrupted by code (fenced)"
    );

    assert_eq!(
        to_html_with_options(
            "| a |\n| - |\n***",
            &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..CompileOptions::gfm()
                },
                ..Options::gfm()
            }
        )?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n<hr />",
        "should be interrupted by a thematic break"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\n# heading?", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n<h1>heading?</h1>",
        "should be interrupted by a heading (ATX)"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\nheading\n=", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>heading</td>\n</tr>\n<tr>\n<td>=</td>\n</tr>\n</tbody>\n</table>",
        "should *not* be interrupted by a heading (setext)"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\nheading\n---", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>heading</td>\n</tr>\n</tbody>\n</table>\n<hr />",
        "should *not* be interrupted by a heading (setext), but interrupt if the underline is also a thematic break"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\nheading\n-", &Options::gfm())?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>heading</td>\n</tr>\n</tbody>\n</table>\n<ul>\n<li></li>\n</ul>",
        "should *not* be interrupted by a heading (setext), but interrupt if the underline is also an empty list item bullet"
    );

    assert_eq!(
        to_html_with_options("a\nb\n-:", &Options::gfm())?,
        "<p>a</p>\n<table>\n<thead>\n<tr>\n<th align=\"right\">b</th>\n</tr>\n</thead>\n</table>",
        "should support a single head row"
    );

    assert_eq!(
        to_html_with_options("> | a |\n> | - |", &Options::gfm())?,
        "<blockquote>\n<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n</blockquote>",
        "should support a table in a container"
    );

    assert_eq!(
        to_html_with_options("> | a |\n| - |", &Options::gfm())?,
        "<blockquote>\n<p>| a |\n| - |</p>\n</blockquote>",
        "should not support a lazy delimiter row if the head row is in a container"
    );

    assert_eq!(
        to_html_with_options("| a |\n> | - |", &Options::gfm())?,
        "<p>| a |</p>\n<blockquote>\n<p>| - |</p>\n</blockquote>",
        "should not support a “piercing” container for the delimiter row, if the head row was not in that container"
    );

    assert_eq!(
        to_html_with_options("> | a |\n> | - |\n| c |", &Options::gfm())?,
        "<blockquote>\n<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n</blockquote>\n<p>| c |</p>",
        "should not support a lazy body row if the head row and delimiter row are in a container"
    );

    assert_eq!(
        to_html_with_options("> | a |\n| - |\n> | c |", &Options::gfm())?,
        "<blockquote>\n<p>| a |\n| - |\n| c |</p>\n</blockquote>",
        "should not support a lazy delimiter row if the head row and a further body row are in a container"
    );

    assert_eq!(
        to_html_with_options("[\na\n:-\n]: b", &Options::gfm())?,
        "<p>[</p>\n<table>\n<thead>\n<tr>\n<th align=\"left\">a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td align=\"left\">]: b</td>\n</tr>\n</tbody>\n</table>",
        "should prefer GFM tables over definitions"
    );

    assert_eq!(
        to_html_with_options("    | a |\n\t| - |\n    | b |", &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        code_indented: false,
                        ..Constructs::gfm()
                    },
                    ..ParseOptions::gfm()
                },
                ..Options::gfm()
            })?,
        "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>b</td>\n</tr>\n</tbody>\n</table>",
        "should support indented rows if code (indented) is off"
    );

    assert_eq!(
        to_html_with_options(
            r###"# Align

## An empty initial cell

| | a|c|
|--|:----:|:---|
|a|b|c|
|a|b|c|

## Missing alignment characters

| a | b | c |
|   |---|---|
| d | e | f |

* * *

| a | b | c |
|---|---|   |
| d | e | f |

## Incorrect characters

| a | b | c |
|---|-*-|---|
| d | e | f |

## Two alignments

|a|
|::|

|a|
|:-:|

## Two at the start or end

|a|
|::-|

|a|
|-::|

## In the middle

|a|
|-:-|

## A space in the middle

|a|
|- -|

## No pipe

a
:-:

a
:-

a
-:

## A single colon

|a|
|:|

a
:

## Alignment on empty cells

| a | b | c | d | e |
| - | - | :- | -: | :-: |
| f |
"###,
            &Options::gfm()
        )?,
        r#"<h1>Align</h1>
<h2>An empty initial cell</h2>
<table>
<thead>
<tr>
<th></th>
<th align="center">a</th>
<th align="left">c</th>
</tr>
</thead>
<tbody>
<tr>
<td>a</td>
<td align="center">b</td>
<td align="left">c</td>
</tr>
<tr>
<td>a</td>
<td align="center">b</td>
<td align="left">c</td>
</tr>
</tbody>
</table>
<h2>Missing alignment characters</h2>
<p>| a | b | c |
|   |---|---|
| d | e | f |</p>
<hr />
<p>| a | b | c |
|---|---|   |
| d | e | f |</p>
<h2>Incorrect characters</h2>
<p>| a | b | c |
|---|-*-|---|
| d | e | f |</p>
<h2>Two alignments</h2>
<p>|a|
|::|</p>
<table>
<thead>
<tr>
<th align="center">a</th>
</tr>
</thead>
</table>
<h2>Two at the start or end</h2>
<p>|a|
|::-|</p>
<p>|a|
|-::|</p>
<h2>In the middle</h2>
<p>|a|
|-:-|</p>
<h2>A space in the middle</h2>
<p>|a|
|- -|</p>
<h2>No pipe</h2>
<table>
<thead>
<tr>
<th align="center">a</th>
</tr>
</thead>
</table>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
</table>
<table>
<thead>
<tr>
<th align="right">a</th>
</tr>
</thead>
</table>
<h2>A single colon</h2>
<p>|a|
|:|</p>
<p>a
:</p>
<h2>Alignment on empty cells</h2>
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
<th align="left">c</th>
<th align="right">d</th>
<th align="center">e</th>
</tr>
</thead>
<tbody>
<tr>
<td>f</td>
<td></td>
<td align="left"></td>
<td align="right"></td>
<td align="center"></td>
</tr>
</tbody>
</table>
"#,
        "should match alignment like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"# Tables

| a | b | c |
| - | - | - |
| d | e | f |

## No body

| a | b | c |
| - | - | - |

## One column

| a |
| - |
| b |
"###,
            &Options::gfm()
        )?,
        r###"<h1>Tables</h1>
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
<th>c</th>
</tr>
</thead>
<tbody>
<tr>
<td>d</td>
<td>e</td>
<td>f</td>
</tr>
</tbody>
</table>
<h2>No body</h2>
<table>
<thead>
<tr>
<th>a</th>
<th>b</th>
<th>c</th>
</tr>
</thead>
</table>
<h2>One column</h2>
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
<tbody>
<tr>
<td>b</td>
</tr>
</tbody>
</table>
"###,
        "should match basic like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"# Tables in things

## In lists

*   Unordered:

    | A | B |
    | - | - |
    | 1 | 2 |

1.  Ordered:

    | A | B |
    | - | - |
    | 1 | 2 |

*   Lazy?
    | A | B |
    | - | - |
   | 1 | 2 |
  | 3 | 4 |
 | 5 | 6 |
| 7 | 8 |

## In block quotes

> W/ space:
> | A | B |
> | - | - |
> | 1 | 2 |

>W/o space:
>| A | B |
>| - | - |
>| 1 | 2 |

> Lazy?
> | A | B |
> | - | - |
> | 1 | 2 |
>| 3 | 4 |
| 5 | 6 |

### List interrupting delimiters

a |
- |

a
-|

a
|-
"###,
            &Options::gfm()
        )?,
        r###"<h1>Tables in things</h1>
<h2>In lists</h2>
<ul>
<li>
<p>Unordered:</p>
<table>
<thead>
<tr>
<th>A</th>
<th>B</th>
</tr>
</thead>
<tbody>
<tr>
<td>1</td>
<td>2</td>
</tr>
</tbody>
</table>
</li>
</ul>
<ol>
<li>
<p>Ordered:</p>
<table>
<thead>
<tr>
<th>A</th>
<th>B</th>
</tr>
</thead>
<tbody>
<tr>
<td>1</td>
<td>2</td>
</tr>
</tbody>
</table>
</li>
</ol>
<ul>
<li>Lazy?
<table>
<thead>
<tr>
<th>A</th>
<th>B</th>
</tr>
</thead>
</table>
</li>
</ul>
<p>| 1 | 2 |
| 3 | 4 |
| 5 | 6 |
| 7 | 8 |</p>
<h2>In block quotes</h2>
<blockquote>
<p>W/ space:</p>
<table>
<thead>
<tr>
<th>A</th>
<th>B</th>
</tr>
</thead>
<tbody>
<tr>
<td>1</td>
<td>2</td>
</tr>
</tbody>
</table>
</blockquote>
<blockquote>
<p>W/o space:</p>
<table>
<thead>
<tr>
<th>A</th>
<th>B</th>
</tr>
</thead>
<tbody>
<tr>
<td>1</td>
<td>2</td>
</tr>
</tbody>
</table>
</blockquote>
<blockquote>
<p>Lazy?</p>
<table>
<thead>
<tr>
<th>A</th>
<th>B</th>
</tr>
</thead>
<tbody>
<tr>
<td>1</td>
<td>2</td>
</tr>
<tr>
<td>3</td>
<td>4</td>
</tr>
</tbody>
</table>
</blockquote>
<p>| 5 | 6 |</p>
<h3>List interrupting delimiters</h3>
<p>a |</p>
<ul>
<li>|</li>
</ul>
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
"###,
        "should match containers like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"| a |
| - |
| - |
| 1 |
"###,
            &Options::gfm()
        )?,
        r###"<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
<tbody>
<tr>
<td>-</td>
</tr>
<tr>
<td>1</td>
</tr>
</tbody>
</table>
"###,
        "should match a double delimiter row like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r"# Examples from GFM

## A

| foo | bar |
| --- | --- |
| baz | bim |

## B

| abc | defghi |
:-: | -----------:
bar | baz

## C

| f\|oo  |
| ------ |
| b `\|` az |
| b **\|** im |

## D

| abc | def |
| --- | --- |
| bar | baz |
> bar

## E

| abc | def |
| --- | --- |
| bar | baz |
bar

bar

## F

| abc | def |
| --- |
| bar |

## G

| abc | def |
| --- | --- |
| bar |
| bar | baz | boo |

## H

| abc | def |
| --- | --- |
",
            &Options::gfm()
        )?,
        r#"<h1>Examples from GFM</h1>
<h2>A</h2>
<table>
<thead>
<tr>
<th>foo</th>
<th>bar</th>
</tr>
</thead>
<tbody>
<tr>
<td>baz</td>
<td>bim</td>
</tr>
</tbody>
</table>
<h2>B</h2>
<table>
<thead>
<tr>
<th align="center">abc</th>
<th align="right">defghi</th>
</tr>
</thead>
<tbody>
<tr>
<td align="center">bar</td>
<td align="right">baz</td>
</tr>
</tbody>
</table>
<h2>C</h2>
<table>
<thead>
<tr>
<th>f|oo</th>
</tr>
</thead>
<tbody>
<tr>
<td>b <code>|</code> az</td>
</tr>
<tr>
<td>b <strong>|</strong> im</td>
</tr>
</tbody>
</table>
<h2>D</h2>
<table>
<thead>
<tr>
<th>abc</th>
<th>def</th>
</tr>
</thead>
<tbody>
<tr>
<td>bar</td>
<td>baz</td>
</tr>
</tbody>
</table>
<blockquote>
<p>bar</p>
</blockquote>
<h2>E</h2>
<table>
<thead>
<tr>
<th>abc</th>
<th>def</th>
</tr>
</thead>
<tbody>
<tr>
<td>bar</td>
<td>baz</td>
</tr>
<tr>
<td>bar</td>
<td></td>
</tr>
</tbody>
</table>
<p>bar</p>
<h2>F</h2>
<p>| abc | def |
| --- |
| bar |</p>
<h2>G</h2>
<table>
<thead>
<tr>
<th>abc</th>
<th>def</th>
</tr>
</thead>
<tbody>
<tr>
<td>bar</td>
<td></td>
</tr>
<tr>
<td>bar</td>
<td>baz</td>
</tr>
</tbody>
</table>
<h2>H</h2>
<table>
<thead>
<tr>
<th>abc</th>
<th>def</th>
</tr>
</thead>
</table>
"#,
        "should match examples from the GFM spec like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r"# Grave accents

## Grave accent in cell

| A            | B |
|--------------|---|
| <kbd>`</kbd> | C |

## Escaped grave accent in “inline code” in cell

| A   |
|-----|
| `\` |

## “Empty” inline code

| 1 | 2    | 3  |
|---|------|----|
| a |   `` |    |
| b |   `` | `` |
| c |    ` | `  |
| d |     `|`   |
| e | `\|` |    |
| f |   \| |    |

## Escaped pipes in code in cells

| `\|\\` |
| --- |
| `\|\\` |

`\|\\`
",
            &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..CompileOptions::gfm()
                },
                ..Options::gfm()
            }
        )?,
        r"<h1>Grave accents</h1>
<h2>Grave accent in cell</h2>
<table>
<thead>
<tr>
<th>A</th>
<th>B</th>
</tr>
</thead>
<tbody>
<tr>
<td><kbd>`</kbd></td>
<td>C</td>
</tr>
</tbody>
</table>
<h2>Escaped grave accent in “inline code” in cell</h2>
<table>
<thead>
<tr>
<th>A</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>\</code></td>
</tr>
</tbody>
</table>
<h2>“Empty” inline code</h2>
<table>
<thead>
<tr>
<th>1</th>
<th>2</th>
<th>3</th>
</tr>
</thead>
<tbody>
<tr>
<td>a</td>
<td>``</td>
<td></td>
</tr>
<tr>
<td>b</td>
<td>``</td>
<td>``</td>
</tr>
<tr>
<td>c</td>
<td>`</td>
<td>`</td>
</tr>
<tr>
<td>d</td>
<td>`</td>
<td>`</td>
</tr>
<tr>
<td>e</td>
<td><code>|</code></td>
<td></td>
</tr>
<tr>
<td>f</td>
<td>|</td>
<td></td>
</tr>
</tbody>
</table>
<h2>Escaped pipes in code in cells</h2>
<table>
<thead>
<tr>
<th><code>|\\</code></th>
</tr>
</thead>
<tbody>
<tr>
<td><code>|\\</code></td>
</tr>
</tbody>
</table>
<p><code>\|\\</code></p>
",
        "should match grave accent like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"# Code

## Indented delimiter row

a
   |-

a
    |-

## Indented body

| a |
 | - |
  | C |
   | D |
    | E |
"###,
            &Options::gfm()
        )?,
        r###"<h1>Code</h1>
<h2>Indented delimiter row</h2>
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
<p>a
|-</p>
<h2>Indented body</h2>
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
<tbody>
<tr>
<td>C</td>
</tr>
<tr>
<td>D</td>
</tr>
</tbody>
</table>
<pre><code>| E |
</code></pre>
"###,
        "should match indent like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"## Blank line

a
:-
b

c

## Block quote

a
:-
b
> c

## Code (fenced)

a
:-
b
```
c
```

## Code (indented)

a
:-
b
    c

## Definition

a
:-
b
[c]: d

## Heading (atx)

a
:-
b
# c


## Heading (setext) (rank 1)

a
:-
b
==
c

## Heading (setext) (rank 2)

a
:-
b
--
c

## HTML (flow, kind 1: raw)

a
:-
b
<pre>
  a
</pre>

## HTML (flow, kind 2: comment)

a
:-
b
<!-- c -->

## HTML (flow, kind 3: instruction)

a
:-
b
<? c ?>

## HTML (flow, kind 4: declaration)

a
:-
b
<!C>

## HTML (flow, kind 5: cdata)

a
:-
b
<![CDATA[c]]>

## HTML (flow, kind 6: basic)

a
:-
b
<div>

## HTML (flow, kind 7: complete)

a
:-
b
<x>

## List (ordered, 1)

a
:-
b
1. c

## List (ordered, other)

a
:-
b
2. c

## List (unordered)

a
:-
b
* c

## List (unordered, blank)

a
:-
b
*
c

## List (unordered, blank start)

a
:-
b
*
  c

## Thematic break

a
:-
b
***
"###,
            &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..CompileOptions::gfm()
                },
                ..Options::gfm()
            }
        )?,
        r#"<h2>Blank line</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<p>c</p>
<h2>Block quote</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<blockquote>
<p>c</p>
</blockquote>
<h2>Code (fenced)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<pre><code>c
</code></pre>
<h2>Code (indented)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<pre><code>c
</code></pre>
<h2>Definition</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
<tr>
<td align="left">[c]: d</td>
</tr>
</tbody>
</table>
<h2>Heading (atx)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<h1>c</h1>
<h2>Heading (setext) (rank 1)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
<tr>
<td align="left">==</td>
</tr>
<tr>
<td align="left">c</td>
</tr>
</tbody>
</table>
<h2>Heading (setext) (rank 2)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
<tr>
<td align="left">--</td>
</tr>
<tr>
<td align="left">c</td>
</tr>
</tbody>
</table>
<h2>HTML (flow, kind 1: raw)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<pre>
  a
</pre>
<h2>HTML (flow, kind 2: comment)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<!-- c -->
<h2>HTML (flow, kind 3: instruction)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<? c ?>
<h2>HTML (flow, kind 4: declaration)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<!C>
<h2>HTML (flow, kind 5: cdata)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<![CDATA[c]]>
<h2>HTML (flow, kind 6: basic)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<div>
<h2>HTML (flow, kind 7: complete)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<x>
<h2>List (ordered, 1)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<ol>
<li>c</li>
</ol>
<h2>List (ordered, other)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<ol start="2">
<li>c</li>
</ol>
<h2>List (unordered)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<ul>
<li>c</li>
</ul>
<h2>List (unordered, blank)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<ul>
<li></li>
</ul>
<p>c</p>
<h2>List (unordered, blank start)</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<ul>
<li>c</li>
</ul>
<h2>Thematic break</h2>
<table>
<thead>
<tr>
<th align="left">a</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left">b</td>
</tr>
</tbody>
</table>
<hr />
"#,
        "should match interrupt like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"# Loose

## Loose

Header 1 | Header 2
-------- | --------
Cell 1   | Cell 2
Cell 3   | Cell 4

## One “column”, loose

a
-
b

## No pipe in first row

a
| - |
"###,
            &Options::gfm()
        )?,
        r###"<h1>Loose</h1>
<h2>Loose</h2>
<table>
<thead>
<tr>
<th>Header 1</th>
<th>Header 2</th>
</tr>
</thead>
<tbody>
<tr>
<td>Cell 1</td>
<td>Cell 2</td>
</tr>
<tr>
<td>Cell 3</td>
<td>Cell 4</td>
</tr>
</tbody>
</table>
<h2>One “column”, loose</h2>
<h2>a</h2>
<p>b</p>
<h2>No pipe in first row</h2>
<table>
<thead>
<tr>
<th>a</th>
</tr>
</thead>
</table>
"###,
        "should match loose tables like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r"# Some more escapes

| Head          |
| ------------- |
| A | Alpha     |
| B \| Bravo    |
| C \\| Charlie |
| D \\\| Delta  |
| E \\\\| Echo  |

Note: GH has a bug where in case C and E, the escaped escape is treated as a
normal escape: <https://github.com/github/cmark-gfm/issues/277>.
",
            &Options::gfm()
        )?,
        r#"<h1>Some more escapes</h1>
<table>
<thead>
<tr>
<th>Head</th>
</tr>
</thead>
<tbody>
<tr>
<td>A</td>
</tr>
<tr>
<td>B | Bravo</td>
</tr>
<tr>
<td>C \</td>
</tr>
<tr>
<td>D \| Delta</td>
</tr>
<tr>
<td>E \\</td>
</tr>
</tbody>
</table>
<p>Note: GH has a bug where in case C and E, the escaped escape is treated as a
normal escape: <a href="https://github.com/github/cmark-gfm/issues/277">https://github.com/github/cmark-gfm/issues/277</a>.</p>
"#,
        "should match loose escapes like GitHub"
    );

    assert_eq!(
        to_mdast(
            "| none | left | right | center |\n| - | :- | -: | :-: |\n| a |\n| b | c | d | e | f |",
            &ParseOptions::gfm()
        )?,
        Node::Root(Root {
            children: vec![Node::Table(Table {
                align: vec![
                    AlignKind::None,
                    AlignKind::Left,
                    AlignKind::Right,
                    AlignKind::Center
                ],
                children: vec![
                    Node::TableRow(TableRow {
                        children: vec![
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "none".into(),
                                    position: Some(Position::new(1, 3, 2, 1, 7, 6))
                                }),],
                                position: Some(Position::new(1, 1, 0, 1, 8, 7))
                            }),
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "left".into(),
                                    position: Some(Position::new(1, 10, 9, 1, 14, 13))
                                }),],
                                position: Some(Position::new(1, 8, 7, 1, 15, 14))
                            }),
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "right".into(),
                                    position: Some(Position::new(1, 17, 16, 1, 22, 21))
                                }),],
                                position: Some(Position::new(1, 15, 14, 1, 23, 22))
                            }),
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "center".into(),
                                    position: Some(Position::new(1, 25, 24, 1, 31, 30))
                                }),],
                                position: Some(Position::new(1, 23, 22, 1, 33, 32))
                            }),
                        ],
                        position: Some(Position::new(1, 1, 0, 1, 33, 32))
                    }),
                    Node::TableRow(TableRow {
                        children: vec![Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "a".into(),
                                position: Some(Position::new(3, 3, 57, 3, 4, 58))
                            }),],
                            position: Some(Position::new(3, 1, 55, 3, 6, 60))
                        }),],
                        position: Some(Position::new(3, 1, 55, 3, 6, 60))
                    }),
                    Node::TableRow(TableRow {
                        children: vec![
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "b".into(),
                                    position: Some(Position::new(4, 3, 63, 4, 4, 64))
                                }),],
                                position: Some(Position::new(4, 1, 61, 4, 5, 65))
                            }),
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "c".into(),
                                    position: Some(Position::new(4, 7, 67, 4, 8, 68))
                                }),],
                                position: Some(Position::new(4, 5, 65, 4, 9, 69))
                            }),
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "d".into(),
                                    position: Some(Position::new(4, 11, 71, 4, 12, 72))
                                }),],
                                position: Some(Position::new(4, 9, 69, 4, 13, 73))
                            }),
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "e".into(),
                                    position: Some(Position::new(4, 15, 75, 4, 16, 76))
                                }),],
                                position: Some(Position::new(4, 13, 73, 4, 17, 77))
                            }),
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "f".into(),
                                    position: Some(Position::new(4, 19, 79, 4, 20, 80))
                                }),],
                                position: Some(Position::new(4, 17, 77, 4, 22, 82))
                            }),
                        ],
                        position: Some(Position::new(4, 1, 61, 4, 22, 82))
                    }),
                ],
                position: Some(Position::new(1, 1, 0, 4, 22, 82))
            })],
            position: Some(Position::new(1, 1, 0, 4, 22, 82))
        }),
        "should support GFM tables as `Table`, `TableRow`, `TableCell`s in mdast"
    );

    assert_eq!(
        to_mdast("| `a\\|b` |\n| - |", &ParseOptions::gfm())?,
        Node::Root(Root {
            children: vec![Node::Table(Table {
                align: vec![AlignKind::None,],
                children: vec![Node::TableRow(TableRow {
                    children: vec![Node::TableCell(TableCell {
                        children: vec![Node::InlineCode(InlineCode {
                            value: "a|b".into(),
                            position: Some(Position::new(1, 3, 2, 1, 9, 8))
                        }),],
                        position: Some(Position::new(1, 1, 0, 1, 11, 10))
                    }),],
                    position: Some(Position::new(1, 1, 0, 1, 11, 10))
                }),],
                position: Some(Position::new(1, 1, 0, 2, 6, 16))
            })],
            position: Some(Position::new(1, 1, 0, 2, 6, 16))
        }),
        "should support weird pipe escapes in code in tables"
    );

    Ok(())
}
