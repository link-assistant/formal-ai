// Issue #1175 R3, browser root: the claim rows of
// data/seed/capability-routing.lino are consulted before a handler runs
// (claimRouteAdmits in js/worker/formal_ai_worker_dispatch.js), as
// rust/tests/unit/issue_1175_claim_routing.rs pins for the native router.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

async function admits(handler, prompt) {
  await seeded;
  return plain(evaluate(worker, `claimRouteAdmits(${JSON.stringify(handler)}, ${JSON.stringify(prompt)})`));
}

test("the claim rows are read from the capability table", async () => {
  await seeded;
  assert.deepEqual(plain(evaluate(worker, "claimRouteRows()")), [
    { handler: "software_project", browserHandler: "trySoftwareProjectRequest", admitsOn: ["object_phrase_artifact", "approval_of_a_proposal"] },
    { handler: "terminal_command", browserHandler: "tryTerminalCommand", admitsOn: ["shell_command_shape", "semantic_shell_task"] },
    { handler: "repository_lineage", browserHandler: "", admitsOn: ["repository_subject"] },
    { handler: "page_query_text", browserHandler: "tryPageQueryText", admitsOn: ["supplied_page"] },
    { handler: "javascript_execution", browserHandler: "tryJavaScriptExecution", admitsOn: ["javascript_program"] },
    { handler: "incompatible_units", browserHandler: "tryIncompatibleUnits", admitsOn: ["incompatible_unit_pair"] },
    { handler: "http_fetch", browserHandler: "", admitsOn: ["fetch_url"] },
    { handler: "url_navigate", browserHandler: "", admitsOn: ["navigation_url"] },
    { handler: "calendar_create_event", browserHandler: "", admitsOn: ["calendar_date_signal"] },
    { handler: "code_debugging", browserHandler: "tryCodeDebugging", admitsOn: ["code_artifact"] },
    { handler: "code_explanation", browserHandler: "tryCodeExplanation", admitsOn: ["code_artifact"] },
    { handler: "code_review", browserHandler: "tryCodeReview", admitsOn: ["code_artifact"] },
    { handler: "summarization_text", browserHandler: "trySummarizationText", admitsOn: ["supplied_text"] },
    { handler: "text_rewrite", browserHandler: "tryTextRewrite", admitsOn: ["supplied_text"] },
    { handler: "statistics", browserHandler: "tryStatistics", admitsOn: ["stated_number"] },
    { handler: "word_problem", browserHandler: "tryWordProblem", admitsOn: ["stated_number"] },
    { handler: "arithmetic", browserHandler: "tryArithmetic", admitsOn: ["calculation_expression", "currency_rate_basis"] },
    { handler: "compound_interest", browserHandler: "tryCompoundInterest", admitsOn: ["investment_terms", "conversion_target_currency"] },
    { handler: "number_constraint_reasoning", browserHandler: "tryNumberConstraintReasoning", admitsOn: ["interval_bounds"] },
    { handler: "unit_conversion", browserHandler: "tryUnitConversion", admitsOn: ["measured_quantity"] },
    { handler: "calendar_reasoning", browserHandler: "tryCalendarReasoning", admitsOn: ["calendar_date_signal", "calendar_anchor"] },
  ]);
});

test("every row names browser functions and evidence kinds the worker knows", async () => {
  await seeded;
  const unknown = plain(evaluate(worker, `claimRouteRows().flatMap((row) => [
    ...row.admitsOn.filter((kind) => typeof CLAIM_EVIDENCE[kind] !== "function"),
    ...(row.browserHandler && typeof self[row.browserHandler] !== "function" ? [row.browserHandler] : []),
  ])`));
  assert.deepEqual(unknown, []);
});

async function evidence(kind, prompt) {
  await seeded;
  return plain(evaluate(worker, `CLAIM_EVIDENCE[${JSON.stringify(kind)}](${JSON.stringify(prompt)}, normalizePrompt(${JSON.stringify(prompt)}))`));
}

test("the native-only rows read the same structure in the worker", async () => {
  assert.equal(await evidence("fetch_url", "Fetch https://example.com"), true);
  assert.equal(await evidence("fetch_url", "Fetch me a summary of the news"), false);
  assert.equal(await evidence("navigation_url", "Navigate to github.com"), true);
  assert.equal(await evidence("navigation_url", "Navigate the menu to the settings screen"), false);
  assert.equal(await evidence("calendar_date_signal", "Schedule a meeting with Anna tomorrow at 3pm"), true);
  assert.equal(await evidence("calendar_date_signal", "Explain how a calendar works."), false);
});

test("a surface word alone is not admitted", async () => {
  assert.equal(await admits("tryTerminalCommand", "Make a 3-day itinerary for a first visit to Rome."), false);
  assert.equal(await admits("tryTerminalCommand", "Find the bug: def average(xs): return sum(xs) / len(xs)"), false);
  assert.equal(await admits("trySoftwareProjectRequest", "Write a regex for a US ZIP code with an optional 4-digit extension"), false);
  assert.equal(await admits("tryPageQueryText", "What command builds the jar on this page?"), false);
  assert.equal(await admits("tryJavaScriptExecution", "Run this JavaScript"), false);
  assert.equal(await admits("tryIncompatibleUnits", "What does a unit test check?"), false);
  assert.equal(await admits("tryCodeDebugging", "Find the bug in my plan for the trip"), false);
  assert.equal(await admits("tryCodeExplanation", "Explain how a compiler works"), false);
  assert.equal(await admits("tryCodeReview", "Review my essay about the ocean"), false);
  assert.equal(await admits("trySummarizationText", "Summarize the rust language"), false);
  assert.equal(await admits("tryTextRewrite", "Make this more formal"), false);
  assert.equal(await admits("tryStatistics", "What is the mean of my test scores?"), false);
  assert.equal(await admits("tryStatistics", "Is this number prime?"), false);
  assert.equal(await admits("tryWordProblem", "How many apples are left if I eat some?"), false);
  assert.equal(await admits("tryArithmetic", "Explain how long division works"), false);
  assert.equal(await admits("tryCompoundInterest", "How does compound interest work when you invest?"), false);
  assert.equal(await admits("tryNumberConstraintReasoning", "Guess the hidden number I am thinking of"), false);
  assert.equal(await admits("tryUnitConversion", "Convert this recipe to metric units"), false);
  assert.equal(await admits("tryCalendarReasoning", "Explain how a calendar works."), false);
});

test("the structural evidence admits", async () => {
  assert.equal(await admits("tryTerminalCommand", "find . -name '*.log' -size +10M"), true);
  assert.equal(await admits("trySoftwareProjectRequest", "Build a web app for tracking habits"), true);
  assert.equal(await admits("tryPageQueryText", "What command builds the jar?\nRun kotlinc hello.kt -include-runtime -d hello.jar."), true);
  assert.equal(await admits("tryJavaScriptExecution", "Run this JavaScript: console.log(1 + 2)"), true);
  assert.equal(await admits("tryIncompatibleUnits", "How many meters are in a kilobyte?"), true);
  assert.equal(await admits("tryCodeDebugging", "Find the bug: def average(xs): return sum(xs) / len(xs)"), true);
  assert.equal(await admits("tryCodeExplanation", "Explain this code: print(sum(range(10)))"), true);
  assert.equal(await admits("tryCodeReview", "Review this code: `const total = items.map((item) => item.price)`"), true);
  assert.equal(await admits("trySummarizationText", "Summarize: The parser reads the file. It builds a tree. The tree is checked."), true);
  assert.equal(await admits("tryTextRewrite", "Make this formal: hey can you send me the file"), true);
  assert.equal(await admits("tryStatistics", "What is the mean of 3, 5 and 7?"), true);
  assert.equal(await admits("tryStatistics", "Is 97 a prime number?"), true);
  assert.equal(await admits("tryWordProblem", "Tom has 5 apples and gets 3 more. How many apples does he have?"), true);
  assert.equal(await admits("tryArithmetic", "What is 2 + 2?"), true);
  assert.equal(await admits("tryArithmetic", "what dollar exchange rate do you use for calculations?"), true);
  assert.equal(await admits("tryCompoundInterest", "If I invest $1000 at 8% annual interest compounded monthly for 5 years, how much will I have?"), true);
  assert.equal(await admits("tryCompoundInterest", "convert the final amount to EUR using current exchange rates from the web."), true);
  assert.equal(await admits("tryNumberConstraintReasoning", "Я загадал число больше 1 но меньше 3. что это за число?"), true);
  assert.equal(await admits("tryUnitConversion", "How many meters are in 3 km?"), true);
  assert.equal(await admits("tryCalendarReasoning", "What day comes after Monday?"), true);
  assert.equal(await admits("tryCalendarReasoning", "What day is today?"), true);
  assert.equal(await admits("tryCalendarReasoning", "What day of the week was 2024-02-29?"), true);
  assert.equal(await admits("tryCalendarReasoning", "What month is 2 months after January?"), true);
});

// The numeric rows must not change an admitted answer: each handler, run on
// its own probes, answers only where its row admits (and the one-number
// primality question still reaches statistics).
test("a numeric handler answers only where its row admits", async () => {
  await seeded;
  const answered = plain(evaluate(worker, `[
    ["tryStatistics", "Is 97 a prime number?"], ["tryStatistics", "What is the mean of my test scores?"],
    ["tryUnitConversion", "How many meters are in 3 km?"], ["tryUnitConversion", "Convert this recipe to metric units"],
    ["tryCalendarReasoning", "What day comes after Monday?"], ["tryCalendarReasoning", "Explain how a calendar works."],
  ].map(([name, prompt]) => {
    const normalized = normalizePrompt(prompt);
    const hit = self[name](prompt, normalized, detectLanguage(prompt));
    return [Boolean(hit), claimRouteAdmits(name, prompt, normalized)];
  })`));
  assert.deepEqual(answered, [[true, true], [false, false], [true, true], [false, false], [true, true], [false, false]]);
});

test("a handler with no claim row is admitted as before", async () => {
  assert.equal(await admits("tryConceptLookup", "What is a monad?"), true);
});
