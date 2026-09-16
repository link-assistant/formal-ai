// Issue #1138, plan 04 L14: the browser worker must formalize a requirement to
// the *same* `ConceptGraph::identity()` the Rust path produces from the same
// committed captures, and must mark every extracted procedure step unverified —
// the browser has no runtime, so it may never claim a step was checked.
//
// The fixtures are `tests/fixtures/issue-1138-b4/`, the same tree
// `tests/unit/issue_1138_formalization_depth.rs` replays, and the expectation
// file is written by `examples/issue_1138_formalization_parity.rs`.

import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../..");
const FIXTURE_DIR = path.join(REPO_ROOT, "tests/fixtures/issue-1138-b4");
const CACHE_DIR = path.join(FIXTURE_DIR, "source-cache");
const CORPUS = path.join(REPO_ROOT, "data/benchmarks/formalization-depth-requirements.lino");

/** The held-out requirements, by language, for one family. */
function requirements(family) {
  const text = readFileSync(CORPUS, "utf8");
  const cases = [];
  let current = null;
  for (const line of text.split("\n")) {
    const trimmed = line.trim();
    if (line.startsWith("  paraphrase ")) {
      if (current) cases.push(current);
      current = { family: "", language: "", prompt: "" };
    } else if (current) {
      if (trimmed.startsWith("family ")) current.family = trimmed.slice("family ".length);
      else if (trimmed.startsWith("language ")) current.language = trimmed.slice("language ".length);
      else if (trimmed.startsWith("prompt ")) {
        current.prompt = trimmed.slice("prompt ".length).replace(/^"|"$/gu, "").replace(/""/gu, '"');
      }
    }
  }
  if (current) cases.push(current);
  return cases.filter((one) => one.family === family);
}

function loadCaptures() {
  const captures = new Map();
  for (const name of readdirSync(CACHE_DIR).filter((entry) => entry.endsWith(".meta"))) {
    const meta = readFileSync(path.join(CACHE_DIR, name), "utf8");
    const field = (key) => {
      const line = meta.split("\n").find((entry) => entry.startsWith(`${key}=`));
      return line ? line.slice(key.length + 1) : "";
    };
    const url = field("url");
    const sha256 = field("sha256");
    if (!url || !sha256) continue;
    const body = readFileSync(path.join(CACHE_DIR, "objects", `${sha256}.body`), "utf8");
    captures.set(url, { body, sha256, fetchedAt: field("fetched_at") });
  }
  return captures;
}

async function bootWorker(captures) {
  const context = createWorkerContext({
    fetch: (url) => {
      const target = String(url);
      const relative = target.split("?")[0].replace(/^\.?\//, "");
      if (!target.startsWith("http")) {
        const onDisk = relative.startsWith("seed/")
          ? path.join(REPO_ROOT, "data", relative)
          : path.join(REPO_ROOT, "src/web", relative);
        try {
          const text = readFileSync(onDisk, "utf8");
          return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(text) });
        } catch {
          return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve("") });
        }
      }
      const capture = captures.get(target);
      if (!capture) {
        return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve("") });
      }
      return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(capture.body) });
    },
  });
  await evaluate(context, "loadSeed()");
  return context;
}

async function formalize(context, text) {
  context.__formalizeText = text;
  return plain(await evaluate(context, 'formalizeDeeply(__formalizeText, "doc:requirement")'));
}

test("the browser produces the same concept-graph identity as the native path", async () => {
  const context = await bootWorker(loadCaptures());
  const expected = JSON.parse(readFileSync(path.join(FIXTURE_DIR, "expected-graphs.json"), "utf8"));
  const cases = requirements("isogram_requirement");
  assert.equal(cases.length, 5, "five languages, one held-out family");

  const identities = new Set();
  for (const one of cases) {
    const graph = await formalize(context, one.prompt);
    assert.ok(graph.concepts.length > 0, `${one.language}: nothing was grounded`);
    identities.add(graph.identity);
    assert.ok(
      expected.some((row) => row.identity === graph.identity),
      `${one.language}: the browser identity is not in the cross-runtime expectation`,
    );
  }
  assert.equal(identities.size, 1, "one requirement, five languages, one identity");
});

test("every extracted procedure step the browser shows is unverified", async () => {
  const context = await bootWorker(loadCaptures());
  const [english] = requirements("lipogram_procedure");
  const graph = await formalize(context, english.prompt);

  assert.ok(graph.procedures.length > 0, "the procedure requirement must extract a procedure");
  for (const procedure of graph.procedures) {
    assert.ok(procedure.steps.length >= 2, "fewer than two steps is not a procedure");
    for (const step of procedure.steps) {
      assert.equal(step.verified, false, "the browser has no runtime and may not claim otherwise");
      assert.match(step.sourceSpan, /@\d+:\d+$/u, "a step names the exact span it came from");
    }
  }
  assert.ok(
    graph.needs.every((need) => need.state !== "satisfied" || need.satisfiedBy),
    "a satisfied need always names the evidence that satisfied it",
  );
});
