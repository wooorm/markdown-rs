extern crate micromark;
use micromark::{micromark, micromark_with_options, Options};

const DANGER: &Options = &Options {
    allow_dangerous_html: true,
    allow_dangerous_protocol: false,
    default_line_ending: None,
};

#[test]
fn html_flow() {
    assert_eq!(
        micromark("<!-- asd -->"),
        "&lt;!-- asd --&gt;",
        "should support a heading w/ rank 1"
    );

    assert_eq!(
        micromark_with_options("<!-- asd -->", DANGER),
        "<!-- asd -->",
        "should support a heading w/ rank 1"
    );

    // To do: turning things off.
    // assert_eq!(
    //   micromark_with_options("<x>", {extensions: [{disable: {null: ["htmlFlow"]}}]}),
    //   "<p>&lt;x&gt;</p>",
    //   "should support turning off html (flow)"
    // );
}

#[test]
fn html_flow_1_raw() {
    assert_eq!(
        micromark_with_options(
            "<pre language=\"haskell\"><code>
import Text.HTML.TagSoup

main :: IO ()
main = print $ parseTags tags
</code></pre>
okay",
            DANGER
        ),
        "<pre language=\"haskell\"><code>
import Text.HTML.TagSoup

main :: IO ()
main = print $ parseTags tags
</code></pre>
<p>okay</p>",
        "should support raw pre tags (type 1)"
    );

    assert_eq!(
        micromark_with_options(
            "<script type=\"text/javascript\">
// JavaScript example

document.getElementById(\"demo\").innerHTML = \"Hello JavaScript!\";
</script>
okay",
            DANGER
        ),
        "<script type=\"text/javascript\">
// JavaScript example

document.getElementById(\"demo\").innerHTML = \"Hello JavaScript!\";
</script>
<p>okay</p>",
        "should support raw script tags"
    );

    assert_eq!(
        micromark_with_options(
            "<style
  type=\"text/css\">
h1 {color:red;}

p {color:blue;}
</style>
okay",
            DANGER
        ),
        "<style
  type=\"text/css\">
h1 {color:red;}

p {color:blue;}
</style>
<p>okay</p>",
        "should support raw style tags"
    );

    assert_eq!(
        micromark_with_options("<style\n  type=\"text/css\">\n\nfoo", DANGER),
        "<style\n  type=\"text/css\">\n\nfoo",
        "should support raw tags w/o ending"
    );

    assert_eq!(
        micromark_with_options("<style>p{color:red;}</style>\n*foo*", DANGER),
        "<style>p{color:red;}</style>\n<p><em>foo</em></p>",
        "should support raw tags w/ start and end on a single line"
    );

    assert_eq!(
        micromark_with_options("<script>\nfoo\n</script>1. *bar*", DANGER),
        "<script>\nfoo\n</script>1. *bar*",
        "should support raw tags w/ more data on ending line"
    );

    assert_eq!(
        micromark_with_options("<script", DANGER),
        "<script",
        "should support an eof directly after a raw tag name"
    );

    assert_eq!(
        micromark_with_options("</script\nmore", DANGER),
        "<p>&lt;/script\nmore</p>",
        "should not support a raw closing tag"
    );

    assert_eq!(
        micromark_with_options("<script/", DANGER),
        "<p>&lt;script/</p>",
        "should not support an eof after a self-closing slash"
    );

    assert_eq!(
        micromark_with_options("<script/\n*asd*", DANGER),
        "<p>&lt;script/\n<em>asd</em></p>",
        "should not support a line ending after a self-closing slash"
    );

    assert_eq!(
        micromark_with_options("<script/>", DANGER),
        "<script/>",
        "should support an eof after a self-closing tag"
    );

    assert_eq!(
        micromark_with_options("<script/>\na", DANGER),
        "<script/>\na",
        "should support a line ending after a self-closing tag"
    );

    assert_eq!(
        micromark_with_options("<script/>a", DANGER),
        "<p><script/>a</p>",
        "should not support other characters after a self-closing tag"
    );

    assert_eq!(
        micromark_with_options("<script>a", DANGER),
        "<script>a",
        "should support other characters after a raw opening tag"
    );

    // Extra.
    assert_eq!(
        micromark_with_options("Foo\n<script", DANGER),
        "<p>Foo</p>\n<script",
        "should support interrupting paragraphs w/ raw tags"
    );

    assert_eq!(
        micromark_with_options("<script>\n  \n  \n</script>", DANGER),
        "<script>\n  \n  \n</script>",
        "should support blank lines in raw"
    );

    assert_eq!(
        micromark_with_options("> <script>\na", DANGER),
        "<blockquote>\n<script>\n</blockquote>\n<p>a</p>",
        "should not support lazyness (1)"
    );

    assert_eq!(
        micromark_with_options("> a\n<script>", DANGER),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<script>",
        "should not support lazyness (2)"
    );
}

#[test]
fn html_flow_2_comment() {
    assert_eq!(
        micromark_with_options("<!-- Foo\n\nbar\n   baz -->\nokay", DANGER),
        "<!-- Foo\n\nbar\n   baz -->\n<p>okay</p>",
        "should support comments (type 2)"
    );

    assert_eq!(
        micromark_with_options("<!-- foo -->*bar*\n*baz*", DANGER),
        "<!-- foo -->*bar*\n<p><em>baz</em></p>",
        "should support comments w/ start and end on a single line"
    );

    assert_eq!(
        micromark_with_options("<!-asd-->", DANGER),
        "<p>&lt;!-asd--&gt;</p>",
        "should not support a single dash to start comments"
    );

    assert_eq!(
        micromark_with_options("<!-->", DANGER),
        "<!-->",
        "should support comments where the start dashes are the end dashes (1)"
    );

    assert_eq!(
        micromark_with_options("<!--->", DANGER),
        "<!--->",
        "should support comments where the start dashes are the end dashes (2)"
    );

    assert_eq!(
        micromark_with_options("<!---->", DANGER),
        "<!---->",
        "should support empty comments"
    );

    // If the `\"` is encoded, we’re in text. If it remains, we’re in HTML.
    assert_eq!(
        micromark_with_options("<!--\n->\n\"", DANGER),
        "<!--\n->\n\"",
        "should not end a comment at one dash (`->`)"
    );
    assert_eq!(
        micromark_with_options("<!--\n-->\n\"", DANGER),
        "<!--\n-->\n<p>&quot;</p>",
        "should end a comment at two dashes (`-->`)"
    );
    assert_eq!(
        micromark_with_options("<!--\n--->\n\"", DANGER),
        "<!--\n--->\n<p>&quot;</p>",
        "should end a comment at three dashes (`--->`)"
    );
    assert_eq!(
        micromark_with_options("<!--\n---->\n\"", DANGER),
        "<!--\n---->\n<p>&quot;</p>",
        "should end a comment at four dashes (`---->`)"
    );

    assert_eq!(
        micromark_with_options("  <!-- foo -->", DANGER),
        "  <!-- foo -->",
        "should support comments w/ indent"
    );

    assert_eq!(
        micromark_with_options("    <!-- foo -->", DANGER),
        "<pre><code>&lt;!-- foo --&gt;\n</code></pre>",
        "should not support comments w/ a 4 character indent"
    );

    // Extra.
    assert_eq!(
        micromark_with_options("Foo\n<!--", DANGER),
        "<p>Foo</p>\n<!--",
        "should support interrupting paragraphs w/ comments"
    );

    assert_eq!(
        micromark_with_options("<!--\n  \n  \n-->", DANGER),
        "<!--\n  \n  \n-->",
        "should support blank lines in comments"
    );

    assert_eq!(
        micromark_with_options("> <!--\na", DANGER),
        "<blockquote>\n<!--\n</blockquote>\n<p>a</p>",
        "should not support lazyness (1)"
    );

    assert_eq!(
        micromark_with_options("> a\n<!--", DANGER),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<!--",
        "should not support lazyness (2)"
    );
}

#[test]
fn html_flow_3_instruction() {
    assert_eq!(
        micromark_with_options("<?php\n\n  echo \">\";\n\n?>\nokay", DANGER),
        "<?php\n\n  echo \">\";\n\n?>\n<p>okay</p>",
        "should support instructions (type 3)"
    );

    assert_eq!(
        micromark_with_options("<?>", DANGER),
        "<?>",
        "should support empty instructions where the `?` is part of both the start and the end"
    );

    assert_eq!(
        micromark_with_options("<??>", DANGER),
        "<??>",
        "should support empty instructions"
    );

    // Extra.
    assert_eq!(
        micromark_with_options("Foo\n<?", DANGER),
        "<p>Foo</p>\n<?",
        "should support interrupting paragraphs w/ instructions"
    );

    assert_eq!(
        micromark_with_options("<?\n  \n  \n?>", DANGER),
        "<?\n  \n  \n?>",
        "should support blank lines in instructions"
    );

    assert_eq!(
        micromark_with_options("> <?\na", DANGER),
        "<blockquote>\n<?\n</blockquote>\n<p>a</p>",
        "should not support lazyness (1)"
    );

    assert_eq!(
        micromark_with_options("> a\n<?", DANGER),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<?",
        "should not support lazyness (2)"
    );
}

#[test]
fn html_flow_4_declaration() {
    assert_eq!(
        micromark_with_options("<!DOCTYPE html>", DANGER),
        "<!DOCTYPE html>",
        "should support declarations (type 4)"
    );

    assert_eq!(
        micromark_with_options("<!123>", DANGER),
        "<p>&lt;!123&gt;</p>",
        "should not support declarations that start w/o an alpha"
    );

    assert_eq!(
        micromark_with_options("<!>", DANGER),
        "<p>&lt;!&gt;</p>",
        "should not support declarations w/o an identifier"
    );

    assert_eq!(
        micromark_with_options("<!a>", DANGER),
        "<!a>",
        "should support declarations w/o a single alpha as identifier"
    );

    // Extra.
    assert_eq!(
        micromark_with_options("Foo\n<!d", DANGER),
        "<p>Foo</p>\n<!d",
        "should support interrupting paragraphs w/ declarations"
    );

    // Note about the lower letter:
    // <https://github.com/commonmark/commonmark-spec/pull/621>
    assert_eq!(
        micromark_with_options("<!a\n  \n  \n>", DANGER),
        "<!a\n  \n  \n>",
        "should support blank lines in declarations"
    );

    assert_eq!(
        micromark_with_options("> <!a\nb", DANGER),
        "<blockquote>\n<!a\n</blockquote>\n<p>b</p>",
        "should not support lazyness (1)"
    );

    assert_eq!(
        micromark_with_options("> a\n<!b", DANGER),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<!b",
        "should not support lazyness (2)"
    );
}

#[test]
fn html_flow_5_cdata() {
    assert_eq!(
    micromark_with_options(
      "<![CDATA[\nfunction matchwo(a,b)\n{\n  if (a < b && a < 0) then {\n    return 1;\n\n  } else {\n\n    return 0;\n  }\n}\n]]>\nokay",
      DANGER
    ),
    "<![CDATA[\nfunction matchwo(a,b)\n{\n  if (a < b && a < 0) then {\n    return 1;\n\n  } else {\n\n    return 0;\n  }\n}\n]]>\n<p>okay</p>",
    "should support cdata (type 5)"
  );

    assert_eq!(
        micromark_with_options("<![CDATA[]]>", DANGER),
        "<![CDATA[]]>",
        "should support empty cdata"
    );

    assert_eq!(
        micromark_with_options("<![CDATA]]>", DANGER),
        "<p>&lt;![CDATA]]&gt;</p>",
        "should not support cdata w/ a missing `[`"
    );

    assert_eq!(
        micromark_with_options("<![CDATA[]]]>", DANGER),
        "<![CDATA[]]]>",
        "should support cdata w/ a single `]` as content"
    );

    // Extra.
    assert_eq!(
        micromark_with_options("Foo\n<![CDATA[", DANGER),
        "<p>Foo</p>\n<![CDATA[",
        "should support interrupting paragraphs w/ cdata"
    );

    // Note: cmjs parses this differently.
    // See: <https://github.com/commonmark/commonmark.js/issues/193>
    assert_eq!(
        micromark_with_options("<![cdata[]]>", DANGER),
        "<p>&lt;![cdata[]]&gt;</p>",
        "should not support lowercase cdata"
    );

    assert_eq!(
        micromark_with_options("<![CDATA[\n  \n  \n]]>", DANGER),
        "<![CDATA[\n  \n  \n]]>",
        "should support blank lines in cdata"
    );

    assert_eq!(
        micromark_with_options("> <![CDATA[\na", DANGER),
        "<blockquote>\n<![CDATA[\n</blockquote>\n<p>a</p>",
        "should not support lazyness (1)"
    );

    assert_eq!(
        micromark_with_options("> a\n<![CDATA[", DANGER),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<![CDATA[",
        "should not support lazyness (2)"
    );
}

#[test]
fn html_flow_6_basic() {
    assert_eq!(
        micromark_with_options(
            "<table><tr><td>\n<pre>\n**Hello**,\n\n_world_.\n</pre>\n</td></tr></table>",
            DANGER
        ),
        "<table><tr><td>\n<pre>\n**Hello**,\n<p><em>world</em>.\n</pre></p>\n</td></tr></table>",
        "should support html (basic)"
    );

    assert_eq!(
        micromark_with_options(
            "<table>
  <tr>
    <td>
           hi
    </td>
  </tr>
</table>

okay.",
            DANGER
        ),
        "<table>
  <tr>
    <td>
           hi
    </td>
  </tr>
</table>
<p>okay.</p>",
        "should support html of type 6 (1)"
    );

    assert_eq!(
        micromark_with_options(" <div>\n  *hello*\n         <foo><a>", DANGER),
        " <div>\n  *hello*\n         <foo><a>",
        "should support html of type 6 (2)"
    );

    assert_eq!(
        micromark_with_options("</div>\n*foo*", DANGER),
        "</div>\n*foo*",
        "should support html starting w/ a closing tag"
    );

    assert_eq!(
        micromark_with_options("<DIV CLASS=\"foo\">\n\n*Markdown*\n\n</DIV>", DANGER),
        "<DIV CLASS=\"foo\">\n<p><em>Markdown</em></p>\n</DIV>",
        "should support html w/ markdown in between"
    );

    assert_eq!(
        micromark_with_options("<div id=\"foo\"\n  class=\"bar\">\n</div>", DANGER),
        "<div id=\"foo\"\n  class=\"bar\">\n</div>",
        "should support html w/ line endings (1)"
    );

    assert_eq!(
        micromark_with_options("<div id=\"foo\" class=\"bar\n  baz\">\n</div>", DANGER),
        "<div id=\"foo\" class=\"bar\n  baz\">\n</div>",
        "should support html w/ line endings (2)"
    );

    assert_eq!(
        micromark_with_options("<div>\n*foo*\n\n*bar*", DANGER),
        "<div>\n*foo*\n<p><em>bar</em></p>",
        "should support an unclosed html element"
    );

    assert_eq!(
        micromark_with_options("<div id=\"foo\"\n*hi*", DANGER),
        "<div id=\"foo\"\n*hi*",
        "should support garbage html (1)"
    );

    assert_eq!(
        micromark_with_options("<div class\nfoo", DANGER),
        "<div class\nfoo",
        "should support garbage html (2)"
    );

    assert_eq!(
        micromark_with_options("<div *???-&&&-<---\n*foo*", DANGER),
        "<div *???-&&&-<---\n*foo*",
        "should support garbage html (3)"
    );

    assert_eq!(
        micromark_with_options("<div><a href=\"bar\">*foo*</a></div>", DANGER),
        "<div><a href=\"bar\">*foo*</a></div>",
        "should support other tags in the opening (1)"
    );

    assert_eq!(
        micromark_with_options("<table><tr><td>\nfoo\n</td></tr></table>", DANGER),
        "<table><tr><td>\nfoo\n</td></tr></table>",
        "should support other tags in the opening (2)"
    );

    assert_eq!(
        micromark_with_options("<div></div>\n``` c\nint x = 33;\n```", DANGER),
        "<div></div>\n``` c\nint x = 33;\n```",
        "should include everything ’till a blank line"
    );

    assert_eq!(
        micromark_with_options("> <div>\n> foo\n\nbar", DANGER),
        "<blockquote>\n<div>\nfoo\n</blockquote>\n<p>bar</p>",
        "should support basic tags w/o ending in containers (1)"
    );

    // To do: list.
    // assert_eq!(
    //     micromark_with_options("- <div>\n- foo", DANGER),
    //     "<ul>\n<li>\n<div>\n</li>\n<li>foo</li>\n</ul>",
    //     "should support basic tags w/o ending in containers (2)"
    // );

    assert_eq!(
        micromark_with_options("  <div>", DANGER),
        "  <div>",
        "should support basic tags w/ indent"
    );

    assert_eq!(
        micromark_with_options("    <div>", DANGER),
        "<pre><code>&lt;div&gt;\n</code></pre>",
        "should not support basic tags w/ a 4 character indent"
    );

    assert_eq!(
        micromark_with_options("Foo\n<div>\nbar\n</div>", DANGER),
        "<p>Foo</p>\n<div>\nbar\n</div>",
        "should support interrupting paragraphs w/ basic tags"
    );

    assert_eq!(
        micromark_with_options("<div>\nbar\n</div>\n*foo*", DANGER),
        "<div>\nbar\n</div>\n*foo*",
        "should require a blank line to end"
    );

    assert_eq!(
        micromark_with_options("<div>\n\n*Emphasized* text.\n\n</div>", DANGER),
        "<div>\n<p><em>Emphasized</em> text.</p>\n</div>",
        "should support interleaving w/ blank lines"
    );

    assert_eq!(
        micromark_with_options("<div>\n*Emphasized* text.\n</div>", DANGER),
        "<div>\n*Emphasized* text.\n</div>",
        "should not support interleaving w/o blank lines"
    );

    assert_eq!(
        micromark_with_options(
            "<table>\n\n<tr>\n\n<td>\nHi\n</td>\n\n</tr>\n\n</table>",
            DANGER
        ),
        "<table>\n<tr>\n<td>\nHi\n</td>\n</tr>\n</table>",
        "should support blank lines between adjacent html"
    );

    assert_eq!(
        micromark_with_options(
            "<table>

  <tr>

    <td>
      Hi
    </td>

  </tr>

</table>",
            DANGER
        ),
        "<table>
  <tr>
<pre><code>&lt;td&gt;
  Hi
&lt;/td&gt;
</code></pre>
  </tr>
</table>",
        "should not support indented, blank-line delimited, adjacent html"
    );

    assert_eq!(
        micromark_with_options("</1>", DANGER),
        "<p>&lt;/1&gt;</p>",
        "should not support basic tags w/ an incorrect name start character"
    );

    assert_eq!(
        micromark_with_options("<div", DANGER),
        "<div",
        "should support an eof directly after a basic tag name"
    );

    assert_eq!(
        micromark_with_options("<div\n", DANGER),
        "<div\n",
        "should support a line ending directly after a tag name"
    );

    assert_eq!(
        micromark_with_options("<div ", DANGER),
        "<div ",
        "should support an eof after a space directly after a tag name"
    );

    assert_eq!(
        micromark_with_options("<div/", DANGER),
        "<p>&lt;div/</p>",
        "should not support an eof directly after a self-closing slash"
    );

    assert_eq!(
        micromark_with_options("<div/\n*asd*", DANGER),
        "<p>&lt;div/\n<em>asd</em></p>",
        "should not support a line ending after a self-closing slash"
    );

    assert_eq!(
        micromark_with_options("<div/>", DANGER),
        "<div/>",
        "should support an eof after a self-closing tag"
    );

    assert_eq!(
        micromark_with_options("<div/>\na", DANGER),
        "<div/>\na",
        "should support a line ending after a self-closing tag"
    );

    assert_eq!(
        micromark_with_options("<div/>a", DANGER),
        "<div/>a",
        "should support another character after a self-closing tag"
    );

    assert_eq!(
        micromark_with_options("<div>a", DANGER),
        "<div>a",
        "should support another character after a basic opening tag"
    );

    // Extra.
    assert_eq!(
        micromark_with_options("Foo\n<div/>", DANGER),
        "<p>Foo</p>\n<div/>",
        "should support interrupting paragraphs w/ self-closing basic tags"
    );

    assert_eq!(
        micromark_with_options("<div\n  \n  \n>", DANGER),
        "<div\n<blockquote>\n</blockquote>",
        "should not support blank lines in basic"
    );

    assert_eq!(
        micromark_with_options("> <div\na", DANGER),
        "<blockquote>\n<div\n</blockquote>\n<p>a</p>",
        "should not support lazyness (1)"
    );

    assert_eq!(
        micromark_with_options("> a\n<div", DANGER),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<div",
        "should not support lazyness (2)"
    );
}

#[test]
fn html_flow_7_complete() {
    assert_eq!(
        micromark_with_options("<a href=\"foo\">\n*bar*\n</a>", DANGER),
        "<a href=\"foo\">\n*bar*\n</a>",
        "should support complete tags (type 7)"
    );

    assert_eq!(
        micromark_with_options("<Warning>\n*bar*\n</Warning>", DANGER),
        "<Warning>\n*bar*\n</Warning>",
        "should support non-html tag names"
    );

    assert_eq!(
        micromark_with_options("<i class=\"foo\">\n*bar*\n</i>", DANGER),
        "<i class=\"foo\">\n*bar*\n</i>",
        "should support non-“block” html tag names (1)"
    );

    assert_eq!(
        micromark_with_options("<del>\n*foo*\n</del>", DANGER),
        "<del>\n*foo*\n</del>",
        "should support non-“block” html tag names (2)"
    );

    assert_eq!(
        micromark_with_options("</ins>\n*bar*", DANGER),
        "</ins>\n*bar*",
        "should support closing tags"
    );

    assert_eq!(
        micromark_with_options("<del>\n\n*foo*\n\n</del>", DANGER),
        "<del>\n<p><em>foo</em></p>\n</del>",
        "should support interleaving"
    );

    assert_eq!(
        micromark_with_options("<del>*foo*</del>", DANGER),
        "<p><del><em>foo</em></del></p>",
        "should not support interleaving w/o blank lines"
    );

    assert_eq!(
        micromark_with_options("<div>\n  \nasd", DANGER),
        "<div>\n<p>asd</p>",
        "should support interleaving w/ whitespace-only blank lines"
    );

    assert_eq!(
        micromark_with_options("Foo\n<a href=\"bar\">\nbaz", DANGER),
        "<p>Foo\n<a href=\"bar\">\nbaz</p>",
        "should not support interrupting paragraphs w/ complete tags"
    );

    assert_eq!(
        micromark_with_options("<x", DANGER),
        "<p>&lt;x</p>",
        "should not support an eof directly after a tag name"
    );

    assert_eq!(
        micromark_with_options("<x/", DANGER),
        "<p>&lt;x/</p>",
        "should not support an eof directly after a self-closing slash"
    );

    assert_eq!(
        micromark_with_options("<x\n", DANGER),
        "<p>&lt;x</p>\n",
        "should not support a line ending directly after a tag name"
    );

    assert_eq!(
        micromark_with_options("<x ", DANGER),
        "<p>&lt;x</p>",
        "should not support an eof after a space directly after a tag name"
    );

    assert_eq!(
        micromark_with_options("<x/", DANGER),
        "<p>&lt;x/</p>",
        "should not support an eof directly after a self-closing slash"
    );

    assert_eq!(
        micromark_with_options("<x/\n*asd*", DANGER),
        "<p>&lt;x/\n<em>asd</em></p>",
        "should not support a line ending after a self-closing slash"
    );

    assert_eq!(
        micromark_with_options("<x/>", DANGER),
        "<x/>",
        "should support an eof after a self-closing tag"
    );

    assert_eq!(
        micromark_with_options("<x/>\na", DANGER),
        "<x/>\na",
        "should support a line ending after a self-closing tag"
    );

    assert_eq!(
        micromark_with_options("<x/>a", DANGER),
        "<p><x/>a</p>",
        "should not support another character after a self-closing tag"
    );

    assert_eq!(
        micromark_with_options("<x>a", DANGER),
        "<p><x>a</p>",
        "should not support another character after an opening tag"
    );

    assert_eq!(
        micromark_with_options("<x y>", DANGER),
        "<x y>",
        "should support boolean attributes in a complete tag"
    );

    assert_eq!(
        micromark_with_options("<x\ny>", DANGER),
        "<p><x\ny></p>",
        "should not support a line ending before an attribute name"
    );

    assert_eq!(
        micromark_with_options("<x\n  y>", DANGER),
        "<p><x\ny></p>",
        "should not support a line ending w/ whitespace before an attribute name"
    );

    assert_eq!(
        micromark_with_options("<x\n  \ny>", DANGER),
        "<p>&lt;x</p>\n<p>y&gt;</p>",
        "should not support a line ending w/ whitespace and another line ending before an attribute name"
    );

    assert_eq!(
        micromark_with_options("<x y\nz>", DANGER),
        "<p><x y\nz></p>",
        "should not support a line ending between attribute names"
    );

    assert_eq!(
        micromark_with_options("<x y   z>", DANGER),
        "<x y   z>",
        "should support whitespace between attribute names"
    );

    assert_eq!(
        micromark_with_options("<x:y>", DANGER),
        "<p>&lt;x:y&gt;</p>",
        "should not support a colon in a tag name"
    );

    assert_eq!(
        micromark_with_options("<x_y>", DANGER),
        "<p>&lt;x_y&gt;</p>",
        "should not support an underscore in a tag name"
    );

    assert_eq!(
        micromark_with_options("<x.y>", DANGER),
        "<p>&lt;x.y&gt;</p>",
        "should not support a dot in a tag name"
    );

    assert_eq!(
        micromark_with_options("<x :y>", DANGER),
        "<x :y>",
        "should support a colon to start an attribute name"
    );

    assert_eq!(
        micromark_with_options("<x _y>", DANGER),
        "<x _y>",
        "should support an underscore to start an attribute name"
    );

    assert_eq!(
        micromark_with_options("<x .y>", DANGER),
        "<p>&lt;x .y&gt;</p>",
        "should not support a dot to start an attribute name"
    );

    assert_eq!(
        micromark_with_options("<x y:>", DANGER),
        "<x y:>",
        "should support a colon to end an attribute name"
    );

    assert_eq!(
        micromark_with_options("<x y_>", DANGER),
        "<x y_>",
        "should support an underscore to end an attribute name"
    );

    assert_eq!(
        micromark_with_options("<x y.>", DANGER),
        "<x y.>",
        "should support a dot to end an attribute name"
    );

    assert_eq!(
        micromark_with_options("<x y123>", DANGER),
        "<x y123>",
        "should support numbers to end an attribute name"
    );

    assert_eq!(
        micromark_with_options("<x data->", DANGER),
        "<x data->",
        "should support a dash to end an attribute name"
    );

    assert_eq!(
        micromark_with_options("<x y=>", DANGER),
        "<p>&lt;x y=&gt;</p>",
        "should not upport an initializer w/o a value"
    );

    assert_eq!(
        micromark_with_options("<x y==>", DANGER),
        "<p>&lt;x y==&gt;</p>",
        "should not support an equals to as an initializer"
    );

    assert_eq!(
        micromark_with_options("<x y=z>", DANGER),
        "<x y=z>",
        "should support a single character as an unquoted attribute value"
    );

    assert_eq!(
        micromark_with_options("<x y=\"\">", DANGER),
        "<x y=\"\">",
        "should support an empty double quoted attribute value"
    );

    assert_eq!(
        micromark_with_options("<x y=\"\">", DANGER),
        "<x y=\"\">",
        "should support an empty single quoted attribute value"
    );

    assert_eq!(
        micromark_with_options("<x y=\"\n\">", DANGER),
        "<p><x y=\"\n\"></p>",
        "should not support a line ending in a double quoted attribute value"
    );

    assert_eq!(
        micromark_with_options("<x y=\"\n\">", DANGER),
        "<p><x y=\"\n\"></p>",
        "should not support a line ending in a single quoted attribute value"
    );

    assert_eq!(
        micromark_with_options("<w x=y\nz>", DANGER),
        "<p><w x=y\nz></p>",
        "should not support a line ending in/after an unquoted attribute value"
    );

    assert_eq!(
        micromark_with_options("<w x=y\"z>", DANGER),
        "<p>&lt;w x=y&quot;z&gt;</p>",
        "should not support a double quote in/after an unquoted attribute value"
    );

    assert_eq!(
        micromark_with_options("<w x=y'z>", DANGER),
        "<p>&lt;w x=y'z&gt;</p>",
        "should not support a single quote in/after an unquoted attribute value"
    );

    assert_eq!(
        micromark_with_options("<x y=\"\"z>", DANGER),
        "<p>&lt;x y=&quot;&quot;z&gt;</p>",
        "should not support an attribute after a double quoted attribute value"
    );

    assert_eq!(
        micromark_with_options("<x>\n  \n  \n>", DANGER),
        "<x>\n<blockquote>\n</blockquote>",
        "should not support blank lines in complete"
    );

    assert_eq!(
        micromark_with_options("> <a>\n*bar*", DANGER),
        "<blockquote>\n<a>\n</blockquote>\n<p><em>bar</em></p>",
        "should not support lazyness (1)"
    );

    // To do: blockquote (lazy).
    // assert_eq!(
    //     micromark_with_options("> a\n<a>", DANGER),
    //     "<blockquote>\n<p>a</p>\n</blockquote>\n<a>",
    //     "should not support lazyness (2)"
    // );
}
