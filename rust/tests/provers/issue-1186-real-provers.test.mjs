// Issue #1186 R4: "the output compiles when `lean`/`coqc` is available".
// rust/tests/web/issue-1186-prover-seam.test.mjs pins the prover step with
// stand-in executables; this suite runs the real ones. It lives outside the
// js tier's glob because it needs both provers installed: the `provers` job
// of .github/workflows/layered-ci.yml installs Lean 4 and Rocq and runs it,
// so a missing prover fails the job instead of skipping the check.

import assert from "node:assert/strict";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { createProverHost, proverCommandIn } from "../../../js/server/prover-host.mjs";
import { createWorkerContext, evaluate, plain } from "../web/support/browser-runtime.mjs";

const PROBES = [
  "Formalize in first-order logic: Every student who studies passes the exam",
  "Формализуй в логике первого порядка: Каждый студент, который учится, сдаёт экзамен",
];

async function answerWith(host, prompt) {
  const worker = createWorkerContext({ formalAiProverHost: host });
  await evaluate(worker, "loadSeed()");
  const literal = JSON.stringify(prompt);
  return plain(evaluate(worker, `tryFormalizationRequest(${literal}, normalizePrompt(${literal}))`)).content;
}

test("both provers are installed where the job put them", () => {
  for (const binary of ["lean", "coqc"]) {
    assert.notEqual(proverCommandIn(process.env.PATH, binary), null, `${binary} is not in PATH`);
  }
});

for (const prompt of PROBES) {
  test(`the Lean and Rocq units of "${prompt}" compile`, async () => {
    const scratch = mkdtempSync(join(tmpdir(), "formal-ai-real-prover-"));
    const answer = await answerWith(createProverHost({ path: process.env.PATH, tmpdir: scratch }), prompt);
    const statuses = [...answer.matchAll(/\b(lean|coqc) (?:was found|найден)[^;]*; (?:it exited with status|код завершения) (\d+)/gu)]
      .map(([, binary, status]) => [binary, Number(status)]);
    assert.deepEqual(statuses, [["lean", 0], ["coqc", 0]], answer);
  });
}
