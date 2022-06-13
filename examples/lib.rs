extern crate micromark;
use micromark::{micromark, micromark_with_options, CompileOptions};

fn main() {
    // Turn on debugging.
    // You can show it with `RUST_LOG=debug cargo run --example lib`
    env_logger::init();

    // Safely turn (untrusted?) markdown into HTML.
    println!("{:?}", micromark("# Hello, world!"));

    // Turn trusted markdown into HTML.
    println!(
        "{:?}",
        micromark_with_options(
            "<div style=\"color: tomato\">\n\n# Hello, tomato!\n\n</div>",
            &CompileOptions {
                allow_dangerous_html: true,
                allow_dangerous_protocol: true
            }
        )
    );
}
