// Issue #1186 R4/R6, JavaScript root: the clause is rendered as the typed
// relative-meta-logic fragment `rml export lean` reads (formalRmlSource, the
// twin of `rml_source` pinned by rust/tests/unit/issue_1186_rml_export.rs),
// and the honesty block states the rml step. The browser has no process to run
// rml in, so its step is "absent" for a universal clause and "outside the
// subset" for the existential and negative readings.
//
// When RML_BIN names a relative-meta-logic `rml` executable (CI checks one out
// in .github/workflows/rml-export.yml), the rendered source is exported for
// real and the exit status must be 0.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

const PROBE_SOURCE = [
  "(U: (Type 0) U)",
  "(Student: (Pi (U x) Prop))",
  "(Studies: (Pi (U x) Prop))",
  "(Passes: (Pi (U x) Prop))",
  "(formalized: (Pi (U x) (Pi ((Student x) h1) (Pi ((Studies x) h2) (Passes x)))))",
  "",
].join("\n");

async function rmlSourceFor(sentence) {
  await seeded;
  const literal = JSON.stringify(sentence);
  return plain(evaluate(
    worker,
    `(() => { const grammar = formalGrammar(); const natural = grammar.natural.find((item) => item.language === "en"); const clause = formalParseQuantifiedClause(${literal}, natural); return clause === null ? undefined : formalRmlSource(clause); })()`,
  ));
}

async function handlerAnswer(prompt) {
  await seeded;
  const literal = JSON.stringify(prompt);
  return plain(evaluate(worker, `tryFormalizationRequest(${literal}, normalizePrompt(${literal}))`)).content;
}

test("a universal conditional renders as the rml typed fragment", async () => {
  assert.equal(await rmlSourceFor("Every student who studies passes"), PROBE_SOURCE);
});

test("an object becomes a constant of the domain", async () => {
  assert.equal(
    await rmlSourceFor("Every student who studies passes the exam"),
    [
      "(U: (Type 0) U)",
      "(Student: (Pi (U x) Prop))",
      "(Studies: (Pi (U x) Prop))",
      "(Passes: (Pi (U x) (Pi (U xx) Prop)))",
      "(exam: U exam)",
      "(formalized: (Pi (U x) (Pi ((Student x) h1) (Pi ((Studies x) h2) (Passes x exam)))))",
      "",
    ].join("\n"),
  );
});

test("existential and negative readings are outside the export subset", async () => {
  assert.equal(await rmlSourceFor("Some bird that sings flies"), null);
  const answer = await handlerAnswer("Formalize in first-order logic: Some bird that sings flies");
  assert.ok(
    answer.includes("The exists reading is outside the relative-meta-logic export subset, which has no binder for it, so rml was not run."),
    answer,
  );
});

test("the honesty block states that the browser did not run rml", async () => {
  const answer = await handlerAnswer("Formalize in first-order logic: Every student who studies passes");
  assert.ok(
    answer.includes("rml was not found in PATH and FORMAL_AI_RML names no executable, so the clause was not exported through relative-meta-logic."),
    answer,
  );
});

test("rml export lean accepts the rendered source", { skip: !process.env.RML_BIN && "RML_BIN is not set" }, () => {
  const directory = mkdtempSync(join(tmpdir(), "formal-ai-rml-"));
  const source = join(directory, "clause.lino");
  const output = join(directory, "clause.lean");
  writeFileSync(source, PROBE_SOURCE);
  const run = spawnSync(process.env.RML_BIN, ["export", "lean", source, "-o", output], { encoding: "utf8" });
  assert.equal(run.status, 0, `${run.stdout}\n${run.stderr}`);
  assert.match(readFileSync(output, "utf8"), /axiom formalized : \(x : U\) -> \(h1 : Student x\) -> \(h2 : Studies x\) -> Passes x/u);
});
