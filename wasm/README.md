# @wooorm/markdown-wasm

WebAssembly bindings for [markdown-rs](https://github.com/wooorm/markdown-rs).

## Features

- CommonMark compliant
- GFM support (tables, strikethrough, autolinks, task lists)
- MDX support (JSX in markdown)
- Pure ESM module
- Works in Node.js 16+
- Single WASM binary (no platform-specific builds)

## Installation

```bash
npm install @wooorm/markdown-wasm
```

## Usage

```javascript
import { toHtml, toHtmlWithOptions } from '@wooorm/markdown-wasm';

// Convert markdown to HTML
const html = await toHtml('# Hello World');

// With GitHub Flavored Markdown
const gfmHtml = await toHtmlWithOptions('~strikethrough~', { 
  gfm: true 
});

// With MDX support
const mdxHtml = await toHtmlWithOptions('<Component />', {
  mdx: true
});
```

## API

### `toHtml(markdown: string): Promise<string>`

Converts markdown to HTML using CommonMark.

```javascript
const html = await toHtml('# Hello World');
```

### `toHtmlWithOptions(markdown: string, options: Object): Promise<string>`

Converts markdown to HTML with options.

**Options:**
- `gfm: boolean` - Enable GitHub Flavored Markdown
- `mdx: boolean` - Enable MDX (JSX in markdown)
- `frontmatter: boolean` - Enable YAML frontmatter
- `allowDangerousHtml: boolean` - Allow raw HTML (default: false)
- `allowDangerousProtocol: boolean` - Allow dangerous protocols (default: false)

## Examples

Run the examples to see the library in action:

```bash
# Basic example
pnpm example:basic

# Options example (GFM, MDX, security)
pnpm example:options
```

## Testing

```bash
# Run tests
pnpm test
```

## Building

```bash
# Build WASM module
pnpm build
```

## Why WASM?

This implementation uses WebAssembly to provide:
- Universal compatibility (one binary for all platforms)
- No native dependencies
- Reliable performance across environments

## License

MIT