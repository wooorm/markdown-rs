# micromark-rs

<!-- To do: enable image when repo is public. -->

<!-- <img align="right" width="106" height="106" alt="" src="https://raw.githubusercontent.com/wooorm/micromark-rs/14f1ad0/logo.svg?sanitize=true"> -->

<!-- To do: enable badges when repo is public/published -->

<!-- To do: link `Downloads`/`crate-badge` to `crate` instead of temporary site. -->

<!-- [![Build][build-badge]][build] -->

<!-- [![Downloads][crate-badge]][docs] -->

<!-- [![Coverage][coverage-badge]][coverage] -->

[![Chat][chat-badge]][chat]

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

micromark is an open source markdown parser written in Rust.
It’s implemented as a state machine (`#![no_std]` + `alloc`) that emits
concrete tokens, so that every byte is accounted for, with positional info.
The API then exposes this information as an AST, which is easier to work with,
or it compiles directly to HTML.

While most markdown parsers work towards compliancy with CommonMark (or GFM),
this project goes further by following how the reference parsers (`cmark`,
`cmark-gfm`) work, which is confirmed with thousands of extra tests.

Other than CommonMark and GFM, this project also supports common extensions
to markdown such as MDX, math, and frontmatter.

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
*   [License](#license)

## Install

With [Rust][] (rust edition 2018+, ±version 1.56+), install with `cargo`:

```sh
cargo install micromark
```

## Use

```rs
extern crate micromark;
use micromark::micromark;

fn main() {
    println!("{}", micromark("## Hello, *world*!"));
}
```

Yields:

```html
<h2>Hello, <em>world</em>!</h2>
```

Extensions (in this case GFM):

```rs
extern crate micromark;
use micromark::{micromark_with_options, Options};

fn main() -> Result<(), String> {
    println!(
        "{}",
        micromark_with_options(
            "* [x] contact@example.com ~~strikethrough~~",
            &Options::gfm()
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
extern crate micromark;
use micromark::{micromark_to_mdast, ParseOptions};

fn main() -> Result<(), String> {
    println!(
        "{:?}",
        micromark_to_mdast("# Hey, *you*!", &ParseOptions::default())?
    );

    Ok(())
}
```

Yields:

```text
Root { children: [Heading { children: [Text { value: "Hey, ", position: Some(1:3-1:8 (2-7)) }, Emphasis { children: [Text { value: "you", position: Some(1:9-1:12 (8-11)) }], position: Some(1:8-1:13 (7-12)) }, Text { value: "!", position: Some(1:13-1:14 (12-13)) }], position: Some(1:1-1:14 (0-13)), depth: 1 }], position: Some(1:1-1:14 (0-13)) }
```

## API

`micromark` exposes
[`micromark`](https://wooorm.com/micromark-rs/micromark/fn.micromark.html),
[`micromark_with_options`](https://wooorm.com/micromark-rs/micromark/fn.micromark_with_options.html),
[`micromark_to_mdast`](https://wooorm.com/micromark-rs/micromark/fn.micromark_to_mdast.html),
[`Options`](https://wooorm.com/micromark-rs/micromark/struct.Options.html),
and a few other structs and enums.

See the [crate docs][docs] for more info.

## Extensions

micromark supports extensions to `CommonMark`.
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

micromark is maintained as a single monolithic crate.

### Overview

The process to parse markdown looks like this:

```txt
                     micromark
+-------------------------------------------------+
|            +-------+         +---------+--html- |
| -markdown->+ parse +-events->+ compile +        |
|            +-------+         +---------+-mdast- |
+-------------------------------------------------+
```

### File structure

The files in `src/` are as follows:

*   `construct/*.rs`
    — CommonMark, GFM, and other extension constructs used in micromark
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

micromark is tested with the \~650 CommonMark tests and more than 1k extra
tests confirmed with CM reference parsers.
Then there’s even more tests for GFM and other extensions.
These tests reach all branches in the code, which means that this project has
100% code coverage.
Fuzz testing is used to check for things that might fall through coverage.

The following bash scripts are useful when working on this project:

*   run examples:
    ```sh
    RUST_BACKTRACE=1 RUST_LOG=debug cargo run --example lib
    ```
*   format:
    ```sh
    cargo fmt
    ```
*   lint:
    ```sh
    cargo fmt --check && cargo clippy --examples --tests --benches
    ```
*   test:
    ```sh
    RUST_BACKTRACE=1 cargo test
    ```
*   docs:
    ```sh
    cargo doc --document-private-items
    ```
*   fuzz:
    ```sh
    cargo install cargo-fuzz
    cargo +nightly fuzz run micromark
    ```

### Version

micromark follows [SemVer](https://semver.org).

### Security

The typical security aspect discussed for markdown is [cross-site scripting
(XSS)][xss] attacks.
Markdown itself is safe if it does not include embedded HTML or dangerous
protocols in links/images (such as `javascript:` or `data:`).
micromark makes any markdown safe by default, even if HTML is embedded or
dangerous protocols are used, as it encodes or drops them.
Turning on the `allow_dangerous_html` or `allow_dangerous_protocol` options for
user-provided markdown opens you up to XSS attacks.

An aspect related to XSS for security is syntax errors: markdown itself has no
syntax errors.
Some syntax extensions (specifically, only MDX) do include syntax errors.
For that reason, `micromark_with_options` returns `Result<(), String>`, of which
the error is a simple string indicating where the problem happened, what
occurred, and what was expected instead.
Make sure to handle your errors when using MDX.

Another security aspect is DDoS attacks.
For example, an attacker could throw a 100mb file at micromark, in which case
it’s going to take a long while to finish.
It is also possible to crash micromark with smaller payloads, notably when
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

> 🚧 **To do**.

<!-- To do: mention Vercel. -->

Support this effort and give back by sponsoring:

*   [GitHub Sponsors](https://github.com/sponsors/wooorm)
    (personal; monthly or one-time)
*   [OpenCollective](https://opencollective.com/unified) or
    [GitHub Sponsors](https://github.com/sponsors/unifiedjs)
    (unified; monthly or one-time)

<!-- To do: origin story -->

## License

[MIT][license] © [Titus Wormer][author]

<!-- To do: public/publish -->

<!-- [build-badge]: https://github.com/wooorm/micromark-rs/workflows/main/badge.svg -->

<!-- [build]: https://github.com/wooorm/micromark-rs/actions -->

<!-- [crate-badge]: https://img.shields.io/crates/d/micromark.svg -->

<!-- [crate]: https://crates.io/crates/micromark -->

[docs]: https://wooorm.com/micromark-rs/micromark/

[chat-badge]: https://img.shields.io/badge/chat-discussions-success.svg

[chat]: https://github.com/wooorm/micromark-rs/discussions

[commonmark]: https://spec.commonmark.org

[cheat]: https://commonmark.org/help/

[rust]: https://www.rust-lang.org

[xss]: https://en.wikipedia.org/wiki/Cross-site_scripting

[improper]: https://github.com/ChALkeR/notes/blob/master/Improper-markup-sanitization.md

[chalker]: https://github.com/ChALkeR

[license]: https://github.com/micromark/micromark/blob/main/license

[author]: https://wooorm.com

[mdast]: https://github.com/syntax-tree/mdast

[contribute]: #contribute

[sponsor]: #sponsor

[extensions]: #extensions

[security]: #security

[test]: #test

[contributing]: .github/contribute.md

[support]: .github/support.md

[coc]: .github/code-of-conduct.md
