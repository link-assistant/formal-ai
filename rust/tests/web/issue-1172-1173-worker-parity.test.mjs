// Browser-worker twins of issue #1172 (word-boundary subject matching, R10)
// and issue #1173 (no canned search description as an answer, R5).
//
// The Rust root pins both in rust/tests/unit/web-engine-core/issue_1172_factual_qa_subject_match.rs
// and rust/tests/unit/issue_1173_fallback_executes_search.rs; these cases hold
// the worker to the same rule.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");

const worker = createWorkerContext({
  fetch: async (url) => {
    const target = String(url);
    if (!target.startsWith("http") || target.startsWith("http://localhost/")) {
      const relative = new URL(target, "http://localhost/").pathname.replace(/^\/+/u, "");
      const onDisk = relative.startsWith("seed/") ? path.join(REPO_ROOT, "data", relative) : path.join(REPO_ROOT, "js", relative);
      try {
        const text = readFileSync(onDisk, "utf8");
        return { ok: true, status: 200, text: async () => text };
      } catch {
        return { ok: false, status: 404, text: async () => "" };
      }
    }
    // Offline: every external provider is unreachable.
    return { ok: false, status: 404, text: async () => "" };
  },
});
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await ready;
  return plain(await worker.solve(prompt, [], {}, {}, [], {}));
}

test("#1172 R10: the alias \"us\" does not fire inside \"australia\"", async () => {
  const answer = await solve("What is the capital of Australia?");
  assert.doesNotMatch(String(answer.content), /Washington/u, JSON.stringify(answer));
});

test("#1172 R10: a seeded alias still answers as a whole word", async () => {
  const answer = await solve("What is the capital of the USA?");
  assert.match(String(answer.content), /Washington/u, JSON.stringify(answer));
});

test("#1173 R5: a canned search description degrades to web_search_unavailable", async () => {
  await ready;
  const guarded = plain(await evaluate(
    worker,
    `guardCannedWebSearchAnswer({ intent: "web_search", content: "Web search requested for: tomato", query: "tomato", evidence: [] }, "en")`,
  ));
  assert.equal(guarded.intent, "web_search");
  assert.doesNotMatch(guarded.content, /Web search requested/u);
  assert.match(guarded.content, /tomato/u);
  assert.ok(guarded.evidence.includes("web_search:canned_description_refused"), JSON.stringify(guarded));
});

test("#1173 R5: an offline research prompt never answers with the canned description", async () => {
  const answer = await solve("Search the web for the history of the zorblax festival");
  assert.doesNotMatch(String(answer.content), /Web search requested|Providers considered/u, JSON.stringify(answer));
});
