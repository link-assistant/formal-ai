// Unit coverage for the browser solver worker (issue #895).
//
// `src/web/formal_ai_worker.js` plus the `src/web/worker/*.js` mirror is the
// production reasoning engine the website runs; it is the largest body of
// browser code in the repository. `createWorkerContext` boots it from those
// sources with the real `data/seed/*.lino` corpus behind `fetch`, so these
// assertions describe the answers the deployed site actually produces.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  createWorkerContext,
  loadWorkerMirror,
  plain,
  REPO_ROOT,
  workerMirrorFiles,
} from "./support/browser-runtime.mjs";

/** One booted worker shared by the read-only assertions below. */
const worker = createWorkerContext();

function solve(prompt, options = {}) {
  return worker.solve(prompt, options.history ?? [], options.preferences ?? {}, options.userContext ?? {}, options.memory ?? [], options.options ?? {});
}

function assertSharedConstructionEvidence(answer, surface) {
  assert.ok(
    answer.evidence.includes(
      "algorithm_construction:meta_algorithm:problem_class_to_shared_ir_to_renderers_to_verification",
    ),
    `${surface} executes the shared meta-algorithm`,
  );
  assert.ok(
    answer.evidence.includes(`algorithm_construction:active_surface:${surface}`),
    `${surface} is the active construction surface`,
  );
  assert.equal(
    answer.evidence.filter((entry) => entry.startsWith("algorithm_construction:stage:")).length,
    7,
    `${surface} executes the common seven-stage trace exactly once`,
  );
}

test("the committed module list is exactly the worker mirror directory", () => {
  const files = workerMirrorFiles();
  assert.ok(files.length > 0, "the mirror has modules");

  // The browser cannot read a directory: `formal_ai_worker.js` imports
  // `worker-modules.js` and loads whatever that list names. The Node harness
  // reads the directory instead, so comparing the two is what proves the
  // deployed site loads every module that exists. Issue #991 made that list a
  // union-merged file, and a union is a superset: it can hold a module twice,
  // or hold one a branch deleted, until
  // `rust-script scripts/normalize-ordered-lists.rs --write` regenerates it.
  // This assertion is the failure that says so.
  const listed = readFileSync(path.join(REPO_ROOT, "src/web/worker-modules.js"), "utf8")
    .split("\n")
    .map((line) => line.match(/^\s*"(worker\/[^"]+)",$/))
    .filter(Boolean)
    .map((match) => `src/web/${match[1]}`);

  assert.deepEqual(
    listed,
    files,
    "src/web/worker-modules.js is stale: run `rust-script scripts/normalize-ordered-lists.rs --write`",
  );
});

test("worker modules are named after their subject, and the numbered prefix stays frozen", () => {
  // Issue #991: `formal_ai_worker_25.js` is the file two branches both create
  // when they each add a module, and git reports that as an add/add conflict no
  // ordering can prevent. `data/meta/merge-conflict-policy.lino` freezes the
  // numbers that predate the policy and requires every later module to be named
  // after what it contains, so two branches produce two different names.
  const FROZEN_PREFIX_MAX = 24;
  const numbered = workerMirrorFiles()
    .map((file) => file.match(/formal_ai_worker_(\d+)\.js$/))
    .filter(Boolean)
    .map((match) => Number(match[1]));

  assert.deepEqual(
    numbered,
    Array.from({ length: FROZEN_PREFIX_MAX + 1 }, (_, index) => index),
    `the frozen numbered modules are 00..${FROZEN_PREFIX_MAX}; name a new module after its subject instead`,
  );
});

test("every coding handler executes the shared meta-algorithm", async () => {
  const installation = await solve(
    "Convert this README installation guide into a sh script: run `npm install`.",
  );
  assert.equal(installation.intent, "installation_conversion");
  assertSharedConstructionEvidence(installation, "installation_conversion");

  const synthesis = await solve(
    "Implement Python function count_vowels(text: str) -> int. Return the number of vowels in the text.",
  );
  assert.equal(synthesis.intent, "write_program");
  assertSharedConstructionEvidence(synthesis, "program_synthesis");

  const catalog = await solve("Write hello world in Rust");
  assert.equal(catalog.intent, "write_program");
  assertSharedConstructionEvidence(catalog, "coding_catalog");

  const numeric = await solve(
    "I have numbers 5, 3, 8, 1, 9 — sort them in JavaScript, give me the code and the result",
  );
  assert.equal(numeric.intent, "write_program");
  assertSharedConstructionEvidence(numeric, "numeric_list");

  const firstPrompt = "Write me a Rust program that lists files in the current directory";
  const first = await solve(firstPrompt);
  const rule = await solve("Sort the results in reverse order", {
    history: [
      { role: "user", content: firstPrompt },
      { role: "assistant", content: first.content },
    ],
  });
  assert.equal(rule.intent, "write_program");
  assertSharedConstructionEvidence(rule, "rule_synthesis");
});

test("prompt normalization collapses whitespace and case", () => {
  const context = loadWorkerMirror();
  assert.equal(context.normalizePrompt("  Hello   World  "), "hello world");
  assert.equal(context.normalizePrompt(""), "");
});

test("language detection recognises each supported script", () => {
  const context = loadWorkerMirror();
  assert.equal(context.detectLanguage("привет как дела"), "ru");
  assert.equal(context.detectLanguage("hello how are you"), "en");
  assert.equal(context.detectLanguage("你好吗"), "zh");
  assert.equal(context.detectLanguage("नमस्ते कैसे हैं आप"), "hi");
});

test("arithmetic is answered from the calculation handler", async () => {
  const answer = await solve("2 + 2");
  assert.equal(answer.intent, "calculation");
  assert.equal(answer.content, "2 + 2 = 4");
  assert.equal(answer.confidence, 1);
  assert.ok(
    answer.evidence.includes("calculation:2 + 2 = 4"),
    "the evidence names the computed result rather than asserting it out of band",
  );

  const product = await solve("12 * 12");
  assert.equal(product.intent, "calculation");
  assert.match(product.content, /144/);
});

test("greetings are answered in the language they were written in", async () => {
  const cases = [
    ["hello", "en", /[A-Za-z]/],
    ["привет", "ru", /[А-Яа-я]/],
    ["你好", "zh", /[一-鿿]/],
  ];

  for (const [prompt, language, script] of cases) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "greeting", `${prompt} is a greeting`);
    assert.match(
      answer.content,
      script,
      `the ${language} greeting is answered in the ${language} script`,
    );
    assert.ok(
      answer.evidence.includes(`trace:language:${language}`),
      `the trace records the detected language ${language}`,
    );
  }
});

test("identity questions answer from the seeded agent info", async () => {
  const answer = await solve("who are you");
  assert.equal(answer.intent, "identity");
  assert.match(answer.content, /formal-ai/i, "the agent names itself from the seed corpus");
});

test("an unknown prompt says so instead of inventing an answer", async () => {
  const answer = await solve("qwertyuiop asdfghjkl zxcvbnm");
  assert.equal(answer.intent, "unknown");
  assert.ok(answer.confidence < 1, "an unknown answer is not reported as certain");
});

// Issue #180: every turn must end with a `deformalize` projection so the answer
// the user sees is always traceable back to the formalized impulse.
test("every answer carries a full impulse → formalize → deformalize trace", async () => {
  for (const prompt of ["2 + 2", "привет", "who are you", "qwertyuiop asdfghjkl"]) {
    const answer = await solve(prompt);
    const steps = plain(answer.steps).map((step) => step.step);

    assert.equal(steps[0], "impulse", `${prompt}: the trace starts at the impulse`);
    assert.ok(steps.includes("formalize"), `${prompt}: the impulse is formalized`);
    assert.equal(
      steps[steps.length - 1],
      "deformalize",
      `${prompt}: the trace ends by projecting the answer back out of the formalization`,
    );
    assert.ok(
      answer.evidence.some((entry) => entry.startsWith("trace:deformalize:")),
      `${prompt}: the deformalization is recorded as evidence`,
    );
  }
});

test("the formalization records the subject, verb and object it derived", async () => {
  const answer = await solve("2 + 2");
  const formalize = plain(answer.steps).find((step) => step.step === "formalize");

  assert.ok(formalize.formalization, "the formalize step carries the structured tuple");
  assert.equal(formalize.formalization.raw, "2 + 2");
  assert.equal(formalize.formalization.subject, "@USER");
  assert.match(
    formalize.formalization.tuple,
    /^\(@USER /,
    "the tuple is rendered in the same notation the Rust solver emits",
  );
});

test("at temperature 0 the same prompt always projects the same answer", async () => {
  // The engine is a projection of an append-only log, not a sampler. Above
  // temperature 0 it deliberately rotates between seeded phrasings; with the
  // rotation switched off the projection must be bit-for-bit reproducible.
  const cold = { temperature: 0 };
  for (const prompt of ["2 + 2", "hello", "who are you"]) {
    const first = plain(await solve(prompt, { preferences: cold }));
    const second = plain(await solve(prompt, { preferences: cold }));
    assert.deepEqual(second, first, `${prompt} is deterministic at temperature 0`);
  }
});

test("above temperature 0 a repeated greeting stays a greeting", async () => {
  // Variation is intentional and bounded: the intent never changes and every
  // phrasing is non-empty, so the rotation cannot degrade the answer.
  const context = createWorkerContext();
  const seen = new Set();
  for (let turn = 0; turn < 6; turn += 1) {
    const answer = await context.solve("hello", [], { temperature: 1 }, {}, [], {});
    assert.equal(answer.intent, "greeting", "every turn is still a greeting");
    assert.ok(answer.content.trim().length > 0, "no turn produces an empty answer");
    seen.add(answer.content);
  }
  assert.ok(seen.size >= 1, "the rotation yields at least one seeded phrasing");
});

test("the worker exposes its handlers through a single dispatch step", async () => {
  const answer = await solve("2 + 2");
  const dispatches = plain(answer.steps).filter((step) => step.step === "dispatch_handler");
  assert.equal(dispatches.length, 1, "exactly one handler claims the prompt");
  assert.ok(dispatches[0].detail, "the trace names the handler that claimed it");
});

test("the seed corpus is hydrated from the shipped .lino files", async () => {
  // With no seed data the worker still answers, but in English only. A localized
  // answer proves the real `data/seed/*.lino` corpus was loaded through fetch.
  const answer = await solve("привет");
  assert.match(
    answer.content,
    /[А-Яа-я]/,
    "the Russian response came from the seed corpus rather than a hardcoded default",
  );
});

test("the web-search component falls back when its bundle is unavailable", () => {
  // `formal_ai_worker.js` installs a bounded local implementation if the
  // optional component bundle is missing; a downstream embed depends on it.
  const context = createWorkerContext({
    importScripts: undefined,
  });
  assert.ok(context.FormalAIWebSearchComponent, "a component is always present");
});

test("issue 989 dialog controls and associative-memory inspection preempt generic routes", async () => {
  const preference = await solve("`quick` is subjective opinion, please don't use these anymore.");
  assert.equal(preference.intent, "conversation_preference");
  assert.match(preference.content, /avoid `quick`/);

  for (const teaching of ["When I say # answer with 42.", "When I say # answer 42."]) {
    const replay = await solve("#", {
      history: [
        { role: "user", content: teaching },
        { role: "assistant", content: "Behavior rule recorded for this dialog." },
      ],
    });
    assert.equal(replay.intent, "behavior_rule_custom", teaching);
    assert.equal(replay.content, "42", teaching);
  }

  const memoryEvents = [{
    id: "event-1",
    kind: "message",
    role: "user",
    content: "hello",
    conversationId: "dialog-1",
  }];
  const options = { options: { memoryEvents } };
  const count = await solve("How many links are in your memory?", options);
  assert.equal(count.intent, "memory_link_count");
  assert.match(count.content, /records: 1; projected links: 17/);

  const inventory = await solve("What is available in your local memory?", options);
  assert.equal(inventory.intent, "memory_inventory");
  assert.match(inventory.content, /kinds: message: 1/);
  assert.match(inventory.content, /conversations: dialog-1: 1/);

  const roots = await solve("Give me root links you have in your memory", options);
  assert.equal(roots.intent, "memory_root_links");
  assert.match(roots.content, /memory_event_[0-9a-f]{8}/);
  assert.match(roots.content, /event-1/);

  const correction = await solve("No that is not about document generation, it is about associative memory data retrieval.", {
    ...options,
    history: [{ role: "user", content: "Give me root links you have in your memory" }],
  });
  assert.equal(correction.intent, "memory_root_links");
});
