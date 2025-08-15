import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

// Dynamic import to handle the ES module
let wasmModule = null;

async function loadWasmModule() {
  if (!wasmModule) {
    wasmModule = await import('../pkg/markdown_rs_wasm.mjs');
  }
  return wasmModule;
}

// Initialize WASM module once
let initialized = false;
let initPromise = null;

async function ensureInitialized() {
  if (initialized) return;
  if (!initPromise) {
    const wasm = await loadWasmModule();
    // Read WASM file for Node.js
    const __dirname = dirname(fileURLToPath(import.meta.url));
    const wasmPath = join(__dirname, '..', 'pkg', 'markdown_rs_wasm_bg.wasm');
    const wasmBuffer = await readFile(wasmPath);

    initPromise = wasm.default({ module_or_path: wasmBuffer }).then(() => {
      initialized = true;
    });
  }
  await initPromise;
}

/**
 * Convert markdown to HTML using CommonMark
 * @param {string} input - Markdown text
 * @returns {Promise<string>} HTML output
 */
export async function toHtml(input) {
  await ensureInitialized();
  const wasm = await loadWasmModule();
  return wasm.to_html(input);
}

/**
 * Convert markdown to HTML with options
 * @param {string} input - Markdown text
 * @param {Object} options - Parsing options
 * @param {boolean} [options.gfm] - Enable GitHub Flavored Markdown
 * @param {boolean} [options.mdx] - Enable MDX
 * @param {boolean} [options.frontmatter] - Enable frontmatter
 * @param {boolean} [options.allowDangerousHtml] - Allow raw HTML
 * @param {boolean} [options.allowDangerousProtocol] - Allow dangerous protocols
 * @returns {Promise<string>} HTML output
 */
export async function toHtmlWithOptions(input, options) {
  await ensureInitialized();
  const wasm = await loadWasmModule();
  return wasm.to_html_with_options(input, options);
}

// Also export sync versions after initialization
export function toHtmlSync(input) {
  if (!initialized || !wasmModule) {
    throw new Error('WASM not initialized. Call toHtml() first or await init()');
  }
  return wasmModule.to_html(input);
}

export function toHtmlWithOptionsSync(input, options) {
  if (!initialized || !wasmModule) {
    throw new Error('WASM not initialized. Call toHtmlWithOptions() first or await init()');
  }
  return wasmModule.to_html_with_options(input, options);
}

export async function init() {
  await ensureInitialized();
}
