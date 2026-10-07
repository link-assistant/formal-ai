// Issue #1164 JS parity (R1164-11): the browser worker's code-example
// decomposer, js/worker/formal_ai_worker_code_examples.js, over the captured
// Hello World pages rust/tests/unit/issue_1164_code_example_knowledge.rs
// reads (through the #1163 browser formalizer), with the same expectations:
// an unregistered grammar is refused by name, each fixture yields its seed
// output call and the printed literal, prose supplies build and run
// commands with their URL, alignment shares the entry-output-literal
// structure, and recomposition builds from the aligned example bodies, so
// Pascal stays held out.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";

import { REPO_ROOT, createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

const FIXTURES = path.join(REPO_ROOT, "rust", "tests", "fixtures", "coding-discovery", "issue-1164");
const page = (name) => readFileSync(path.join(FIXTURES, `${name}-hello-world.html`), "utf8");
const RUST_HELLO = 'fn main() {\n    println!("Hello, world!");\n}\n';
const GO_HELLO = 'package main\n\nimport "fmt"\n\nfunc main() {\n    fmt.Println("Hello, world!")\n}\n';
const KOTLIN_URL = "https://kotlinlang.org/docs/command-line.html";

async function call(expression) {
  await seeded;
  return plain(evaluate(worker, expression));
}

const literal = (value) => JSON.stringify(value);
const decompose = (source, language, prose, url) =>
  `decomposeCodeExample(${literal(source)}, ${literal(language)}, ${literal(prose || [])}, ${literal(url || "")})`;
const procedureOf = (...examples) =>
  `generalizeCodeExamples([${examples.map((example) => `${example}.ok`).join(", ")}])`;

test("an unregistered grammar is refused by name, Pascal included", async () => {
  assert.deepEqual(await call(decompose('print("hello")', "zig")), {
    error: { kind: "unknown_grammar", language: "zig" },
  });
  assert.deepEqual(await call(decompose("program HelloWorld;\nbegin\nend.", "pascal")), {
    error: { kind: "unknown_grammar", language: "pascal" },
  });
});

test("Rust decomposes into entry point, output operation and literal", async () => {
  const node = (await call(decompose(RUST_HELLO, "rust"))).ok;
  const kinds = node.parts.map((part) => `${part.kind}:${part.sourceText}`);
  assert.deepEqual(kinds, ["entry_point:main", "output_operation:println!", "string_literal:Hello, world!"]);
  assert.equal(node.programBody, RUST_HELLO.replace("Hello, world!", "{literal}"));
});

test("every captured fixture decomposes through the browser formalizer", async () => {
  for (const [name, language, outputCall] of [
    ["rust", "rust", "println!"],
    ["go", "go", "fmt.Println"],
    ["kotlin", "kotlin", "println"],
    ["swift", "swift", "print"],
    ["scala", "scala", "println"],
  ]) {
    const code = `formalizePage(${literal(page(name))}, "text/html", null).blocks.find((block) => block.kind === "code_block").text`;
    const node = await call(`decomposeCodeExample(${code}, ${literal(language)}, [], "")`);
    assert.ok(node.ok, `${language}: ${JSON.stringify(node)}`);
    assert.ok(node.ok.parts.some((part) => part.kind === "output_operation" && part.sourceText === outputCall), language);
    assert.ok(node.ok.parts.some((part) => part.kind === "string_literal" && part.sourceText === "Hello, world!"), language);
    if (language === "swift") assert.ok(node.ok.parts.every((part) => part.kind !== "entry_point"));
  }
});

test("prose links become build and run commands that keep their URL", async () => {
  const prose = [
    { relation: "compile with", text: "kotlinc hello.kt -include-runtime -d hello.jar", sourceUrl: KOTLIN_URL },
    { relation: "run_with", text: "java -jar hello.jar", sourceUrl: KOTLIN_URL },
  ];
  const node = (await call(decompose('fun main() {\n    println("Hello, world!")\n}\n', "kotlin", prose))).ok;
  const build = node.parts.find((part) => part.kind === "build_command");
  const run = node.parts.find((part) => part.kind === "run_command");
  assert.equal(build.sourceText, prose[0].text);
  assert.equal(build.sourceUrl, KOTLIN_URL);
  assert.equal(run.sourceText, "java -jar hello.jar");
});

test("Rust and Go align into a shared structure with per-language parameters", async () => {
  const procedure = await call(procedureOf(decompose(RUST_HELLO, "rust"), decompose(GO_HELLO, "go")));
  assert.equal(procedure.id, "generalized:go+rust");
  assert.deepEqual(procedure.sharedStructure, ["entry_point", "output_operation", "string_literal"]);
  const parameter = (name) => procedure.parameters.find((candidate) => candidate.name === name).perLanguage;
  assert.deepEqual(parameter("output_literal"), { rust: "Hello, world!", go: "Hello, world!" });
  assert.deepEqual(parameter("output_call"), { rust: "println!", go: "fmt.Println" });
  const steps = await call(`codeExampleProcedureSteps(${procedureOf(decompose(RUST_HELLO, "rust"), decompose(GO_HELLO, "go"))})`);
  assert.deepEqual(steps.map((step) => step.ordinal), [1, 2, 3]);
  assert.ok(steps.every((step) => step.licenseName === "" && step.licenseUrl === ""));
});

test("recomposition builds from the aligned example bodies", async () => {
  const procedure = procedureOf(decompose(RUST_HELLO, "rust"), decompose(GO_HELLO, "go", [], "https://go.dev/tour/welcome/1"));
  const bound = literal({ output_literal: "Hello, Formal AI!" });
  const go = await call(`recomposeCodeExample(${procedure}, ${bound}, "go")`);
  assert.equal(go.ok.source, GO_HELLO.replace("Hello, world!", "Hello, Formal AI!"));
  assert.deepEqual(go.ok.partSourceUrls, ["https://go.dev/tour/welcome/1"]);
  const same = await call(`recomposeCodeExample(${procedure}, {}, "rust")`);
  assert.equal(same.ok.source, RUST_HELLO);
  const escaped = await call(`recomposeCodeExample(${procedure}, ${literal({ output_literal: 'say "hi"' })}, "rust")`);
  assert.ok(escaped.ok.source.includes('println!("say \\"hi\\"");'), escaped.ok.source);
  const unaligned = await call(`recomposeCodeExample(${procedure}, ${literal({ output_call: "eprintln!" })}, "rust")`);
  assert.equal(unaligned.error.kind, "no_program_body");
  const kotlin = await call(`recomposeCodeExample(${procedure}, ${bound}, "kotlin")`);
  assert.deepEqual(kotlin, { error: { kind: "no_program_body", language: "kotlin" } });
});

test("Pascal stays held out: no stored program can produce it", async () => {
  const seed = readFileSync(path.join(REPO_ROOT, "data", "seed", "code-example-parts.lino"), "utf8");
  assert.ok(!seed.includes("program_shape"));
  assert.ok(!seed.includes("part_language pascal"));
  const bound = literal({ output_literal: "Hello, Formal AI!", output_call: "writeln" });
  const pascal = await call(`recomposeCodeExample(${procedureOf(decompose(RUST_HELLO, "rust"))}, ${bound}, "pascal")`);
  assert.deepEqual(pascal, { error: { kind: "no_program_body", language: "pascal" } });
});

test("records render as Links Notation", async () => {
  const node = await call(`decomposedCodeExampleNotation(${decompose(RUST_HELLO, "rust")}.ok)`);
  assert.ok(node.startsWith("decomposed_code_node\n  language_slug rust\n"), node);
  assert.ok(node.includes("    kind entry_point\n"));
  const procedure = await call(`generalizedCodeExampleNotation(${procedureOf(decompose(RUST_HELLO, "rust"))})`);
  assert.ok(procedure.includes("  shared_structure output_operation\n"));
  assert.ok(procedure.includes("    name output_literal\n"));
  assert.ok(procedure.includes('    source "fn main() {\\n    println!(\\"{literal}\\");\\n}\\n"'), procedure);
});
