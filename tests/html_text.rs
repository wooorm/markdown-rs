extern crate micromark;
use micromark::{micromark, micromark_with_options, CompileOptions};

const DANGER: &CompileOptions = &CompileOptions {
    allow_dangerous_html: true,
    allow_dangerous_protocol: false,
};

#[test]
fn html_text() {
    assert_eq!(
        micromark("a <b> c"),
        "<p>a &lt;b&gt; c</p>",
        "should encode dangerous html by default"
    );

    assert_eq!(
        micromark_with_options("<a><bab><c2c>", DANGER),
        "<p><a><bab><c2c></p>",
        "should support opening tags"
    );

    assert_eq!(
        micromark_with_options("<a/><b2/>", DANGER),
        "<p><a/><b2/></p>",
        "should support self-closing tags"
    );

    // To do: line endings.
    // assert_eq!(
    //     micromark_with_options("<a  /><b2\ndata=\"foo\" >", DANGER),
    //     "<p><a  /><b2\ndata=\"foo\" ></p>",
    //     "should support whitespace in tags"
    // );

    // To do: line endings.
    // assert_eq!(
    //     micromark_with_options(
    //         "<a foo=\"bar\" bam = \"baz <em>\"</em>\"\n_boolean zoop:33=zoop:33 />",
    //         DANGER
    //     ),
    //     "<p><a foo=\"bar\" bam = \"baz <em>\"</em>\"\n_boolean zoop:33=zoop:33 /></p>",
    //     "should support attributes on tags"
    // );

    assert_eq!(
        micromark_with_options("Foo <responsive-image src=\"foo.jpg\" />", DANGER),
        "<p>Foo <responsive-image src=\"foo.jpg\" /></p>",
        "should support non-html tags"
    );

    assert_eq!(
        micromark_with_options("<33> <__>", DANGER),
        "<p>&lt;33&gt; &lt;__&gt;</p>",
        "should not support nonconforming tag names"
    );

    assert_eq!(
        micromark_with_options("<a h*#ref=\"hi\">", DANGER),
        "<p>&lt;a h*#ref=&quot;hi&quot;&gt;</p>",
        "should not support nonconforming attribute names"
    );

    assert_eq!(
        micromark_with_options("<a href=\"hi'> <a href=hi'>", DANGER),
        "<p>&lt;a href=&quot;hi'&gt; &lt;a href=hi'&gt;</p>",
        "should not support nonconforming attribute values"
    );

    // To do: line endings.
    // assert_eq!(
    //     micromark_with_options("< a><\nfoo><bar/ >\n<foo bar=baz\nbim!bop />", DANGER),
    //     "<p>&lt; a&gt;&lt;\nfoo&gt;&lt;bar/ &gt;\n&lt;foo bar=baz\nbim!bop /&gt;</p>",
    //     "should not support nonconforming whitespace"
    // );

    assert_eq!(
        micromark_with_options("<a href='bar'title=title>", DANGER),
        "<p>&lt;a href='bar'title=title&gt;</p>",
        "should not support missing whitespace"
    );

    assert_eq!(
        micromark_with_options("</a></foo >", DANGER),
        "<p></a></foo ></p>",
        "should support closing tags"
    );

    assert_eq!(
        micromark_with_options("</a href=\"foo\">", DANGER),
        "<p>&lt;/a href=&quot;foo&quot;&gt;</p>",
        "should not support closing tags w/ attributes"
    );

    // To do: line endings.
    //     assert_eq!(
    //         micromark_with_options("foo <!-- this is a\ncomment - with hyphen -->", DANGER),
    //         "<p>foo <!-- this is a\ncomment - with hyphen --></p>",
    //         "should support comments"
    //     );

    assert_eq!(
        micromark_with_options("foo <!-- not a comment -- two hyphens -->", DANGER),
        "<p>foo &lt;!-- not a comment -- two hyphens --&gt;</p>",
        "should not support comments w/ two dashes inside"
    );

    assert_eq!(
        micromark_with_options("foo <!--> foo -->", DANGER),
        "<p>foo &lt;!--&gt; foo --&gt;</p>",
        "should not support nonconforming comments (1)"
    );

    assert_eq!(
        micromark_with_options("foo <!-- foo--->", DANGER),
        "<p>foo &lt;!-- foo---&gt;</p>",
        "should not support nonconforming comments (2)"
    );

    assert_eq!(
        micromark_with_options("foo <?php echo $a; ?>", DANGER),
        "<p>foo <?php echo $a; ?></p>",
        "should support instructions"
    );

    assert_eq!(
        micromark_with_options("foo <!ELEMENT br EMPTY>", DANGER),
        "<p>foo <!ELEMENT br EMPTY></p>",
        "should support declarations"
    );

    assert_eq!(
        micromark_with_options("foo <![CDATA[>&<]]>", DANGER),
        "<p>foo <![CDATA[>&<]]></p>",
        "should support cdata"
    );

    assert_eq!(
        micromark_with_options("foo <a href=\"&ouml;\">", DANGER),
        "<p>foo <a href=\"&ouml;\"></p>",
        "should support (ignore) character references"
    );

    assert_eq!(
        micromark_with_options("foo <a href=\"\\*\">", DANGER),
        "<p>foo <a href=\"\\*\"></p>",
        "should not support character escapes (1)"
    );

    assert_eq!(
        micromark_with_options("<a href=\"\\\"\">", DANGER),
        "<p>&lt;a href=&quot;&quot;&quot;&gt;</p>",
        "should not support character escapes (2)"
    );

    // Extra:
    assert_eq!(
        micromark_with_options("foo <!1>", DANGER),
        "<p>foo &lt;!1&gt;</p>",
        "should not support non-comment, non-cdata, and non-named declaration"
    );

    assert_eq!(
        micromark_with_options("foo <!-not enough!-->", DANGER),
        "<p>foo &lt;!-not enough!--&gt;</p>",
        "should not support comments w/ not enough dashes"
    );

    assert_eq!(
        micromark_with_options("foo <!---ok-->", DANGER),
        "<p>foo <!---ok--></p>",
        "should support comments that start w/ a dash, if it’s not followed by a greater than"
    );

    assert_eq!(
        micromark_with_options("foo <!--->", DANGER),
        "<p>foo &lt;!---&gt;</p>",
        "should not support comments that start w/ `->`"
    );

    assert_eq!(
        micromark_with_options("foo <!-- -> -->", DANGER),
        "<p>foo <!-- -> --></p>",
        "should support `->` in a comment"
    );

    assert_eq!(
        micromark_with_options("foo <!--", DANGER),
        "<p>foo &lt;!--</p>",
        "should not support eof in a comment (1)"
    );

    assert_eq!(
        micromark_with_options("foo <!--a", DANGER),
        "<p>foo &lt;!--a</p>",
        "should not support eof in a comment (2)"
    );

    assert_eq!(
        micromark_with_options("foo <!--a-", DANGER),
        "<p>foo &lt;!--a-</p>",
        "should not support eof in a comment (3)"
    );

    assert_eq!(
        micromark_with_options("foo <!--a--", DANGER),
        "<p>foo &lt;!--a--</p>",
        "should not support eof in a comment (4)"
    );

    // Note: cmjs parses this differently.
    // See: <https://github.com/commonmark/commonmark.js/issues/193>
    assert_eq!(
        micromark_with_options("foo <![cdata[]]>", DANGER),
        "<p>foo &lt;![cdata[]]&gt;</p>",
        "should not support lowercase “cdata”"
    );

    assert_eq!(
        micromark_with_options("foo <![CDATA", DANGER),
        "<p>foo &lt;![CDATA</p>",
        "should not support eof in a CDATA (1)"
    );

    assert_eq!(
        micromark_with_options("foo <![CDATA[", DANGER),
        "<p>foo &lt;![CDATA[</p>",
        "should not support eof in a CDATA (2)"
    );

    assert_eq!(
        micromark_with_options("foo <![CDATA[]", DANGER),
        "<p>foo &lt;![CDATA[]</p>",
        "should not support eof in a CDATA (3)"
    );

    assert_eq!(
        micromark_with_options("foo <![CDATA[]]", DANGER),
        "<p>foo &lt;![CDATA[]]</p>",
        "should not support eof in a CDATA (4)"
    );

    assert_eq!(
        micromark_with_options("foo <![CDATA[asd", DANGER),
        "<p>foo &lt;![CDATA[asd</p>",
        "should not support eof in a CDATA (5)"
    );

    assert_eq!(
        micromark_with_options("foo <![CDATA[]]]]>", DANGER),
        "<p>foo <![CDATA[]]]]></p>",
        "should support end-like constructs in CDATA"
    );

    assert_eq!(
        micromark_with_options("foo <!doctype", DANGER),
        "<p>foo &lt;!doctype</p>",
        "should not support eof in declarations"
    );

    assert_eq!(
        micromark_with_options("foo <?php", DANGER),
        "<p>foo &lt;?php</p>",
        "should not support eof in instructions (1)"
    );

    assert_eq!(
        micromark_with_options("foo <?php?", DANGER),
        "<p>foo &lt;?php?</p>",
        "should not support eof in instructions (2)"
    );

    assert_eq!(
        micromark_with_options("foo <???>", DANGER),
        "<p>foo <???></p>",
        "should support question marks in instructions"
    );

    assert_eq!(
        micromark_with_options("foo </3>", DANGER),
        "<p>foo &lt;/3&gt;</p>",
        "should not support closing tags that don’t start w/ alphas"
    );

    assert_eq!(
        micromark_with_options("foo </a->", DANGER),
        "<p>foo </a-></p>",
        "should support dashes in closing tags"
    );

    assert_eq!(
        micromark_with_options("foo </a   >", DANGER),
        "<p>foo </a   ></p>",
        "should support whitespace after closing tag names"
    );

    assert_eq!(
        micromark_with_options("foo </a!>", DANGER),
        "<p>foo &lt;/a!&gt;</p>",
        "should not support other characters after closing tag names"
    );

    assert_eq!(
        micromark_with_options("foo <a->", DANGER),
        "<p>foo <a-></p>",
        "should support dashes in opening tags"
    );

    assert_eq!(
        micromark_with_options("foo <a   >", DANGER),
        "<p>foo <a   ></p>",
        "should support whitespace after opening tag names"
    );

    assert_eq!(
        micromark_with_options("foo <a!>", DANGER),
        "<p>foo &lt;a!&gt;</p>",
        "should not support other characters after opening tag names"
    );

    assert_eq!(
        micromark_with_options("foo <a !>", DANGER),
        "<p>foo &lt;a !&gt;</p>",
        "should not support other characters in opening tags (1)"
    );

    assert_eq!(
        micromark_with_options("foo <a b!>", DANGER),
        "<p>foo &lt;a b!&gt;</p>",
        "should not support other characters in opening tags (2)"
    );

    assert_eq!(
        micromark_with_options("foo <a b/>", DANGER),
        "<p>foo <a b/></p>",
        "should support a self-closing slash after an attribute name"
    );

    assert_eq!(
        micromark_with_options("foo <a b>", DANGER),
        "<p>foo <a b></p>",
        "should support a greater than after an attribute name"
    );

    assert_eq!(
        micromark_with_options("foo <a b=<>", DANGER),
        "<p>foo &lt;a b=&lt;&gt;</p>",
        "should not support less than to start an unquoted attribute value"
    );

    assert_eq!(
        micromark_with_options("foo <a b=>>", DANGER),
        "<p>foo &lt;a b=&gt;&gt;</p>",
        "should not support greater than to start an unquoted attribute value"
    );

    assert_eq!(
        micromark_with_options("foo <a b==>", DANGER),
        "<p>foo &lt;a b==&gt;</p>",
        "should not support equals to to start an unquoted attribute value"
    );

    assert_eq!(
        micromark_with_options("foo <a b=`>", DANGER),
        "<p>foo &lt;a b=`&gt;</p>",
        "should not support grave accent to start an unquoted attribute value"
    );

    assert_eq!(
        micromark_with_options("foo <a b=\"asd", DANGER),
        "<p>foo &lt;a b=&quot;asd</p>",
        "should not support eof in double quoted attribute value"
    );

    assert_eq!(
        micromark_with_options("foo <a b='asd", DANGER),
        "<p>foo &lt;a b='asd</p>",
        "should not support eof in single quoted attribute value"
    );

    assert_eq!(
        micromark_with_options("foo <a b=asd", DANGER),
        "<p>foo &lt;a b=asd</p>",
        "should not support eof in unquoted attribute value"
    );

    // To do: line endings.
    // assert_eq!(
    //     micromark_with_options("foo <a b=\nasd>", DANGER),
    //     "<p>foo <a b=\nasd></p>",
    //     "should support an eol before an attribute value"
    // );

    assert_eq!(
micromark_with_options("<x> a", DANGER),
"<p><x> a</p>",
"should support starting a line w/ a tag if followed by anything other than an eol (after optional space/tabs)"
);

    assert_eq!(
        micromark_with_options("<span foo=", DANGER),
        "<p>&lt;span foo=</p>",
        "should support an EOF before an attribute value"
    );

    // To do: line endings.
    // assert_eq!(
    //     micromark_with_options("a <!b\nc>", DANGER),
    //     "<p>a <!b\nc></p>",
    //     "should support an EOL in a declaration"
    // );
    // To do: line endings.
    // assert_eq!(
    //     micromark_with_options("a <![CDATA[\n]]>", DANGER),
    //     "<p>a <![CDATA[\n]]></p>",
    //     "should support an EOL in cdata"
    // );

    // To do: line endings.
    // // Note: cmjs parses this differently.
    // // See: <https://github.com/commonmark/commonmark.js/issues/196>
    // assert_eq!(
    //     micromark_with_options("a <?\n?>", DANGER),
    //     "<p>a <?\n?></p>",
    //     "should support an EOL in an instruction"
    // );

    //     // To do: extensions.
    //     // assert_eq!(
    //     //     micromark_with_options("a <x>", {extensions: [{disable: {null: ["htmlText"]}}]}),
    //     //     "<p>a &lt;x&gt;</p>",
    //     //     "should support turning off html (text)"
    //     // );
}
