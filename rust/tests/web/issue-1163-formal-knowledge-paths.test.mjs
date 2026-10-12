// Issue #1163 JS parity for the paths that put the formalizer to work, the
// browser twins of rust/tests/unit/issue_1163_formal_knowledge_paths.rs:
// R3 (the generic formalizer covers each bespoke extractor's statements on
// its own fixture), R6 (the store replays a remembered capture as a cache
// hit), R7 (computed trust features), R10 (the page_query_text route and the
// shared cross-runtime-synthesis case), and R13 (HTML and Markdown read as
// document-conversion sources).

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";

import { REPO_ROOT, createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

const FIXTURES = path.join(REPO_ROOT, "rust", "tests", "fixtures", "issue-1163");
const fixture = (name) => readFileSync(path.join(FIXTURES, name), "utf8");
const KOTLINLANG_URL = "https://kotlinlang.org/docs/command-line.html";
const MIRROR_URL = "https://mirror.example.org/kotlin/command-line.html";
const KOTLINLANG_PAGE = fixture("command-line.html");
const WIKIDATA_PAYLOAD = fixture("wikidata_entity_v1/kotlin.json");

async function call(expression) {
  await seeded;
  return plain(evaluate(worker, expression));
}

const literal = (value) => JSON.stringify(value);

test("R3: the generic formalizer covers every bespoke extractor's statements", async () => {
  const cases = [
    ["wordnet_sense_v1/variance.json", "application/json", [
      "the quality of being subject to variation",
      "an official document that permits something not normally allowed",
    ]],
    ["mediawiki_summary_v1/variance.json", "application/json", [
      "In probability theory and statistics, variance is the expected value of the squared deviation from the mean of a random variable.",
    ]],
    ["wikidata_entity_v1/kotlin.json", "application/json", ["general-purpose programming language"]],
    ["oeis_sequence_v1/fibonacci.json", "application/json", [
      "Fibonacci numbers: F(n) = F(n-1) + F(n-2) with F(0) = 0 and F(1) = 1.",
      "0,1,1,2,3,5,8,13,21,34,55,89,144,233,377,610",
      "Also called Lamé's sequence.",
      "F(n+2) = number of binary sequences of length n that have no consecutive 0's.",
    ]],
    ["python_docs_v1/functions.html", "text/html", [
      "sorted(iterable, /, *, key=None, reverse=False)",
      "Return a new sorted list from the items in iterable.",
      "sum(iterable, /, start=0)",
      "Sums start and the items of an iterable from left to right and returns the total.",
    ]],
  ];
  for (const [name, mime, statements] of cases) {
    const network = `formalizePage(${literal(fixture(name))}, ${literal(mime)}, null)`;
    assert.equal(await call(`pageCoversStatements(${network}, ${literal(statements)})`), true, name);
  }
  const definitionTerm = await call(`formalizePage(${literal(fixture("python_docs_v1/functions.html"))}, "text/html", null)`);
  assert.deepEqual(
    definitionTerm.blocks.filter((block) => block.kind === "list_item").map((block) => block.text),
    ["sorted(iterable, /, *, key=None, reverse=False)", "sum(iterable, /, start=0)"],
  );
});

test("R6: a stored page's capture is replayed as a cache hit", async () => {
  const outcome = await call(`(() => {
    const store = createFormalizedPageStore();
    const capture = { url: ${literal(KOTLINLANG_URL)}, sha256: "c".repeat(64), fetchedAt: "1759795200", cached: false, text: ${literal(KOTLINLANG_PAGE)} };
    formalizedPageStoreInsert(store, formalizedPageFromCapture(capture, "kotlin", 1, 10, "text/html"));
    const replayed = formalizedPageStoreCapture(store, ${literal(KOTLINLANG_URL)});
    return { cached: replayed.cached, sha256: replayed.sha256, original: capture.cached, missing: formalizedPageStoreCapture(store, "https://example.org/") };
  })()`);
  assert.deepEqual(outcome, { cached: true, sha256: "c".repeat(64), original: false, missing: null });
});

test("R7: the trust features are computed from the page and what is known about it", async () => {
  assert.deepEqual(await call(`pageOfficialWebsites(${literal(WIKIDATA_PAYLOAD)})`), ["https://kotlinlang.org/"]);
  const features = await call(`(() => {
    const store = createFormalizedPageStore();
    const mirror = { url: ${literal(MIRROR_URL)}, sha256: "d".repeat(64), fetchedAt: "1759795200", cached: false, text: ${literal(KOTLINLANG_PAGE)} };
    formalizedPageStoreInsert(store, formalizedPageFromCapture(mirror, "kotlin", 1, 0, null));
    return pageTrustFeatures(${literal(KOTLINLANG_URL)}, ${literal(KOTLINLANG_PAGE)}, store, pageOfficialWebsites(${literal(WIKIDATA_PAYLOAD)}));
  })()`);
  assert.deepEqual(features, { officialSite: true, https: true, primacy: null, openLicense: false, agreementPages: 1 });
  assert.equal(await call(`pageTrustScore(${literal(features)})`), 48);
  const python = await call(
    `pageTrustFeatures("https://docs.python.org/3.12/library/functions.html", ${literal(fixture("python_docs_v1/functions.html"))}, createFormalizedPageStore(), [])`,
  );
  assert.equal(python.primacy, "self_published");
  const licensed = await call(
    `pageTrustFeatures("http://example.org/notes", "<p>Text is available under https://creativecommons.org/licenses/by-sa/4.0/ terms.</p>", createFormalizedPageStore(), [])`,
  );
  assert.deepEqual(licensed, { officialSite: false, https: false, primacy: null, openLicense: true, agreementPages: 0 });
});

test("R10: the page_query_text route answers the shared parity case", async () => {
  const registry = await call("WORKER_HANDLER_REGISTRY.workerHandlers");
  assert.equal(registry.page_query_text, "tryPageQueryText");
  const cases = JSON.parse(readFileSync(path.join(REPO_ROOT, "data/parity/cross-runtime-synthesis.json"), "utf8"));
  const parity = cases.find((entry) => entry.id === "e1163_supplied_page_command_query");
  assert.ok(parity, "the parity corpus carries the #1163 case");
  await seeded;
  const answer = plain(await worker.solve(parity.prompt, [], {}, {}, [], {}));
  assert.equal(answer.intent, parity.expectedIntent, JSON.stringify(answer));
  assert.equal(answer.content, "kotlinc hello.kt -include-runtime -d hello.jar");
  for (const prefix of parity.browserExpectedEvidencePrefixes) {
    assert.ok(answer.evidence.some((link) => link.startsWith(prefix)), `${prefix}: ${JSON.stringify(answer.evidence)}`);
  }
  assert.equal(await call(`tryPageQueryText("the command in the paragraph that mentions compiler")`), null);
});

test("R13: HTML and Markdown are document-conversion sources", async () => {
  const markdown = await call(`formalizeDocumentSource("Markdown", ${literal(fixture("rust-book.md"))})`);
  assert.equal(markdown.sourceFormat, "markdown");
  assert.equal(markdown.mimeHint, "text/markdown");
  assert.deepEqual(markdown.codeLanguages, ["rust", "shell"]);
  const html = await call(`formalizeDocumentSource("html", ${literal(KOTLINLANG_PAGE)})`);
  assert.deepEqual(html.codeLanguages, ["kotlin"]);
  assert.deepEqual(html.blocks[0], ["heading", "Command-line compiler"]);
  assert.deepEqual((await call(`formalizeDocumentSource("md", "# Title\\n\\nbody")`)).blocks, [["heading", "Title"], ["paragraph", "body"]]);
  assert.equal(await call(`formalizeDocumentSource("docx", "body")`), null);
});
