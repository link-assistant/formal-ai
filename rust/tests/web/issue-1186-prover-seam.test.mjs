// Issue #1186 R4, JavaScript root: the theorem-prover step. Every `prover`
// record of data/seed/formal-targets.lino turns its target's rendering into a
// self-contained compile unit (formalProverUnit), and the honesty block states
// in the answer language what ran. The browser installs no prover host, so
// both provers are absent there; the JavaScript server installs
// createProverHost (js/server/prover-host.mjs), which looks the binary up in
// the PATH it is given and runs it. These tests stub PATH with a scratch
// directory holding fake `lean` and `coqc` executables, so the "ran" branch is
// pinned without either prover installed. Twin of
// rust/tests/unit/issue_1186_prover_step.rs.

import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  PROVER_DIRECTORY,
  createProverHost,
  proverCommandIn,
  proverFileStem,
} from "../../../js/server/prover-host.mjs";
import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const PROBE = "Formalize in first-order logic: Every student who studies passes the exam";

const LEAN_UNIT = [
  "axiom U : Type",
  "axiom Student : U → Prop",
  "axiom Studies : U → Prop",
  "axiom Passes : U → U → Prop",
  "axiom exam : U",
  "",
  "theorem formalized : ∀ (x : U), Student x ∧ Studies x → Passes x exam := by sorry",
  "",
].join("\n");

const ROCQ_UNIT = [
  "Parameter U : Type.",
  "Parameter Student : U -> Prop.",
  "Parameter Studies : U -> Prop.",
  "Parameter Passes : U -> U -> Prop.",
  "Parameter exam : U.",
  "",
  "Theorem formalized : forall (x : U), Student x /\\ Studies x -> Passes x exam.",
  "Proof.",
  "Admitted.",
  "",
].join("\n");

/** A scratch PATH directory with fake provers that copy their input aside. */
function stubProvers(exits) {
  const bin = mkdtempSync(join(tmpdir(), "formal-ai-prover-path-"));
  for (const [binary, exit] of Object.entries(exits)) {
    const script = join(bin, binary);
    writeFileSync(script, `#!/bin/sh\ncp "$1" "${bin}/${binary}.seen"\nexit ${exit}\n`);
    chmodSync(script, 0o755);
  }
  return bin;
}

async function answerWith(overrides, prompt) {
  const worker = createWorkerContext(overrides);
  await evaluate(worker, "loadSeed()");
  const literal = JSON.stringify(prompt);
  return plain(evaluate(worker, `tryFormalizationRequest(${literal}, normalizePrompt(${literal}))`)).content;
}

/** The answer paragraph that opens with the prover summary. */
function honestyParagraph(answer, opening) {
  const found = answer.split("\n\n").find((paragraph) => paragraph.startsWith(opening));
  assert.ok(found !== undefined, answer);
  return found;
}

const RML_ABSENT =
  "rml was not found in PATH and FORMAL_AI_RML names no executable, so the clause was not exported through relative-meta-logic.";
const SEAM_TAIL =
  "The Lean and Rocq texts come from the seed-grammar exporter; the relative-meta-logic crate takes its place in process once it is published (link-foundation/relative-meta-logic#185).";

test("each prover's compile unit declares the domain, predicates and objects before the theorem", async () => {
  const worker = createWorkerContext();
  await evaluate(worker, "loadSeed()");
  const units = plain(evaluate(
    worker,
    `(() => { const grammar = formalGrammar(); const natural = grammar.natural.find((item) => item.language === "en"); const clause = formalParseQuantifiedClause("Every student who studies passes the exam", natural); return formalProverRecords().map((prover) => [prover.binary, prover.extension, formalProverUnit(grammar, clause, prover)]); })()`,
  ));
  assert.deepEqual(units, [
    ["lean", "lean", LEAN_UNIT],
    ["coqc", "v", ROCQ_UNIT],
  ]);
});

test("without a prover host (the browser) no prover is invoked", async () => {
  const answer = await answerWith({}, PROBE);
  assert.equal(
    honestyParagraph(answer, "No theorem prover"),
    [
      "No theorem prover was invoked.",
      "lean was not found in PATH, so the lean text was not compiled.",
      "coqc was not found in PATH, so the coqc text was not compiled.",
      RML_ABSENT,
      SEAM_TAIL,
    ].join(" "),
  );
});

test("an empty PATH finds no prover and runs nothing", async () => {
  const scratch = mkdtempSync(join(tmpdir(), "formal-ai-prover-tmp-"));
  const answer = await answerWith({ formalAiProverHost: createProverHost({ path: "", tmpdir: scratch }) }, PROBE);
  assert.equal(
    honestyParagraph(answer, "No theorem prover"),
    [
      "No theorem prover was invoked.",
      "lean was not found in PATH, so the lean text was not compiled.",
      "coqc was not found in PATH, so the coqc text was not compiled.",
      RML_ABSENT,
      SEAM_TAIL,
    ].join(" "),
  );
});

test("provers found in a stubbed PATH run on their compile units and report their exit status", async () => {
  const bin = stubProvers({ lean: 0, coqc: 1 });
  const scratch = mkdtempSync(join(tmpdir(), "formal-ai-prover-tmp-"));
  const answer = await answerWith({ formalAiProverHost: createProverHost({ path: bin, tmpdir: scratch }) }, PROBE);
  const leanPath = join(scratch, PROVER_DIRECTORY, `${proverFileStem(LEAN_UNIT)}.lean`);
  const rocqPath = join(scratch, PROVER_DIRECTORY, `${proverFileStem(ROCQ_UNIT)}.v`);
  assert.equal(
    honestyParagraph(answer, "A theorem prover"),
    [
      "A theorem prover was invoked: lean, coqc.",
      `lean was found in PATH and ran on ${leanPath}; it exited with status 0.`,
      `coqc was found in PATH and ran on ${rocqPath}; it exited with status 1.`,
      RML_ABSENT,
      SEAM_TAIL,
    ].join(" "),
  );
  assert.equal(readFileSync(join(bin, "lean.seen"), "utf8"), LEAN_UNIT);
  assert.equal(readFileSync(join(bin, "coqc.seen"), "utf8"), ROCQ_UNIT);
});

test("only the prover present in PATH runs, and the status text follows the answer language", async () => {
  const bin = stubProvers({ lean: 0 });
  const scratch = mkdtempSync(join(tmpdir(), "formal-ai-prover-tmp-"));
  const answer = await answerWith(
    { formalAiProverHost: createProverHost({ path: bin, tmpdir: scratch }) },
    "Формализуй в логике первого порядка: Каждый студент, который учится, сдаёт экзамен",
  );
  const paragraph = honestyParagraph(answer, "Вызван теорем-прувер");
  const sentences = paragraph.split(/(?<=\.) /u);
  assert.equal(sentences[0], "Вызван теорем-прувер: lean.");
  assert.match(sentences[1], /^lean найден в PATH и запущен на .+clause_[0-9a-f]{16}\.lean; код завершения 0\.$/u);
  assert.equal(sentences[2], "coqc не найден в PATH, поэтому текст coqc не компилировался.");
});

test("the PATH lookup and file naming are deterministic", () => {
  const bin = stubProvers({ lean: 0 });
  assert.equal(proverCommandIn(bin, "lean"), join(bin, "lean"));
  assert.equal(proverCommandIn(bin, "coqc"), null);
  assert.equal(proverCommandIn(undefined, "lean"), null);
  // FNV-1a 64 of the empty string is its offset basis.
  assert.equal(proverFileStem(""), "clause_cbf29ce484222325");
  assert.equal(proverFileStem(LEAN_UNIT), proverFileStem(LEAN_UNIT));
});
