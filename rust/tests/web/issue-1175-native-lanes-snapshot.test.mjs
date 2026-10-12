// Issue #1175 / PR #1188: the browser twins of two native-only branches.
//
// * how_it_works, bare form: "how does it work?" reads its topic from the
//   previous assistant reply (try_how_it_works in rust/src/solver_handler_how.rs)
//   — a known topic answers through the concept lookup, an unrecorded one with
//   the seeded `how_it_works_prior_topic` response, and no prior reply with the
//   seeded `how_it_works_no_context` explanation.
// * network_query, snapshot branch: "show me the network" renders the network
//   the browser has loaded as Links Notation records in the shape of
//   knowledge_links_notation (rust/src/engine.rs), claiming only records the
//   browser holds.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { createWorkerContext, evaluate, plain, REPO_ROOT } from "./support/browser-runtime.mjs";

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
    return { ok: false, status: 404, text: async () => "" };
  },
});

let loaded = null;
async function solve(prompt, history = []) {
  loaded ??= evaluate(worker, "loadSeed()");
  await loaded;
  return plain(await worker.solve(prompt, history, {}, {}, [], {}));
}

test("a bare how-it-works question with no prior reply explains how to ask", async () => {
  for (const prompt of ["how it works?", "как это работает?"]) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "meta_explanation", prompt);
    assert.equal(answer.content, plain(evaluate(worker, 'answerFor("how_it_works_no_context", "en")')), prompt);
    assert.match(answer.content, /how does Wikipedia work\?/u);
  }
});

test("a how-you-work question stays with the meta explanation, as natively", async () => {
  const answer = await solve("How does it work?");
  assert.equal(answer.intent, "meta_explanation");
  assert.equal(answer.content, plain(evaluate(worker, 'answerFor("meta_explanation", "en")')));
});

test("a bare how-it-works question answers the prior reply's known topic through the concept lookup", async () => {
  const first = await solve("What is Wikipedia?");
  assert.equal(first.intent, "concept_lookup");
  const answer = await solve("how it works?", [
    { role: "user", content: "What is Wikipedia?" },
    { role: "assistant", content: first.content },
  ]);
  assert.equal(answer.intent, "concept_lookup");
  assert.equal(answer.content, first.content);
  assert.ok(answer.evidence.includes("followup:subject:prior_reply:wikipedia"), answer.evidence.join(" | "));
});

test("a bare how-it-works question names an unrecorded prior topic", async () => {
  const answer = await solve("how it works", [
    { role: "user", content: "tell me about the gadget" },
    { role: "assistant", content: "Zorblaxium (gadget): a fictional device." },
  ]);
  assert.equal(answer.intent, "concept_elaboration_missing");
  assert.match(answer.content, /^To explain how zorblaxium works: /u);
  assert.ok(answer.evidence.includes("followup:subject:prior_reply_no_record:zorblaxium"));
});

test("the prior-topic scan skips seeded stop words and short tokens", () => {
  assert.equal(evaluate(worker, 'howItWorksPriorTopic("The answer is Rust, a language.")'), "rust");
  assert.equal(evaluate(worker, 'howItWorksPriorTopic("no capitals here")'), null);
});

test("an inline subject still gets the mechanism discovery plan", async () => {
  const answer = await solve("How does a zorblaxium engine work?");
  assert.equal(answer.intent, "how_it_works");
});

for (const prompt of ["export the network", "show me the current network", "show me the network"]) {
  test(`"${prompt}" renders the browser's loaded network snapshot`, async () => {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "network_snapshot");
    const fence = /```links\n([\s\S]*)\n```/u.exec(answer.content);
    assert.ok(fence, answer.content);
    const snapshot = fence[1];
    assert.match(snapshot, /^formal_ai_knowledge\n  model "formal-ai"\n/u);
    assert.match(snapshot, /\n\nconcept_index\n  greeting "intent: greeting"\n/u);
    const greeting = plain(evaluate(worker, 'answerFor("greeting", "en")'));
    assert.ok(snapshot.includes(`rule_greeting\n  intent "greeting"\n  response_link "response:greeting"\n`), snapshot);
    assert.ok(snapshot.includes(greeting.replace(/\n/gu, "\\n")));
    // The native write_program rule has no seeded response, so the browser
    // does not claim it; every rule it does render is counted.
    assert.ok(!snapshot.includes("rule_write_program"));
    const rules = snapshot.match(/^rule_\w+$/gmu) || [];
    assert.ok(rules.length >= 7);
    assert.match(snapshot, new RegExp(`\\n  rule_count "${rules.length}"\\n`, "u"));
  });
}
