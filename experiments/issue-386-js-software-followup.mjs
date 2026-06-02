// Issue #386 — parity for the software-project follow-up detector.
//
// `detectSoftwareFollowUp` in src/web/formal_ai_worker.js must recognise which
// follow-up a prompt evidences by *meaning* (the software_followup_* roles in
// data/seed/meanings-software-project.lino), not a hardcoded marker table —
// mirroring `follow_up_kind` in src/solver_handlers/software_project_followup.rs
// and its multilingual unit test. Verification outranks execution outranks
// demonstration, and the surface words work in every supported language.
// Run: `node experiments/issue-386-js-software-followup.mjs`.

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

const kindOf = (prompt) => {
  const detected = sandbox.detectSoftwareFollowUp(prompt, sandbox.normalizePrompt(prompt));
  return detected ? detected.kind : null;
};

console.log("=== follow-up kind by meaning, every supported language ===");
// Verification — mirrors software_project_followup_detects_verbs_across_supported_languages.
for (const [lang, prompt] of [
  ["en", "test it on wikipedia.org and show me the top 10 most frequent words"],
  ["ru", "теперь протестируй его на wikipedia.org"],
  ["hi", "अब इसका परीक्षण करो"],
  ["zh", "现在测试它"],
]) {
  check(`verification (${lang}): ${prompt}`, kindOf(prompt) === "verification", kindOf(prompt));
}

// Execution.
for (const [lang, prompt] of [
  ["en", "run it now"],
  ["ru", "запусти его"],
  ["hi", "इसे चलाओ"],
  ["zh", "运行它"],
]) {
  check(`execution (${lang}): ${prompt}`, kindOf(prompt) === "execution", kindOf(prompt));
}

// Demonstration.
for (const [lang, prompt] of [
  ["en", "demo it for me"],
  ["ru", "покажи результат"],
  ["hi", "इसे दिखाओ"],
  ["zh", "显示结果"],
]) {
  check(`demonstration (${lang}): ${prompt}`, kindOf(prompt) === "demonstration", kindOf(prompt));
}

console.log("\n=== precedence: verification outranks execution outranks demonstration ===");
check("test + run -> verification", kindOf("test it and run it") === "verification", kindOf("test it and run it"));
check("run + show -> execution", kindOf("run it and show me the result") === "execution", kindOf("run it and show me the result"));

console.log("\n=== unrelated prompts do not trip the detector ===");
for (const prompt of ["what is the capital of France", "напиши стихотворение про осень"]) {
  check(`no follow-up: ${prompt}`, kindOf(prompt) === null, String(kindOf(prompt)));
}

console.log("\n" + (fail.length ? "FAILURES: " + fail.join(", ") : "ALL CHECKS PASSED"));
process.exit(fail.length ? 1 : 0);
