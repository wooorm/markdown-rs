# micromark-rs

<!-- To do: enable image when repo is public. -->

<!-- <img align="right" width="106" height="106" alt="" src="https://raw.githubusercontent.com/wooorm/micromark-rs/14f1ad0/logo.svg?sanitize=true"> -->

<!-- To do: enable badges when repo is public/published -->
<!-- To do: link `Downloads`/`crate-badge` to `crate` instead of temporary site. -->

<!-- [![Build][build-badge]][build] -->
<!-- [![Downloads][crate-badge]][docs] -->
<!-- [![Coverage][coverage-badge]][coverage] -->

[![Sponsors][sponsors-badge]][opencollective]
[![Backers][backers-badge]][opencollective]
[![Chat][chat-badge]][chat]

A [`CommonMark`][commonmark-spec] compliant markdown parser in [Rust][] with
positional info, concrete tokens, and extensions.

## Feature highlights

- [x] **[compliant][commonmark]** (100% to CommonMark)
- [x] **[extensions][]** (100% GFM, 100% MDX, frontmatter, math)
- [x] **[safe][security]** (100% safe rust, also 100% safe HTML by default)
- [x] **[robust][test]** (2300+ tests, 100% coverage, fuzz testing)

It’s also `#![no_std]` + `alloc`, has tons of docs, and has a single dependency
(for optional debug logging).

> 🐣 **Note**: coverage is currently within progress.

## When to use this

- If you _just_ want to turn markdown into HTML (with maybe a few extensions)
- If you want to do _really complex things_ with markdown

See [§ Comparison][comparison] for more info

## Intro

micromark is markdown parser in Rust.
It uses a state machine to parse the entirety of markdown into concrete
tokens.
Its API compiles to HTML, but its parts are made to be used separately, so as to
generate syntax trees or compile to other output formats.
`micromark-rs` has a sibling in JavaScript, [`micromark-js`][micromark-js].

<!-- To do: link to unified etc if this repo gets moved there? -->

- to learn markdown, see this [cheatsheet and tutorial][cheat]
- for questions, see [Discussions][chat]
- to help, see [contribute][] or [sponsor][] below

## Contents

- [Install](#install)
- [Use](#use)
- [API](#api)
- [Extensions](#extensions)
- [Architecture](#architecture)
- [Examples](#examples)
- [Markdown](#markdown)
- [Project](#project)
- [License](#license)

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
use micromark::{micromark_with_options, Constructs, Options};

fn main() -> Result<(), String> {
    println!(
        "{}",
        micromark_with_options(
            "* [x] contact@example.com ~~strikethrough~~",
            &Options {
                constructs: Constructs::gfm(),
                ..Options::default()
            }
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

## API

`micromark` exposes
[`micromark`](https://wooorm.com/micromark-rs/micromark/fn.micromark.html),
[`micromark_with_options`](https://wooorm.com/micromark-rs/micromark/fn.micromark_with_options.html), and
[`Options`](https://wooorm.com/micromark-rs/micromark/struct.Options.html).
See [crate docs][docs] for more info.

## Extensions

micromark supports extensions.
These extensions are maintained in this project.
They are not enabled by default but can be turned on with options.

- frontmatter
- GFM
  - autolink literal
  - footnote
  - strikethrough
  - table
  - tagfilter
  - task list item
- math
- MDX
  - ESM
  - expressions
  - JSX

It is not a goal of this project to support lots of different extensions.
It’s instead a goal to support incredibly common, somewhat standardized,
extensions.

## Architecture

micromark is maintained as a single monolithic package.

### Overview

The process to parse markdown looks like this:

```txt
                     micromark
+------------------------------------------------+
|            +-------+         +---------+       |
| -markdown->+ parse +-events->+ compile +-html- |
|            +-------+         +---------+       |
+------------------------------------------------+
```

### File structure

The files in `src/` are as follows:

- `construct/*.rs`
  — CommonMark, GFM, and other extension constructs used in micromark
- `util/*.rs`
  — helpers often needed when parsing markdown
- `event.rs`
  — things with meaning happening somewhere
- `lib.rs`
  — core module
- `mdast.rs`
  — syntax tree
- `parser.rs`
  — turn a string of markdown into events
- `resolve.rs`
  — steps to process events
- `state.rs`
  — steps of the state machine
- `subtokenize.rs`
  — handle content in other content
- `to_html.rs`
  — turns events into a string of HTML
- `to_mdast.rs`
  — turns events into a syntax tree
- `tokenizer.rs`
  — glue the states of the state machine together

## Examples

<!-- To do: example section with more full-fledged examples, on GFM, math, frontmatter, etc. -->

> 🚧 **To do**.

### Example: syntax highlighting code

This example shows how `micromark-rs` can be used to turn markdown into an HTML
file.
When the HTML is opened in a browser, the code examples that were in the
markdown are then syntax highlighted by client side JavaScript using
[`starry-night`][starry-night].
The `starry-night` library matches how GitHub highlights code on their platform.

Say we have this `example.rs`:

```rs
extern crate micromark;
use micromark::{micromark_with_options, Constructs, Options};
use std::fs;

fn main() -> Result<(), String> {
    let markdown = r###"
# Hello

…world!

~~~js
console.log('it works!')
~~~
"###;

    let html = micromark_with_options(
        markdown,
        &Options {
            constructs: Constructs::gfm(),
            ..Options::default()
        },
    )?;

    let js = r###"
import {createStarryNight, common} from 'https://esm.sh/@wooorm/starry-night@1?bundle'
import {toDom} from 'https://esm.sh/hast-util-to-dom@3?bundle'

const starryNight = await createStarryNight(common)
const prefix = 'language-'

const nodes = Array.from(document.body.querySelectorAll('code'))

for (const node of nodes) {
  const className = Array.from(node.classList).find((d) => d.startsWith(prefix))
  if (!className) continue
  const scope = starryNight.flagToScope(className.slice(prefix.length))
  if (!scope) continue
  const tree = starryNight.highlight(node.textContent, scope)
  node.replaceChildren(toDom(tree, {fragment: true}))
}
"###;

    let html = format!(
        "<!doctype html>
<meta charset=utf8>
<title>Hello</title>
<link rel=stylesheet href=\"https://esm.sh/@wooorm/starry-night@1/style/both.css\">
<body>
{}
<script type=module>
{}
</script>
</body>
",
        html, js
    );

    match fs::write("index.html", html) {
        Ok(()) => {}
        Err(error) => return Err(format!("Could not write `index.html`: {:?}", error)),
    }

    Ok(())
}
```

The code example in the markdown as HTML will first look like this:

```html
<pre><code class="language-js">console.log('it works!')
</code></pre>
```

Opening that page in a browser, we’d see that being swapped with:

<!-- prettier-ignore -->
```html
<pre><code class="language-js"><span class="pl-en">console</span>.<span class="pl-c1">log</span>(<span class="pl-s"><span class="pl-pds">'</span>it works!<span class="pl-pds">'</span></span>)
</code></pre>
```

## Markdown

### CommonMark

The first definition of “Markdown” gave several examples of how it worked,
showing input Markdown and output HTML, and came with a reference implementation
(`Markdown.pl`).
When new implementations followed, they mostly followed the first definition,
but deviated from the first implementation, and added extensions, thus making
the format a family of formats.

Some years later, an attempt was made to standardize the differences between
implementations, by specifying how several edge cases should be handled, through
more input and output examples.
This is known as [CommonMark][commonmark-spec], and many implementations now
work towards some degree of CommonMark compliancy.
Still, CommonMark describes what the output in HTML should be given some
input, which leaves many edge cases up for debate, and does not answer what
should happen for other output formats.

micromark passes all tests from CommonMark and has many more tests to match the
CommonMark reference parsers.

### Grammar

The syntax of markdown can be described in Backus–Naur form (BNF) as:

```bnf
markdown = .*
```

No, that’s [not a typo](http://trevorjim.com/a-specification-for-markdown/):
markdown has no syntax errors; anything thrown at it renders _something_.

For more practical examples of how things roughly work in BNF, see the module docs of each `src/construct`.

## Project

### Comparison

> 🚧 **To do**.

<!-- To do. -->

### Test

micromark is tested with the \~650 CommonMark tests and more than 1.7k extra
tests confirmed with CM reference parsers.
Then there’s even more tests for GFM and other extensions.
These tests reach all branches in the code, which means that this project has
100% code coverage.
Fuzz testing is used to check for things that might fall through coverage.

The following scripts are useful when working on this project:

- run examples:
  ```sh
  RUST_BACKTRACE=1 RUST_LOG=debug cargo run --example lib
  ```
- format:
  ```sh
  cargo fmt
  ```
- lint:
  ```sh
  cargo fmt --check && cargo clippy -- -D clippy::pedantic -D clippy::cargo -A clippy::doc_link_with_quotes -A clippy::unnecessary_wraps
  ```
- test:
  ```sh
  RUST_BACKTRACE=1 cargo test
  ```
- docs:
  ```sh
  cargo doc --document-private-items
  ```
- fuzz:
  ```sh
  cargo install cargo-fuzz
  cargo +nightly fuzz run micromark
  ```

### Version

micromark adheres to [SemVer](https://semver.org).

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

<!-- To do: CoC. -->

### Sponsor

> 🚧 **To do**.

<!-- To do: mention Vercel. -->

Support this effort and give back by sponsoring:

- [GitHub Sponsors](https://github.com/sponsors/wooorm)
  (personal; monthly or one-time)
- [OpenCollective](https://opencollective.com/unified) or
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

[sponsors-badge]: https://opencollective.com/unified/sponsors/badge.svg
[backers-badge]: https://opencollective.com/unified/backers/badge.svg
[opencollective]: https://opencollective.com/unified
[docs]: https://wooorm.com/micromark-rs/micromark/
[chat-badge]: https://img.shields.io/badge/chat-discussions-success.svg
[chat]: https://github.com/wooorm/micromark-rs/discussions
[commonmark-spec]: https://spec.commonmark.org
[cheat]: https://commonmark.org/help/
[gfm-spec]: https://github.github.com/gfm/
[rust]: https://www.rust-lang.org
[cmsm]: https://github.com/micromark/common-markup-state-machine
[micromark-js]: https://github.com/micromark/micromark
[xss]: https://en.wikipedia.org/wiki/Cross-site_scripting
[improper]: https://github.com/ChALkeR/notes/blob/master/Improper-markup-sanitization.md
[chalker]: https://github.com/ChALkeR
[license]: https://github.com/micromark/micromark/blob/main/license
[author]: https://wooorm.com
[starry-night]: https://github.com/wooorm/starry-night
[contribute]: #contribute
[sponsor]: #sponsor
[commonmark]: #commonmark
[extensions]: #extensions
[security]: #security
[test]: #test
[comparison]: #comparison
[contributing]: .github/contribute.md
[support]: .github/support.md
