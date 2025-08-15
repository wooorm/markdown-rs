import assert from 'node:assert';
import { describe, it } from 'node:test';
import { toHtml, toHtmlWithOptions, toHtmlSync, toHtmlWithOptionsSync } from '../lib/index.mjs';

describe('markdown-rs WASM', () => {
  describe('Basic functionality', () => {
    it('converts heading to HTML', async () => {
      const result = await toHtml('# Hello World');
      assert.strictEqual(result, '<h1>Hello World</h1>');
    });

    it('converts paragraph to HTML', async () => {
      const result = await toHtml('This is a paragraph.');
      assert.strictEqual(result, '<p>This is a paragraph.</p>');
    });

    it('converts emphasis to HTML', async () => {
      const result = await toHtml('*italic* and **bold**');
      assert.strictEqual(result, '<p><em>italic</em> and <strong>bold</strong></p>');
    });

    it('converts links to HTML', async () => {
      const result = await toHtml('[GitHub](https://github.com)');
      assert.strictEqual(result, '<p><a href="https://github.com">GitHub</a></p>');
    });

    it('converts code blocks to HTML', async () => {
      const result = await toHtml('```js\nconst x = 1;\n```');
      assert(result.includes('<pre>'));
      assert(result.includes('<code'));
      assert(result.includes('const x = 1;'));
    });
  });

  describe('GFM options', () => {
    it('enables strikethrough with GFM', async () => {
      const result = await toHtmlWithOptions('~strikethrough~', { gfm: true });
      assert(result.includes('<del>strikethrough</del>'));
    });

    it('enables tables with GFM', async () => {
      const markdown = '| a | b |\n|---|---|\n| c | d |';
      const result = await toHtmlWithOptions(markdown, { gfm: true });
      assert(result.includes('<table>'));
      assert(result.includes('<td>c</td>'));
    });

    it('enables autolinks with GFM', async () => {
      const result = await toHtmlWithOptions('https://example.com', { gfm: true });
      assert(result.includes('<a href="https://example.com">'));
    });
  });

  describe('MDX options', () => {
    it('handles JSX with MDX enabled', async () => {
      const result = await toHtmlWithOptions('# Hello <Component />', { mdx: true });
      assert(result.includes('<h1>'));
    });
  });

  describe('Security options', () => {
    it('blocks dangerous HTML by default', async () => {
      const html = '<script>alert("test")</script>';
      const result = await toHtmlWithOptions(html, {});
      assert(!result.includes('<script>'));
    });

    it('allows dangerous HTML when enabled', async () => {
      const html = '<script>alert("test")</script>';
      const result = await toHtmlWithOptions(html, { allowDangerousHtml: true });
      assert(result.includes('<script>'));
    });
  });

  describe('Sync functions', () => {
    it('sync functions work after initialization', async () => {
      // First initialize with async call
      await toHtml('# Init');
      
      // Now sync should work
      const result = toHtmlSync('# Hello Sync');
      assert.strictEqual(result, '<h1>Hello Sync</h1>');
    });

    it('sync with options works', async () => {
      await toHtml('# Init');
      const result = toHtmlWithOptionsSync('~strike~', { gfm: true });
      assert(result.includes('<del>strike</del>'));
    });
  });

  describe('Edge cases', () => {
    it('handles empty input', async () => {
      const result = await toHtml('');
      assert.strictEqual(result, '');
    });

    it('handles complex markdown', async () => {
      const markdown = `
# Title

Paragraph with **bold** and *italic*.

- List item 1
- List item 2

\`\`\`js
console.log('code');
\`\`\`

> Blockquote
`;
      const result = await toHtml(markdown);
      assert(result.includes('<h1>Title</h1>'));
      assert(result.includes('<ul>'));
      assert(result.includes('<blockquote>'));
    });
  });
});