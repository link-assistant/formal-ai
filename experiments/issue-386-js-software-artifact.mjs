// Issue #386 — parity for the software-project artifact/action detector.
//
// `detectSoftwareAction` and `detectSoftwareArtifact` in
// src/web/formal_ai_worker.js must recognise the authoring verb and the
// artifact kind by *meaning* (the software_authoring_action and
// software_artifact_kind roles in data/seed/meanings*.lino), not a hardcoded
// English table — mirroring action_surface_table / artifact_surface_table and
// the CJK-aware word-boundary scan in src/solver_handlers/software_project.rs.
// The verb resolves to its language-independent slug; the artifact resolves to
// its canonical English label; and a short surface like `апи` must never match
// inside the Cyrillic verb `напиши`.
// Run: `node experiments/issue-386-js-software-artifact.mjs`.

import fs from "node:fs";
import vm from "node:vm";
import { TextEncoder, TextDecoder } from "node:util";

const src = fs.readFileSync(new URL("../src/web/formal_ai_worker.js", import.meta.url), "utf8");

const sandbox = {};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.console = console;
sandbox.WebAssembly = WebAssembly;
sandbox.importScripts = () => { throw new Error("no importScripts in node"); };
sandbox.postMessage = () => {};
sandbox.setTimeout = setTimeout;
sandbox.fetch = async () => { throw new Error("no fetch"); };
sandbox.location = { search: "", origin: "http://localhost" };
sandbox.TextEncoder = TextEncoder;
sandbox.TextDecoder = TextDecoder;
sandbox.crypto = globalThis.crypto;
sandbox.URL = URL;
vm.createContext(sandbox);
vm.runInContext(src, sandbox, { filename: "formal_ai_worker.js" });

const fail = [];
function check(name, cond, extra) {
  console.log(`${cond ? "PASS" : "FAIL"}: ${name}${extra ? " :: " + extra : ""}`);
  if (!cond) fail.push(name);
}

const actionOf = (prompt) => sandbox.detectSoftwareAction(sandbox.normalizePrompt(prompt));
const artifactOf = (prompt) => {
  const match = sandbox.detectSoftwareArtifact(sandbox.normalizePrompt(prompt));
  return match ? match.label : null;
};

console.log("=== authoring verb resolves to its slug, every supported language ===");
// The reused program-request verbs (make/write/create/generate/build) and the
// new ones (implement/develop/design/scaffold) all carry their English-equal
// slug, so the slug is stable across the language the surface came from.
for (const [lang, prompt, slug] of [
  ["en", "write an application for me", "write"],
  ["en", "implement a dashboard", "implement"],
  ["en", "scaffold a library", "scaffold"],
  ["ru", "напиши приложение", "write"],
  ["ru", "реализуй библиотеку", "implement"],
  ["ru", "разработай сервис", "develop"],
  ["hi", "एक एप्लिकेशन बनाओ", "make"],
  ["zh", "开发一个网站", "develop"],
  ["zh", "写一个应用", "write"],
]) {
  check(`action (${lang}): ${prompt}`, actionOf(prompt) === slug, String(actionOf(prompt)));
}

console.log("\n=== artifact kind resolves to its canonical label, every language ===");
for (const [lang, prompt, label] of [
  ["en", "build a browser extension", "browser extension"],
  ["en", "create a command line tool", "command-line tool"],
  ["en", "make an api", "API"],
  ["ru", "создай библиотеку", "library"],
  ["ru", "напиши сайт", "website"],
  ["hi", "एक डैशबोर्ड बनाओ", "dashboard"],
  ["zh", "开发一个网站", "website"],
  ["zh", "写一个应用程序", "application"],
]) {
  check(`artifact (${lang}): ${prompt}`, artifactOf(prompt) === label, String(artifactOf(prompt)));
}

console.log("\n=== specific-before-generic via the end-boundary check, not order ===");
check("web app beats app", artifactOf("build a web app for travel") === "web app", artifactOf("build a web app for travel"));
check("application beats app", artifactOf("write an application") === "application", artifactOf("write an application"));
check("browser extension beats extension", artifactOf("build a browser extension") === "browser extension", artifactOf("build a browser extension"));

console.log("\n=== Cyrillic word boundary: апи must not match inside напиши ===");
// The factorial prompt is a write-program request with no software artifact;
// the short artifact surface `апи` (API) must not match inside `напиши`, so the
// software-project handler must decline (mirrors the Rust regression test
// russian_program_request_with_unknown_task_is_not_unknown).
const factorial = "Напиши программу на Python, которая вычисляет факториал числа";
check("no false API artifact in напиши", artifactOf(factorial) === null, String(artifactOf(factorial)));
check("software project declines factorial prompt", sandbox.formalizeSoftwareProjectRequest(factorial) === null, String(sandbox.formalizeSoftwareProjectRequest(factorial)));

console.log("\n=== unrelated prompts do not trip the detector ===");
for (const prompt of ["what is the capital of France", "what time is it in Tokyo"]) {
  check(`no software project: ${prompt}`, sandbox.formalizeSoftwareProjectRequest(prompt) === null, String(sandbox.formalizeSoftwareProjectRequest(prompt)));
}

console.log("\n=== a full multilingual request still formalizes end to end ===");
const ru = sandbox.formalizeSoftwareProjectRequest("создай расширение браузера для перевода страниц");
check("ru browser extension formalizes", ru && ru.artifact === "browser extension" && ru.action === "create", ru ? `${ru.action}/${ru.artifact}` : "null");

console.log("\n=== requirement category resolves from the lexicon, declaration order ===");
// classifySoftwareRequirement walks the software_requirement_category meanings
// in declaration order (state_tracking → … → project_behavior catch-all),
// mirroring classify_requirement in src/solver_handlers/software_project.rs.
const classify = (req, game = false) => sandbox.classifySoftwareRequirement(req, game);
for (const [req, category] of [
  ["track hp", "state_tracking"],
  ["import csv", "data_exchange"],
  ["send a weekly reminder", "automation"],
  ["validate the input", "validation"],
  ["discord integration", "integration"],
  ["dashboard view", "user_interface"],
  ["send invoice to customer", "project_behavior"],
]) {
  check(`classify: ${req}`, classify(req) === category, classify(req));
}
check("game tracker forces state_tracking", classify("send invoice to customer", true) === "state_tracking", classify("send invoice to customer", true));

console.log("\n=== requirement markers come from the lexicon, not a hardcoded list ===");
const features = sandbox.extractSoftwareFeatures(
  "Add expense tracking, import CSV, and send weekly reminders.",
);
check("three feature clauses extracted", features.length === 3, JSON.stringify(features));
const fallback = sandbox.extractSoftwareFeatures("Hello there friend.");
check(
  "no markers -> single fallback feature",
  fallback.length === 1 && fallback[0].startsWith("Capture state"),
  JSON.stringify(fallback),
);

console.log("\n" + (fail.length ? "FAILURES: " + fail.join(", ") : "ALL CHECKS PASSED"));
process.exit(fail.length ? 1 : 0);
