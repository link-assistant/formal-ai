// Browser-worker drafts of issue #1172 R2 (subject Q-id gate), R6 (seeded
// comparisons) and R7 (questions over prompt-supplied text), JavaScript first.
//
// The native twins are pinned in rust/tests/unit/issue_1172_factual_qa_gate.rs.
// Every case runs offline: an external fetch is counted, and the R7 cases
// assert none happened.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");
const external = [];

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
    external.push(target);
    return { ok: false, status: 404, text: async () => "" };
  },
});
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await ready;
  return plain(await worker.solve(prompt, [], {}, {}, [], {}));
}

async function call(expression) {
  await ready;
  return plain(await evaluate(worker, expression));
}

// ---- R2: the formalized subject's Q-id gates every seeded record ----------

test("#1172 R2: the subject slot resolves to the seeded record's Q-id", async () => {
  assert.equal((await call(`factSubjectGate(normalizePrompt("What is the capital of the USA?"))`)).qid, "Q30");
  assert.equal((await call(`factSubjectGate(normalizePrompt("Какова столица США?"))`)).qid, "Q30");
  assert.equal((await call(`factSubjectGate(normalizePrompt("जापान की राजधानी क्या है?"))`)).qid, "Q17");
  assert.equal((await call(`factSubjectGate(normalizePrompt("美国的首都是什么？"))`)).qid, "Q30");
  // Australia has no seeded record, so its slot resolves to nothing.
  assert.equal((await call(`factSubjectGate(normalizePrompt("What is the capital of Australia?"))`)).qid, "");
});

test("#1172 R2: the longest label wins and a cross-Q-id tie resolves to nothing", async () => {
  // "us" (Q30) is a whole word here, but "japan" is the longer label.
  assert.equal((await call(`resolveFactSubject("tell us the capital of japan")`)).qid, "Q17");
  assert.equal(await call(`resolveFactSubject("japan china")`), null);
});

test("#1172 R2: a seeded answer carries the subject gate and the native intent", async () => {
  const usa = await solve("What is the capital of the USA?");
  assert.equal(usa.intent, "fact_lookup", JSON.stringify(usa));
  assert.match(usa.content, /Washington/u);
  assert.ok(usa.evidence.includes("fact_lookup:subject_gate:subject_qid:Q30"), JSON.stringify(usa.evidence));

  const japan = await solve("Tell us the capital of Japan");
  assert.equal(japan.intent, "fact_lookup");
  assert.match(japan.content, /Tokyo/u);
});

test("#1172 R2: an unseeded subject is never answered from another record", async () => {
  for (const prompt of ["What is the capital of Australia?", "What is the capital of Canada, not the USA?"]) {
    const answer = await solve(prompt);
    assert.notEqual(answer.intent, "fact_lookup", `${prompt}: ${JSON.stringify(answer)}`);
    assert.doesNotMatch(String(answer.content), /Washington/u, prompt);
  }
});

test("#1172 R2: Q-id-less records keep the word-boundary alias hint as the last resort", async () => {
  const answer = await solve("Who painted the Mona Lisa?");
  assert.equal(answer.intent, "fact_lookup");
  assert.ok(answer.evidence.includes("fact_lookup:subject_gate:surface_hint"), JSON.stringify(answer.evidence));
});

// ---- R6: seeded comparisons ----------------------------------------------

test("#1172 R6: two seeded subjects are compared on aligned relations in every language", async () => {
  const cases = [
    ["Compare Japan and Russia.", /Tokyo/u, /Moscow/u],
    ["Сравни Японию и Россию", /Токио/u, /Москва/u],
    ["जापान और रूस की तुलना करें", /टोक्यो/u, /मास्को/u],
    ["比较日本和俄罗斯", /东京/u, /莫斯科/u],
  ];
  for (const [prompt, left, right] of cases) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "fact_comparison", `${prompt}: ${JSON.stringify(answer)}`);
    assert.match(answer.content, left, prompt);
    assert.match(answer.content, right, prompt);
    assert.ok(answer.evidence.includes("fact_comparison:difference:capital"), JSON.stringify(answer.evidence));
    assert.ok(answer.evidence.includes("fact_comparison:left:Q17"), JSON.stringify(answer.evidence));
    assert.ok(answer.evidence.includes("fact_comparison:right:Q159"), JSON.stringify(answer.evidence));
  }
});

test("#1172 R6: a comparison with one unseeded side names the missing side honestly", async () => {
  const answer = await solve("Compare Japan and Australia");
  assert.equal(answer.intent, "fact_comparison_gap", JSON.stringify(answer));
  assert.match(answer.content, /Australia/u);
  assert.match(answer.content, /Tokyo/u);
  assert.ok(answer.evidence.includes("fact_comparison:missing:Australia"), JSON.stringify(answer.evidence));
});

test("#1172 R6: subjects without seeded facts leave the comparison route alone", async () => {
  assert.equal(await call(`tryFactComparison("Compare Rust and Go for writing web servers.", normalizePrompt("Compare Rust and Go for writing web servers."))`), null);
  assert.equal(await call(`tryFactComparisonGap("Compare Rust and Go", normalizePrompt("Compare Rust and Go"))`), null);
  assert.equal(await call(`tryFactComparison("Compare Japan and Japan", "compare japan and japan")`), null);
});

// ---- R7: questions over prompt-supplied text -------------------------------

test("#1172 R7: a question over a quoted passage is answered from its own sentence with no network call", async () => {
  const prompts = [
    "Read this and answer: \"The meeting moved from Tuesday to Thursday at 3 pm in room 204.\" When and where is the meeting?",
    "Given the text: 'The meeting moved from Tuesday to Thursday at 3 pm in room 204.' When is the meeting?",
    "Given the text: \"Мы встретимся в четверг в комнате 204.\" Где встреча?",
    "根据文本：「会议改到星期四下午三点，在204房间。」会议在哪里？",
  ];
  for (const prompt of prompts) {
    external.length = 0;
    const answer = await solve(prompt);
    assert.equal(answer.intent, "prompt_text_answer", `${prompt}: ${JSON.stringify(answer)}`);
    assert.match(answer.content, /204/u, prompt);
    assert.ok(answer.evidence.includes("prompt_text:network:none"), JSON.stringify(answer.evidence));
    assert.deepEqual(external, [], `${prompt} made network calls`);
  }
});

test("#1172 R7: the sentence covering the question's words is the one quoted", async () => {
  const answer = await solve(
    "Given the text: \"The library opens at 9 am. The cafe closes at 6 pm. Parking is free on Sundays.\" When does the cafe close?",
  );
  assert.equal(answer.intent, "prompt_text_answer");
  assert.match(answer.content, /The cafe closes at 6 pm\./u);
  assert.doesNotMatch(answer.content, /library/u);
});

test("#1172 R7: nothing in the text covers the question, so the text does not say", async () => {
  external.length = 0;
  const answer = await solve("Given the text: 'The meeting moved to Thursday. Alice chairs it.' Who is the CEO?");
  assert.equal(answer.intent, "prompt_text_gap", JSON.stringify(answer));
  assert.match(answer.content, /does not say/u);
  assert.match(answer.content, /ceo/u);
  assert.deepEqual(external, []);
});

const MEETING = "The meeting moved from Tuesday to Thursday at 3 pm in room 204.";
const MEETING_STATEMENT = "As a statement: subject «The meeting»; predicate «moved»; "
  + "time «from Tuesday to Thursday at 3 pm»; place «in room 204»";

test("#1172 R7: the covering sentence is projected into subject/predicate/object/time/place", async () => {
  const both = await solve(`Read this and answer: "${MEETING}" When and where is the meeting?`);
  assert.equal(both.content, [
    "From the text — time «from Tuesday to Thursday at 3 pm»; place «in room 204».",
    `The text answers this: «${MEETING}»`,
    MEETING_STATEMENT,
  ].join("\n"));
  assert.ok(both.evidence.includes("prompt_text:statement:subject=The meeting;predicate=moved;time=from Tuesday to Thursday at 3 pm;place=in room 204"), JSON.stringify(both.evidence));
  assert.ok(both.evidence.includes("prompt_text:asked:time,place"));

  const when = await solve(`Given the text: '${MEETING}' When is the meeting?`);
  assert.equal(when.content, [
    "From the text — time «from Tuesday to Thursday at 3 pm».",
    `The text answers this: «${MEETING}»`,
    MEETING_STATEMENT,
  ].join("\n"));

  const close = await solve(
    "Given the text: \"The library opens at 9 am. The cafe closes at 6 pm. Parking is free on Sundays.\" When does the cafe close?",
  );
  assert.equal(close.content, [
    "From the text — time «at 6 pm».",
    "The text answers this: «The cafe closes at 6 pm.»",
    "As a statement: subject «The cafe»; predicate «closes»; time «at 6 pm»",
  ].join("\n"));

  const russian = await solve("Given the text: \"Мы встретимся в четверг в комнате 204.\" Где встреча?");
  assert.equal(russian.content, [
    "По тексту — место «в комнате 204».",
    "Ответ есть в тексте: «Мы встретимся в четверг в комнате 204.»",
    "Как утверждение: субъект «Мы»; предикат «встретимся»; время «в четверг»; место «в комнате 204»",
  ].join("\n"));

  const cafe = await solve(
    "Given the text: \"The library opens at 9 am. The cafe closes at 6 pm. Parking is free on Sundays.\" When is parking free?",
  );
  assert.equal(cafe.content, [
    "From the text — time «on Sundays».",
    "The text answers this: «Parking is free on Sundays.»",
    "As a statement: subject «Parking»; predicate «is free»; time «on Sundays»",
  ].join("\n"), "an inflected weekday is a time; the copula joins the predicate");
});

test("#1172 R7: an asked slot the question already says is not claimed; CJK keeps the quote", async () => {
  const who = await solve(`Given the text: '${MEETING}' Who moved the meeting?`);
  assert.equal(who.intent, "prompt_text_answer");
  assert.equal(who.content.split("\n")[0], `The text answers this: «${MEETING}»`, "subject «The meeting» answers who? only as the statement");
  assert.ok(!who.evidence.some((entry) => entry.startsWith("prompt_text:asked:")), JSON.stringify(who.evidence));

  const chinese = await solve("根据文本：「会议改到星期四下午三点，在204房间。」会议在哪里？");
  assert.equal(chinese.content, "文本中的答案：「会议改到星期四下午三点，在204房间。」");

  const comma = await call(`promptTextStatement("On Monday, Alice sent the report to Bob.")`);
  assert.deepEqual(comma, [["Alice"], ["sent"], ["the", "report", "to", "Bob"], ["On", "Monday"], []]);
  assert.equal(await call(`promptTextStatement("Alice chairs it.")`), null, "no seeded adjunct, no projection");
  const alice = await solve("Given the text: 'The meeting moved to Thursday. Alice chairs it.' Who chairs it?");
  assert.equal(alice.content, "The text answers this: «Alice chairs it.»");
});

test("#1172 R7: a quoted phrase inside an instruction is not a text to read", async () => {
  assert.equal(await call(`tryPromptTextQuestion("Translate 'I would like to order a coffee please' into French", "")`), null);
  assert.equal(await call(`tryPromptTextQuestion("Proofread 'the quick brown fox jumps over the lazy dog'", "")`), null);
});

// ---- R12: the shared cross-runtime parity case ------------------------------

test("#1172 R12: the worker matches the shared parity case the native solver pins", async () => {
  const cases = JSON.parse(readFileSync(path.join(REPO_ROOT, "data/parity/cross-runtime-synthesis.json"), "utf8"));
  const parity = cases.find((entry) => entry.id === "e1172_seeded_capital_subject_gate");
  assert.ok(parity, "the parity corpus carries the #1172 case");
  const answer = await solve(parity.prompt);
  assert.equal(answer.intent, parity.expectedIntent, JSON.stringify(answer));
  for (const fragment of parity.expectedAnswerFragments) assert.ok(answer.content.includes(fragment), fragment);
  for (const fragment of parity.forbiddenAnswerFragments) assert.ok(!answer.content.includes(fragment), fragment);
  for (const prefix of parity.browserExpectedEvidencePrefixes || parity.expectedEvidencePrefixes) {
    assert.ok(answer.evidence.some((link) => link.startsWith(prefix)), `${prefix}: ${JSON.stringify(answer.evidence)}`);
  }
});
