import fs from "fs/promises";
import path from "path";

const cleanTestName = (name) => {
  return name
    .replace(/[^a-zA-Z0-9]+/g, "_")
    .split("_")
    .filter((s) => s.length > 0)
    .join("_");
};

const createTest = ({ input, output, testName }) => {
  return `
#[test]
fn ${testName}() {
    let input = include_str!("${input}");
    let output = include_str!("${output}");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}`;
};

const remarkFixtures = async () => {
  const inputPath = "./fixtures/input";
  const outputPath = "./fixtures/tree";

  const fileNames = await fs.readdir(`./fixtures/input`);

  const histogram = {};

  const files = fileNames.map((name) => {
    const partial = name.slice(0, -5);

    let testName = partial
      .replace(/[^a-zA-Z0-9]+/g, "_")
      .split("_")
      .filter((s) => s.length > 0)
      .join("_");

    if (!histogram[testName]) {
      histogram[testName] = 1;
    } else {
      histogram[testName] += 1;
      testName = testName + "_" + histogram[testName];
    }

    return {
      input: `${inputPath}/${name}`,
      output: `${outputPath}/${partial}.json`,
      testName: `test_${testName}`,
    };
  });

  const tests = files.map(createTest).join("\n");

  console.log(tests);
};

const mdastFixtures = async () => {
  const basePath = path.resolve(".");
  const testsDir = path.join(basePath, "tests");
  const fixturesDir = path.join(testsDir, "fixtures2");

  const fileNames = await fs.readdir(fixturesDir);

  const importPath = path.relative(testsDir, fixturesDir);

  const files = fileNames
    .filter((s) => s.endsWith(".md"))
    .map((name) => {
      const fileStem = name.slice(0, -3);

      const testName = cleanTestName(fileStem);

      return {
        input: path.join(importPath, name),
        output: path.join(importPath, `${fileStem}.json`),
        testName: `test_${testName}`,
      };
    });

  const tests = files.map(createTest).join("\n");

  console.log(tests);
};

await mdastFixtures();
