// Server parity for evidence links and program reports (R1013, R1015).
//
// js/server/evidence-links.mjs ports rust/src/event_log.rs `EventLog::append`
// and `build_evidence_links`; js/server/program-report.mjs ports
// rust/src/engine.rs `execution_report`. The expected strings below were
// computed by the Rust server: the prompt link the server-parity run printed,
// the evidence trail recorded in
// docs/case-studies/issue-870/agent-cli-evidence/red-regression.log, and the
// native write-program replies the server-parity corpus replays as history
// (rust/tests/fixtures/server-parity/requests.lino, `chat_learning_trace`).

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { before, test } from "node:test";

import { EventLog, buildEvidenceLinks, evidenceLink } from "../../../js/server/evidence-links.mjs";
import { executionReport, nativeProgramAnswer } from "../../../js/server/program-report.mjs";
import { REPO_ROOT, WorkerHost } from "../../../js/server/worker-host.mjs";
import { solveSymbolic, solverEvidenceLinks } from "../../../js/server/solve.mjs";

let host;
before(async () => {
  host = new WorkerHost();
  await host.boot();
});

test("the prompt link is the one the Rust server answers with", () => {
  const links = buildEvidenceLinks("What is 2 + 2?", new EventLog(), "response:calculation");
  assert.deepEqual(links, ["prompt:prompt_b888e028816d6739", "response:calculation"]);
});

test("event ids hash kind, position and payload exactly as EventLog::append", () => {
  const prompt = "Проверь какие процессы запущены на моём компьютере";
  const log = new EventLog();
  log.append("impulse", prompt);
  log.append("language", "ru");
  while (log.events.length < 38) log.append("filler", String(log.events.length));
  log.append("validation", "accepted_without_extra_constraints");
  log.append("response", "response:unknown_reasoning");
  log.append("trace:simplification", "smallest_sufficient");
  log.append("trace", "unknown");
  const links = buildEvidenceLinks(prompt, log, "response:unknown_reasoning");
  assert.equal(links[0], "prompt:prompt_3a8b7f8b62a4c8dd");
  assert.equal(links[1], "impulse:impulse_34e5fa029d92eba8");
  assert.equal(links[2], "language:ru");
  assert.deepEqual(links.slice(-4), [
    "validation:validation_7b533e4da55ec7af",
    "response:unknown_reasoning",
    "trace:simplification:trace:simplification_5e34b97eeb2b76bd",
    "trace:trace_4609a73a80331e83",
  ]);
});

test("every arm shape of build_evidence_links", () => {
  const link = (kind, payload, id = `${kind}_0000000000000000`) => evidenceLink({ kind, payload, id });
  assert.equal(link("statement_weight", "formalization:0 weight=1.000000 empty"),
    "statement_weight:formalization:0 weight=1.000000 empty");
  assert.equal(link("intent_formalization_cache", "miss impulse_72b1d40558b327fd"),
    "intent_formalization_cache:miss:impulse_72b1d40558b327fd");
  assert.equal(link("intent_formalization:kind", "unknown"), "intent_formalization:kind:unknown");
  assert.equal(link("search:local", "anything", "search:local_71adfb20bea55ac3"),
    "search:local:search:local_71adfb20bea55ac3");
  assert.equal(link("search:external", "skipped:offline"), "policy:offline");
  assert.equal(link("search:external", "ran", "search:external_1"), "search:external:search:external_1");
  assert.equal(link("source:http", "https://a.example 200 cached"), "source:http:https://a.example:200:cached");
  assert.equal(link("skill_gap", "no route for x"), "skill_gap:no_route_for_x");
  assert.equal(link("spelling_correction", "teh -> the"), "spelling_correction:teh->the");
  assert.equal(link("program_parameters", "write_program(language=rust, task=hello_world)"),
    "program_parameters:write_program(language=rust::task=hello_world)");
  assert.equal(link("release_timeline:hit", "rust 1.90"), "release_timeline:rust 1.90");
  assert.equal(link("docs_method:source", "https://docs.rs"), "source:https://docs.rs");
  assert.equal(link("fact_query:cache:miss", "whatever"), "fact_query:cache:miss");
  assert.equal(link("policy:clarify_under_ambiguity", "top=formalization:0"), "policy:clarify_under_ambiguity");
  assert.equal(link("verifiable_task:check", "sum=4"), "verifiable_task:check:sum=4");
  assert.equal(link("response", "response:calculation"), "response:calculation");
  assert.equal(link("formalization", "x", "formalization_1"), "formalization:formalization_1");
});

test("the Responses evidence projects the worker's native log", async () => {
  const result = await host.solve("What is 2 + 2?");
  const links = solverEvidenceLinks(result, []);
  assert.equal(links[0], "prompt:prompt_b888e028816d6739");
  assert.equal(links[1], `impulse:${new EventLog([{ kind: "impulse", payload: "What is 2 + 2?" }]).events[0].id}`);
  assert.equal(links[2], "language:en");
  assert.ok(links.includes("calculation:engine:link-calculator"));
  assert.ok(links.includes("intent:calculation"));
  assert.ok(links.includes("response:calculation"));
  assert.ok(links.some((entry) => entry.startsWith("trace:simplification:trace:simplification_")));
  assert.ok(!links.some((entry) => entry.startsWith("trace:impulse:")), "no browser trace in the native links");
});

const RUST_REPORT_RU = [
  "Статус выполнения: скомпилировано и запущено в среде «issue-8 local verification harness (isolated sandbox)».",
  "Check command: `rustc main.rs -o main`",
  "Run command: `./main`",
  "Вывод:",
  "```text",
  "README.md",
  "data.txt",
  "main.rs",
  "```",
  "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
].join("\n");

test("execution_report renders the recorded toolchain run", () => {
  const program = { language: "rust", checkCommand: "rustc main.rs -o main", runCommand: "./main", output: "README.md\ndata.txt\nmain.rs" };
  assert.equal(executionReport(program, "ru"), RUST_REPORT_RU);
  assert.match(executionReport({ ...program, language: "go", checkCommand: null, runCommand: "go run main.go" }, "en"),
    /^Execution status: compiled and ran in issue-8 local verification harness \(isolated sandbox\)\.\nRun command: `go run main\.go`\nOutput:\n/);
  assert.equal(nativeProgramAnswer("a\nBLOCK\nb", { ...program, block: "BLOCK", responseLanguage: "ru" }), `a\n${RUST_REPORT_RU}\nb`);
  assert.equal(nativeProgramAnswer("unchanged", undefined), "unchanged");
});

test("a follow-up program edit carries the native execution report", async () => {
  const corpus = readFileSync(path.join(REPO_ROOT, "rust/tests/fixtures/server-parity/requests.lino"), "utf8");
  const line = corpus.split("\n").find((entry) => entry.includes("Сделай сортировку результатов")).trim();
  const { messages } = JSON.parse(line.replace(/^body '/, "").replace(/'$/, ""));
  const answer = await solveSymbolic({ worker: host }, messages.at(-1).content, messages.slice(0, -1));
  const report = RUST_REPORT_RU.replace("README.md\ndata.txt\nmain.rs", "main.rs\ndata.txt\nREADME.md");
  assert.ok(answer.answer.includes(`\`\`\`\n\n${report}\n\nКак это работает:\n`), answer.answer);
  assert.ok(!answer.answer.includes("браузерная песочница"));
});
