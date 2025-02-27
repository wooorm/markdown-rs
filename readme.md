<p align="center">
  <br>
  <img width="192" src="media/logo-chromatic.svg" alt="">
  <br>
  <br>
  <br>
</p>

# markdown-rs

[![Build][build-badge]][build]
[![Coverage][coverage-badge]][coverage]
[![GitHub][repo-badge]][repo]
[![docs.rs][docs-badge]][docs]
[![crates.io][crate-badge]][crate]

> 👉 **Note**: this is a new crate that reuses an old name.
> The old crate (`0.3.0` and lower) has a bunch of problems.
> Make sure to use the new crate, currently in alpha at `1.0.0-alpha.22`.

CommonMark compliant markdown parser in Rust with ASTs and extensions.

## Feature highlights

*   [x] **[compliant][commonmark]** (100% to CommonMark)
*   [x] **[extensions][]** (100% GFM, 100% MDX, frontmatter, math)
*   [x] **[safe][security]** (100% safe Rust, also 100% safe HTML by default)
*   [x] **[robust][test]** (2300+ tests, 100% coverage, fuzz testing)
*   [x] **[ast][mdast]** (mdast)

## When should I use this?

*   If you *just* want to turn markdown into HTML (with maybe a few extensions)
*   If you want to do *really complex things* with markdown

## What is this?

`markdown-rs` is an open source markdown parser written in Rust.
It’s implemented as a state machine (`#![no_std]` + `alloc`) that emits
concrete tokens, so that every byte is accounted for, with positional info.
The API then exposes this information as an AST, which is easier to work with,
or it compiles directly to HTML.

While most markdown parsers work towards compliancy with CommonMark (or GFM),
this project goes further by following how the reference parsers (`cmark`,
`cmark-gfm`) work, which is confirmed with thousands of extra tests.

Other than CommonMark and GFM, this project also supports common extensions
to markdown such as MDX, math, and frontmatter.

This Rust crate has a sibling project in JavaScript: [`micromark`][micromark]
(and [`mdast-util-from-markdown`][mdast-util-from-markdown] for the AST).

P.S. if you want to *compile* MDX, use [`mdxjs-rs`][mdxjs-rs].

## Questions

*   to learn markdown, see this [cheatsheet and tutorial][cheat]
*   for the API, see the [crate docs][docs]
*   for questions, see [Discussions][chat]
*   to help, see [contribute][] or [sponsor][] below

## Contents

*   [Install](#install)
*   [Use](#use)
*   [API](#api)
*   [Extensions](#extensions)
*   [Project](#project)
    *   [Overview](#overview)
    *   [File structure](#file-structure)
    *   [Test](#test)
    *   [Version](#version)
    *   [Security](#security)
    *   [Contribute](#contribute)
    *   [Sponsor](#sponsor)
    *   [Thanks](#thanks)
*   [Related](#related)
*   [License](#license)

## Install

With [Rust][] (rust edition 2018+, ±version 1.56+), install with `cargo`:

```sh
cargo add markdown@1.0.0-alpha.22
```

> 👉 **Note**: this is a new crate that reuses an old name.
> The old crate (`0.3.0` and lower) has a bunch of problems.
> Make sure to use the new crate, currently in alpha at `1.0.0-alpha.22`.

## Use

```rs
fn main() {
    println!("{}", markdown::to_html("## Hello, *world*!"));
}
```

Yields:

```html
<h2>Hello, <em>world</em>!</h2>
```

Extensions (in this case GFM):

```rs
fn main() -> Result<(), markdown::message::Message> {
    println!(
        "{}",
        markdown::to_html_with_options(
            "* [x] contact@example.com ~~strikethrough~~",
            &markdown::Options::gfm()
        )?
    );

    Ok(())
}
```

Yields:

```html
<ul>
  <li>
    <input checked="" disabled="" type="checkbox" />
    <a href="mailto:contact@example.com">contact@example.com</a>
    <del>strikethrough</del>
  </li>
</ul>
```

Syntax tree ([mdast][]):

```rs
fn main() -> Result<(), markdown::message::Message> {
    println!(
        "{:?}",
        markdown::to_mdast("# Hey, *you*!", &markdown::ParseOptions::default())?
    );

    Ok(())
}
```

Yields:

```text
Root { children: [Heading { children: [Text { value: "Hey, ", position: Some(1:3-1:8 (2-7)) }, Emphasis { children: [Text { value: "you", position: Some(1:9-1:12 (8-11)) }], position: Some(1:8-1:13 (7-12)) }, Text { value: "!", position: Some(1:13-1:14 (12-13)) }], position: Some(1:1-1:14 (0-13)), depth: 1 }], position: Some(1:1-1:14 (0-13)) }
```

## API

`markdown-rs` exposes
[`to_html`](https://docs.rs/markdown/1.0.0-alpha.22/markdown/fn.to_html.html),
[`to_html_with_options`](https://docs.rs/markdown/1.0.0-alpha.22/markdown/fn.to_html_with_options.html),
[`to_mdast`](https://docs.rs/markdown/1.0.0-alpha.22/markdown/fn.to_mdast.html),
[`Options`](https://docs.rs/markdown/1.0.0-alpha.22/markdown/struct.Options.html),
and a few other structs and enums.

See the [crate docs][docs] for more info.

## Extensions

`markdown-rs` supports extensions to `CommonMark`.
These extensions are maintained in this project.
They are not enabled by default but can be turned on with options.

*   frontmatter
*   GFM
    *   autolink literal
    *   footnote
    *   strikethrough
    *   table
    *   tagfilter
    *   task list item
*   math
*   MDX
    *   ESM
    *   expressions
    *   JSX

It is not a goal of this project to support lots of different extensions.
It’s instead a goal to support very common and mostly standardized extensions.

## Project

`markdown-rs` is maintained as a single monolithic crate.

### Overview

The process to parse markdown looks like this:

```txt
                    markdown-rs
+-------------------------------------------------+
|            +-------+         +---------+--html- |
| -markdown->+ parse +-events->+ compile +        |
|            +-------+         +---------+-mdast- |
+-------------------------------------------------+
```

### File structure

The files in `src/` are as follows:

*   `construct/*.rs`
    — CommonMark, GFM, and other extension constructs used in markdown
*   `util/*.rs`
    — helpers often needed when parsing markdown
*   `event.rs`
    — things with meaning happening somewhere
*   `lib.rs`
    — public API
*   `mdast.rs`
    — syntax tree
*   `parser.rs`
    — turn a string of markdown into events
*   `resolve.rs`
    — steps to process events
*   `state.rs`
    — steps of the state machine
*   `subtokenize.rs`
    — handle content in other content
*   `to_html.rs`
    — turns events into a string of HTML
*   `to_mdast.rs`
    — turns events into a syntax tree
*   `tokenizer.rs`
    — glue the states of the state machine together
*   `unist.rs`
    — point and position, used in mdast

### Test

`markdown-rs` is tested with the \~650 CommonMark tests and more than 1k extra
tests confirmed with CM reference parsers.
Then there’s even more tests for GFM and other extensions.
These tests reach all branches in the code, which means that this project has
100% code coverage.
Fuzz testing is used to check for things that might fall through coverage.

The following bash scripts are useful when working on this project:

*   generate code (latest CM tests and Unicode info):
    ```sh
    cargo run --manifest-path generate/Cargo.toml
    ```
*   run examples:
    ```sh
    RUST_BACKTRACE=1 RUST_LOG=trace cargo run --example lib --features log
    ```
*   format:
    ```sh
    cargo fmt && cargo fix --all-features --all-targets --workspace
    ```
*   lint:
    ```sh
    cargo fmt --check && cargo clippy --all-features --all-targets --workspace
    ```
*   test:
    ```sh
    RUST_BACKTRACE=1 cargo test --all-features --workspace
    ```
*   docs:
    ```sh
    cargo doc --document-private-items --examples --workspace
    ```
*   fuzz:
    ```sh
    cargo install cargo-fuzz
    cargo install honggfuzz
    cargo +nightly fuzz run markdown_libfuzz
    cargo hfuzz run markdown_honggfuzz
    ```

### Version

`markdown-rs` follows [SemVer](https://semver.org).

### Security

The typical security aspect discussed for markdown is [cross-site scripting
(XSS)][xss] attacks.
Markdown itself is safe if it does not include embedded HTML or dangerous
protocols in links/images (such as `javascript:`).
`markdown-rs` makes any markdown safe by default, even if HTML is embedded or
dangerous protocols are used, as it encodes or drops them.

Some very old browsers (such as Opera 12 from 2012) did not respect the HTML
specification and executed scripts in images, making the use of external images
dangerous. However, all modern browsers respect the HTML specification and 
prevent this, making the use of external images safe.

Turning on the `allow_dangerous_html` or `allow_dangerous_protocol` options for
user-provided markdown opens you up to XSS attacks.
 - `allow_dangerous_html` allows HTML tags to be rendered, including tags that may trigger the execution of scripts
 - `allow_dangerous_protocol` allows the use of protocols like `javascript:`, in links/images
   - when the option `allow_any_img_src` is enabled, `allow_dangerous_protocol` will only apply to links.
     - The [HTML specification](https://html.spec.whatwg.org/multipage/images.html#images-processing-model) does not allow the execution of scripts in images, whatever the protocol they use. All modern browsers respect this.

An aspect related to XSS for security is syntax errors: markdown itself has no
syntax errors.
Some syntax extensions (specifically, only MDX) do include syntax errors.
For that reason, `to_html_with_options` returns `Result<String, Message>`, of
which the error is a struct indicating where the problem happened, what
occurred, and what was expected instead.
Make sure to handle your errors when using MDX.

Another security aspect is DDoS attacks.
For example, an attacker could throw a 100mb file at `markdown-rs`, in which
case it’s going to take a long while to finish.
It is also possible to crash `markdown-rs` with smaller payloads, notably when
thousands of links, images, emphasis, or strong are opened but not closed.
It is wise to cap the accepted size of input (500kb can hold a big book) and to
process content in a different thread so that it can be stopped when needed.

For more information on markdown sanitation, see
[`improper-markup-sanitization.md`][improper] by [**@chalker**][chalker].

### Contribute

See [`contributing.md`][contributing] for ways to help.
See [`support.md`][support] for ways to get help.
See [`code-of-conduct.md`][coc] for how to communicate in and around this
project.

### Sponsor

Support this effort and give back by sponsoring:

*   [GitHub Sponsors](https://github.com/sponsors/wooorm)
    (personal; monthly or one-time)
*   [OpenCollective](https://opencollective.com/unified) or
    [GitHub Sponsors](https://github.com/sponsors/unifiedjs)
    (unified; monthly or one-time)

### Thanks

Special thanks go out to:

*   [Vercel][] for funding the initial development
*   [**@Murderlon**][murderlon] for the design of the logo
*   [**@johannhof**][johannhof] for the crate name

## Related

*   [`micromark`][micromark]
    — same as `markdown-rs` but in JavaScript
*   [`mdxjs-rs`][mdxjs-rs]
    — wraps `markdown-rs` to *compile* MDX to JavaScript

## License

[MIT][license] © [Titus Wormer][author]

[build-badge]: https://github.com/wooorm/markdown-rs/workflows/main/badge.svg

[build]: https://github.com/wooorm/markdown-rs/actions

[coverage-badge]: https://img.shields.io/codecov/c/github/wooorm/markdown-rs.svg

[coverage]: https://codecov.io/github/wooorm/markdown-rs

[repo-badge]: https://img.shields.io/badge/GitHub-wooorm%2Fmarkdown--rs-brightgreen

[repo]: https://github.com/wooorm/markdown-rs

[docs-badge]: https://img.shields.io/docsrs/markdown

[docs]: https://docs.rs/markdown/1.0.0-alpha.22/markdown/

[crate-badge]: https://img.shields.io/crates/v/markdown

[crate]: https://crates.io/crates/markdown/1.0.0-alpha.22

[chat]: https://github.com/wooorm/markdown-rs/discussions

[commonmark]: https://spec.commonmark.org

[cheat]: https://commonmark.org/help/

[rust]: https://www.rust-lang.org

[xss]: https://en.wikipedia.org/wiki/Cross-site_scripting

[improper]: https://github.com/ChALkeR/notes/blob/master/Improper-markup-sanitization.md

[chalker]: https://github.com/ChALkeR

[license]: license

[author]: https://wooorm.com

[mdast]: https://github.com/syntax-tree/mdast

[micromark]: https://github.com/micromark/micromark

[mdxjs-rs]: https://github.com/wooorm/mdxjs-rs

[mdast-util-from-markdown]: https://github.com/syntax-tree/mdast-util-from-markdown

[vercel]: https://vercel.com

[murderlon]: https://github.com/murderlon

[johannhof]: https://github.com/johannhof

[contribute]: #contribute

[sponsor]: #sponsor

[extensions]: #extensions

[security]: #security

[test]: #test

[contributing]: .github/contribute.md

[support]: .github/support.md

[coc]: .github/code-of-conduct.md
