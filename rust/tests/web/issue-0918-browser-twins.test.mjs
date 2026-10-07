// Issue #918 (E71, R914-6), browser root: the `memory_program` and
// `memory_program_gap` rows of the worker's solve table answer exactly as the
// native memory surface does. Both runtimes compile a request against
// data/seed/memory-programs.lino and render the outcome through the seeded
// `memory_program_*` responses. Prompts, stores and expected wording are the
// ones rust/tests/unit/issue_918_browser_twins.rs asserts against
// `execute_memory_query_with_options`.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const ready = evaluate(worker, "loadSeed()");

const RENAME_REQUEST = "List every fact I contributed about X and rename X to Y in all of them.";
const RENAME_EN = "Memory program memory_program_41ef0c9602340676 matched 1 event(s), changed 1, and stopped at fixpoint after 2 iteration(s).";
const RENAME_EMPTY_RU = "Программа памяти memory_program_41ef0c9602340676 сопоставила событий: 0, изменила: 0 и остановилась с результатом fixpoint после итераций: 1.";
const DESTRUCTIVE_EN = "Destructive memory actions require explicit human confirmation. No memory event was erased or retracted.";
const GAP_EN = "I could not compile this memory request without dropping a step. program_gap:no_complete_seeded_family";

const factStore = () => [{ id: "fact", kind: "fact", role: "user", content: "X" }];

async function solve(prompt, memoryEvents = []) {
  await ready;
  return worker.solve(prompt, [], {}, {}, [], { memoryEvents });
}

// FNV-1a 64 as rust/src/web_engine_core.rs::stable_id computes it.
function stableId(prefix, text) {
  let hash = 0xcbf29ce484222325n;
  for (const byte of new TextEncoder().encode(text)) {
    hash ^= BigInt(byte);
    hash = (hash * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  return `${prefix}_${hash.toString(16).padStart(16, "0")}`;
}

test("a seeded rename program runs to fixpoint with the seeded wording", async () => {
  const response = await solve(RENAME_REQUEST, factStore());
  assert.equal(response.intent, "memory_program");
  assert.equal(response.content, RENAME_EN);
  assert.equal(response.memoryOperation.updates[0].fields.content, "Y");
});

test("the program id is the native stable id of the canonical program", async () => {
  await ready;
  const compiled = plain(evaluate(worker, `compileMemoryProgramForWorker(${JSON.stringify(RENAME_REQUEST)})`));
  assert.equal(compiled.status, "compiled");
  assert.equal(compiled.program.canonical.split("\n")[0], "family=contributed_fact_rename");
  assert.equal(stableId("memory_program", compiled.program.canonical), "memory_program_41ef0c9602340676");
});

test("the russian request compiles to the same program and answers in russian", async () => {
  const response = await solve("Перечисли все факты, которые я добавил о X, и переименуй X в Y во всех них.");
  assert.equal(response.intent, "memory_program");
  assert.equal(response.content, RENAME_EMPTY_RU);
  assert.equal(response.memoryOperation, undefined);
});

test("a destructive program is refused without confirmation", async () => {
  const response = await solve("Delete every fact I contributed about X.", factStore());
  assert.equal(response.intent, "memory_program_refused");
  assert.equal(response.content, DESTRUCTIVE_EN);
  assert.equal(response.memoryOperation, undefined);
});

test("a memory request no seeded family covers names its gap", async () => {
  const response = await solve("Transpose every fact matrix in memory.");
  assert.equal(response.intent, "memory_program_gap");
  assert.equal(response.content, GAP_EN);
});

// Issue #918 (R914-6): the compound_interest twins read one policy block and
// one set of seeded responses. Two deliberate parity fixes: the native row
// took its rate from the calculator while the worker took a table of its own
// (both now read the seeded default rates), and the worker appended a
// USD -> USD "conversion" whenever a principal was spelled in dollars (a
// conversion now skips its source currency, and the native row gains the
// ruble target the worker already had).
const COMPOUND_REPORT = "Compound interest calculation\n\nFormula: A = P(1 + r/n)^(n*t)\nP = 1000 USD\nr = 0.08 (8% annual)\nn = 12 (monthly)\nt = 5 years\n\nStep 1: periodic rate = r/n = 0.08/12 = 0.006666666666667\nStep 2: number of periods = n*t = 12*5 = 60\nStep 3: A = 1000 * (1 + 0.006666666666667)^60\nFinal amount: 1489.85 USD";
const COMPOUND_EUR = "\n\nConversion: USD -> EUR\n1 USD in EUR = 0.92 EUR\n1489.85 USD * 0.92 = 1370.66 EUR\nRate detail: Exchange rate: 1 USD = 0.92 EUR (source: default (hardcoded))\nLive web freshness is not independently verified here; this uses the exchange-rate source available through the local calculator.";
const COMPOUND_RUB = "\n\nConversion: USD -> RUB\n1 USD in RUB = 89.5 RUB\n1489.85 USD * 89.5 = 133341.57 RUB\nRate detail: Exchange rate: 1 USD = 89.5 RUB (source: default (hardcoded))";
const COMPOUND_PROMPT = "Invest $1000 at 8% annual interest compounded monthly for 5 years";

async function solveWith(prompt, history) {
  await ready;
  return worker.solve(prompt, history, {}, {}, [], {});
}

test("a compound-interest report converts at the seeded euro and ruble rates", async () => {
  for (const [prompt, expected] of [
    [`${COMPOUND_PROMPT} and convert the final amount to EUR using current exchange rates from the web.`, COMPOUND_REPORT + COMPOUND_EUR],
    [`${COMPOUND_PROMPT} and convert the final amount to rubles.`, COMPOUND_REPORT + COMPOUND_RUB],
  ]) {
    const response = await solveWith(prompt, []);
    assert.equal(response.intent, "calculation", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

test("a principal spelled in dollars is never converted to dollars", async () => {
  const response = await solveWith("invest 1000 dollars at 8,5% interest compounded daily for 2 years", []);
  assert.equal(response.content, "Compound interest calculation\n\nFormula: A = P(1 + r/n)^(n*t)\nP = 1000 USD\nr = 0.085 (8.5% annual)\nn = 365 (daily)\nt = 2 years\n\nStep 1: periodic rate = r/n = 0.085/365 = 0.000232876712329\nStep 2: number of periods = n*t = 365*2 = 730\nStep 3: A = 1000 * (1 + 0.000232876712329)^730\nFinal amount: 1185.28 USD");
});

test("a follow-up converts the earlier final amount", async () => {
  const history = [
    { role: "user", content: COMPOUND_PROMPT },
    { role: "assistant", content: COMPOUND_REPORT },
  ];
  const response = await solveWith("Convert the final amount to rubles", history);
  assert.equal(response.intent, "calculation");
  assert.equal(response.content, `Final amount conversion\nSource amount: 1489.85 USD${COMPOUND_RUB}`);
});
