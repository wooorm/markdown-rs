use markdown::{
    mdast::{Html, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    CompileOptions, Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn html_text() -> Result<(), message::Message> {
    let danger = Options {
        compile: CompileOptions {
            allow_dangerous_html: true,
            allow_dangerous_protocol: true,
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html("a <b> c"),
        "<p>a &lt;b&gt; c</p>",
        "should encode dangerous html by default"
    );

    assert_eq!(
        to_html_with_options("<a><bab><c2c>", &danger)?,
        "<p><a><bab><c2c></p>",
        "should support opening tags"
    );

    assert_eq!(
        to_html_with_options("<a/><b2/>", &danger)?,
        "<p><a/><b2/></p>",
        "should support self-closing tags"
    );

    assert_eq!(
        to_html_with_options("<a  /><b2\ndata=\"foo\" >", &danger)?,
        "<p><a  /><b2\ndata=\"foo\" ></p>",
        "should support whitespace in tags"
    );

    assert_eq!(
        to_html_with_options(
            "<a foo=\"bar\" bam = 'baz <em>\"</em>'\n_boolean zoop:33=zoop:33 />",
            &danger
        )?,
        "<p><a foo=\"bar\" bam = 'baz <em>\"</em>'\n_boolean zoop:33=zoop:33 /></p>",
        "should support attributes on tags"
    );

    assert_eq!(
        to_html_with_options("Foo <responsive-image src=\"foo.jpg\" />", &danger)?,
        "<p>Foo <responsive-image src=\"foo.jpg\" /></p>",
        "should support non-html tags"
    );

    assert_eq!(
        to_html_with_options("<33> <__>", &danger)?,
        "<p>&lt;33&gt; &lt;__&gt;</p>",
        "should not support nonconforming tag names"
    );

    assert_eq!(
        to_html_with_options("<a h*#ref=\"hi\">", &danger)?,
        "<p>&lt;a h*#ref=&quot;hi&quot;&gt;</p>",
        "should not support nonconforming attribute names"
    );

    assert_eq!(
        to_html_with_options("<a href=\"hi'> <a href=hi'>", &danger)?,
        "<p>&lt;a href=&quot;hi'&gt; &lt;a href=hi'&gt;</p>",
        "should not support nonconforming attribute values"
    );

    assert_eq!(
        to_html_with_options("< a><\nfoo><bar/ >\n<foo bar=baz\nbim!bop />", &danger)?,
        "<p>&lt; a&gt;&lt;\nfoo&gt;&lt;bar/ &gt;\n&lt;foo bar=baz\nbim!bop /&gt;</p>",
        "should not support nonconforming whitespace"
    );

    assert_eq!(
        to_html_with_options("<a href='bar'title=title>", &danger)?,
        "<p>&lt;a href='bar'title=title&gt;</p>",
        "should not support missing whitespace"
    );

    assert_eq!(
        to_html_with_options("</a></foo >", &danger)?,
        "<p></a></foo ></p>",
        "should support closing tags"
    );

    assert_eq!(
        to_html_with_options("</a href=\"foo\">", &danger)?,
        "<p>&lt;/a href=&quot;foo&quot;&gt;</p>",
        "should not support closing tags w/ attributes"
    );

    assert_eq!(
        to_html_with_options("foo <!-- this is a\ncomment - with hyphen -->", &danger)?,
        "<p>foo <!-- this is a\ncomment - with hyphen --></p>",
        "should support comments"
    );

    assert_eq!(
        to_html_with_options("foo <!-- not a comment -- two hyphens -->", &danger)?,
        "<p>foo <!-- not a comment -- two hyphens --></p>",
        "should support comments w/ two dashes inside"
    );

    assert_eq!(
        to_html_with_options("foo <!--> foo -->", &danger)?,
        "<p>foo <!--> foo --&gt;</p>",
        "should support nonconforming comments (1)"
    );

    assert_eq!(
        to_html_with_options("foo <!-- foo--->", &danger)?,
        "<p>foo <!-- foo---></p>",
        "should support nonconforming comments (2)"
    );

    assert_eq!(
        to_html_with_options("foo <?php echo $a; ?>", &danger)?,
        "<p>foo <?php echo $a; ?></p>",
        "should support instructions"
    );

    assert_eq!(
        to_html_with_options("foo <!ELEMENT br EMPTY>", &danger)?,
        "<p>foo <!ELEMENT br EMPTY></p>",
        "should support declarations"
    );

    assert_eq!(
        to_html_with_options("foo <![CDATA[>&<]]>", &danger)?,
        "<p>foo <![CDATA[>&<]]></p>",
        "should support cdata"
    );

    assert_eq!(
        to_html_with_options("foo <a href=\"&ouml;\">", &danger)?,
        "<p>foo <a href=\"&ouml;\"></p>",
        "should support (ignore) character references"
    );

    assert_eq!(
        to_html_with_options("foo <a href=\"\\*\">", &danger)?,
        "<p>foo <a href=\"\\*\"></p>",
        "should not support character escapes (1)"
    );

    assert_eq!(
        to_html_with_options("<a href=\"\\\"\">", &danger)?,
        "<p>&lt;a href=&quot;&quot;&quot;&gt;</p>",
        "should not support character escapes (2)"
    );

    // Extra:
    assert_eq!(
        to_html_with_options("foo <!1>", &danger)?,
        "<p>foo &lt;!1&gt;</p>",
        "should not support non-comment, non-cdata, and non-named declaration"
    );

    assert_eq!(
        to_html_with_options("foo <!-not enough!-->", &danger)?,
        "<p>foo &lt;!-not enough!--&gt;</p>",
        "should not support comments w/ not enough dashes"
    );

    assert_eq!(
        to_html_with_options("foo <!---ok-->", &danger)?,
        "<p>foo <!---ok--></p>",
        "should support comments that start w/ a dash, if it’s not followed by a greater than"
    );

    assert_eq!(
        to_html_with_options("foo <!--->", &danger)?,
        "<p>foo <!---></p>",
        "should support comments that start w/ `->`"
    );

    assert_eq!(
        to_html_with_options("foo <!-- -> -->", &danger)?,
        "<p>foo <!-- -> --></p>",
        "should support `->` in a comment"
    );

    assert_eq!(
        to_html_with_options("foo <!--", &danger)?,
        "<p>foo &lt;!--</p>",
        "should not support eof in a comment (1)"
    );

    assert_eq!(
        to_html_with_options("foo <!--a", &danger)?,
        "<p>foo &lt;!--a</p>",
        "should not support eof in a comment (2)"
    );

    assert_eq!(
        to_html_with_options("foo <!--a-", &danger)?,
        "<p>foo &lt;!--a-</p>",
        "should not support eof in a comment (3)"
    );

    assert_eq!(
        to_html_with_options("foo <!--a--", &danger)?,
        "<p>foo &lt;!--a--</p>",
        "should not support eof in a comment (4)"
    );

    // Note: cmjs parses this differently.
    // See: <https://github.com/commonmark/commonmark.js/issues/193>
    assert_eq!(
        to_html_with_options("foo <![cdata[]]>", &danger)?,
        "<p>foo &lt;![cdata[]]&gt;</p>",
        "should not support lowercase “cdata”"
    );

    assert_eq!(
        to_html_with_options("foo <![CDATA", &danger)?,
        "<p>foo &lt;![CDATA</p>",
        "should not support eof in a CDATA (1)"
    );

    assert_eq!(
        to_html_with_options("foo <![CDATA[", &danger)?,
        "<p>foo &lt;![CDATA[</p>",
        "should not support eof in a CDATA (2)"
    );

    assert_eq!(
        to_html_with_options("foo <![CDATA[]", &danger)?,
        "<p>foo &lt;![CDATA[]</p>",
        "should not support eof in a CDATA (3)"
    );

    assert_eq!(
        to_html_with_options("foo <![CDATA[]]", &danger)?,
        "<p>foo &lt;![CDATA[]]</p>",
        "should not support eof in a CDATA (4)"
    );

    assert_eq!(
        to_html_with_options("foo <![CDATA[asd", &danger)?,
        "<p>foo &lt;![CDATA[asd</p>",
        "should not support eof in a CDATA (5)"
    );

    assert_eq!(
        to_html_with_options("foo <![CDATA[]]]]>", &danger)?,
        "<p>foo <![CDATA[]]]]></p>",
        "should support end-like constructs in CDATA"
    );

    assert_eq!(
        to_html_with_options("foo <!doctype", &danger)?,
        "<p>foo &lt;!doctype</p>",
        "should not support eof in declarations"
    );

    assert_eq!(
        to_html_with_options("foo <?php", &danger)?,
        "<p>foo &lt;?php</p>",
        "should not support eof in instructions (1)"
    );

    assert_eq!(
        to_html_with_options("foo <?php?", &danger)?,
        "<p>foo &lt;?php?</p>",
        "should not support eof in instructions (2)"
    );

    assert_eq!(
        to_html_with_options("foo <???>", &danger)?,
        "<p>foo <???></p>",
        "should support question marks in instructions"
    );

    assert_eq!(
        to_html_with_options("foo </3>", &danger)?,
        "<p>foo &lt;/3&gt;</p>",
        "should not support closing tags that don’t start w/ alphas"
    );

    assert_eq!(
        to_html_with_options("foo </a->", &danger)?,
        "<p>foo </a-></p>",
        "should support dashes in closing tags"
    );

    assert_eq!(
        to_html_with_options("foo </a   >", &danger)?,
        "<p>foo </a   ></p>",
        "should support whitespace after closing tag names"
    );

    assert_eq!(
        to_html_with_options("foo </a!>", &danger)?,
        "<p>foo &lt;/a!&gt;</p>",
        "should not support other characters after closing tag names"
    );

    assert_eq!(
        to_html_with_options("foo <a->", &danger)?,
        "<p>foo <a-></p>",
        "should support dashes in opening tags"
    );

    assert_eq!(
        to_html_with_options("foo <a   >", &danger)?,
        "<p>foo <a   ></p>",
        "should support whitespace after opening tag names"
    );

    assert_eq!(
        to_html_with_options("foo <a!>", &danger)?,
        "<p>foo &lt;a!&gt;</p>",
        "should not support other characters after opening tag names"
    );

    assert_eq!(
        to_html_with_options("foo <a !>", &danger)?,
        "<p>foo &lt;a !&gt;</p>",
        "should not support other characters in opening tags (1)"
    );

    assert_eq!(
        to_html_with_options("foo <a b!>", &danger)?,
        "<p>foo &lt;a b!&gt;</p>",
        "should not support other characters in opening tags (2)"
    );

    assert_eq!(
        to_html_with_options("foo <a b/>", &danger)?,
        "<p>foo <a b/></p>",
        "should support a self-closing slash after an attribute name"
    );

    assert_eq!(
        to_html_with_options("foo <a b>", &danger)?,
        "<p>foo <a b></p>",
        "should support a greater than after an attribute name"
    );

    assert_eq!(
        to_html_with_options("foo <a b=<>", &danger)?,
        "<p>foo &lt;a b=&lt;&gt;</p>",
        "should not support less than to start an unquoted attribute value"
    );

    assert_eq!(
        to_html_with_options("foo <a b=>>", &danger)?,
        "<p>foo &lt;a b=&gt;&gt;</p>",
        "should not support greater than to start an unquoted attribute value"
    );

    assert_eq!(
        to_html_with_options("foo <a b==>", &danger)?,
        "<p>foo &lt;a b==&gt;</p>",
        "should not support equals to to start an unquoted attribute value"
    );

    assert_eq!(
        to_html_with_options("foo <a b=`>", &danger)?,
        "<p>foo &lt;a b=`&gt;</p>",
        "should not support grave accent to start an unquoted attribute value"
    );

    assert_eq!(
        to_html_with_options("foo <a b=\"asd", &danger)?,
        "<p>foo &lt;a b=&quot;asd</p>",
        "should not support eof in double quoted attribute value"
    );

    assert_eq!(
        to_html_with_options("foo <a b='asd", &danger)?,
        "<p>foo &lt;a b='asd</p>",
        "should not support eof in single quoted attribute value"
    );

    assert_eq!(
        to_html_with_options("foo <a b=asd", &danger)?,
        "<p>foo &lt;a b=asd</p>",
        "should not support eof in unquoted attribute value"
    );

    assert_eq!(
        to_html_with_options("foo <a b=\nasd>", &danger)?,
        "<p>foo <a b=\nasd></p>",
        "should support an eol before an attribute value"
    );

    assert_eq!(
to_html_with_options("<x> a", &danger)?,
"<p><x> a</p>",
"should support starting a line w/ a tag if followed by anything other than an eol (after optional space/tabs)"
);

    assert_eq!(
        to_html_with_options("<span foo=", &danger)?,
        "<p>&lt;span foo=</p>",
        "should support an EOF before an attribute value"
    );

    assert_eq!(
        to_html_with_options("a <!b\nc>", &danger)?,
        "<p>a <!b\nc></p>",
        "should support an EOL in a declaration"
    );
    assert_eq!(
        to_html_with_options("a <![CDATA[\n]]>", &danger)?,
        "<p>a <![CDATA[\n]]></p>",
        "should support an EOL in cdata"
    );

    // Note: cmjs parses this differently.
    // See: <https://github.com/commonmark/commonmark.js/issues/196>
    assert_eq!(
        to_html_with_options("a <?\n?>", &danger)?,
        "<p>a <?\n?></p>",
        "should support an EOL in an instruction"
    );

    assert_eq!(
        to_html_with_options(
            "a <x>",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        html_text: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>a &lt;x&gt;</p>",
        "should support turning off html (text)"
    );

    assert_eq!(
        to_mdast("alpha <i>bravo</b> charlie.", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "alpha ".into(),
                        position: Some(Position::new(1, 1, 0, 1, 7, 6))
                    }),
                    Node::Html(Html {
                        value: "<i>".into(),
                        position: Some(Position::new(1, 7, 6, 1, 10, 9))
                    }),
                    Node::Text(Text {
                        value: "bravo".into(),
                        position: Some(Position::new(1, 10, 9, 1, 15, 14))
                    }),
                    Node::Html(Html {
                        value: "</b>".into(),
                        position: Some(Position::new(1, 15, 14, 1, 19, 18))
                    }),
                    Node::Text(Text {
                        value: " charlie.".into(),
                        position: Some(Position::new(1, 19, 18, 1, 28, 27))
                    })
                ],
                position: Some(Position::new(1, 1, 0, 1, 28, 27))
            })],
            position: Some(Position::new(1, 1, 0, 1, 28, 27))
        }),
        "should support HTML (text) as `Html`s in mdast"
    );

    Ok(())
}
