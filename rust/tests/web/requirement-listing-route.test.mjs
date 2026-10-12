// R1188-U20: "List the requirements of this issue: <text>" is answered in
// chat, in every seeded language, with a localized intro and exactly the
// requirements the extractor returns, one per line.
//
// The route is data: the `requirement_listing` rule set of
// data/seed/handler-rules.lino reads the `requirement_listing_action` role in
// the command head and captures `value requirements transform
// requirement_list`, the generic text-transform value source over the
// request's free-text payload. The native twin is
// rust/tests/unit/requirement_listing_route.rs, which pins the same answers.

import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import test from "node:test";

import { createWorkerContext, evaluate, plain, REPO_ROOT } from "./support/browser-runtime.mjs";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
import { installNodeHost } from "../../../js/agentic/node-host.mjs";
import { extractRequirements } from "../../../js/agentic/crate/requirement_extraction.mjs";

await installNodeHost(new WorkerHost());

const worker = createWorkerContext();
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await ready;
  return worker.solve(prompt, [], {}, {}, [], {});
}

/** Prompts and the exact answers both runtimes give them. */
const ANSWERS = [
  [
    "List the requirements of this issue: The page must load in under a second. "
      + "Add a progress bar to the upload form. The cache works well.",
    "Requirements stated in the text:\n"
      + "- The page must load in under a second.\n"
      + "- Add a progress bar to the upload form.",
  ],
  [
    "Перечисли требования этой задачи: Страница должна загружаться быстрее секунды. "
      + "Добавь индикатор загрузки. Кэш работает хорошо.",
    "Требования, изложенные в тексте:\n"
      + "- Страница должна загружаться быстрее секунды.\n"
      + "- Добавь индикатор загрузки.",
  ],
  [
    "列出这个问题的需求：页面必须在一秒内加载。添加上传进度条。缓存运行良好。",
    "文本中提出的需求：\n- 页面必须在一秒内加载。\n- 添加上传进度条。",
  ],
  [
    "List the requirements of this issue\n"
      + "## Acceptance criteria\n"
      + "- The upload resumes after a reload\n"
      + "- [ ] Errors are shown in the user language",
    "Requirements stated in the text:\n"
      + "- The upload resumes after a reload\n"
      + "- Errors are shown in the user language",
  ],
  [
    "Extract the requirements from \"The page must load quickly. Add a progress bar.\"",
    "Requirements stated in the text:\n- The page must load quickly.\n- Add a progress bar.",
  ],
];

test("a requirement-listing request answers the extracted requirements, one per line", async () => {
  for (const [prompt, expected] of ANSWERS) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "requirement_listing", prompt);
    assert.equal(answer.content, expected, prompt);
    assert.ok(answer.evidence.includes("text_transform:requirement_list"), prompt);
  }
});

test("without a payload, or with a payload stating no requirement, the rule declines", async () => {
  for (const prompt of [
    "List the requirements of this issue",
    "List the requirements of this issue: The cache was added last year. It works well.",
  ]) {
    const answer = await solve(prompt);
    assert.notEqual(answer.intent, "requirement_listing", prompt);
  }
});

test("a listing phrase inside the pasted text never routes: the command head decides", async () => {
  const answer = await solve(
    "Summarize: We need to list the requirements before the meeting. The team met on Monday. "
      + "Everyone agreed on the plan.",
  );
  assert.equal(answer.intent, "summarization_free_text");
});

test("the full-width colon of CJK text ends a command head, and CJK characters count as words", async () => {
  await ready;
  assert.equal(
    plain(evaluate(worker, 'textTransformFreeTextPayload("列出需求：页面必须加载。")')),
    "页面必须加载。",
  );
  assert.equal(plain(evaluate(worker, 'textTransformPayloadWordCount("页面 必须 load")')), 5);
  assert.equal(plain(evaluate(worker, 'normalizePrompt(textTransformCommandHead("列出需求：页面必须加载。"))')), "列出需求");
});

test("the worker extractor is the crate extractor over the whole benchmark corpus", async () => {
  await ready;
  const corpus = path.join(REPO_ROOT, "data/benchmarks/issue-requirements");
  const files = readdirSync(corpus).filter((name) => name.endsWith(".md"));
  assert.ok(files.length > 100, String(files.length));
  for (const file of files) {
    worker.corpusText = readFileSync(path.join(corpus, file), "utf8");
    assert.deepEqual(
      plain(evaluate(worker, "extractRequirements(corpusText)")),
      extractRequirements(worker.corpusText),
      file,
    );
  }
});
