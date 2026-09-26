// Issue #1138 plans 06/08: browser and Telegram execution claims are driven
// by shared corpora, and an unavailable browser executor is an explicit
// unverified result rather than a value guessed by a JavaScript mirror.

import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";

import { createWorkerContext, evaluate, plain, REPO_ROOT } from "./support/browser-runtime.mjs";

function parseFixture(context, relativePath) {
  const text = readFileSync(path.join(REPO_ROOT, relativePath), "utf8");
  context.__fixture = text;
  return plain(evaluate(context, "FormalAiSeed.parse(__fixture)"));
}

function children(node, name) {
  return (node?.children || []).filter((child) => child.name === name);
}

function field(node, name) {
  return children(node, name)[0]?.value || "";
}

function verifiablePrompts(context) {
  const suite = parseFixture(context, "data/benchmarks/verifiable-task-paraphrases.lino");
  const cases = (suite.children || []).filter(
    (candidate) => field(candidate, "record_type") === "benchmark_case_family",
  );
  return cases.flatMap((family) => children(family, "paraphrase").map((paraphrase) => ({
    family: field(family, "id"),
    language: field(paraphrase, "language"),
    prompt: field(paraphrase, "prompt"),
  })));
}

test("browser projects every held-out verifiable task and stays explicitly unverified", async () => {
  const context = createWorkerContext();
  await evaluate(context, "loadSeed()");
  const prompts = verifiablePrompts(context);
  assert.equal(prompts.length, 30, "the canonical corpus remains six families by five languages");

  for (const fixture of prompts) {
    context.__prompt = fixture.prompt;
    const answer = plain(await evaluate(context, "solve(__prompt, [], {}, {}, [])"));
    assert.equal(answer.intent, "verifiable_task", `${fixture.family}/${fixture.language}`);
    assert.equal(
      answer.diagnostics?.verifiableTask?.status,
      "unverified",
      `${fixture.family}/${fixture.language}: browser execution absence is explicit`,
    );
    assert.equal(
      answer.diagnostics?.verifiableTask?.language,
      fixture.language,
      `${fixture.family}/${fixture.language}: recognition keeps the corpus language`,
    );
    assert.ok(
      answer.evidence.some((item) => item.startsWith("verifiable_task:recognised:")),
      `${fixture.family}/${fixture.language}: recognition evidence is retained`,
    );
    assert.ok(
      answer.evidence.includes("verifiable_task:unverified:browser_execution_unavailable"),
      `${fixture.family}/${fixture.language}: no computed value is presented as observed`,
    );
    assert.ok(
      !answer.evidence.some((item) => item.startsWith("verifiable_task:executed:")),
      `${fixture.family}/${fixture.language}: the browser did not execute a derivation`,
    );
  }
});

test("the Telegram refused/granted table is complete and points at shared response intents", async () => {
  const context = createWorkerContext();
  await evaluate(context, "loadSeed()");
  const table = parseFixture(context, "data/benchmarks/telegram-execution-outcomes.lino");
  const outcomes = children(table, "language_outcome");
  assert.equal(outcomes.length, Number(field(table, "language_count")));

  const languages = new Set();
  for (const outcome of outcomes) {
    const language = field(outcome, "language");
    languages.add(language);
    const refused = children(outcome, "refused")[0];
    const granted = children(outcome, "granted")[0];
    assert.equal(field(refused, "status"), "refused", language);
    assert.equal(field(granted, "status"), "observed", language);
    assert.equal(field(refused, "response_intent"), "code_execution_refused", language);
    assert.equal(field(granted, "response_intent"), "code_execution_observed", language);
    assert.equal(field(granted, "output"), "55", language);
    assert.equal(field(granted, "evidence_kind"), "command_exit", language);
    assert.ok(evaluate(context, `answerFor("code_execution_refused", ${JSON.stringify(language)})`));
    assert.ok(evaluate(context, `answerFor("code_execution_observed", ${JSON.stringify(language)})`));
  }
  assert.equal(languages.size, outcomes.length, "one outcome row per registered test language");
});
