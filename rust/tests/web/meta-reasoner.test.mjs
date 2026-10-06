// The recursive meta reasoner (js/worker/formal_ai_worker_meta_reasoner.js,
// data/seed/meta-reasoning.lino). Every prompt here is absent from the code
// and the seed: the answers are derived — grounded, enumerated, verified —
// never looked up. Dictionary knowledge comes only from real captures under
// rust/tests/fixtures/meta-reasoner (sha256 in capture-manifest.lino).
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");
const FIXTURES = path.join(REPO_ROOT, "rust/tests/fixtures/meta-reasoner");

/** The committed captures as `url → body`, each checked against the manifest. */
function captures() {
  const manifest = readFileSync(path.join(FIXTURES, "capture-manifest.lino"), "utf8");
  const out = new Map();
  for (const block of manifest.split("\n  capture ").slice(1)) {
    const word = block.split("\n")[0].trim();
    const url = /url "([^"]+)"/u.exec(block)[1];
    const sha256 = /sha256 ([0-9a-f]{64})/u.exec(block)[1];
    const body = readFileSync(path.join(FIXTURES, "captures", `${word}.json`), "utf8");
    assert.equal(createHash("sha256").update(body).digest("hex"), sha256, `${word} capture is unedited`);
    out.set(url, body);
  }
  return out;
}

const replay = captures();
const worker = createWorkerContext({
  fetch: async (url) => {
    const target = String(url);
    if (!target.startsWith("http") || target.startsWith("http://localhost/")) {
      // Web-root assets: scripts from js/, seed files from data/seed/.
      const relative = new URL(target, "http://localhost/").pathname.replace(/^\/+/u, "");
      const onDisk = relative.startsWith("seed/") ? path.join(REPO_ROOT, "data", relative) : path.join(REPO_ROOT, "js", relative);
      try {
        const text = readFileSync(onDisk, "utf8");
        return { ok: true, status: 200, text: async () => text };
      } catch {
        return { ok: false, status: 404, text: async () => "" };
      }
    }
    const body = replay.get(target);
    if (body !== undefined) return { ok: true, status: 200, text: async () => body };
    // The repository's pre-cached Wiktionary captures (data/cache/wiktionary).
    const cached = /^https:\/\/api\.dictionaryapi\.dev\/api\/v2\/entries\/en\/([a-z]+)$/u.exec(target);
    const file = cached && path.join(REPO_ROOT, "data/cache/wiktionary/en", `${cached[1]}.json`);
    if (file && existsSync(file)) {
      const text = readFileSync(file, "utf8");
      return { ok: true, status: 200, text: async () => text };
    }
    return { ok: false, status: 404, text: async () => "" };
  },
});
const ready = evaluate(worker, "loadSeed()");

async function reason(prompt) {
  await ready;
  return plain(await evaluate(
    worker,
    `metaReasonTurn(${JSON.stringify(prompt)}, "en", {}).then((r) => ({
      goal: r.goal, status: r.status, needs: r.needs, lookups: r.lookups,
      program: r.program, verification: r.verification, probe: r.probe || null,
      unknowns: r.unknowns, events: r.trace.events.map((e) => e.kind + ": " + e.detail),
      lino: r.derivationLino }))`,
  ));
}

async function solve(prompt) {
  await ready;
  return worker.solve(prompt, [], {}, {}, [], {});
}

test("examples alone are a goal state: the shortest typed program with difference 0 is found and verified", async () => {
  const result = await reason("'ab cd' -> 'ba dc', 'hello' -> 'olleh'");
  assert.equal(result.goal, "synthesize_from_examples");
  assert.equal(result.status, "solved");
  assert.deepEqual(result.program.steps, ["split_words", "each(reverse_text)", "join_words"]);
  assert.deepEqual([result.verification.passed, result.verification.total], [2, 2]);
  assert.ok(result.events.some((event) => event.startsWith("search: length 3")), result.events.join("\n"));
});

test("a parameter is inferred from the example, not memorized", async () => {
  const result = await reason("[1, 2, 3] -> [3, 6, 9]");
  assert.deepEqual(result.program.steps, ["each(multiply_by)"]);
  assert.equal(result.program.parameter, 3);
  assert.match(result.program.source, /input \* 3/u);
});

test("a nonce word the request defines breaks the tie the examples leave", async () => {
  const bare = await reason("2 -> 4");
  assert.ok(bare.events.some((event) => event.startsWith("tie:")), bare.events.join("\n"));
  const defined = await reason("A zorp doubles a number. Write a zorp: 2 -> 4");
  assert.ok(defined.events.some((event) => event.includes("zorp") === false && event.includes("doubles → multiply_by")), defined.events.join("\n"));
  assert.deepEqual(defined.program.steps, ["multiply_by"]);
});

test("an unknown word is looked up, its gloss grounded one level deeper, and the result learned", async () => {
  const first = await reason("write a function that turns each word backwards");
  assert.match(first.lookups.join(";"), /^backwards:\d+$/u, first.events.join("\n"));
  assert.ok(first.events.some((event) => event.startsWith("subgoal: ↳") || event.startsWith("subgoal: understand backwards")), first.events.join("\n"));
  assert.ok(first.events.some((event) => /grounded: backwards → reverse_(text|list) via https:\/\/api\.dictionaryapi\.dev/u.test(event)), first.events.join("\n"));
  assert.deepEqual(first.program.steps, ["split_words", "each(reverse_text)", "join_words"]);
  assert.equal(first.probe.output, "olleh dlrow");
  assert.ok(first.events.some((event) => event.startsWith("chunk: learned backwards")), first.events.join("\n"));
  // The learned chunk answers the paraphrase with no lookup at all.
  const again = await reason("make a function that spins every word backwards");
  assert.deepEqual(again.lookups.filter((round) => round.includes("backwards")), []);
  assert.ok(again.events.some((event) => event.startsWith("recall: backwards → reverse_")), again.events.join("\n"));
});

test("a word no source reaches stays an open unknown and the answer asks the deciding question", async () => {
  const answer = await solve("Write a function that counts the qwzx of a text");
  assert.equal(answer.intent, "meta_reasoned_program_unverified", answer.content);
  assert.match(answer.content, /Still unknown: qwzx/u);
  assert.match(answer.content, /one input and the output you expect/u);
});

test("a specialized handler's admitted impasse is resolved by the general loop", async () => {
  const answer = await solve("Write a JavaScript function that counts the lines of a text");
  assert.equal(answer.intent, "meta_reasoned_program_unverified", answer.content);
  assert.match(answer.content, /split_lines/u);
  const steps = answer.steps.map((step) => step.step);
  assert.ok(steps.includes("meta_impasse"), steps.join(","));
  assert.equal(steps[steps.length - 1], "deformalize");
});

test("every answer carries its derivation in links notation", async () => {
  for (const prompt of ["2 + 2", "hello", "'x y' -> 'y x'"]) {
    const answer = await solve(prompt);
    assert.match(String(answer.derivation), /^derivation\n {2}goal /u, prompt);
    assert.match(String(answer.derivation), /\n {2}event 1\n {4}kind impulse/u, prompt);
  }
  const answer = await solve("'x y' -> 'y x'");
  assert.equal(answer.intent, "meta_reasoned_program");
  assert.match(answer.content, /function solution\(input\)/u);
});

test("the same request yields the same derivation", async () => {
  const one = await reason("[3, 1, 2] -> [1, 2, 3]");
  const two = await reason("[3, 1, 2] -> [1, 2, 3]");
  assert.equal(one.lino, two.lino);
});

test("every rung of the task ladder is derived, never handled", async () => {
  const ladder = readFileSync(path.join(FIXTURES, "ladder.lino"), "utf8");
  const rungs = [...ladder.matchAll(/^ {2}rung "((?:[^"\\]|\\.)*)"\n {4}steps "([^"]*)"/gmu)];
  assert.ok(rungs.length >= 10, `${rungs.length} rungs`);
  for (const [, quoted, steps] of rungs) {
    const prompt = JSON.parse(`"${quoted}"`);
    const answer = await solve(prompt);
    assert.match(answer.intent, /^meta_reasoned_program/u, `${prompt}: ${answer.intent}`);
    const derived = /\n {2}program "([^"]*)"/u.exec(answer.derivation);
    assert.equal(derived && derived[1].split(" ∘ ").join(" "), steps, prompt);
  }
});

test("a derived file program runs in node and agrees with a direct count", async () => {
  const { mkdtempSync, writeFileSync, rmSync } = await import("node:fs");
  const { tmpdir } = await import("node:os");
  const { createRequire } = await import("node:module");
  const folder = mkdtempSync(path.join(tmpdir(), "meta-ladder-"));
  try {
    writeFileSync(path.join(folder, "long.txt"), "x\n".repeat(5));
    writeFileSync(path.join(folder, "short.txt"), "x\n");
    const answer = await solve("Write a Node.js script that prints every file in a folder with more than 3 lines");
    const source = /```javascript\n([\s\S]*?)```/u.exec(answer.content)[1];
    // eslint-disable-next-line no-new-func -- running the derived answer is the check.
    const solution = new Function("require", `${source}\nreturn solution;`)(createRequire(import.meta.url));
    assert.deepEqual(solution(folder), [path.join(folder, "long.txt")]);
  } finally {
    rmSync(folder, { recursive: true, force: true });
  }
});
