use markdown::to_html;
use pretty_assertions::assert_eq;

#[test]
fn dangerous_protocol_autolink() {
    assert_eq!(
        to_html("<javascript:alert(1)>"),
        "<p><a href=\"\">javascript:alert(1)</a></p>",
        "should be safe by default"
    );

    assert_eq!(
        to_html("<http://a>"),
        "<p><a href=\"http://a\">http://a</a></p>",
        "should allow `http:`"
    );

    assert_eq!(
        to_html("<https://a>"),
        "<p><a href=\"https://a\">https://a</a></p>",
        "should allow `https:`"
    );

    assert_eq!(
        to_html("<irc:///help>"),
        "<p><a href=\"irc:///help\">irc:///help</a></p>",
        "should allow `irc:`"
    );

    assert_eq!(
        to_html("<mailto:a>"),
        "<p><a href=\"mailto:a\">mailto:a</a></p>",
        "should allow `mailto:`"
    );
}

#[test]
fn dangerous_protocol_image() {
    assert_eq!(
        to_html("![](javascript:alert(1))"),
        "<p><img src=\"\" alt=\"\" /></p>",
        "should be safe by default"
    );

    assert_eq!(
        to_html("![](http://a)"),
        "<p><img src=\"http://a\" alt=\"\" /></p>",
        "should allow `http:`"
    );

    assert_eq!(
        to_html("![](https://a)"),
        "<p><img src=\"https://a\" alt=\"\" /></p>",
        "should allow `https:`"
    );

    assert_eq!(
        to_html("![](irc:///help)"),
        "<p><img src=\"\" alt=\"\" /></p>",
        "should not allow `irc:`"
    );

    assert_eq!(
        to_html("![](mailto:a)"),
        "<p><img src=\"\" alt=\"\" /></p>",
        "should not allow `mailto:`"
    );

    assert_eq!(
        to_html("![](#a)"),
        "<p><img src=\"#a\" alt=\"\" /></p>",
        "should allow a hash"
    );

    assert_eq!(
        to_html("![](?a)"),
        "<p><img src=\"?a\" alt=\"\" /></p>",
        "should allow a search"
    );

    assert_eq!(
        to_html("![](/a)"),
        "<p><img src=\"/a\" alt=\"\" /></p>",
        "should allow an absolute"
    );

    assert_eq!(
        to_html("![](./a)"),
        "<p><img src=\"./a\" alt=\"\" /></p>",
        "should allow an relative"
    );

    assert_eq!(
        to_html("![](../a)"),
        "<p><img src=\"../a\" alt=\"\" /></p>",
        "should allow an upwards relative"
    );

    assert_eq!(
        to_html("![](a#b:c)"),
        "<p><img src=\"a#b:c\" alt=\"\" /></p>",
        "should allow a colon in a hash"
    );

    assert_eq!(
        to_html("![](a?b:c)"),
        "<p><img src=\"a?b:c\" alt=\"\" /></p>",
        "should allow a colon in a search"
    );

    assert_eq!(
        to_html("![](a/b:c)"),
        "<p><img src=\"a/b:c\" alt=\"\" /></p>",
        "should allow a colon in a path"
    );
}

#[test]
fn dangerous_protocol_link() {
    assert_eq!(
        to_html("[](javascript:alert(1))"),
        "<p><a href=\"\"></a></p>",
        "should be safe by default"
    );

    assert_eq!(
        to_html("[](http://a)"),
        "<p><a href=\"http://a\"></a></p>",
        "should allow `http:`"
    );

    assert_eq!(
        to_html("[](https://a)"),
        "<p><a href=\"https://a\"></a></p>",
        "should allow `https:`"
    );

    assert_eq!(
        to_html("[](irc:///help)"),
        "<p><a href=\"irc:///help\"></a></p>",
        "should allow `irc:`"
    );

    assert_eq!(
        to_html("[](mailto:a)"),
        "<p><a href=\"mailto:a\"></a></p>",
        "should allow `mailto:`"
    );

    assert_eq!(
        to_html("[](#a)"),
        "<p><a href=\"#a\"></a></p>",
        "should allow a hash"
    );

    assert_eq!(
        to_html("[](?a)"),
        "<p><a href=\"?a\"></a></p>",
        "should allow a search"
    );

    assert_eq!(
        to_html("[](/a)"),
        "<p><a href=\"/a\"></a></p>",
        "should allow an absolute"
    );

    assert_eq!(
        to_html("[](./a)"),
        "<p><a href=\"./a\"></a></p>",
        "should allow an relative"
    );

    assert_eq!(
        to_html("[](../a)"),
        "<p><a href=\"../a\"></a></p>",
        "should allow an upwards relative"
    );

    assert_eq!(
        to_html("[](a#b:c)"),
        "<p><a href=\"a#b:c\"></a></p>",
        "should allow a colon in a hash"
    );

    assert_eq!(
        to_html("[](a?b:c)"),
        "<p><a href=\"a?b:c\"></a></p>",
        "should allow a colon in a search"
    );

    assert_eq!(
        to_html("[](a/b:c)"),
        "<p><a href=\"a/b:c\"></a></p>",
        "should allow a colon in a path"
    );
}

#[test]
fn dangerous_protocol_image_with_option() {
    use markdown::{to_html_with_options, CompileOptions, Options};

    let options = Options {
        compile: CompileOptions {
            allow_any_img_src: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let result = to_html_with_options("![](javascript:alert(1))", &options).unwrap();
    assert_eq!(
        result, "<p><img src=\"javascript:alert(1)\" alt=\"\" /></p>",
        "should allow javascript protocol with allow_any_img_src option"
    );

    let result = to_html_with_options("![](irc:///help)", &options).unwrap();
    assert_eq!(
        result, "<p><img src=\"irc:///help\" alt=\"\" /></p>",
        "should allow irc protocol with allow_any_img_src option"
    );

    let result = to_html_with_options("![](mailto:a)", &options).unwrap();
    assert_eq!(
        result, "<p><img src=\"mailto:a\" alt=\"\" /></p>",
        "should allow mailto protocol with allow_any_img_src option"
    );
}
