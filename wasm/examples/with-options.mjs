import { toHtmlWithOptions } from '../lib/index.mjs';

console.log('\n╔════════════════════════════════════════════════╗');
console.log('║     markdown-rs WASM - Options Examples        ║');
console.log('╚════════════════════════════════════════════════╝\n');

// Example 1: GitHub Flavored Markdown
console.log('\n┌────────────────────────────────────────────────┐');
console.log('│  Example 1: GitHub Flavored Markdown (GFM)     │');
console.log('└────────────────────────────────────────────────┘\n');

const gfmMarkdown = `
## GFM Features

### Tables
| Feature | Supported |
|---------|-----------|
| Tables  | Yes       |
| Tasks   | Yes       |

### Task Lists
- [x] Completed task
- [ ] Pending task

### Strikethrough
~strikethrough text~

### Autolinks
https://github.com/wooorm/markdown-rs
`;

const gfmHtml = await toHtmlWithOptions(gfmMarkdown, { gfm: true });
console.log('Input Markdown:');
console.log('```markdown');
console.log(gfmMarkdown.trim());
console.log('```\n');
console.log('Output HTML:');
console.log('```html');
console.log(gfmHtml.trim());
console.log('```');

// Example 2: MDX Support
console.log('\n┌────────────────────────────────────────────────┐');
console.log('│  Example 2: MDX (JSX in Markdown)              │');
console.log('└────────────────────────────────────────────────┘\n');

const mdxMarkdown = `
# MDX Example

<CustomComponent prop="value" />

Regular markdown with **JSX** components.
`;

const mdxHtml = await toHtmlWithOptions(mdxMarkdown, { mdx: true });
console.log('Input Markdown:');
console.log('```mdx');
console.log(mdxMarkdown.trim());
console.log('```\n');
console.log('Output HTML:');
console.log('```html');
console.log(mdxHtml.trim());
console.log('```');

// Example 3: Frontmatter
console.log('\n┌────────────────────────────────────────────────┐');
console.log('│  Example 3: Frontmatter Support                │');
console.log('└────────────────────────────────────────────────┘\n');

const frontmatterMarkdown = `---
title: Example Post
date: 2024-01-01
---

# Content

This is the actual content after frontmatter.
`;

const frontmatterHtml = await toHtmlWithOptions(frontmatterMarkdown, {
  frontmatter: true
});
console.log('Input Markdown:');
console.log('```markdown');
console.log(frontmatterMarkdown.trim());
console.log('```\n');
console.log('Output HTML:');
console.log('```html');
console.log(frontmatterHtml.trim());
console.log('```');

// Example 4: Security Options
console.log('\n┌────────────────────────────────────────────────┐');
console.log('│  Example 4: Security Options                   │');
console.log('└────────────────────────────────────────────────┘\n');

const dangerousMarkdown = `
<script>console.log('This is JavaScript');</script>

[Dangerous Link](javascript:alert('XSS'))
`;

console.log('Input Markdown:');
console.log('```markdown');
console.log(dangerousMarkdown.trim());
console.log('```\n');

console.log('Safe mode output (default):');
console.log('```html');
const safeHtml = await toHtmlWithOptions(dangerousMarkdown, {});
console.log(safeHtml.trim());
console.log('```\n');

console.log('Dangerous mode output (be careful!):');
console.log('```html');
const dangerousHtml = await toHtmlWithOptions(dangerousMarkdown, {
  allowDangerousHtml: true,
  allowDangerousProtocol: true
});
console.log(dangerousHtml.trim());
console.log('```');

// Example 5: Combined Options
console.log('\n┌────────────────────────────────────────────────┐');
console.log('│  Example 5: Combined Options (GFM + MDX)       │');
console.log('└────────────────────────────────────────────────┘\n');

const combinedMarkdown = `
| GFM | MDX |
|-----|-----|
| Yes | Yes |

~strikethrough~ and <Component />
`;

const combinedHtml = await toHtmlWithOptions(combinedMarkdown, {
  gfm: true,
  mdx: true
});
console.log('Input Markdown:');
console.log('```markdown');
console.log(combinedMarkdown.trim());
console.log('```\n');
console.log('Output HTML:');
console.log('```html');
console.log(combinedHtml.trim());
console.log('```');

console.log('\n╔════════════════════════════════════════════════╗');
console.log('║            Examples Completed!                 ║');
console.log('╚════════════════════════════════════════════════╝\n');
