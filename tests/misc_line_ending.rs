extern crate micromark;
use micromark::{micromark, micromark_with_options, Options};

const DANGER: &Options = &Options {
    allow_dangerous_html: true,
    allow_dangerous_protocol: true,
    default_line_ending: None,
};

#[test]
fn line_ending() {
    assert_eq!(
        micromark("a\nb"),
        "<p>a\nb</p>",
        "should support a line feed for a line ending inside a paragraph"
    );

    assert_eq!(
        micromark("a\rb"),
        "<p>a\rb</p>",
        "should support a carriage return for a line ending inside a paragraph"
    );

    assert_eq!(
        micromark("a\r\nb"),
        "<p>a\r\nb</p>",
        "should support a carriage return + line feed for a line ending inside a paragraph"
    );

    assert_eq!(
        micromark("\ta\n\tb"),
        "<pre><code>a\nb\n</code></pre>",
        "should support a line feed in indented code (and prefer it)"
    );

    assert_eq!(
        micromark("\ta\r\tb"),
        "<pre><code>a\rb\r</code></pre>",
        "should support a carriage return in indented code (and prefer it)"
    );

    assert_eq!(
        micromark("\ta\r\n\tb"),
        "<pre><code>a\r\nb\r\n</code></pre>",
        "should support a carriage return + line feed in indented code (and prefer it)"
    );

    assert_eq!(
        micromark("***\n### Heading"),
        "<hr />\n<h3>Heading</h3>",
        "should support a line feed between flow"
    );

    assert_eq!(
        micromark("***\r### Heading"),
        "<hr />\r<h3>Heading</h3>",
        "should support a carriage return between flow"
    );

    assert_eq!(
        micromark("***\r\n### Heading"),
        "<hr />\r\n<h3>Heading</h3>",
        "should support a carriage return + line feed between flow"
    );

    assert_eq!(
        micromark("***\n\n\n### Heading\n"),
        "<hr />\n<h3>Heading</h3>\n",
        "should support several line feeds between flow"
    );

    assert_eq!(
        micromark("***\r\r\r### Heading\r"),
        "<hr />\r<h3>Heading</h3>\r",
        "should support several carriage returns between flow"
    );

    assert_eq!(
        micromark("***\r\n\r\n\r\n### Heading\r\n"),
        "<hr />\r\n<h3>Heading</h3>\r\n",
        "should support several carriage return + line feeds between flow"
    );

    assert_eq!(
        micromark("```x\n\n\ny\n\n\n```\n\n\n"),
        "<pre><code class=\"language-x\">\n\ny\n\n\n</code></pre>\n",
        "should support several line feeds in fenced code"
    );

    assert_eq!(
        micromark("```x\r\r\ry\r\r\r```\r\r\r"),
        "<pre><code class=\"language-x\">\r\ry\r\r\r</code></pre>\r",
        "should support several carriage returns in fenced code"
    );

    assert_eq!(
        micromark("```x\r\n\r\n\r\ny\r\n\r\n\r\n```\r\n\r\n\r\n"),
        "<pre><code class=\"language-x\">\r\n\r\ny\r\n\r\n\r\n</code></pre>\r\n",
        "should support several carriage return + line feeds in fenced code"
    );

    assert_eq!(
        micromark("A\r\nB\r\n-\r\nC"),
        "<h2>A\r\nB</h2>\r\n<p>C</p>",
        "should support a carriage return + line feed in content"
    );

    assert_eq!(
        micromark_with_options("<div\n", DANGER),
        "<div\n",
        "should support a line feed after html"
    );

    assert_eq!(
        micromark_with_options("<div\r", DANGER),
        "<div\r",
        "should support a carriage return after html"
    );

    assert_eq!(
        micromark_with_options("<div\r\n", DANGER),
        "<div\r\n",
        "should support a carriage return + line feed after html"
    );

    assert_eq!(
        micromark_with_options("<div>\n\nx", DANGER),
        "<div>\n<p>x</p>",
        "should support a blank line w/ line feeds after html"
    );

    assert_eq!(
        micromark_with_options("<div>\r\rx", DANGER),
        "<div>\r<p>x</p>",
        "should support a blank line w/ carriage returns after html"
    );

    assert_eq!(
        micromark_with_options("<div>\r\n\r\nx", DANGER),
        "<div>\r\n<p>x</p>",
        "should support a blank line w/ carriage return + line feeds after html"
    );

    assert_eq!(
        micromark_with_options("<div>\nx", DANGER),
        "<div>\nx",
        "should support a non-blank line w/ line feed in html"
    );

    assert_eq!(
        micromark_with_options("<div>\rx", DANGER),
        "<div>\rx",
        "should support a non-blank line w/ carriage return in html"
    );

    assert_eq!(
        micromark_with_options("<div>\r\nx", DANGER),
        "<div>\r\nx",
        "should support a non-blank line w/ carriage return + line feed in html"
    );
}
