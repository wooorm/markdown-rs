use markdown::{mdast::Node, message, to_html_with_options, to_mdast, Options, ParseOptions};
use pretty_assertions::assert_eq;

fn options() -> Options {
    Options {
        parse: ParseOptions {
            cjk_friendly_emphasis: true,
            ..ParseOptions::default()
        },
        ..Options::default()
    }
}

#[test]
fn cjk_punctuation() -> Result<(), message::Message> {
    for (source, expected) in [
        (
            "**意义：**AI 训练数据",
            "<p><strong>意义：</strong>AI 训练数据</p>",
        ),
        (
            "**可信度：较高。**多家媒体",
            "<p><strong>可信度：较高。</strong>多家媒体</p>",
        ),
        ("前**「引用」**後", "<p>前<strong>「引用」</strong>後</p>"),
        (
            "**この文。**次の文",
            "<p><strong>この文。</strong>次の文</p>",
        ),
        (
            "**강조(parenthesis)**가",
            "<p><strong>강조(parenthesis)</strong>가</p>",
        ),
        ("前**(引用)**後", "<p>前<strong>(引用)</strong>後</p>"),
        ("前*「引用」*後", "<p>前<em>「引用」</em>後</p>"),
        (
            "前**半角｢引用｣**後",
            "<p>前<strong>半角｢引用｣</strong>後</p>",
        ),
        (
            "**意义：**正文 **可信度：高。**后文",
            "<p><strong>意义：</strong>正文 <strong>可信度：高。</strong>后文</p>",
        ),
    ] {
        assert_eq!(
            to_html_with_options(source, &options())?,
            expected,
            "{}",
            source
        );
    }
    Ok(())
}

#[test]
fn nested_inline_content() -> Result<(), message::Message> {
    for (source, expected) in [
        ("**意义：&#32;**正文", "<p><strong>意义： </strong>正文</p>"),
        (
            "**外层*内层*。**后文",
            "<p><strong>外层<em>内层</em>。</strong>后文</p>",
        ),
        (
            "**含有`代码`。**后文",
            "<p><strong>含有<code>代码</code>。</strong>后文</p>",
        ),
        (
            "**链接：[文档](https://example.com)。**后文",
            "<p><strong>链接：<a href=\"https://example.com\">文档</a>。</strong>后文</p>",
        ),
    ] {
        assert_eq!(
            to_html_with_options(source, &options())?,
            expected,
            "{}",
            source
        );
    }
    Ok(())
}

#[test]
fn whitespace_code_escapes_and_ascii_keep_their_meaning() -> Result<(), message::Message> {
    for source in [
        "**意义： **正文",
        "** 意义：**正文",
        "**意义：\t**正文",
        "**意义：\u{3000}**正文",
        "**意义：",
        "****",
        "`**意义：**正文`",
        "```md\n**意义：**正文\n```",
        "    **意义：**正文",
        r"\*\*意义：\*\*正文",
        "[链接](https://example.com/**x**)",
        "![图](https://example.com/**x**.png)",
        "**label:**text",
        "a**(label)**b",
        "foo_bar_baz",
        "中_文_字",
        "a_\"b\"_c",
    ] {
        assert_eq!(
            to_html_with_options(source, &options())?,
            to_html_with_options(source, &Options::default())?,
            "{}",
            source
        );
    }
    Ok(())
}

#[test]
fn cjk_and_emoji_are_distinct() -> Result<(), message::Message> {
    for (source, expected) in [
        ("**好。**😀", "<p><strong>好。</strong>😀</p>"),
        ("**a!**😀", "<p><strong>a!</strong>😀</p>"),
        ("**a😀**b", "<p>**a😀**b</p>"),
        ("**a㊗**b", "<p><strong>a㊗</strong>b</p>"),
        ("😀**(a)**b", "<p>😀**(a)**b</p>"),
        ("㊗**(a)**中", "<p>㊗<strong>(a)</strong>中</p>"),
        ("\u{1160}**(a)**中", "<p>\u{1160}<strong>(a)</strong>中</p>"),
        ("𠮷**(a)**中", "<p>𠮷<strong>(a)</strong>中</p>"),
    ] {
        assert_eq!(
            to_html_with_options(source, &options())?,
            expected,
            "{}",
            source
        );
    }
    Ok(())
}

#[test]
fn variation_selectors() -> Result<(), message::Message> {
    for (source, expected) in [
        ("**字。\u{FE00}**a", "<p><strong>字。\u{FE00}</strong>a</p>"),
        ("**字。\u{FE0E}**a", "<p><strong>字。\u{FE0E}</strong>a</p>"),
        ("**字。\u{FE0F}**a", "<p><strong>字。\u{FE0F}</strong>a</p>"),
        ("**a!\u{FE00}**b", "<p>**a!\u{FE00}**b</p>"),
        ("**a’\u{FE01}**b", "<p><strong>a’\u{FE01}</strong>b</p>"),
        ("**a’\u{FE00}**b", "<p>**a’\u{FE00}**b</p>"),
        (
            "字\u{E0100}**(a)**中",
            "<p>字\u{E0100}<strong>(a)</strong>中</p>",
        ),
        ("**a!\u{E0100}**b", "<p><strong>a!\u{E0100}</strong>b</p>"),
    ] {
        assert_eq!(
            to_html_with_options(source, &options())?,
            expected,
            "{}",
            source
        );
    }
    for source in ["** \u{FE00}**x", "**\u{FE00}**x", "**\u{E0100}\u{FE00}**x"] {
        let _ = to_html_with_options(source, &options())?;
    }
    Ok(())
}

#[test]
fn opt_in_for_each_parse() -> Result<(), message::Message> {
    let source = "**意义：**正文";
    for preset in [
        Options::default(),
        Options::gfm(),
        Options {
            parse: ParseOptions::mdx(),
            ..Options::default()
        },
    ] {
        assert!(!preset.parse.cjk_friendly_emphasis);
        assert_eq!(
            to_html_with_options(source, &preset)?,
            "<p>**意义：**正文</p>"
        );
    }
    let mut enabled = options();
    assert_eq!(
        to_html_with_options(source, &enabled)?,
        "<p><strong>意义：</strong>正文</p>"
    );
    enabled.parse.cjk_friendly_emphasis = false;
    assert_eq!(
        to_html_with_options(source, &enabled)?,
        "<p>**意义：**正文</p>"
    );
    Ok(())
}

#[test]
fn gfm_strikethrough_is_separate() -> Result<(), message::Message> {
    let mut enabled = Options::gfm();
    enabled.parse.cjk_friendly_emphasis = true;
    for source in ["~~删除。~~后文", "~删除。~后文", "~~a!~~字"] {
        assert_eq!(
            to_html_with_options(source, &enabled)?,
            to_html_with_options(source, &Options::gfm())?
        );
    }
    assert_eq!(
        to_html_with_options("~~删除~~，**意义：**正文", &enabled)?,
        "<p><del>删除</del>，<strong>意义：</strong>正文</p>"
    );
    assert_eq!(to_html_with_options("| 列 |\n| --- |\n| **意义：**正文 |", &enabled)?, "<table>\n<thead>\n<tr>\n<th>列</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td><strong>意义：</strong>正文</td>\n</tr>\n</tbody>\n</table>");
    Ok(())
}

#[test]
fn mdast_keeps_original_source_positions() -> Result<(), message::Message> {
    let source = "前 **意义：**正文";
    let tree = to_mdast(source, &options().parse)?;
    let paragraph = &tree.children().unwrap()[0];
    let children = paragraph.children().unwrap();
    assert!(matches!(children[1], Node::Strong(_)));
    let position = children[1].position().unwrap();
    assert_eq!(
        &source[position.start.offset..position.end.offset],
        "**意义：**"
    );
    let text = &children[1].children().unwrap()[0];
    let text_position = text.position().unwrap();
    assert_eq!(
        &source[text_position.start.offset..text_position.end.offset],
        "意义："
    );
    assert_eq!(text.to_string(), "意义：");
    assert_eq!(children[2].to_string(), "正文");
    let mut mdx = ParseOptions::mdx();
    mdx.cjk_friendly_emphasis = true;
    let mdx_tree = to_mdast("**意义：**正文", &mdx)?;
    assert!(matches!(
        mdx_tree.children().unwrap()[0].children().unwrap()[0],
        Node::Strong(_)
    ));
    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn configuration_roundtrip_and_missing_field() {
    let json = serde_json::to_string(&options().parse).unwrap();
    assert!(json.contains("\"cjkFriendlyEmphasis\":true"));
    let restored: ParseOptions = serde_json::from_str(&json).unwrap();
    assert!(restored.cjk_friendly_emphasis);
    let old = json.replace("\"cjkFriendlyEmphasis\":true,", "");
    let restored: ParseOptions = serde_json::from_str(&old).unwrap();
    assert!(!restored.cjk_friendly_emphasis);
}
