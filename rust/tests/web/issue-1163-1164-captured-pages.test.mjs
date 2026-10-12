// Issues #1163 R14 and #1164 R12 JS parity: the browser formalizer and
// decomposer over byte-for-byte captures of the real documentation pages
// (rust/tests/fixtures/coding-discovery/captured/), the twin of
// rust/tests/unit/issue_1163_1164_captured_pages.rs. Each capture is pinned by
// the SHA-256 it was recorded with, its code blocks carry the language tags the
// page itself declares (a `language-` class, kotlinlang's `data-lang`, a
// Markdown fence, or the class of the enclosing container), and every Hello
// World page yields its seed output call and printed literal.

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";

import { REPO_ROOT, createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");
const CAPTURED = path.join(REPO_ROOT, "rust", "tests", "fixtures", "coding-discovery", "captured");
const literal = (value) => JSON.stringify(value);

// [file, mime, recorded SHA-256, the page's code-block language tags]
const PAGES = [
  ["issue-1163/command-line.html", "text/html", "0847a47edc7136be89ddb103aed1178c4eb95a0ba03bdfe396f1b207ab806e8e",
    ["bash", "bash", "bash", "kotlin", "bash", "bash", "bash", "bash", "bash", "kotlin", "bash", "bash"]],
  ["issue-1163/ch01-02-hello-world.md", "text/markdown", "645189892417cf5431fafb29c4e9b413db2c8cb07fb99e993efaf3c3e9d7a143",
    ["console", "cmd", "rust", "console", "powershell", "rust", "rust", "console", "console", "cmd", "console"]],
  ["issue-1163/taste-hello-world.html", "text/html", "913485460cbb046dab070e46eca56fc97cc3f1bc43fb57ae931a2f711145ef2b",
    ["scala", "scala", "bash", "plaintext", "scala", "scala", "scala", "bash", "bash", "scala"]],
  ["issue-1164/kotlin-tour-hello-world.html", "text/html", "916f03525029483ef10ca64b75b7c110c538aadce2c7021dbb6a56e8ef8bd92a",
    ["kotlin", "kotlin", "kotlin", "kotlin", "kotlin"]],
  ["issue-1164/ch01-02-hello-world.html", "text/html", "98d3e31d5c41a6572a3536ee16c51a1058b3974e6f22e15ba5fb2a5e964ceb4e",
    ["console", "cmd", "rust", "console", "powershell", "rust", "rust", "console", "console", "cmd", "console"]],
  ["issue-1164/getting-started.html", "text/html", "a7048d2cce6a94ae835c9503d6af5e96f88954a459d831fe03f9a7d82b60ef2c",
    Array(10).fill("unknown")],
];

const read = (file) => readFileSync(path.join(CAPTURED, file));

async function codeBlocks(file, mime) {
  await seeded;
  const page = plain(evaluate(worker, `formalizePage(${literal(read(file).toString("utf8"))}, ${literal(mime)}, null)`));
  return page.blocks.filter((block) => block.kind === "code_block");
}

test("every capture is the recorded bytes and keeps the page's language tags", async () => {
  for (const [file, mime, sha256, languages] of PAGES) {
    assert.equal(createHash("sha256").update(read(file)).digest("hex"), sha256, file);
    assert.deepEqual((await codeBlocks(file, mime)).map((block) => block.language), languages, file);
  }
});

test("R1163-14: kotlinlang's numbered steps keep the compile and run commands", async () => {
  const texts = (await codeBlocks("issue-1163/command-line.html", "text/html")).map((block) => block.text.trim());
  assert.ok(texts.includes("kotlinc hello.kt -include-runtime -d hello.jar"), texts.join("\n"));
  assert.ok(texts.includes("java -jar hello.jar"), texts.join("\n"));
});

test("R1163-11 offline half: the paragraph mentioning compiling introduces the kotlinc command", async () => {
  await seeded;
  const text = read("issue-1163/command-line.html").toString("utf8");
  const commands = plain(evaluate(worker, `(() => {
    const store = createFormalizedPageStore();
    const capture = { url: "https://kotlinlang.org/docs/command-line.html", sha256: "", text: ${literal(text)} };
    formalizedPageStoreInsert(store, formalizedPageFromCapture(capture, "how to compile a Kotlin program from the command line", 1, 0, "text/html"));
    return formalizedPageCommandMentioning(store, "Compile").map((block) => block.text.trim());
  })()`));
  assert.deepEqual(commands, ["kotlinc hello.kt -include-runtime -d hello.jar"]);
});

test("R1164-12: every captured Hello World page decomposes", async () => {
  const cases = [
    ["issue-1163/command-line.html", "text/html", "kotlin", ["entry_point:main", "output_operation:println", "string_literal:Hello, World!"]],
    ["issue-1164/kotlin-tour-hello-world.html", "text/html", "kotlin", ["entry_point:main", "output_operation:println", "string_literal:Hello, world!"]],
    ["issue-1164/ch01-02-hello-world.html", "text/html", "rust", ["entry_point:main", "output_operation:println!", "string_literal:Hello, world!"]],
    ["issue-1164/getting-started.html", "text/html", "go", ["entry_point:main", "output_operation:fmt.Println", "string_literal:Hello, World!"]],
    ["issue-1164/GuidedTour.md", "text/markdown", "swift", ["output_operation:print", "string_literal:Hello, world!"]],
    ["issue-1163/taste-hello-world.html", "text/html", "scala", ["entry_point:main", "output_operation:println", "string_literal:Hello, World!"]],
  ];
  for (const [file, mime, language, parts] of cases) {
    // The page's example is its first block, tagged with the language or
    // untagged (go.dev declares no tag), that decomposes into an output call.
    let found = null;
    for (const block of await codeBlocks(file, mime)) {
      if (block.language !== language && block.language !== "unknown") continue;
      const result = plain(evaluate(worker, `decomposeCodeExample(${literal(block.text)}, ${literal(language)}, [], "")`));
      if (result.ok && result.ok.parts.some((part) => part.kind === "output_operation")) {
        found = result.ok.parts.map((part) => `${part.kind}:${part.sourceText}`);
        break;
      }
    }
    assert.deepEqual(found, parts, file);
  }
});

test("R1165-1: a code block keeps its indentation, prose still collapses", async () => {
  const blocks = await codeBlocks("issue-1164/ch01-02-hello-world.html", "text/html");
  assert.ok(blocks.some((block) => block.text === 'fn main() {\n    println!("Hello, world!");\n}'), JSON.stringify(blocks));
  const kotlin = await codeBlocks("issue-1163/command-line.html", "text/html");
  assert.ok(kotlin.some((block) => block.text === 'fun main() {\n    println("Hello, World!")\n}'), JSON.stringify(kotlin));
  const page = plain(evaluate(worker, `formalizePage(${literal(
    "<p>Some   spaced\n   prose</p><pre><code>\n\n      if x {\n<span>      </span>    y()\n      }\n\n</code></pre>",
  )}, "text/html", null)`));
  // The shared indentation and the blank lines around the block are markup;
  // a highlighter span around leading spaces is indentation.
  assert.deepEqual(page.blocks.map((block) => block.text), ["Some spaced prose", "if x {\n    y()\n}"]);
});
