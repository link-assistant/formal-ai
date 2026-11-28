import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";
import { startCli } from "./support/structured-log-fixture.mjs";

const worker = createWorkerContext();
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await ready;
  return worker.solve(prompt, [], {}, {}, [], {});
}

test("the runtime seed exposes resolution families without benchmark cases", async () => {
  await ready;
  assert.equal(evaluate(worker, "trizFamilies().length"), 12);
  assert.equal(evaluate(worker, "seedRawText(SEED_RAW, 'triz-principles.lino').includes('triz_benchmark_task')"), false);
});

test("a Russian contradiction produces a localized worksheet", async () => {
  const answer = await solve("Как разрешить противоречие: деталь должна быть жёсткой и одновременно гибкой?");
  assert.equal(answer.intent, "triz_resolution");
  assert.match(answer.content, /Конфликтующие требования.*деталь должна быть жёсткой \/ гибкой/u);
  assert.match(answer.content, /Range selection/u);
  assert.match(answer.content, /Dimension change/u);
});

test("unseen demands produce their own worksheet without recalled cases", async () => {
  for (const [prompt, expected] of [
    ["Apply TRIZ to latency vs acoustic damping", "latency / acoustic damping"],
    ["Un dilema, contradicción: latencia frente a fidelidad", "latencia / fidelidad"],
    ["TRIZ: कम विलंब बनाम अधिक सटीकता", "कम विलंब / अधिक सटीकता"],
    ["矛盾：延迟与隔音", "延迟 / 隔音"],
    ["TRIZ: A {families} vs B {left}", "A {families} / B {left}"],
    ["TRIZ: ratio 1:2 vs equal allocation", "ratio 1:2 / equal allocation"],
  ]) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "triz_resolution", prompt);
    assert.ok(answer.content.includes(expected), answer.content);
    assert.ok(answer.evidence.includes("triz_solver:demands:" + expected), answer.evidence.join(", "));
    assert.doesNotMatch(answer.content, /pills|umbrella|wheat grains/u);
  }
});

test("all twenty benchmark cases are explicit parser inputs", async () => {
  await ready;
  const corpus = readFileSync(new URL("../fixtures/triz/benchmark-tasks.lino", import.meta.url), "utf8");
  const demands = Array.from(corpus.matchAll(/^  contradiction "([^"\n]+)"$/gmu), ([, value]) => value);
  assert.equal(demands.length, 20);
  for (const text of demands) {
    const extracted = evaluate(worker, `trizDemands(${JSON.stringify(`TRIZ: ${text}`)})`);
    assert.deepEqual(Array.from(extracted ?? []), text.split(" vs "), text);
  }
});

test("missing or repeated demands produce a routed request for details", async () => {
  for (const prompt of ["Explain a paradox: zoetrope", "TRIZ: speed vs speed"]) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "triz_resolution");
    assert.match(answer.content, /State the two conflicting demands/u);
    assert.doesNotMatch(answer.content, /Range selection|Precedent/u);
    assert.ok(answer.evidence.includes("triz_resolution:refusal:two demands required"));
  }
});

test("ordinary prompts decline", async () => {
  await ready;
  for (const prompt of ["What is the capital of France?", "Напомни мне встречу в 20:00"]) {
    assert.equal(evaluate(worker, `tryTrizResolution(${JSON.stringify(prompt)}, normalizePrompt(${JSON.stringify(prompt)}))`), null);
  }
});

const nativeCli = process.env.FORMAL_AI_TEST_NATIVE_CLI;
test("the native public gateway derives the same demands and asks when they are missing", { skip: !nativeCli, timeout: 30_000 }, async (t) => {
  const run = await startCli(t, { binary: nativeCli, silent: true, lifetime: 30_000, env: { FORMAL_AI_OFFLINE: "1" } });
  await run.ready();
  for (const [prompt, expected] of [
    ["Apply TRIZ to latency vs acoustic damping", "latency / acoustic damping"],
    ["TRIZ: ratio 1:2 vs equal allocation", "ratio 1:2 / equal allocation"],
    ["Explain a paradox: zoetrope", "State the two conflicting demands"],
    ["矛盾：延迟与隔音", "延迟 / 隔音"],
  ]) {
    const response = await fetch(`${run.url}/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      signal: t.signal,
      body: JSON.stringify({ model: "formal-ai", messages: [{ role: "user", content: prompt }] }),
    });
    assert.ok(response.ok, await response.clone().text());
    const answer = (await response.json()).choices[0].message.content;
    assert.ok(answer.includes(expected), answer);
    assert.doesNotMatch(answer, /pills|umbrella|wheat grains/u);
  }
});
