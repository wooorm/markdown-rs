use markdown::{
    mdast::{List, ListItem, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    CompileOptions, Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn list() -> Result<(), message::Message> {
    let danger = Options {
        compile: CompileOptions {
            allow_dangerous_html: true,
            allow_dangerous_protocol: true,
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html(
            "A paragraph\nwith two lines.\n\n    indented code\n\n> A block quote."),
        "<p>A paragraph\nwith two lines.</p>\n<pre><code>indented code\n</code></pre>\n<blockquote>\n<p>A block quote.</p>\n</blockquote>",
        "should support documents"
    );

    assert_eq!(
        to_html("1.  a\n    b.\n\n        c\n\n    > d."),
        "<ol>\n<li>\n<p>a\nb.</p>\n<pre><code>c\n</code></pre>\n<blockquote>\n<p>d.</p>\n</blockquote>\n</li>\n</ol>",
        "should support documents in list items"
    );

    assert_eq!(
        to_html("- one\n\n two"),
        "<ul>\n<li>one</li>\n</ul>\n<p>two</p>",
        "should not support 1 space for a two-character list prefix"
    );

    assert_eq!(
        to_html("- a\n\n  b"),
        "<ul>\n<li>\n<p>a</p>\n<p>b</p>\n</li>\n</ul>",
        "should support blank lines in list items"
    );

    assert_eq!(
        to_html(" -    one\n\n     two"),
        "<ul>\n<li>one</li>\n</ul>\n<pre><code> two\n</code></pre>",
        "should support indented code after lists"
    );

    assert_eq!(
        to_html("   > > 1.  one\n>>\n>>     two"),
        "<blockquote>\n<blockquote>\n<ol>\n<li>\n<p>one</p>\n<p>two</p>\n</li>\n</ol>\n</blockquote>\n</blockquote>",
        "should support proper indent mixed w/ block quotes (1)"
    );

    assert_eq!(
        to_html(">>- one\n>>\n  >  > two"),
        "<blockquote>\n<blockquote>\n<ul>\n<li>one</li>\n</ul>\n<p>two</p>\n</blockquote>\n</blockquote>",
        "should support proper indent mixed w/ block quotes (2)"
    );

    assert_eq!(
        to_html("-one\n\n2.two"),
        "<p>-one</p>\n<p>2.two</p>",
        "should not support a missing space after marker"
    );

    assert_eq!(
        to_html("- foo\n\n\n  bar"),
        "<ul>\n<li>\n<p>foo</p>\n<p>bar</p>\n</li>\n</ul>",
        "should support multiple blank lines between items"
    );

    assert_eq!(
        to_html("1.  foo\n\n    ```\n    bar\n    ```\n\n    baz\n\n    > bam"),
        "<ol>\n<li>\n<p>foo</p>\n<pre><code>bar\n</code></pre>\n<p>baz</p>\n<blockquote>\n<p>bam</p>\n</blockquote>\n</li>\n</ol>",
        "should support flow in items"
    );

    assert_eq!(
        to_html("- Foo\n\n      bar\n\n\n      baz"),
        "<ul>\n<li>\n<p>Foo</p>\n<pre><code>bar\n\n\nbaz\n</code></pre>\n</li>\n</ul>",
        "should support blank lines in indented code in items"
    );

    assert_eq!(
        to_html("123456789. ok"),
        "<ol start=\"123456789\">\n<li>ok</li>\n</ol>",
        "should support start on the first list item"
    );

    assert_eq!(
        to_html("1234567890. not ok"),
        "<p>1234567890. not ok</p>",
        "should not support ordered item values over 10 digits"
    );

    assert_eq!(
        to_html("0. ok"),
        "<ol start=\"0\">\n<li>ok</li>\n</ol>",
        "should support ordered item values of `0`"
    );

    assert_eq!(
        to_html("003. ok"),
        "<ol start=\"3\">\n<li>ok</li>\n</ol>",
        "should support ordered item values starting w/ `0`s"
    );

    assert_eq!(
        to_html("-1. not ok"),
        "<p>-1. not ok</p>",
        "should not support “negative” ordered item values"
    );

    assert_eq!(
        to_html("- foo\n\n      bar"),
        "<ul>\n<li>\n<p>foo</p>\n<pre><code>bar\n</code></pre>\n</li>\n</ul>",
        "should support indented code in list items (1)"
    );

    assert_eq!(
        to_html("  10.  foo\n\n           bar"),
        "<ol start=\"10\">\n<li>\n<p>foo</p>\n<pre><code>bar\n</code></pre>\n</li>\n</ol>",
        "should support indented code in list items (2)"
    );

    assert_eq!(
        to_html("    indented code\n\nparagraph\n\n    more code"),
        "<pre><code>indented code\n</code></pre>\n<p>paragraph</p>\n<pre><code>more code\n</code></pre>",
        "should support indented code in list items (3)"
    );

    assert_eq!(
        to_html("1.     indented code\n\n   paragraph\n\n       more code"),
        "<ol>\n<li>\n<pre><code>indented code\n</code></pre>\n<p>paragraph</p>\n<pre><code>more code\n</code></pre>\n</li>\n</ol>",
        "should support indented code in list items (4)"
    );

    assert_eq!(
        to_html("1.      indented code\n\n   paragraph\n\n       more code"),
        "<ol>\n<li>\n<pre><code> indented code\n</code></pre>\n<p>paragraph</p>\n<pre><code>more code\n</code></pre>\n</li>\n</ol>",
        "should support indented code in list items (5)"
    );

    assert_eq!(
        to_html("   foo\n\nbar"),
        "<p>foo</p>\n<p>bar</p>",
        "should support indented code in list items (6)"
    );

    assert_eq!(
        to_html("-    foo\n\n  bar"),
        "<ul>\n<li>foo</li>\n</ul>\n<p>bar</p>",
        "should support indented code in list items (7)"
    );

    assert_eq!(
        to_html("-  foo\n\n   bar"),
        "<ul>\n<li>\n<p>foo</p>\n<p>bar</p>\n</li>\n</ul>",
        "should support indented code in list items (8)"
    );

    assert_eq!(
        to_html("-\n  foo\n-\n  ```\n  bar\n  ```\n-\n      baz"),
        "<ul>\n<li>foo</li>\n<li>\n<pre><code>bar\n</code></pre>\n</li>\n<li>\n<pre><code>baz\n</code></pre>\n</li>\n</ul>",
        "should support blank first lines (1)"
    );

    assert_eq!(
        to_html("-   \n  foo"),
        "<ul>\n<li>foo</li>\n</ul>",
        "should support blank first lines (2)"
    );

    assert_eq!(
        to_html("-\n\n  foo"),
        "<ul>\n<li></li>\n</ul>\n<p>foo</p>",
        "should support empty only items"
    );

    assert_eq!(
        to_html("- foo\n-\n- bar"),
        "<ul>\n<li>foo</li>\n<li></li>\n<li>bar</li>\n</ul>",
        "should support empty continued items"
    );

    assert_eq!(
        to_html("- foo\n-   \n- bar"),
        "<ul>\n<li>foo</li>\n<li></li>\n<li>bar</li>\n</ul>",
        "should support blank continued items"
    );

    assert_eq!(
        to_html("1. foo\n2.\n3. bar"),
        "<ol>\n<li>foo</li>\n<li></li>\n<li>bar</li>\n</ol>",
        "should support empty continued items (ordered)"
    );

    assert_eq!(
        to_html("*"),
        "<ul>\n<li></li>\n</ul>",
        "should support a single empty item"
    );

    assert_eq!(
        to_html("foo\n*\n\nfoo\n1."),
        "<p>foo\n*</p>\n<p>foo\n1.</p>",
        "should not support empty items to interrupt paragraphs"
    );

    assert_eq!(
        to_html(
            " 1.  A paragraph\n     with two lines.\n\n         indented code\n\n     > A block quote."),
        "<ol>\n<li>\n<p>A paragraph\nwith two lines.</p>\n<pre><code>indented code\n</code></pre>\n<blockquote>\n<p>A block quote.</p>\n</blockquote>\n</li>\n</ol>",
        "should support indenting w/ 1 space"
    );

    assert_eq!(
        to_html(
            "  1.  A paragraph\n      with two lines.\n\n          indented code\n\n      > A block quote."),
        "<ol>\n<li>\n<p>A paragraph\nwith two lines.</p>\n<pre><code>indented code\n</code></pre>\n<blockquote>\n<p>A block quote.</p>\n</blockquote>\n</li>\n</ol>",
        "should support indenting w/ 2 spaces"
    );

    assert_eq!(
        to_html(
            "   1.  A paragraph\n       with two lines.\n\n           indented code\n\n       > A block quote."),
        "<ol>\n<li>\n<p>A paragraph\nwith two lines.</p>\n<pre><code>indented code\n</code></pre>\n<blockquote>\n<p>A block quote.</p>\n</blockquote>\n</li>\n</ol>",
        "should support indenting w/ 3 spaces"
    );

    assert_eq!(
        to_html(
            "    1.  A paragraph\n        with two lines.\n\n            indented code\n\n        > A block quote."),
        "<pre><code>1.  A paragraph\n    with two lines.\n\n        indented code\n\n    &gt; A block quote.\n</code></pre>",
        "should not support indenting w/ 4 spaces"
    );

    assert_eq!(
        to_html(
            "  1.  A paragraph\nwith two lines.\n\n          indented code\n\n      > A block quote."),
        "<ol>\n<li>\n<p>A paragraph\nwith two lines.</p>\n<pre><code>indented code\n</code></pre>\n<blockquote>\n<p>A block quote.</p>\n</blockquote>\n</li>\n</ol>",
        "should support lazy lines"
    );

    assert_eq!(
        to_html("  1.  A paragraph\n    with two lines."),
        "<ol>\n<li>A paragraph\nwith two lines.</li>\n</ol>",
        "should support partially lazy lines"
    );

    assert_eq!(
        to_html("> 1. > Blockquote\ncontinued here."),
        "<blockquote>\n<ol>\n<li>\n<blockquote>\n<p>Blockquote\ncontinued here.</p>\n</blockquote>\n</li>\n</ol>\n</blockquote>",
        "should support lazy lines combined w/ other containers"
    );

    assert_eq!(
        to_html("> 1. > Blockquote\n> continued here."),
        "<blockquote>\n<ol>\n<li>\n<blockquote>\n<p>Blockquote\ncontinued here.</p>\n</blockquote>\n</li>\n</ol>\n</blockquote>",
        "should support partially continued, partially lazy lines combined w/ other containers"
    );

    assert_eq!(
        to_html("- [\na"),
        "<ul>\n<li>[\na</li>\n</ul>",
        "should support lazy, definition-like lines"
    );

    assert_eq!(
        to_html("- [a]: b\nc"),
        "<ul>\n<li>c</li>\n</ul>",
        "should support a definition, followed by a lazy paragraph"
    );

    assert_eq!(
        to_html("- foo\n  - bar\n    - baz\n      - boo"),
        "<ul>\n<li>foo\n<ul>\n<li>bar\n<ul>\n<li>baz\n<ul>\n<li>boo</li>\n</ul>\n</li>\n</ul>\n</li>\n</ul>\n</li>\n</ul>",
        "should support sublists w/ enough spaces (1)"
    );

    assert_eq!(
        to_html("- foo\n - bar\n  - baz\n   - boo"),
        "<ul>\n<li>foo</li>\n<li>bar</li>\n<li>baz</li>\n<li>boo</li>\n</ul>",
        "should not support sublists w/ too few spaces"
    );

    assert_eq!(
        to_html("10) foo\n    - bar"),
        "<ol start=\"10\">\n<li>foo\n<ul>\n<li>bar</li>\n</ul>\n</li>\n</ol>",
        "should support sublists w/ enough spaces (2)"
    );

    assert_eq!(
        to_html("10) foo\n   - bar"),
        "<ol start=\"10\">\n<li>foo</li>\n</ol>\n<ul>\n<li>bar</li>\n</ul>",
        "should not support sublists w/ too few spaces (2)"
    );

    assert_eq!(
        to_html("- - foo"),
        "<ul>\n<li>\n<ul>\n<li>foo</li>\n</ul>\n</li>\n</ul>",
        "should support sublists (1)"
    );

    assert_eq!(
        to_html("1. - 2. foo"),
        "<ol>\n<li>\n<ul>\n<li>\n<ol start=\"2\">\n<li>foo</li>\n</ol>\n</li>\n</ul>\n</li>\n</ol>",
        "should support sublists (2)"
    );

    assert_eq!(
        to_html("- # Foo\n- Bar\n  ---\n  baz"),
        "<ul>\n<li>\n<h1>Foo</h1>\n</li>\n<li>\n<h2>Bar</h2>\nbaz</li>\n</ul>",
        "should support headings in list items"
    );

    assert_eq!(
        to_html("- foo\n- bar\n+ baz"),
        "<ul>\n<li>foo</li>\n<li>bar</li>\n</ul>\n<ul>\n<li>baz</li>\n</ul>",
        "should support a new list by changing the marker (unordered)"
    );

    assert_eq!(
        to_html("1. foo\n2. bar\n3) baz"),
        "<ol>\n<li>foo</li>\n<li>bar</li>\n</ol>\n<ol start=\"3\">\n<li>baz</li>\n</ol>",
        "should support a new list by changing the marker (ordered)"
    );

    assert_eq!(
        to_html("Foo\n- bar\n- baz"),
        "<p>Foo</p>\n<ul>\n<li>bar</li>\n<li>baz</li>\n</ul>",
        "should support interrupting a paragraph"
    );

    assert_eq!(
        to_html("a\n2. b"),
        "<p>a\n2. b</p>",
        "should not support interrupting a paragraph with a non-1 numbered item"
    );

    assert_eq!(
        to_html("\n2. a"),
        "<ol start=\"2\">\n<li>a</li>\n</ol>",
        "should “interrupt” a blank line (1)"
    );

    assert_eq!(
        to_html("a\n\n2. b"),
        "<p>a</p>\n<ol start=\"2\">\n<li>b</li>\n</ol>",
        "should “interrupt” a blank line (2)"
    );

    assert_eq!(
        to_html("a\n1. b"),
        "<p>a</p>\n<ol>\n<li>b</li>\n</ol>",
        "should support interrupting a paragraph with a 1 numbered item"
    );

    assert_eq!(
        to_html("- foo\n\n- bar\n\n\n- baz"),
        "<ul>\n<li>\n<p>foo</p>\n</li>\n<li>\n<p>bar</p>\n</li>\n<li>\n<p>baz</p>\n</li>\n</ul>",
        "should support blank lines between items (1)"
    );

    assert_eq!(
        to_html("- foo\n  - bar\n    - baz\n\n\n      bim"),
        "<ul>\n<li>foo\n<ul>\n<li>bar\n<ul>\n<li>\n<p>baz</p>\n<p>bim</p>\n</li>\n</ul>\n</li>\n</ul>\n</li>\n</ul>",
        "should support blank lines between items (2)"
    );

    assert_eq!(
        to_html_with_options("- foo\n- bar\n\n<!-- -->\n\n- baz\n- bim", &danger)?,
        "<ul>\n<li>foo</li>\n<li>bar</li>\n</ul>\n<!-- -->\n<ul>\n<li>baz</li>\n<li>bim</li>\n</ul>",
        "should support HTML comments between lists"
    );

    assert_eq!(
        to_html_with_options("-   foo\n\n    notcode\n\n-   foo\n\n<!-- -->\n\n    code", &danger)?,
        "<ul>\n<li>\n<p>foo</p>\n<p>notcode</p>\n</li>\n<li>\n<p>foo</p>\n</li>\n</ul>\n<!-- -->\n<pre><code>code\n</code></pre>",
        "should support HTML comments between lists and indented code"
    );

    assert_eq!(
        to_html("- a\n - b\n  - c\n   - d\n  - e\n - f\n- g"),
        "<ul>\n<li>a</li>\n<li>b</li>\n<li>c</li>\n<li>d</li>\n<li>e</li>\n<li>f</li>\n<li>g</li>\n</ul>",
        "should not support lists in lists w/ too few spaces (1)"
    );

    assert_eq!(
        to_html("1. a\n\n  2. b\n\n   3. c"),
        "<ol>\n<li>\n<p>a</p>\n</li>\n<li>\n<p>b</p>\n</li>\n<li>\n<p>c</p>\n</li>\n</ol>",
        "should not support lists in lists w/ too few spaces (2)"
    );

    assert_eq!(
        to_html("- a\n - b\n  - c\n   - d\n    - e"),
        "<ul>\n<li>a</li>\n<li>b</li>\n<li>c</li>\n<li>d\n- e</li>\n</ul>",
        "should not support lists in lists w/ too few spaces (3)"
    );

    assert_eq!(
        to_html("1. a\n\n  2. b\n\n    3. c"),
        "<ol>\n<li>\n<p>a</p>\n</li>\n<li>\n<p>b</p>\n</li>\n</ol>\n<pre><code>3. c\n</code></pre>",
        "should not support lists in lists w/ too few spaces (3)"
    );

    assert_eq!(
        to_html("- a\n- b\n\n- c"),
        "<ul>\n<li>\n<p>a</p>\n</li>\n<li>\n<p>b</p>\n</li>\n<li>\n<p>c</p>\n</li>\n</ul>",
        "should support loose lists w/ a blank line between (1)"
    );

    assert_eq!(
        to_html("* a\n*\n\n* c"),
        "<ul>\n<li>\n<p>a</p>\n</li>\n<li></li>\n<li>\n<p>c</p>\n</li>\n</ul>",
        "should support loose lists w/ a blank line between (2)"
    );

    assert_eq!(
        to_html("- a\n- b\n\n  c\n- d"),
        "<ul>\n<li>\n<p>a</p>\n</li>\n<li>\n<p>b</p>\n<p>c</p>\n</li>\n<li>\n<p>d</p>\n</li>\n</ul>",
        "should support loose lists w/ a blank line in an item (1)"
    );

    assert_eq!(
        to_html("- a\n- b\n\n  [ref]: /url\n- d"),
        "<ul>\n<li>\n<p>a</p>\n</li>\n<li>\n<p>b</p>\n</li>\n<li>\n<p>d</p>\n</li>\n</ul>",
        "should support loose lists w/ a blank line in an item (2)"
    );

    assert_eq!(
        to_html("- a\n- ```\n  b\n\n\n  ```\n- c"),
        "<ul>\n<li>a</li>\n<li>\n<pre><code>b\n\n\n</code></pre>\n</li>\n<li>c</li>\n</ul>",
        "should support tight lists w/ a blank line in fenced code"
    );

    assert_eq!(
        to_html("- a\n  - b\n\n    c\n- d"),
        "<ul>\n<li>a\n<ul>\n<li>\n<p>b</p>\n<p>c</p>\n</li>\n</ul>\n</li>\n<li>d</li>\n</ul>",
        "should support tight lists w/ a blank line in a sublist"
    );

    assert_eq!(
        to_html("* a\n  > b\n  >\n* c"),
        "<ul>\n<li>a\n<blockquote>\n<p>b</p>\n</blockquote>\n</li>\n<li>c</li>\n</ul>",
        "should support tight lists w/ a blank line in a block quote"
    );

    assert_eq!(
        to_html("- a\n  > b\n  ```\n  c\n  ```\n- d"),
        "<ul>\n<li>a\n<blockquote>\n<p>b</p>\n</blockquote>\n<pre><code>c\n</code></pre>\n</li>\n<li>d</li>\n</ul>",
        "should support tight lists w/ flow w/o blank line"
    );

    assert_eq!(
        to_html("- a"),
        "<ul>\n<li>a</li>\n</ul>",
        "should support tight lists w/ a single content"
    );

    assert_eq!(
        to_html("- a\n  - b"),
        "<ul>\n<li>a\n<ul>\n<li>b</li>\n</ul>\n</li>\n</ul>",
        "should support tight lists w/ a sublist"
    );

    assert_eq!(
        to_html("1. ```\n   foo\n   ```\n\n   bar"),
        "<ol>\n<li>\n<pre><code>foo\n</code></pre>\n<p>bar</p>\n</li>\n</ol>",
        "should support loose lists w/ a blank line in an item"
    );

    assert_eq!(
        to_html("* foo\n  * bar\n\n  baz"),
        "<ul>\n<li>\n<p>foo</p>\n<ul>\n<li>bar</li>\n</ul>\n<p>baz</p>\n</li>\n</ul>",
        "should support loose lists w/ tight sublists (1)"
    );

    assert_eq!(
        to_html("- a\n  - b\n  - c\n\n- d\n  - e\n  - f"),
        "<ul>\n<li>\n<p>a</p>\n<ul>\n<li>b</li>\n<li>c</li>\n</ul>\n</li>\n<li>\n<p>d</p>\n<ul>\n<li>e</li>\n<li>f</li>\n</ul>\n</li>\n</ul>",
        "should support loose lists w/ tight sublists (2)"
    );

    // Extra.
    assert_eq!(
        to_html("* a\n*\n\n  \n\t\n* b"),
        "<ul>\n<li>\n<p>a</p>\n</li>\n<li></li>\n<li>\n<p>b</p>\n</li>\n</ul>",
        "should support continued list items after an empty list item w/ many blank lines"
    );

    assert_eq!(
        to_html("*\n  ~~~p\n\n  ~~~"),
        "<ul>\n<li>\n<pre><code class=\"language-p\">\n</code></pre>\n</li>\n</ul>",
        "should support blank lines in code after an initial blank line"
    );

    assert_eq!(
        to_html(
            "* a tight item that ends with an html element: `x`\n\nParagraph"),
        "<ul>\n<li>a tight item that ends with an html element: <code>x</code></li>\n</ul>\n<p>Paragraph</p>",
        "should ignore line endings after tight items ending in tags"
    );

    assert_eq!(
        to_html("*   foo\n\n*\n\n*   bar"),
        "<ul>\n<li>\n<p>foo</p>\n</li>\n<li></li>\n<li>\n<p>bar</p>\n</li>\n</ul>",
        "should support empty items in a spread list"
    );

    assert_eq!(
        to_html("- ```\n\n  ```"),
        "<ul>\n<li>\n<pre><code>\n</code></pre>\n</li>\n</ul>",
        "should remove indent of code (fenced) in list (0 space)"
    );

    assert_eq!(
        to_html("- ```\n \n  ```"),
        "<ul>\n<li>\n<pre><code>\n</code></pre>\n</li>\n</ul>",
        "should remove indent of code (fenced) in list (1 space)"
    );

    assert_eq!(
        to_html("- ```\n  \n  ```"),
        "<ul>\n<li>\n<pre><code>\n</code></pre>\n</li>\n</ul>",
        "should remove indent of code (fenced) in list (2 spaces)"
    );

    assert_eq!(
        to_html("- ```\n   \n  ```"),
        "<ul>\n<li>\n<pre><code> \n</code></pre>\n</li>\n</ul>",
        "should remove indent of code (fenced) in list (3 spaces)"
    );

    assert_eq!(
        to_html("- ```\n    \n  ```"),
        "<ul>\n<li>\n<pre><code>  \n</code></pre>\n</li>\n</ul>",
        "should remove indent of code (fenced) in list (4 spaces)"
    );

    assert_eq!(
        to_html("- ```\n\t\n  ```"),
        "<ul>\n<li>\n<pre><code>  \n</code></pre>\n</li>\n</ul>",
        "should remove indent of code (fenced) in list (1 tab)"
    );

    assert_eq!(
        to_html("- +\n-"),
        "<ul>\n<li>\n<ul>\n<li></li>\n</ul>\n</li>\n<li></li>\n</ul>",
        "should support complex nested and empty lists (1)"
    );

    assert_eq!(
        to_html("- 1.\n-"),
        "<ul>\n<li>\n<ol>\n<li></li>\n</ol>\n</li>\n<li></li>\n</ul>",
        "should support complex nested and empty lists (2)"
    );

    assert_eq!(
        to_html("* - +\n* -"),
        "<ul>\n<li>\n<ul>\n<li>\n<ul>\n<li></li>\n</ul>\n</li>\n</ul>\n</li>\n<li>\n<ul>\n<li></li>\n</ul>\n</li>\n</ul>",
        "should support complex nested and empty lists (3)"
    );

    assert_eq!(
        to_html_with_options("* a\n\n<!---->\n\n* b", &danger)?,
        "<ul>\n<li>a</li>\n</ul>\n<!---->\n<ul>\n<li>b</li>\n</ul>",
        "should support the common list breaking comment method"
    );

    assert_eq!(
        to_html_with_options(
            "- one\n\n two",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        list_item: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>- one</p>\n<p>two</p>",
        "should support turning off lists"
    );

    assert_eq!(
        to_mdast("* a", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::List(List {
                ordered: false,
                spread: false,
                start: None,
                children: vec![Node::ListItem(ListItem {
                    checked: None,
                    spread: false,
                    children: vec![Node::Paragraph(Paragraph {
                        children: vec![Node::Text(Text {
                            value: "a".into(),
                            position: Some(Position::new(1, 3, 2, 1, 4, 3))
                        }),],
                        position: Some(Position::new(1, 3, 2, 1, 4, 3))
                    })],
                    position: Some(Position::new(1, 1, 0, 1, 4, 3))
                })],
                position: Some(Position::new(1, 1, 0, 1, 4, 3))
            })],
            position: Some(Position::new(1, 1, 0, 1, 4, 3))
        }),
        "should support lists, list items as `List`, `ListItem`s in mdast"
    );

    assert_eq!(
        to_mdast("3. a\n4. b", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::List(List {
                ordered: true,
                spread: false,
                start: Some(3),
                children: vec![
                    Node::ListItem(ListItem {
                        checked: None,
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "a".into(),
                                position: Some(Position::new(1, 4, 3, 1, 5, 4))
                            }),],
                            position: Some(Position::new(1, 4, 3, 1, 5, 4))
                        })],
                        position: Some(Position::new(1, 1, 0, 1, 5, 4))
                    }),
                    Node::ListItem(ListItem {
                        checked: None,
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "b".into(),
                                position: Some(Position::new(2, 4, 8, 2, 5, 9))
                            }),],
                            position: Some(Position::new(2, 4, 8, 2, 5, 9))
                        })],
                        position: Some(Position::new(2, 1, 5, 2, 5, 9))
                    })
                ],
                position: Some(Position::new(1, 1, 0, 2, 5, 9))
            })],
            position: Some(Position::new(1, 1, 0, 2, 5, 9))
        }),
        "should support `start` fields on `List` w/ `ordered: true` in mdast"
    );

    assert_eq!(
        to_mdast("* a\n\n  b\n* c", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::List(List {
                ordered: false,
                spread: false,
                start: None,
                children: vec![
                    Node::ListItem(ListItem {
                        checked: None,
                        spread: true,
                        children: vec![
                            Node::Paragraph(Paragraph {
                                children: vec![Node::Text(Text {
                                    value: "a".into(),
                                    position: Some(Position::new(1, 3, 2, 1, 4, 3))
                                }),],
                                position: Some(Position::new(1, 3, 2, 1, 4, 3))
                            }),
                            Node::Paragraph(Paragraph {
                                children: vec![Node::Text(Text {
                                    value: "b".into(),
                                    position: Some(Position::new(3, 3, 7, 3, 4, 8))
                                }),],
                                position: Some(Position::new(3, 3, 7, 3, 4, 8))
                            })
                        ],
                        position: Some(Position::new(1, 1, 0, 3, 4, 8))
                    }),
                    Node::ListItem(ListItem {
                        checked: None,
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "c".into(),
                                position: Some(Position::new(4, 3, 11, 4, 4, 12))
                            }),],
                            position: Some(Position::new(4, 3, 11, 4, 4, 12))
                        })],
                        position: Some(Position::new(4, 1, 9, 4, 4, 12))
                    })
                ],
                position: Some(Position::new(1, 1, 0, 4, 4, 12))
            })],
            position: Some(Position::new(1, 1, 0, 4, 4, 12))
        }),
        "should support `spread` fields on `List`, `ListItem`s in mdast"
    );

    Ok(())
}
