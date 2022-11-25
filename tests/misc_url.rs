use markdown::to_html;
use pretty_assertions::assert_eq;

#[test]
fn url() {
    assert_eq!(
        to_html("<https://%>"),
        "<p><a href=\"https://%25\">https://%</a></p>",
        "should support incorrect percentage encoded values (0)"
    );

    assert_eq!(
        to_html("[](<%>)"),
        "<p><a href=\"%25\"></a></p>",
        "should support incorrect percentage encoded values (1)"
    );

    assert_eq!(
        to_html("[](<%%20>)"),
        "<p><a href=\"%25%20\"></a></p>",
        "should support incorrect percentage encoded values (2)"
    );

    assert_eq!(
        to_html("[](<%a%20>)"),
        "<p><a href=\"%25a%20\"></a></p>",
        "should support incorrect percentage encoded values (3)"
    );

    // Note: Surrogate handling not needed in Rust.
    //   assert_eq!(
    //     to_html("[](<foo\u{D800}bar>)"),
    //     "<p><a href=\"foo%EF%BF%BDbar\"></a></p>",
    //     "should support a lone high surrogate (lowest)"
    //   );

    // Surrogate handling not needed in Rust.
    //   assert_eq!(
    //     to_html("[](<foo\u{DBFF}bar>)"),
    //     "<p><a href=\"foo%EF%BF%BDbar\"></a></p>",
    //     "should support a lone high surrogate (highest)"
    //   );

    // Surrogate handling not needed in Rust.
    //   assert_eq!(
    //     to_html("[](<\u{D800}bar>)"),
    //     "<p><a href=\"%EF%BF%BDbar\"></a></p>",
    //     "should support a lone high surrogate at the start (lowest)"
    //   );

    // Surrogate handling not needed in Rust.
    //   assert_eq!(
    //     to_html("[](<\u{DBFF}bar>)"),
    //     "<p><a href=\"%EF%BF%BDbar\"></a></p>",
    //     "should support a lone high surrogate at the start (highest)"
    //   );

    // Surrogate handling not needed in Rust.
    //   assert_eq!(
    //     to_html("[](<foo\u{D800}>)"),
    //     "<p><a href=\"foo%EF%BF%BD\"></a></p>",
    //     "should support a lone high surrogate at the end (lowest)"
    //   );

    // Surrogate handling not needed in Rust.
    //   assert_eq!(
    //     to_html("[](<foo\u{DBFF}>)"),
    //     "<p><a href=\"foo%EF%BF%BD\"></a></p>",
    //     "should support a lone high surrogate at the end (highest)"
    //   );

    // Surrogate handling not needed in Rust.
    //   assert_eq!(
    //     to_html("[](<foo\u{DC00}bar>)"),
    //     "<p><a href=\"foo%EF%BF%BDbar\"></a></p>",
    //     "should support a lone low surrogate (lowest)"
    //   );

    // Surrogate handling not needed in Rust.
    //   assert_eq!(
    //     to_html("[](<foo\u{DFFF}bar>)"),
    //     "<p><a href=\"foo%EF%BF%BDbar\"></a></p>",
    //     "should support a lone low surrogate (highest)"
    //   );

    // Surrogate handling not needed in Rust.
    //   assert_eq!(
    //     to_html("[](<\u{DC00}bar>)"),
    //     "<p><a href=\"%EF%BF%BDbar\"></a></p>",
    //     "should support a lone low surrogate at the start (lowest)"
    //   );

    // Surrogate handling not needed in Rust.
    //   assert_eq!(
    //     to_html("[](<\u{DFFF}bar>)"),
    //     "<p><a href=\"%EF%BF%BDbar\"></a></p>",
    //     "should support a lone low surrogate at the start (highest)"
    //   );

    // Surrogate handling not needed in Rust.
    //   assert_eq!(
    //     to_html("[](<foo\u{DC00}>)"),
    //     "<p><a href=\"foo%EF%BF%BD\"></a></p>",
    //     "should support a lone low surrogate at the end (lowest)"
    //   );

    // Surrogate handling not needed in Rust.
    //   assert_eq!(
    //     to_html("[](<foo\u{DFFF}>)"),
    //     "<p><a href=\"foo%EF%BF%BD\"></a></p>",
    //     "should support a lone low surrogate at the end (highest)"
    //   );

    assert_eq!(
        to_html("[](<🤔>)"),
        "<p><a href=\"%F0%9F%A4%94\"></a></p>",
        "should support an emoji"
    );

    let mut ascii = Vec::with_capacity(129);
    let mut code = 0;

    while code < 128 {
        // LF and CR can’t be in resources.
        if code == 10 || code == 13 {
            code += 1;
            continue;
        }

        // `<`, `>`, `\` need to be escaped.
        if code == 60 || code == 62 || code == 92 {
            ascii.push('\\');
        }

        ascii.push(char::from_u32(code).unwrap());

        code += 1;
    }

    let ascii_in = ascii.into_iter().collect::<String>();
    let ascii_out = "%EF%BF%BD%01%02%03%04%05%06%07%08%09%0B%0C%0E%0F%10%11%12%13%14%15%16%17%18%19%1A%1B%1C%1D%1E%1F%20!%22#$%25&amp;'()*+,-./0123456789:;%3C=%3E?@ABCDEFGHIJKLMNOPQRSTUVWXYZ%5B%5C%5D%5E_%60abcdefghijklmnopqrstuvwxyz%7B%7C%7D~%7F";
    assert_eq!(
        to_html(&format!("[](<{}>)", ascii_in)),
        format!("<p><a href=\"{}\"></a></p>", ascii_out),
        "should support ascii characters"
    );
}
