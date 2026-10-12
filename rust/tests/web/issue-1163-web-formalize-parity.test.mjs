// Issue #1163 JS parity (R1163-10): the browser worker's generic page
// formalizer, js/worker/formal_ai_worker_web_formalize.js, run over the same
// committed fixtures rust/tests/unit/issue_1163_web_formalize.rs pins, with
// the same expectations: structure and code blocks from HTML, Markdown
// fences tagged with the `unknown` fallback, the class-prefix language tag,
// JSON statements covering the bespoke ones, the URL-and-hash store, the two
// page queries answering with block links, the trust score and the
// rediscovery record.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";

import { REPO_ROOT, createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

const FIXTURES = path.join(REPO_ROOT, "rust", "tests", "fixtures", "issue-1163");
const KOTLINLANG_URL = "https://kotlinlang.org/docs/command-line.html";
const KOTLINLANG_PAGE = readFileSync(path.join(FIXTURES, "command-line.html"), "utf8");
const RUST_BOOK_PAGE = readFileSync(path.join(FIXTURES, "rust-book.md"), "utf8");
const SCALA_DOCS_PAGE = readFileSync(path.join(FIXTURES, "scala-docs.html"), "utf8");
const WIKTIONARY_PAYLOAD = JSON.stringify([
  {
    word: "variance",
    meanings: [
      {
        partOfSpeech: "noun",
        definitions: [{ definition: "The fact of varying; a departure from a standard." }],
      },
    ],
  },
]);
const KOTLIN_SHA = "a".repeat(64);
const KOTLIN_QUERY = "how to compile a Kotlin program from the command line";

async function call(expression) {
  await seeded;
  return plain(evaluate(worker, expression));
}

function literal(value) {
  return JSON.stringify(value);
}

async function formalize(text, mime, url) {
  return call(`formalizePage(${literal(text)}, ${literal(mime)}, ${literal(url || null)})`);
}

function kotlinPageExpression(sha) {
  const capture = { url: KOTLINLANG_URL, sha256: sha, fetchedAt: "1759795200", cached: false, text: KOTLINLANG_PAGE };
  const trust = `pageTrustScore({officialSite: true, https: true, primacy: "self_published", openLicense: true, agreementPages: 2})`;
  return `formalizedPageFromCapture(${literal(capture)}, ${literal(KOTLIN_QUERY)}, 1, ${trust}, "text/html")`;
}

test("an HTML page formalizes structure, a table, a list and its tagged code", async () => {
  const network = await formalize(KOTLINLANG_PAGE, "text/html", KOTLINLANG_URL);
  const texts = network.blocks.map((block) => block.text);
  assert.ok(texts.includes("Command-line compiler"), texts.join("\n"));
  assert.ok(texts.some((text) => text.includes("Kotlin runtime")));
  assert.ok(texts.includes("-include-runtime | Include the Kotlin runtime into the resulting jar"));
  assert.ok(texts.some((text) => text.includes("JDK 8 or higher")));
  const code = network.blocks.find((block) => block.kind === "code_block" && block.language === "kotlin");
  assert.ok(code, "the kotlin code block is reached through html, body and pre");
  assert.ok(code.text.includes("kotlinc hello.kt -include-runtime -d hello.jar"));
  assert.ok(!texts.some((text) => text.includes("Trimmed capture")), "the HTML comment is skipped");
  assert.equal(network.domain, "kotlinlang.org");
});

test("a page with no hint is sniffed, and opaque tags are skipped whole", async () => {
  const sniffed = await formalize(KOTLINLANG_PAGE, null);
  assert.equal(sniffed.mime, "html");
  const scripted = await formalize(
    "<html><body><script>var p = '<p>not prose</p>';</script><p>prose</p></body></html>",
    "text/html",
  );
  assert.deepEqual(scripted.blocks.map((block) => block.text), ["prose"]);
  const nested = await formalize("<pre>x</pre><p>after the pre</p>", "text/html");
  assert.deepEqual(nested.blocks.map((block) => block.kind), ["code_block", "paragraph"]);
});

test("Markdown fences tag languages, and a bare fence falls back to unknown", async () => {
  const network = await formalize(RUST_BOOK_PAGE, "text/markdown");
  assert.ok(network.blocks.some((block) => block.text.includes("println!(\"Hello, world!\")")));
  const languages = network.blocks.filter((block) => block.kind === "code_block").map((block) => block.language);
  assert.deepEqual(languages, ["rust", "shell"]);
  const bare = await formalize("```\nplain text block\n```", "text/markdown");
  assert.equal(bare.blocks[0].language, "unknown");
});

test("the shebang and URL-extension heuristics tag an unannotated block", async () => {
  const rules = await call("pageFormalizationRules()");
  const shebang = await call(
    `pageResolveLanguage(pageFormalizationRules(), null, null, "#!/usr/bin/env python3\\nprint(1)", null)`,
  );
  assert.equal(shebang, "python");
  const extension = await call(
    `pageResolveLanguage(pageFormalizationRules(), null, null, "fn main() {}", "https://example.com/src/main.rs")`,
  );
  assert.equal(extension, "rust");
  assert.equal(rules.tagFallback, "unknown");
});

test("a GitHub-style highlight class tags the Scala block", async () => {
  const network = await formalize(SCALA_DOCS_PAGE, "text/html");
  const code = network.blocks.find((block) => block.language === "scala");
  assert.ok(code && code.text.includes("println(\"Hello, world!\")"));
});

test("the generic formalizer covers the bespoke wiktionary statements", async () => {
  const network = `formalizePage(${literal(WIKTIONARY_PAYLOAD)}, "application/json", null)`;
  const bespoke = ["The fact of varying; a departure from a standard.", "noun"];
  assert.equal(await call(`pageCoversStatements(${network}, ${literal(bespoke)})`), true);
  const missing = ["the second moment of a distribution about its mean"];
  assert.equal(await call(`pageCoversStatements(${network}, ${literal(missing)})`), false);
});

test("plain and PDF text formalize as paragraphs", async () => {
  const plainText = await formalize("first paragraph\n\nsecond paragraph", "text/plain");
  assert.deepEqual(plainText.blocks.map((block) => block.text), ["first paragraph", "second paragraph"]);
  const pdf = await formalize("extracted pdf line one\n\nextracted pdf line two", "application/pdf");
  assert.equal(pdf.blocks[0].text, "extracted pdf line one");
});

test("the store keys pages by URL and hash and the queries answer with block links", async () => {
  await seeded;
  const outcome = plain(
    evaluate(
      worker,
      `(() => {
        const store = createFormalizedPageStore();
        const first = ${kotlinPageExpression(KOTLIN_SHA)};
        const key = formalizedPageStoreInsert(store, first);
        formalizedPageStoreInsert(store, first);
        const sizeAfterRepeat = store.pages.size;
        const blocks = formalizedPageCodeBlocksOn(store, "kotlinlang.org", "-d");
        const compiling = formalizedPageCommandMentioning(store, "compil");
        const running = formalizedPageCommandMentioning(store, "jar");
        const parsedBlocks = formalizedPageQuery(store, "Code blocks on KotlinLang.org whose text contains -d?");
        const parsedCommands = formalizedPageQuery(store, "the command in the paragraph that mentions compile");
        const unparsed = formalizedPageQuery(store, "what is the weather today");
        formalizedPageStoreInsert(store, ${kotlinPageExpression("b".repeat(64))});
        return { key, sizeAfterRepeat, sizeAfterChange: store.pages.size, blocks, compiling, running,
          rediscovery: first.network.rediscovery, trust: first.trust, parsedBlocks, parsedCommands, unparsed };
      })()`,
    ),
  );
  assert.equal(outcome.key, `page:${KOTLIN_SHA}:${KOTLINLANG_URL}`);
  assert.equal(outcome.sizeAfterRepeat, 1);
  assert.equal(outcome.sizeAfterChange, 2);
  assert.equal(outcome.blocks.length, 1);
  assert.equal(outcome.blocks[0].kind, "code_block");
  // Only the paragraph that mentions compiling answers, with the command it
  // introduces; "java -jar hello.jar" mentions nothing about compiling.
  assert.equal(outcome.compiling.length, 1);
  assert.ok(outcome.compiling[0].text.startsWith("kotlinc"), JSON.stringify(outcome.compiling));
  assert.deepEqual(outcome.running.map((block) => block.text), ["java -jar hello.jar"]);
  assert.equal(outcome.rediscovery.url, KOTLINLANG_URL);
  assert.equal(outcome.rediscovery.sha256, KOTLIN_SHA);
  assert.equal(outcome.rediscovery.query, KOTLIN_QUERY);
  assert.equal(outcome.rediscovery.rank, "1");
  assert.equal(outcome.rediscovery.fetched_at, "1759795200");
  assert.equal(outcome.trust, 88);
  // The memory-query surfaces come from the seed's page_query templates.
  assert.deepEqual(outcome.parsedBlocks, outcome.blocks);
  assert.deepEqual(outcome.parsedCommands, outcome.compiling);
  assert.equal(outcome.unparsed, null);
});

test("the trust score ranks without excluding", async () => {
  const trusted = await call(
    `pageTrustScore({officialSite: true, https: true, primacy: "first_hand_record", openLicense: true, agreementPages: 3})`,
  );
  const cited = await call(`pageTrustScore({primacy: "citation"})`);
  assert.equal(trusted, 100);
  assert.ok(cited < trusted && cited > 0, `${cited}`);
});
