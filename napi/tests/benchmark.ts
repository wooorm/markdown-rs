import { run, bench, boxplot, summary } from "mitata";
import { parse, toHtml } from "../";
import { remark } from "remark";
import remarkMdx from "remark-mdx";
import remarkGfm from "remark-gfm";
import remarkHtml from "remark-html";

const remarkProcessor = remark().use(remarkMdx).use(remarkGfm).use(remarkHtml);

async function main() {
  let longMdxContent = await fetch(
    "https://raw.githubusercontent.com/colinhacks/zod/0a49fa39348b7c72b19ddedc3b0f879bd395304b/packages/docs/content/packages/v3.mdx",
  ).then((x) => x.text());

  summary(() => {
    bench("markdown-rs parse", () => {
      const res = parse(longMdxContent, {
        mdx: true,
        mdxExpressionParse: true,
        mdxEsmParse: true,
        gfm: true,
      });
    });

    bench("remark parse", () => {
      remarkProcessor.processSync(longMdxContent);
    });
  });
  summary(() => {
    bench("markdown-rs parse without parsing js", () => {
      const res = parse(longMdxContent, {
        mdx: true,
        mdxExpressionParse: false,
        mdxEsmParse: false,
        gfm: true,
      });
    });

    bench("remark parse", () => {
      remarkProcessor.processSync(longMdxContent);
    });
  });
  summary(() => {
    bench("markdown-rs html", () => {
      const res = toHtml(longMdxContent, {
        parse: {
          mdx: true,
          mdxExpressionParse: true,
          mdxEsmParse: true,
          gfm: true,
        },
      });
    });

    bench("remark html", () => {
      const file = remarkProcessor.processSync(longMdxContent);
      const res = String(file);
    });
  });

  await run();
}

main();
