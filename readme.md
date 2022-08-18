# micromark-rs

A [CommonMark][] compliant, `no_std` + `alloc`, markdown parser, with extensions,
in Rust.

Crate docs currently at
[`wooorm.com/micromark-rs/micromark/`][docs].

## To do

### Docs

- [ ] (1) Add overview docs on how everything works

### Refactor

- [ ] (1) Improve `interrupt`, `concrete`, `lazy` fields somehow?
- [ ] (?) Remove last box: the one around the child tokenizer?
- [ ] (1) Add helper to get byte at, get char before/after, etc.
- [ ] (?) Use smaller things that usizes?

### Test

- [ ] (1) Make sure positional info is perfect
- [ ] (3) Share tests with `micromark-js`
- [ ] (3) Add tests for a zillion attention markers, tons of lists, tons of labels, etc?

### Misc

- [ ] (?) Improve document performance (potential 50%)
- [ ] (?) Improve paragraph performance (potential 15%)
- [ ] (?) Improve label (link, image) performance (potential 7%)
- [ ] (3) Read through rust docs to figure out what useful functions there are,
      and fix stuff I’m doing manually now
- [ ] (5) Do some research on rust best practices for APIs, e.g., what to accept,
      how to integrate with streams or so?
- [ ] (3) Write comparison to other parsers
- [ ] (3) Add node/etc bindings?
- [ ] (3) Bunch of docs
- [ ] (5) Site

### Extensions

The extensions below are listed from top to bottom from more important to less
important.

- [x] (1) frontmatter (yaml, toml) (flow)
      — [`micromark-extension-frontmatter`](https://github.com/micromark/micromark-extension-frontmatter)
- [x] (3) autolink literal (GFM) (text)
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

#### After

- [ ] (8) After all extensions, including MDX, are done, see if we can integrate
      this with SWC to compile MDX

## Scripts

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
cargo fmt --all -- --check && cargo clippy -- -D clippy::pedantic -D clippy::cargo -A clippy::doc_link_with_quotes
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

[commonmark]: https://spec.commonmark.org
[docs]: https://wooorm.com/micromark-rs/micromark/
