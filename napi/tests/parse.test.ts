import { describe, expect, it } from "vitest";
import { parse } from "../";

describe("parse", () => {
  it("parse accepts options parameter", () => {
    const ast = parse("# Hello ~world~", {
      gfmStrikethroughSingleTilde: true,
    });
    expect(stripPositions(ast)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "children": [
              {
                "type": "text",
                "value": "Hello ~world~",
              },
            ],
            "depth": 1,
            "type": "heading",
          },
        ],
        "type": "root",
      }
    `);
  });

  it("parse parses tables with gfm option", () => {
    const ast = parse(
      `
      | Name  | Age        |
      |-------|------------|
      | Alice | <div />    |
      | Bob   | 23         |
      `,
      { gfm: true, mdx: true },
    );
    expect(stripPositions(ast)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "align": [
              null,
              null,
            ],
            "children": [
              {
                "children": [
                  {
                    "children": [
                      {
                        "type": "text",
                        "value": "Name",
                      },
                    ],
                    "type": "tableCell",
                  },
                  {
                    "children": [
                      {
                        "type": "text",
                        "value": "Age",
                      },
                    ],
                    "type": "tableCell",
                  },
                ],
                "type": "tableRow",
              },
              {
                "children": [
                  {
                    "children": [
                      {
                        "type": "text",
                        "value": "Alice",
                      },
                    ],
                    "type": "tableCell",
                  },
                  {
                    "children": [
                      {
                        "attributes": [],
                        "children": [],
                        "name": "div",
                        "type": "mdxJsxTextElement",
                      },
                    ],
                    "type": "tableCell",
                  },
                ],
                "type": "tableRow",
              },
              {
                "children": [
                  {
                    "children": [
                      {
                        "type": "text",
                        "value": "Bob",
                      },
                    ],
                    "type": "tableCell",
                  },
                  {
                    "children": [
                      {
                        "type": "text",
                        "value": "23",
                      },
                    ],
                    "type": "tableCell",
                  },
                ],
                "type": "tableRow",
              },
            ],
            "type": "table",
          },
        ],
        "type": "root",
      }
    `);
  });

  it("parse handles level 2 heading", () => {
    const ast = parse("## My Subheading");
    expect(stripPositions(ast)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "children": [
              {
                "type": "text",
                "value": "My Subheading",
              },
            ],
            "depth": 2,
            "type": "heading",
          },
        ],
        "type": "root",
      }
    `);
  });

  it("parse code block with metastring", () => {
    const ast = parse("```js metastring\nconsole.log('hello world');\n```");
    expect(stripPositions(ast)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "lang": "js",
            "meta": "metastring",
            "type": "code",
            "value": "console.log('hello world');",
          },
        ],
        "type": "root",
      }
    `);
  });

  it("parse with mdx option accepts options parameter", () => {
    const ast = parse("# Hello {expression}", {
      mdx: true,
      mdxExpressionParse: true,
      gfmStrikethroughSingleTilde: false,
    });
    expect(stripPositions(ast)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "children": [
              {
                "type": "text",
                "value": "Hello ",
              },
              {
                "_markdownRsStops": [
                  [
                    0,
                    9,
                  ],
                ],
                "type": "mdxTextExpression",
                "value": "expression",
              },
            ],
            "depth": 1,
            "type": "heading",
          },
        ],
        "type": "root",
      }
    `);
  });

  it("parse works without options parameter", () => {
    const ast = parse("# Hello world");
    expect(stripPositions(ast)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "children": [
              {
                "type": "text",
                "value": "Hello world",
              },
            ],
            "depth": 1,
            "type": "heading",
          },
        ],
        "type": "root",
      }
    `);
  });

  it("parse with mdx option works without other options", () => {
    const ast = parse("# Hello {expression}", { mdx: true });
    expect(stripPositions(ast)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "children": [
              {
                "type": "text",
                "value": "Hello ",
              },
              {
                "_markdownRsStops": [
                  [
                    0,
                    9,
                  ],
                ],
                "type": "mdxTextExpression",
                "value": "expression",
              },
            ],
            "depth": 1,
            "type": "heading",
          },
        ],
        "type": "root",
      }
    `);
  });

  it("options affect math parsing behavior", () => {
    const astWithSingleDollar = parse("$math$", {
      mathTextSingleDollar: true,
    });
    const astWithoutSingleDollar = parse("$math$", {
      mathTextSingleDollar: false,
    });
    expect(stripPositions(astWithSingleDollar)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "children": [
              {
                "type": "text",
                "value": "$math$",
              },
            ],
            "type": "paragraph",
          },
        ],
        "type": "root",
      }
    `);
    expect(stripPositions(astWithoutSingleDollar)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "children": [
              {
                "type": "text",
                "value": "$math$",
              },
            ],
            "type": "paragraph",
          },
        ],
        "type": "root",
      }
    `);
  });

  it("options affect strikethrough parsing", () => {
    const astWithSingleTilde = parse("~strikethrough~", {
      gfmStrikethroughSingleTilde: true,
    });
    const astWithoutSingleTilde = parse("~strikethrough~", {
      gfmStrikethroughSingleTilde: false,
    });
    expect(stripPositions(astWithSingleTilde)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "children": [
              {
                "type": "text",
                "value": "~strikethrough~",
              },
            ],
            "type": "paragraph",
          },
        ],
        "type": "root",
      }
    `);
    expect(stripPositions(astWithoutSingleTilde)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "children": [
              {
                "type": "text",
                "value": "~strikethrough~",
              },
            ],
            "type": "paragraph",
          },
        ],
        "type": "root",
      }
    `);
  });

  it("mdx options work with parse function", () => {
    const ast = parse("import foo from 'bar'\n\n{expression}", {
      mdx: true,
      mdxEsmParse: true,
      mdxExpressionParse: true,
    });
    expect(stripPositions(ast)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "_markdownRsStops": [
              [
                0,
                0,
              ],
            ],
            "type": "mdxjsEsm",
            "value": "import foo from 'bar'",
          },
          {
            "_markdownRsStops": [
              [
                0,
                24,
              ],
            ],
            "type": "mdxFlowExpression",
            "value": "expression",
          },
        ],
        "type": "root",
      }
    `);
  });

  it("handles broken jsx in mdx without panic", () => {
    const error = catchErrorValue(() =>
      parse(
        `
paragraph

<Callout type="info">

    `,
        { mdx: true },
      ),
    );
    expect(error).toMatchInlineSnapshot(
      `[Error: Message { place: Some(Point(6:5 (39))), reason: "Expected a closing tag for \`<Callout>\` (4:1)", rule_id: "end-tag-mismatch", source: "markdown-rs" }]`,
    );
  });

  it("returns mdast", () => {
    const ast = parse(
      `
import something from 'package'
export const x = 9

# Hello

this is a paragraph

here is some mdx {expression}

> quote

<Callout type="info">
this is a callout

</Callout>
    `,
      { mdx: true },
    );
    expect(stripPositions(ast)).toMatchInlineSnapshot(`
      {
        "children": [
          {
            "children": [
              {
                "type": "text",
                "value": "import something from 'package'
      export const x = 9",
              },
            ],
            "type": "paragraph",
          },
          {
            "children": [
              {
                "type": "text",
                "value": "Hello",
              },
            ],
            "depth": 1,
            "type": "heading",
          },
          {
            "children": [
              {
                "type": "text",
                "value": "this is a paragraph",
              },
            ],
            "type": "paragraph",
          },
          {
            "children": [
              {
                "type": "text",
                "value": "here is some mdx ",
              },
              {
                "_markdownRsStops": [
                  [
                    0,
                    101,
                  ],
                ],
                "type": "mdxTextExpression",
                "value": "expression",
              },
            ],
            "type": "paragraph",
          },
          {
            "children": [
              {
                "children": [
                  {
                    "type": "text",
                    "value": "quote",
                  },
                ],
                "type": "paragraph",
              },
            ],
            "type": "blockquote",
          },
          {
            "attributes": [
              {
                "name": "type",
                "type": "mdxJsxAttribute",
                "value": "info",
              },
            ],
            "children": [
              {
                "children": [
                  {
                    "type": "text",
                    "value": "this is a callout",
                  },
                ],
                "type": "paragraph",
              },
            ],
            "name": "Callout",
            "type": "mdxJsxFlowElement",
          },
        ],
        "type": "root",
      }
    `);
  });
});

/**
 * Removes all "position" fields from an mdast/unist tree node (in-place).
 * @param node - The tree node to clean.
 * @returns The node without "position" fields.
 */
export function stripPositions<T extends Record<string, any>>(node: T): T {
  if (Array.isArray(node)) {
    // @ts-ignore
    return node.map(stripPositions);
  } else if (node && typeof node === "object") {
    const newNode: any = {};
    for (const key in node) {
      if (key === "position") continue;
      // Recursively clean properties that may be nodes or arrays of nodes
      newNode[key] = stripPositions(node[key]);
    }
    return newNode;
  }
  return node;
}

/**
 * Executes a function and returns either its value or the error thrown.
 * @param fn - The function to execute.
 * @returns The returned value, or the caught error.
 */
export function catchErrorValue<T>(fn: () => T): T | Error {
  try {
    return fn();
  } catch (e) {
    return e;
  }
}
