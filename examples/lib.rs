fn main() -> Result<(), String> {
    // Turn on debugging.
    // You can show it with `RUST_LOG=debug cargo run --features log --example lib`
    env_logger::init();

    // Safely turn (untrusted?) markdown into HTML.
    println!("{:?}", markdown::to_html("## Hello, *world*!"));

    // Turn trusted markdown into HTML.
    println!(
        "{:?}",
        markdown::to_html_with_options(
            "<div style=\"color: tomato\">\n\n# Hello, tomato!\n\n</div>",
            &markdown::Options {
                compile: markdown::CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..markdown::CompileOptions::default()
                },
                ..markdown::Options::default()
            }
        )
    );

    // Support GFM extensions.
    println!(
        "{}",
        markdown::to_html_with_options(
            "* [x] contact@example.com ~~strikethrough~~",
            &markdown::Options::gfm()
        )?
    );

    // Access syntax tree and support MDX extensions:
    println!(
        "{:?}",
        markdown::to_mdast(
            "# <HelloMessage />, {username}!",
            &markdown::ParseOptions::mdx()
        )?
    );

    Ok(())
}
