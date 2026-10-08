// Issue #918 (E71, R914-6), browser root, second migration file: the
// `diagnostic` prelude row and the `coreference` row read only seed data in
// both runtimes. The native pins are
// rust/tests/unit/issue_918_handler_rules_batch_2.rs.
//
// `diagnostic`: the per-message marker is the `diagnostic_marker` table of
// data/seed/handler-rules.lino (the Rust prelude tested a literal). The worker
// had no twin, so a marked prompt fell through to the unknown answer; it now
// solves the prompt without the marker and appends the evidence and trace
// lines under it, as rust/src/meta_method_dispatch.rs `try_diagnostic` does.
//
// `coreference`: the pronouns, antecedents and bodies are data/seed/coreference.lino.
// The worker read only the last user turn and answered only through a seeded
// fact, so "Why is it safer than C?" after "I love Rust." was unknown; it now
// resolves the nearest earlier user turn naming an antecedent and answers the
// antecedent's seeded body, as the native row does.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt, history = []) {
  await ready;
  return worker.solve(prompt, history, {}, {}, [], {});
}

const RUST_BODY = "`it` resolves to Rust from your prior turn. Compared with C, Rust adds ownership, borrowing, and stronger compile-time checks so memory-safety errors are caught before the program runs while retaining native-code performance.";

const turn = (role, content) => ({ role, content });

test("the diagnostic marker is a seed table row, not a literal in code", async () => {
  await ready;
  assert.deepEqual(plain(evaluate(worker, 'handlerRulesTableKeys("diagnostic_marker")')), ["[diagnostic]"]);
});

test("a marked prompt answers the unmarked answer with its evidence and trace under the marker", async () => {
  const inner = await solve("What is 2 + 2?");
  const marked = await solve("[diagnostic] What is 2 + 2?");
  const evidence = inner.evidence.map((link) => `evidence: ${link}`).join("\n");
  assert.equal(marked.intent, inner.intent);
  assert.equal(marked.content, `${inner.content}\n\n[diagnostic]\n${evidence}\ntrace: ${inner.intent}\n`);
  assert.equal(marked.evidence[0], "diagnostic_mode:active");
});

test("only the exact seeded marker is recognized, so an upper-case one never re-enters itself", async () => {
  const response = await solve("[DIAGNOSTIC] What is 2 + 2?");
  assert.ok(!response.content.includes("\n\n[diagnostic]\n"), response.content);
});

test("a pronoun resolves to the nearest earlier antecedent and answers its seeded body", async () => {
  const history = [
    turn("user", "I love Rust."),
    turn("assistant", "Rust is a systems programming language."),
    turn("user", "The weather is pleasant."),
    turn("assistant", "It is."),
  ];
  const response = await solve("Why is it safer than C?", history);
  assert.equal(response.intent, "coreference_rust");
  assert.equal(response.content, RUST_BODY);
  for (const link of ["coreference:resolved:it=Rust", "wikidata:Q575650", "response:coreference"]) {
    assert.ok(response.evidence.includes(link), `${link} in ${response.evidence}`);
  }
});

test("a pronoun with no antecedent in any earlier user turn is not a coreference", async () => {
  const response = await solve("Why is it safer than C?", [turn("user", "The weather is pleasant."), turn("assistant", "It is.")]);
  assert.ok(!String(response.intent).startsWith("coreference"), response.intent);
});

// `nl_tool`: the worker had no twin, so an explicit tool call answered unknown
// (the calculator) or ran the general web search over the request's words. It
// now reads the same roles and seeded `nl_tool_*` responses as
// rust/src/solver_handlers/natural_language_tools.rs: refused outside agent
// mode exactly as natively, the browser calculator in agent mode, and an
// explicit cannot-run-here answer, in the prompt's language, for the tools the
// browser has no executor for.
async function solveAgent(prompt) {
  await ready;
  return worker.solve(prompt, [], { agentMode: true }, {}, [], {});
}

const AGENT_MODE_REFUSAL_EN = "Execution status: refused. Natural-language tool calls require explicit agent mode before `tool:calculator` can run.";
const AGENT_MODE_REFUSAL_RU = "Статус выполнения: отказ. Вызовы инструментов на естественном языке требуют явно включённого режима агента, прежде чем `tool:calculator` сможет запуститься.";

test("a tool call outside agent mode is refused with the native seeded sentence", async () => {
  for (const [prompt, expected] of [
    ["Call the calculator API with `2 + 2`", AGENT_MODE_REFUSAL_EN],
    ["Вызови инструмент калькулятор с `2 + 2`", AGENT_MODE_REFUSAL_RU],
  ]) {
    const response = await solve(prompt);
    assert.equal(response.intent, "tool_call_refused", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

test("an allowed calculator call renders the native report", async () => {
  const response = await solveAgent("Call the calculator API with `2 + 2`");
  assert.equal(response.intent, "natural_language_api_call");
  assert.equal(response.content, "Execution status: executed.\nTool call: calculator\nInput: `2 + 2`\nResult: 4");
});

test("a tool the browser cannot run is refused explicitly in the prompt language", async () => {
  for (const [prompt, expected] of [
    ["Call the web_search API with query `Rust ownership`", "Execution status: refused. This browser session has no executor for the `web_search` tool, so the call cannot run here. Ask the native Formal AI (the CLI or the server) in agent mode to run it."],
    ["Call the local_shell tool with `ls`", "Execution status: refused. This browser session has no executor for the `local_shell` tool, so the call cannot run here. Ask the native Formal AI (the CLI or the server) in agent mode to run it."],
  ]) {
    const response = await solveAgent(prompt);
    assert.equal(response.intent, "tool_call_refused", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

// `installation_conversion`: the request cues, source and target markers, prose
// function words, verb-to-step map and project marker are the installation_*
// tables and policy of data/seed/handler-rules.lino, and every sentence a
// seeded installation_* response, read here as natively; the native pin of the
// same prompt is in rust/tests/unit/issue_918_handler_rules_batch_2.rs.
test("an installation guide converts to both scripts from the seed tables", async () => {
  const response = await solve("Convert this installation guide into both sh and PowerShell scripts:\n- `make`\n- `./app --help`\n- `deno`");
  assert.equal(response.intent, "installation_conversion");
  assert.equal(response.content.split("\n")[0], "Converted installation instructions for the project.");
  const steps = "# Build the project\nmake\n# Verify the installation\n./app --help\n# Run deno\ndeno\n```";
  assert.equal(
    response.content.slice(response.content.indexOf("Bash script:")),
    `Bash script:\n\`\`\`bash\n#!/usr/bin/env bash\nset -euo pipefail\n\n${steps}\n\nPowerShell script:\n\`\`\`powershell\n$ErrorActionPreference = 'Stop'\n\n${steps}`,
  );
});

// `translation`: the gap answers are the seeded translation_gap_* responses,
// byte-identical to the native answers pinned by
// rust/tests/unit/specification/translation_via_links.rs.
test("a translation gap answers the seeded wording", async () => {
  for (const [prompt, expected] of [
    ["Translate \"zzqxqv\" to Russian", "I could not translate \"zzqxqv\" from en to ru with the available formalization data. I recorded this as a translation gap for follow-up."],
    ["Translate to Russian", "I could not identify a source phrase to translate from en to ru."],
  ]) {
    const response = await solve(prompt);
    assert.equal(response.intent, "translate_en_to_ru", prompt);
    assert.equal(response.content, expected, prompt);
  }
});
