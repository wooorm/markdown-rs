extern crate micromark;
use micromark::{micromark, micromark_with_options, CompileOptions};

const DANGER: &CompileOptions = &CompileOptions {
    allow_dangerous_html: true,
    allow_dangerous_protocol: true,
};

#[test]
fn character_reference() {
    assert_eq!(
      micromark(
        "&nbsp; &amp; &copy; &AElig; &Dcaron;\n&frac34; &HilbertSpace; &DifferentialD;\n&ClockwiseContourIntegral; &ngE;"
      ),
      "<p>  &amp; © Æ Ď\n¾ ℋ ⅆ\n∲ ≧̸</p>",
      "should support named character references"
    );

    assert_eq!(
        micromark("&#35; &#1234; &#992; &#0;"),
        "<p># Ӓ Ϡ �</p>",
        "should support decimal character references"
    );

    assert_eq!(
        micromark("&#X22; &#XD06; &#xcab;"),
        "<p>&quot; ആ ಫ</p>",
        "should support hexadecimal character references"
    );

    assert_eq!(
      micromark(
        "&nbsp &x; &#; &#x;\n&#987654321;\n&#abcdef0;\n&ThisIsNotDefined; &hi?;"
      ),
      "<p>&amp;nbsp &amp;x; &amp;#; &amp;#x;\n&amp;#987654321;\n&amp;#abcdef0;\n&amp;ThisIsNotDefined; &amp;hi?;</p>",
      "should not support other things that look like character references"
    );

    assert_eq!(
        micromark("&copy"),
        "<p>&amp;copy</p>",
        "should not support character references w/o semicolon"
    );

    assert_eq!(
        micromark("&MadeUpEntity;"),
        "<p>&amp;MadeUpEntity;</p>",
        "should not support unknown named character references"
    );

    assert_eq!(
        micromark_with_options("<a href=\"&ouml;&ouml;.html\">", DANGER),
        "<a href=\"&ouml;&ouml;.html\">",
        "should not care about character references in html"
    );

    // To do: link (resource).
    // assert_eq!(
    //     micromark("[foo](/f&ouml;&ouml; \"f&ouml;&ouml;\")"),
    //     "<p><a href=\"/f%C3%B6%C3%B6\" title=\"föö\">foo</a></p>",
    //     "should support character references in resource URLs and titles"
    // );

    // To do: definition.
    // assert_eq!(
    //     micromark("[foo]: /f&ouml;&ouml; \"f&ouml;&ouml;\"\n\n[foo]"),
    //     "<p><a href=\"/f%C3%B6%C3%B6\" title=\"föö\">foo</a></p>",
    //     "should support character references in definition URLs and titles"
    // );

    assert_eq!(
        micromark("``` f&ouml;&ouml;\nfoo\n```"),
        "<pre><code class=\"language-föö\">foo\n</code></pre>",
        "should support character references in code language"
    );

    assert_eq!(
        micromark("`f&ouml;&ouml;`"),
        "<p><code>f&amp;ouml;&amp;ouml;</code></p>",
        "should not support character references in text code"
    );

    assert_eq!(
        micromark("    f&ouml;f&ouml;"),
        "<pre><code>f&amp;ouml;f&amp;ouml;\n</code></pre>",
        "should not support character references in indented code"
    );

    // To do: attention.
    // assert_eq!(
    //     micromark("&#42;foo&#42;\n*foo*"),
    //     "<p>*foo*\n<em>foo</em></p>",
    //     "should not support character references as construct markers (1)"
    // );

    // To do: list.
    // assert_eq!(
    //     micromark("&#42; foo\n\n* foo"),
    //     "<p>* foo</p>\n<ul>\n<li>foo</li>\n</ul>",
    //     "should not support character references as construct markers (2)"
    // );

    // To do: link.
    // assert_eq!(
    //     micromark("[a](url &quot;tit&quot;)"),
    //     "<p>[a](url &quot;tit&quot;)</p>",
    //     "should not support character references as construct markers (3)"
    // );

    assert_eq!(
        micromark("foo&#10;&#10;bar"),
        "<p>foo\n\nbar</p>",
        "should not support character references as whitespace (1)"
    );

    assert_eq!(
        micromark("&#9;foo"),
        "<p>\tfoo</p>",
        "should not support character references as whitespace (2)"
    );

    // Extra:
    assert_eq!(
        micromark("&CounterClockwiseContourIntegral;"),
        "<p>∳</p>",
        "should support the longest possible named character reference"
    );

    assert_eq!(
        micromark("&#xff9999;"),
        "<p>�</p>",
        "should “support” a longest possible hexadecimal character reference"
    );

    assert_eq!(
        micromark("&#9999999;"),
        "<p>�</p>",
        "should “support” a longest possible decimal character reference"
    );

    assert_eq!(
        micromark("&CounterClockwiseContourIntegrali;"),
        "<p>&amp;CounterClockwiseContourIntegrali;</p>",
        "should not support the longest possible named character reference"
    );

    assert_eq!(
        micromark("&#xff99999;"),
        "<p>&amp;#xff99999;</p>",
        "should not support a longest possible hexadecimal character reference"
    );

    assert_eq!(
        micromark("&#99999999;"),
        "<p>&amp;#99999999;</p>",
        "should not support a longest possible decimal character reference"
    );

    assert_eq!(
        micromark("&-;"),
        "<p>&amp;-;</p>",
        "should not support the other characters after `&`"
    );

    assert_eq!(
        micromark("&#-;"),
        "<p>&amp;#-;</p>",
        "should not support the other characters after `#`"
    );

    assert_eq!(
        micromark("&#x-;"),
        "<p>&amp;#x-;</p>",
        "should not support the other characters after `#x`"
    );

    assert_eq!(
        micromark("&lt-;"),
        "<p>&amp;lt-;</p>",
        "should not support the other characters inside a name"
    );

    assert_eq!(
        micromark("&#9-;"),
        "<p>&amp;#9-;</p>",
        "should not support the other characters inside a demical"
    );

    assert_eq!(
        micromark("&#x9-;"),
        "<p>&amp;#x9-;</p>",
        "should not support the other characters inside a hexademical"
    );

    // To do: extensions.
    // assert_eq!(
    //   micromark("&amp;", {
    //     extensions: [{disable: {null: ["characterReferences"]}}]
    //   }),
    //   "<p>&amp;</p>",
    //   "should support turning off character references"
    // );
}