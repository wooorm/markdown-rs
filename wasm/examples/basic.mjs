import { toHtml } from '../lib/index.mjs';

const markdown = `# Hello World

This is a **markdown** document with *emphasis*.

## Features

- Fast parsing
- CommonMark compliant
- WebAssembly powered

## Code Example

\`\`\`javascript
const result = await toHtml(markdown);
console.log(result);
\`\`\`

## Links

Check out [markdown-rs](https://github.com/wooorm/markdown-rs) for more information.
`;

// Display header
console.log('\n╔════════════════════════════════════════════════╗');
console.log('║     markdown-rs WASM - Basic Example           ║');
console.log('╚════════════════════════════════════════════════╝\n');

// Show input
console.log('Input Markdown:');
console.log('```markdown');
console.log(markdown.trim());
console.log('```\n');

// Convert markdown to HTML
console.log('Converting to HTML...\n');
const html = await toHtml(markdown);

// Show output
console.log('Output HTML:');
console.log('```html');
console.log(html.trim());
console.log('```\n');

console.log('Conversion complete!');
