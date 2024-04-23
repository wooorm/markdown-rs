use markdown::{
    mdast::{Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    CompileOptions, Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn character_reference() -> Result<(), message::Message> {
    assert_eq!(
        to_html(
            "&nbsp; &amp; &copy; &AElig; &Dcaron;\n&frac34; &HilbertSpace; &DifferentialD;\n&ClockwiseContourIntegral; &ngE;"
        ),
        "<p>\u{a0} &amp; © Æ Ď\n¾ ℋ ⅆ\n∲ ≧̸</p>",
        "should support named character references"
    );

    assert_eq!(
        to_html("&#35; &#1234; &#992; &#0;"),
        "<p># Ӓ Ϡ �</p>",
        "should support decimal character references"
    );

    assert_eq!(
        to_html("&#X22; &#XD06; &#xcab;"),
        "<p>&quot; ആ ಫ</p>",
        "should support hexadecimal character references"
    );

    assert_eq!(
      to_html(
        "&nbsp &x; &#; &#x;\n&#987654321;\n&#abcdef0;\n&ThisIsNotDefined; &hi?;"),
      "<p>&amp;nbsp &amp;x; &amp;#; &amp;#x;\n&amp;#987654321;\n&amp;#abcdef0;\n&amp;ThisIsNotDefined; &amp;hi?;</p>",
      "should not support other things that look like character references"
    );

    assert_eq!(
        to_html("&copy"),
        "<p>&amp;copy</p>",
        "should not support character references w/o semicolon"
    );

    assert_eq!(
        to_html("&MadeUpEntity;"),
        "<p>&amp;MadeUpEntity;</p>",
        "should not support unknown named character references"
    );

    assert_eq!(
        to_html_with_options(
            "<a href=\"&ouml;&ouml;.html\">",
            &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<a href=\"&ouml;&ouml;.html\">",
        "should not care about character references in html"
    );

    assert_eq!(
        to_html("[foo](/f&ouml;&ouml; \"f&ouml;&ouml;\")"),
        "<p><a href=\"/f%C3%B6%C3%B6\" title=\"föö\">foo</a></p>",
        "should support character references in resource URLs and titles"
    );

    assert_eq!(
        to_html("[foo]: /f&ouml;&ouml; \"f&ouml;&ouml;\"\n\n[foo]"),
        "<p><a href=\"/f%C3%B6%C3%B6\" title=\"föö\">foo</a></p>",
        "should support character references in definition URLs and titles"
    );

    assert_eq!(
        to_html("``` f&ouml;&ouml;\nfoo\n```"),
        "<pre><code class=\"language-föö\">foo\n</code></pre>",
        "should support character references in code language"
    );

    assert_eq!(
        to_html("`f&ouml;&ouml;`"),
        "<p><code>f&amp;ouml;&amp;ouml;</code></p>",
        "should not support character references in text code"
    );

    assert_eq!(
        to_html("    f&ouml;f&ouml;"),
        "<pre><code>f&amp;ouml;f&amp;ouml;\n</code></pre>",
        "should not support character references in indented code"
    );

    assert_eq!(
        to_html("&#42;foo&#42;\n*foo*"),
        "<p>*foo*\n<em>foo</em></p>",
        "should not support character references as construct markers (1)"
    );

    assert_eq!(
        to_html("&#42; foo\n\n* foo"),
        "<p>* foo</p>\n<ul>\n<li>foo</li>\n</ul>",
        "should not support character references as construct markers (2)"
    );

    assert_eq!(
        to_html("[a](url &quot;tit&quot;)"),
        "<p>[a](url &quot;tit&quot;)</p>",
        "should not support character references as construct markers (3)"
    );

    assert_eq!(
        to_html("foo&#10;&#10;bar"),
        "<p>foo\n\nbar</p>",
        "should not support character references as whitespace (1)"
    );

    assert_eq!(
        to_html("&#9;foo"),
        "<p>\tfoo</p>",
        "should not support character references as whitespace (2)"
    );

    // Extra:
    assert_eq!(
        to_html("&CounterClockwiseContourIntegral;"),
        "<p>∳</p>",
        "should support the longest possible named character reference"
    );

    assert_eq!(
        to_html("&#xff9999;"),
        "<p>�</p>",
        "should “support” a longest possible hexadecimal character reference"
    );

    assert_eq!(
        to_html("&#9999999;"),
        "<p>�</p>",
        "should “support” a longest possible decimal character reference"
    );

    assert_eq!(
        to_html("&CounterClockwiseContourIntegrali;"),
        "<p>&amp;CounterClockwiseContourIntegrali;</p>",
        "should not support the longest possible named character reference"
    );

    assert_eq!(
        to_html("&#xff99999;"),
        "<p>&amp;#xff99999;</p>",
        "should not support a longest possible hexadecimal character reference"
    );

    assert_eq!(
        to_html("&#99999999;"),
        "<p>&amp;#99999999;</p>",
        "should not support a longest possible decimal character reference"
    );

    assert_eq!(
        to_html("&-;"),
        "<p>&amp;-;</p>",
        "should not support the other characters after `&`"
    );

    assert_eq!(
        to_html("&#-;"),
        "<p>&amp;#-;</p>",
        "should not support the other characters after `#`"
    );

    assert_eq!(
        to_html("&#x-;"),
        "<p>&amp;#x-;</p>",
        "should not support the other characters after `#x`"
    );

    assert_eq!(
        to_html("&lt-;"),
        "<p>&amp;lt-;</p>",
        "should not support the other characters inside a name"
    );

    assert_eq!(
        to_html("&#9-;"),
        "<p>&amp;#9-;</p>",
        "should not support the other characters inside a demical"
    );

    assert_eq!(
        to_html("&#x9-;"),
        "<p>&amp;#x9-;</p>",
        "should not support the other characters inside a hexademical"
    );

    assert_eq!(
        to_html_with_options(
            "&amp;",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        character_reference: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>&amp;amp;</p>",
        "should support turning off character references"
    );

    assert_eq!(
        to_mdast("&nbsp; &amp; &copy; &AElig; &Dcaron;\n&frac34; &HilbertSpace; &DifferentialD;\n&ClockwiseContourIntegral; &ngE;\n&#35; &#1234; &#992; &#0;\n&#X22; &#XD06; &#xcab;", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::Text(Text {
                    value: "\u{a0} & © Æ Ď\n¾ ℋ ⅆ\n∲ ≧̸\n# Ӓ Ϡ �\n\" ആ ಫ".into(),
                    position: Some(Position::new(1, 1, 0, 5, 23, 158))
                }),],
                position: Some(Position::new(1, 1, 0, 5, 23, 158))
            })],
            position: Some(Position::new(1, 1, 0, 5, 23, 158))
        }),
        "should support character references as `Text`s in mdast"
    );

    Ok(())
}
