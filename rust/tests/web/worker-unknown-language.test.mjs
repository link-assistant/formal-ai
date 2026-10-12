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
