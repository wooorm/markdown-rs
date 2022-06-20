# micromark-rs

Crate docs are currently at
[`wooorm.com/micromark-rs/micromark/`](https://wooorm.com/micromark-rs/micromark/).

Here be dragons!
🐉
There’s a lot to do.
Some major to dos are described here, more smaller ones are in the code.

## Some useful scripts for now

Run examples:

```sh
RUST_BACKTRACE=1 RUST_LOG=debug cargo run --example lib
```

Format:

```sh
cargo fmt --all
```

Lint:

```sh
cargo fmt --all -- --check && cargo clippy -- -W clippy::pedantic
```

Tests:

```sh
RUST_BACKTRACE=1 cargo test
```

Docs:

```sh
cargo doc --document-private-items
```

(add `--open` to open them in a browser)

## To do

### Some major obstacles

- [ ] (8) Can paragraphs (and to a lesser extent string data and text data)
      operate more performantly than checking whether other flow constructs
      start a line, before exiting and actually attempting flow constructs?
- [ ] (5) Figure out sharing definition and identifiers, and references before
      definitions
- [ ] (3) Interrupting: sometimes flow can or cannot start depending on the
      previous construct (typically paragraph)
- [ ] (5) Containers: this will be rather messy, and depends a lot on how
      subtokenization is solved
- [ ] (3) Concrete constructs: HTML or code (fenced) cannot be “pierced” into by
      containers
- [ ] (3) Lazy lines, in containers, in flow in a paragraph, a line does not
      need to be indented
- [ ] (5) There’s a lot of rust-related choosing whether to pass (mutable)
      references or whatever around that should be refactored
- [ ] (5) Figure out extensions
- [ ] (1) Support turning off constructs

### Small things

- [ ] (1) Handle BOM at start
- [ ] (1) Parse initial and final whitespace of paragraphs (in text)
- [ ] (1) Add docs to subtokenize
- [ ] (1) Add module docs to parser
- [ ] (1) Add overview docs on how everything works
- [ ] (1) Move safe protocols to constants
- [ ] (3) Clean compiler
- [ ] (1) Use preferred line ending style in markdown
- [ ] (1) Add tests for `default-line-ending`, `line-ending`
- [ ] (1) Make sure tabs are handled properly and that positional info is perfect
- [ ] (1) Make sure crlf/cr/lf are working perfectly
- [ ] (3) Figure out lifetimes of things (see `life time` in source)
- [ ] (3) Use `commonmark` tests
- [ ] (3) Share a bunch of tests with `micromark-js`
- [ ] (5) Do some research on rust best practices for APIs, e.g., what to accept,
      how to integrate with streams or so?
- [ ] (1) Go through clippy rules, and such, to add strict code styles
- [ ] (1) Make sure that rust character groups match CM character groups (e.g., is
      `unicode_whitespace` or so the same?)
- [ ] (1) Any special handling of surrogates?
- [ ] (1) Make sure debugging is useful for other folks
- [ ] (3) Add some benchmarks, do some perf testing
- [ ] (3) Write comparison to other parsers
- [ ] (3) Add node/etc bindings?
- [ ] (8) After all extensions, including MDX, are done, see if we can integrate
      this with SWC to compile MDX
- [ ] (3) Bunch of docs
- [ ] (5) Site

### Constructs

- [ ] (5) attention (strong, emphasis) (text)
- [x] autolink
- [x] blank line
- [ ] (5) block quote
- [x] character escape
- [x] character reference
- [x] code (fenced)
- [x] code (indented)
- [x] code (text)
- [x] definition
- [x] hard break (escape)
- [x] hard break (trailing)
- [x] heading (atx)
- [x] heading (setext)
- [x] html (flow)
- [x] html (text)
- [ ] (3) label end
- [ ] (3) label start (image)
- [ ] (3) label start (link)
- [ ] (8) list
- [x] paragraph
- [x] thematic break

### Content types

- [ ] (8) container
  - [ ] block quote
  - [ ] list
- [x] flow
  - [x] blank line
  - [x] code (fenced)
  - [x] code (indented)
  - [x] definition
  - [x] heading (atx)
  - [x] heading (setext)
  - [x] html (flow)
  - [x] paragraph
  - [x] thematic break
- [ ] (5) text
  - [ ] attention (strong, emphasis) (text)
  - [x] autolink
  - [x] character escape
  - [x] character reference
  - [x] code (text)
  - [x] hard break (escape)
  - [x] hard break (trailing)
  - [x] html (text)
  - [ ] label end
  - [ ] label start (image)
  - [ ] label start (link)
- [x] string
  - [x] character escape
  - [x] character reference

### Done

- [x] (8) Subtokenization: figure out a good, fast way to deal with constructs in
      one content type that also are another content type
- [x] (3) Encode urls
- [x] (1) Optionally remove dangerous protocols when compiling
- [x] (1) Add docs to html (text)
- [x] (1) Add docs on bnf
- [x] (1) Reorganize to split util
- [x] (1) Add examples to `CompileOptions` docs
- [x] (3) Fix deep subtokenization
- [x] (1) text in heading
- [x] (1) Setext headings, solved in flow
- [x] (1) Add docs to partials
- [x] (1) Remove all `pub fn`s from constructs, except for start
- [x] (1) Remove `content` content type, as it is no longer needed
- [x] (1) Paragraph
- [x] (1) Parse whitespace in each flow construct
- [x] (1) Connect `ChunkString` in label, destination, title
- [x] (1) Add support for line endings in `string`

### Extensions

The main thing here is is to figure out if folks could extend from the outside
with their own code, or if we need to maintain it all here.
Regardless, it is essential for the launch of `micromark-rs` that extensions
are theoretically or practically possible.
The extensions below are listed from top to bottom from more important to less
important.

- [ ] (1) frontmatter (yaml, toml) (flow)
      — [`micromark-extension-frontmatter`](https://github.com/micromark/micromark-extension-frontmatter)
- [ ] (3) autolink literal (GFM) (text)
      — [`micromark-extension-gfm-autolink-literal`](https://github.com/micromark/micromark-extension-gfm-autolink-literal)
- [ ] (3) footnote (GFM) (flow, text)
      — [`micromark-extension-gfm-footnote`](https://github.com/micromark/micromark-extension-gfm-footnote)
- [ ] (3) strikethrough (GFM) (text)
      — [`micromark-extension-gfm-strikethrough`](https://github.com/micromark/micromark-extension-gfm-strikethrough)
- [ ] (5) table (GFM) (flow)
      — [`micromark-extension-gfm-table`](https://github.com/micromark/micromark-extension-gfm-table)
- [ ] (1) task list item (GFM) (text)
      — [`micromark-extension-gfm-task-list-item`](https://github.com/micromark/micromark-extension-gfm-task-list-item)
- [ ] (3) math (flow, text)
      — [`micromark-extension-math`](https://github.com/micromark/micromark-extension-math)
- [ ] (8) directive (flow, text)
      — [`micromark-extension-directive`](https://github.com/micromark/micromark-extension-directive)
- [ ] (8) expression (MDX) (flow, text)
      — [`micromark-extension-mdx-expression`](https://github.com/micromark/micromark-extension-mdx-expression)
- [ ] (5) JSX (MDX) (flow, text)
      — [`micromark-extension-mdx-jsx`](https://github.com/micromark/micromark-extension-mdx-jsx)
- [ ] (3) ESM (MDX) (flow)
      — [`micromark-extension-mdxjs-esm`](https://github.com/micromark/micromark-extension-mdxjs-esm)
- [ ] (1) tagfilter (GFM) (n/a, renderer)
      — [`micromark-extension-gfm-tagfilter`](https://github.com/micromark/micromark-extension-gfm-tagfilter)
