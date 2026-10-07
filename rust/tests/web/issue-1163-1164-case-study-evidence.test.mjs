// Issues #1163 and #1164: the case-study evidence and changelog fragments the
// issues ask for (R1163-15, R1163-16, R1164-14, R1164-15), checked against the
// engine instead of only for presence.
//
// R1163-15: the self-use formalizer run names the committed capture by its
// SHA-256, and formalizing that capture again with the browser engine yields
// the recorded block count, the recorded code blocks, and the recorded compile
// and run commands as code blocks of the page.
// R1164-14: the decomposed example is reproducible: each page's SHA-256 is the
// committed capture's, and decomposing and generalizing the captures again
// gives the recorded decomposed nodes verbatim and the recorded generalized
// procedure (its program bodies compared by language, since the HTML walker's
// whitespace handling is a recorded limit that may still improve).

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import test from "node:test";

import { REPO_ROOT, createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");
const read = (relative) => readFileSync(path.join(REPO_ROOT, relative), "utf8");
const sha256 = (relative) => createHash("sha256").update(readFileSync(path.join(REPO_ROOT, relative))).digest("hex");
const CAPTURED = "rust/tests/fixtures/coding-discovery/captured";
const literal = (value) => JSON.stringify(value);

async function call(expression) {
  await seeded;
  return plain(evaluate(worker, expression));
}

// The changelog fragment for an issue: front matter `bump: minor` and an
// `### Added` section with at least one entry. Once a release collects it,
// the fragment leaves changelog.d/ for the fragment-release map, and its
// entries live in that version's CHANGELOG section, which is checked instead.
function assertMinorAddedFragment(suffix) {
  const name = readdirSync(path.join(REPO_ROOT, "changelog.d")).find((file) => /^\d{8}_\d{6}_/.test(file) && file.endsWith(suffix));
  if (name) {
    const text = read(`changelog.d/${name}`);
    assert.match(text, /^---\nbump: minor\n---\n/);
    assert.match(text, /\n### Added\n- \S/);
    return text;
  }
  const row = read("docs/case-studies/issue-711/fragment-release-map.tsv").split("\n")
    .map((line) => line.split("\t")).find(([fragment]) => fragment && fragment.endsWith(suffix));
  assert.ok(row, `changelog.d or the fragment-release map has a <timestamp>_${suffix} fragment`);
  const changelogs = ["CHANGELOG.md", ...readdirSync(path.join(REPO_ROOT, "docs", "changelog")).map((file) => `docs/changelog/${file}`)];
  for (const file of changelogs) {
    const text = read(file);
    const start = text.indexOf(`## [${row[1]}]`);
    if (start < 0) continue;
    const next = text.indexOf("\n## [", start + 1);
    const section = text.slice(start, next < 0 ? text.length : next);
    assert.match(section, /\n### Added\n/);
    return section;
  }
  assert.fail(`no changelog section for release ${row[1]}`);
}

test("R1163-16: the issue-1163 changelog fragment bumps minor with an Added section", () => {
  assert.match(assertMinorAddedFragment("_issue-1163-internet-formal-knowledge.md"), /issue #1163/);
});

test("R1164-15: the issue-1164 changelog fragment bumps minor with an Added section", () => {
  assert.match(assertMinorAddedFragment("_issue-1164-code-node-decomposition.md"), /issue #1164/);
});

test("R1163-15: the design plan, the Agent CLI run and the self-use trace are present", () => {
  const plan = read("docs/case-studies/issue-1163/plans/00-formalizer-design.md");
  assert.match(plan, /^# /m);
  const agentRun = read("docs/case-studies/issue-1163/agent-cli-evidence/kotlinlang-formalization/README.md");
  const prompt = read("docs/case-studies/issue-1163/self-use/kotlinlang-compile/prompt.txt").trim();
  assert.ok(agentRun.includes(`Prompt: \`${prompt}\``), "the Agent CLI run used the self-use prompt");
  assert.match(agentRun, /@link-assistant\/agent` \d+\.\d+\.\d+/);
  assert.match(agentRun, /Final answer, verbatim:/);
  for (const file of ["answer.txt", "thinking-steps.txt", "formalizer-run.json", "run.env"]) {
    assert.ok(existsSync(path.join(REPO_ROOT, "docs/case-studies/issue-1163/self-use/kotlinlang-compile", file)), file);
  }
});

test("R1163-15: the self-use formalizer run reproduces from the committed capture", async () => {
  const run = JSON.parse(read("docs/case-studies/issue-1163/self-use/kotlinlang-compile/formalizer-run.json"));
  const env = read("docs/case-studies/issue-1163/self-use/kotlinlang-compile/run.env");
  const capture = env.match(/^committed_capture=(\S+)/m)[1];
  assert.equal(sha256(capture), run.sha256, "the committed capture is the page the run formalized");
  assert.ok(env.includes(`formalizer_sha256=${run.sha256}`));
  const page = await call(`(() => {
    const page = formalizedPageFromCapture({ url: ${literal(run.url)}, sha256: ${literal(run.sha256)}, fetchedAt: "", cached: false, text: ${literal(read(capture))} }, "", 1, 0, "text/html");
    return { blockCount: page.network.blocks.length, codeBlocks: page.codeBlocks.map((block) => ({ language: block.language, text: block.text.trim() })) };
  })()`);
  assert.equal(page.blockCount, run.blockCount);
  assert.deepEqual(page.codeBlocks.map((block) => `${block.language} | ${block.text.split("\n")[0]}`), run.codeBlocks);
  const blockTexts = page.codeBlocks.map((block) => block.text);
  for (const command of [...run.compile, ...run.run]) {
    assert.ok(blockTexts.includes(command), `the recorded command ${command} is a code block of the page`);
  }
  assert.ok(run.compile.some((command) => command.startsWith("kotlinc ")), "the run found the kotlinc compile command");
});

test("R1164-14: the decomposed example and generalized procedure reproduce from the captures", async () => {
  const recorded = read("docs/case-studies/issue-1164/decomposed-example/hello-world-run.lino");
  const generalizedAt = recorded.indexOf("\n# generalized\n");
  assert.ok(generalizedAt > 0, "the run records the generalized procedure");
  const sections = recorded.slice(0, generalizedAt).split(/\n(?=# )/)
    .map((section) => section.match(/^# (\S+) sha256=([0-9a-f]{64})\n([\s\S]*)$/))
    .filter(Boolean);
  assert.ok(sections.length >= 1, "at least one decomposed example");
  const nodes = [];
  for (const [, file, hash, notation] of sections) {
    const relative = `${CAPTURED}/${file}`;
    assert.equal(sha256(relative), hash, `${file} is the committed capture`);
    const language = notation.match(/^ {2}language_slug (\S+)$/m)[1];
    const url = notation.match(/source_url "([^"]+)"/)[1];
    const mime = file.endsWith(".md") ? "text/markdown" : "text/html";
    // The first block in the page's language (or untagged) with an output call.
    const node = await call(`(() => {
      const blocks = formalizePage(${literal(read(relative))}, ${literal(mime)}, ${literal(url)}).blocks
        .filter((block) => block.kind === "code_block" && (!block.language || block.language === "unknown" || block.language === ${literal(language)}));
      for (const block of blocks) {
        const result = decomposeCodeExample(block.text, ${literal(language)}, [], ${literal(url)});
        if (result.ok && result.ok.parts.some((part) => part.kind === "output_operation")) {
          return { node: result.ok, notation: decomposedCodeExampleNotation(result.ok) };
        }
      }
      return null;
    })()`);
    assert.ok(node, `${file} decomposes`);
    assert.equal(node.notation.trim(), notation.trim(), `${file}: the recorded decomposed node is the engine's`);
    nodes.push(node.node);
  }
  const generalized = await call(`generalizedCodeExampleNotation(generalizeCodeExamples(${literal(nodes)}))`);
  const withoutBodies = (text) => text.trim().split("\n").filter((line) => !/^ {4}source "/.test(line)).join("\n");
  const recordedGeneralized = recorded.slice(generalizedAt + "\n# generalized\n".length);
  assert.equal(withoutBodies(String(generalized)), withoutBodies(recordedGeneralized));
  assert.match(recordedGeneralized, /^generalized_procedure$/m);
});
