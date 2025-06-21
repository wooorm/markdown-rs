import { describe, expect, it } from "vitest";
import { toHtml } from "../";

describe("toHtml", () => {
  it("converts basic markdown to HTML", () => {
    const html = toHtml("# Hello world");
    expect(html).toMatchInlineSnapshot(`"<h1>Hello world</h1>"`);
  });

  it("works without options parameter", () => {
    const html = toHtml("**bold** and *italic*");
    expect(html).toMatchInlineSnapshot(`"<p><strong>bold</strong> and <em>italic</em></p>"`);
  });

  it("handles GFM strikethrough with parse options", () => {
    const html = toHtml("~~strikethrough~~", {
      parse: { gfm: true }
    });
    expect(html).toMatchInlineSnapshot(`"<p><del>strikethrough</del></p>"`);
  });

  it("handles GFM table with parse options", () => {
    const html = toHtml(`| Name | Age |
|------|-----|
| John | 30  |
| Jane | 25  |`, {
      parse: { gfm: true }
    });
    expect(html).toMatchInlineSnapshot(`
      "<table>
      <thead>
      <tr>
      <th>Name</th>
      <th>Age</th>
      </tr>
      </thead>
      <tbody>
      <tr>
      <td>John</td>
      <td>30</td>
      </tr>
      <tr>
      <td>Jane</td>
      <td>25</td>
      </tr>
      </tbody>
      </table>"
    `);
  });

  it("allows dangerous HTML when enabled", () => {
    const html = toHtml('<script>alert("xss")</script>', {
      compile: { allowDangerousHtml: true }
    });
    expect(html).toMatchInlineSnapshot(`"<script>alert("xss")</script>"`);
  });

  it("escapes dangerous HTML by default", () => {
    const html = toHtml('<script>alert("xss")</script>');
    expect(html).toMatchInlineSnapshot(`"&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;"`);
  });

  it("allows dangerous protocols when enabled", () => {
    const html = toHtml('<javascript:alert(1)>', {
      compile: { allowDangerousProtocol: true }
    });
    expect(html).toMatchInlineSnapshot(`"<p><a href="javascript:alert(1)">javascript:alert(1)</a></p>"`);
  });

  it("blocks dangerous protocols by default", () => {
    const html = toHtml('<javascript:alert(1)>');
    expect(html).toMatchInlineSnapshot(`"<p><a href="">javascript:alert(1)</a></p>"`);
  });

  it("handles GFM footnotes with custom back label", () => {
    const html = toHtml(`Here is a footnote[^1].

[^1]: This is the footnote.`, {
      parse: { gfm: true },
      compile: { gfmFootnoteBackLabel: "Return to text" }
    });
    expect(html).toContain('aria-label="Return to text"');
  });

  it("handles GFM task lists", () => {
    const html = toHtml(`- [x] Task 1
- [ ] Task 2`, {
      parse: { gfm: true }
    });
    expect(html).toMatchInlineSnapshot(`
      "<ul>
      <li><input type="checkbox" disabled="" checked="" /> Task 1</li>
      <li><input type="checkbox" disabled="" /> Task 2</li>
      </ul>"
    `);
  });

  it("makes task list checkboxes checkable when enabled", () => {
    const html = toHtml(`- [x] Task 1
- [ ] Task 2`, {
      parse: { gfm: true },
      compile: { gfmTaskListItemCheckable: true }
    });
    expect(html).toMatchInlineSnapshot(`
      "<ul>
      <li><input type="checkbox" checked="" /> Task 1</li>
      <li><input type="checkbox" /> Task 2</li>
      </ul>"
    `);
  });

  it("handles different line endings", () => {
    const html = toHtml("> quote", {
      compile: { defaultLineEnding: "crlf" }
    });
    expect(html).toMatchInlineSnapshot(`
      "<blockquote>
      <p>quote</p>
      </blockquote>"
    `);
  });

  it("handles MDX with parse options", () => {
    const html = toHtml("# Hello {name}", {
      parse: { mdx: true }
    });
    expect(html).toMatchInlineSnapshot(`"<h1>Hello </h1>"`);
  });

  it("handles frontmatter when enabled", () => {
    const html = toHtml(`---
title: Test
---

# Content`, {
      parse: { frontmatter: true }
    });
    expect(html).toMatchInlineSnapshot(`"<h1>Content</h1>"`);
  });

  it("treats frontmatter as content when disabled", () => {
    const html = toHtml(`---
title: Test
---

# Content`);
    expect(html).toMatchInlineSnapshot(`
      "<hr />
      <h2>title: Test</h2>
      <h1>Content</h1>"
    `);
  });

  it("combines parse and compile options", () => {
    const html = toHtml(`<div>

~~strikethrough~~

</div>`, {
      parse: { gfm: true },
      compile: { allowDangerousHtml: true }
    });
    expect(html).toMatchInlineSnapshot(`
      "<div>
      <p><del>strikethrough</del></p>
      </div>"
    `);
  });

  it("handles GFM tagfilter", () => {
    const html = toHtml('<iframe>test</iframe>', {
      parse: { gfm: true },
      compile: { 
        allowDangerousHtml: true,
        gfmTagfilter: true 
      }
    });
    expect(html).toMatchInlineSnapshot(`"&lt;iframe>test&lt;/iframe>"`);
  });

  it("allows iframe without tagfilter", () => {
    const html = toHtml('<iframe>test</iframe>', {
      parse: { gfm: true },
      compile: { 
        allowDangerousHtml: true,
        gfmTagfilter: false 
      }
    });
    expect(html).toMatchInlineSnapshot(`"<iframe>test</iframe>"`);
  });

  it("handles custom footnote settings", () => {
    const html = toHtml(`Here is a footnote[^1].

[^1]: This is the footnote.`, {
      parse: { gfm: true },
      compile: { 
        gfmFootnoteLabel: "Notes",
        gfmFootnoteLabelTagName: "h3",
        gfmFootnoteLabelAttributes: 'class="footnote-title"',
        gfmFootnoteClobberPrefix: ""
      }
    });
    expect(html).toContain('<h3 id="footnote-label" class="footnote-title">Notes</h3>');
    expect(html).toContain('href="#fn-');  // no prefix
  });

  it("handles math text with single dollar disabled", () => {
    const html = toHtml("$math$", {
      parse: { mathTextSingleDollar: false }
    });
    expect(html).toMatchInlineSnapshot(`"<p>$math$</p>"`);
  });

  it("handles strikethrough with single tilde disabled", () => {
    const html = toHtml("~strikethrough~", {
      parse: { 
        gfm: true,
        gfmStrikethroughSingleTilde: false 
      }
    });
    expect(html).toMatchInlineSnapshot(`"<p>~strikethrough~</p>"`);
  });
});