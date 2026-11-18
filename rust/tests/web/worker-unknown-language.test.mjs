import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
const host = new WorkerHost();

test("unchanged Persian request matches the entire native authored answer", async () => {
  const source = fs.readFileSync("rust/tests/unit/specification/multilingual.rs", "utf8");
  const region = source.slice(source.indexOf("fn unknown_language_prompts_fall_back"));
  const literal = region.match(/response\.answer,\s*("(?:[^"\\]|\\.)*")/u)[1];
  const answer = await host.solve("لطفاً سلام بگو");
  assert.equal(answer.content, JSON.parse(literal));
  assert.ok(answer.evidence.includes("language:unknown"));
  for (const source of ["link_memory", "public_knowledge_cache", "source_cache"]) {
    assert.ok(answer.rawSolverEvents.some((event) => event.kind === "reasoning:gather_attempt" && event.payload === source + ":لطفاً سلام بگو"));
    assert.ok(answer.rawSolverEvents.some((event) => event.kind === "reasoning:gather_result" && event.payload === source + ":miss"));
  }
});

test("held-out Arabic script keeps the full unresolved focus and real failure invitation", async () => {
  const answer = await host.solve("هذه تعليمة غير معروفة");
  const quote = String.fromCharCode(96);
  assert.ok(answer.content.includes(quote + "هذه تعليمة غير معروفة" + quote));
  assert.ok(answer.content.includes("Reply " + quote + "Report issue" + quote));
});

test("empty and supported-language prompts retain their existing routes", async () => {
  assert.equal(await host.run('solverUnknownLanguage("", "unknown", [], [], {})'), null);
  assert.equal(await host.run('solverUnknownLanguage("Hello", "en", [], [], {})'), null);
  assert.equal((await host.solve("Hello")).intent, "greeting");
});

test("matching unverified local text is recorded without claiming a verified answer", async () => {
  const answer = await host.run('solverUnknownLanguage(__focus, "unknown", [{ content: __focus }], [], { offline: true })', { __focus: "لطفاً سلام بگو" });
  assert.ok(answer.events.some((event) => event.payload === "link_memory:no_verified_answer"));
  assert.ok(answer.events.some((event) => event.payload === "allowed_external_api:skipped_offline"));
});

test("a localized unknown answer keeps its full seed text without an opener pool", async () => {
  const seed = await host.run('answerFor("unknown", "es")');
  assert.ok(seed.startsWith("Todavía no sé"), seed);
  const answer = await host.run('unknownAnswerWithVariation("¿Cómo funciona la fotosíntesis submarina de xyzzy?", "es")');
  assert.equal(answer, seed);
});

test("the fallback pool stays available without injecting it into another language", async () => {
  assert.deepEqual(await host.run('unknownOpenersFor("es")'), await host.run('unknownOpenersFor("en")'));
  assert.equal(await host.run('selectUnknownOpener("¿Qué es xyzzy?", "es")'), "");
  const opener = await host.run('selectUnknownOpener("a held-out unknown prompt", "en")');
  const english = await host.run('unknownOpenersFor("en")');
  assert.ok(english.includes(opener));
});


test("a detected language outside the declared answer languages keeps the entire native unresolved body", { timeout: 15000 }, async () => {
  const source = fs.readFileSync("rust/tests/unit/issue_706_any_language.rs", "utf8");
  const region = source.slice(source.indexOf("fn a_language_without_localized_openers_reports_a_gap_not_english()"));
  const prompts = [...region.matchAll(/\.answer\("([^"\n]+)"\)/gu)].map((match) => match[1]);
  assert.equal(prompts.length, 2);
  const literal = region.match(/answer\.answer,\s*("(?:[^"\\]|\\[\s\S])*")/u)[1];
  const expected = JSON.parse(literal.replace(/\\\r?\n\s*/gu, ""));
  const answer = await host.solve(prompts[1]);
  assert.equal(answer.intent, "unknown");
  assert.equal(answer.content, expected);
  assert.ok(answer.rawSolverEvents.some((event) => event.kind === "reasoning:known" && event.payload.startsWith("language=es ")));
  for (const source of ["link_memory", "public_knowledge_cache", "source_cache"]) {
    assert.ok(answer.rawSolverEvents.some((event) => event.kind === "reasoning:gather_attempt" && event.payload === source + ":" + prompts[1].slice(0, -1)));
    assert.ok(answer.rawSolverEvents.some((event) => event.kind === "reasoning:gather_result" && event.payload === source + ":miss"));
  }
});

test("declared answer languages control the fallback independently of detection rules", async () => {
  const result = await host.run(`(() => {
    const original = AGENT_INFO.supported_languages;
    try {
      AGENT_INFO.supported_languages = '("en" "held-out")';
      const supported = solverUnknownLanguage("a held-out request", "held-out", [], [], {});
      AGENT_INFO.supported_languages = '("en")';
      const unsupported = solverUnknownLanguage("a held-out request?", "held-out", [], [], { offline: true });
      return { supported, unsupported };
    } finally { AGENT_INFO.supported_languages = original; }
  })()`);
  assert.equal(result.supported, null);
  assert.ok(result.unsupported?.events.some((event) => event.payload === "missing_answer_for:a held-out request"));
  assert.ok(result.unsupported.events.some((event) => event.payload === "allowed_external_api:skipped_offline"));
});
