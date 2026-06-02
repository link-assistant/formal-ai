// Issue #386 — parity for the software-project delivery/language/approval/
// game-tracker detectors in src/web/formal_ai_worker.js.
//
// detectSoftwareDeliveryMode, detectSoftwareImplementationLanguage,
// softwareApprovalGates, isGameUnitTracker, and isSoftwareApprovalPrompt must
// recognise their concepts by *meaning* (the software_delivery_mode,
// software_implementation_language, software_feature, game_tracker_domain,
// game_tracker_mechanic, software_step_granularity, software_bash_command, and
// software_approval_trigger roles in data/seed/meanings-software-project.lino),
// not a hardcoded English table — mirroring detect_delivery_mode /
// detect_implementation_language / approval_gates / is_game_unit_tracker /
// is_approval_prompt in src/solver_handlers/software_project.rs.
//
// The 22 dialogue examples below are byte-for-byte the prompts and expectations
// asserted on the Rust side in
// tests/unit/software_project.rs::software_project_dialogue_examples_…, so a
// PASS here means the browser worker classifies them identically to the solver.
// Run: `node experiments/issue-386-js-software-delivery.mjs`.

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

// prompt, artifact, delivery_mode, language, extra_gate, gameTracker.
// gameTracker is true exactly for the two examples whose Rust implementation
// needle is `mitigateDamage` (domain + combat mechanic both evidenced).
const examples = [
  ["Write an extension for Owlbear that tracks HP, Protection, Resistance, damage, and cooldowns", "extension", "code_generation", "typescript", "generated_code", true],
  ["Build a browser extension for reading progress that tracks pages and exports CSV", "browser extension", "code_generation", "typescript", "generated_code", false],
  ["Create a JavaScript Discord bot for scheduling game sessions with reminders", "bot", "code_generation", "javascript", "generated_code", false],
  ["Implement a React web app for invoices that tracks overdue payments and exports reports", "web app", "code_generation", "typescript", "generated_code", false],
  ["Make a plugin for a tabletop map that tracks unit status effects", "plugin", "code_generation", "typescript", "generated_code", false],
  ["Develop a Rust command line tool for renaming photos by date", "command-line tool", "code_generation", "rust", "generated_code", false],
  ["Generate a mobile app for habit tracking with notifications and backups", "mobile app", "code_generation", "typescript", "generated_code", false],
  ["Design a service for importing customer invoices and sending payment reminders", "service", "code_generation", "typescript", "generated_code", false],
  ["Scaffold a website for event schedules that exports calendar data", "website", "code_generation", "typescript", "generated_code", false],
  ["Create a Python API for tracking equipment status and maintenance dates", "API", "code_generation", "python", "generated_code", false],
  ["Build a bot for project reports that sends weekly notifications", "bot", "code_generation", "typescript", "generated_code", false],
  ["Make an add-on for a tabletop token that tracks hp and damage", "extension", "code_generation", "typescript", "generated_code", true],
  ["Build a Python CLI tool for importing CSV tasks and exporting weekly reports with manual instructions", "command-line tool", "manual_instructions", "python", "manual_instructions", false],
  ["Write a Python scraper that imports product prices and stores history", "scraper", "code_generation", "python", "generated_code", false],
  ["Implement a Rust library for validating configuration files", "library", "code_generation", "rust", "generated_code", false],
  ["Build an admin dashboard that filters users and exports audit logs", "dashboard", "code_generation", "typescript", "generated_code", false],
  ["Make a GitHub Action that checks changelog fragments on pull requests", "action", "code_generation", "typescript", "generated_code", false],
  ["Implement a plugin for a design tool that syncs assets and reports conflicts", "plugin", "code_generation", "typescript", "generated_code", false],
  ["Build a TypeScript SDK for uploading files with retries and progress events", "SDK", "code_generation", "typescript", "generated_code", false],
  ["Create a Telegram bot that tracks expenses and sends weekly reports", "bot", "code_generation", "typescript", "generated_code", false],
  ["Generate a command line tool with shell commands for backing up project files and validating upload status", "command-line tool", "script_generation", "typescript", "generated_script", false],
  ["Develop a web app for incident reports, run commands in WebVM, and approve each step", "web app", "immediate_execution", "typescript", "each_step", false],
];

console.log("=== 22 dialogue examples: artifact / delivery_mode / language / gate / tracker ===");
for (const [prompt, artifact, mode, language, gate, tracker] of examples) {
  const m = sandbox.formalizeSoftwareProjectRequest(prompt);
  const tag = prompt.slice(0, 42);
  if (!m) {
    check(`formalizes :: ${tag}`, false, "null");
    continue;
  }
  check(`artifact ${artifact} :: ${tag}`, m.artifact === artifact, m.artifact);
  check(`delivery ${mode} :: ${tag}`, m.deliveryMode === mode, m.deliveryMode);
  check(`language ${language} :: ${tag}`, m.implementationLanguage === language, m.implementationLanguage);
  check(`gate ${gate} :: ${tag}`, m.approvalGates.includes(gate), m.approvalGates.join(","));
  check(`task_formalization gate :: ${tag}`, m.approvalGates.includes("task_formalization"));
  check(`implementation_plan gate :: ${tag}`, m.approvalGates.includes("implementation_plan"));
  check(`gameTracker ${tracker} :: ${tag}`, m.gameTracker === tracker, String(m.gameTracker));
}

// The "each step" go-ahead in example 22 must also surface the each_step gate.
const ex22 = sandbox.formalizeSoftwareProjectRequest(
  "Develop a web app for incident reports, run commands in WebVM, and approve each step",
);
check("immediate_execution adds generated_script+bash_command", ex22 &&
  ex22.approvalGates.includes("generated_script") && ex22.approvalGates.includes("bash_command"),
  ex22 ? ex22.approvalGates.join(",") : "null");

console.log("\n=== single command (singular) must NOT trip script mode ===");
const cli = sandbox.formalizeSoftwareProjectRequest("Develop a Rust command line tool for renaming photos by date");
check("rust cli stays code_generation (singular 'command')", cli && cli.deliveryMode === "code_generation", cli ? cli.deliveryMode : "null");

console.log("\n=== game-tracker needs BOTH a domain and a mechanic ===");
check("tabletop+token + hp/damage => tracker", sandbox.isGameUnitTracker(sandbox.normalizePrompt("a tabletop token that tracks hp and damage")));
check("unit status effects (no mechanic) => not a tracker", !sandbox.isGameUnitTracker(sandbox.normalizePrompt("a tabletop map that tracks unit status effects")));
check("hp alone (no domain) => not a tracker", !sandbox.isGameUnitTracker(sandbox.normalizePrompt("a chart that tracks hp over time")));

console.log("\n=== boundary safety: short surfaces must not match inside longer words ===");
check("'hp' does not match inside 'php'", !sandbox.isGameUnitTracker(sandbox.normalizePrompt("a php dnd tool")));

console.log("\n=== approval prompt: whole-prompt go-aheads only ===");
const approvals = ["approve", "approved", "approve plan", "yes", "yes proceed", "proceed", "go ahead", "looks good", "do it", "start implementation", "generate code", "convert to code"];
for (const phrase of approvals) {
  check(`approval: ${phrase}`, sandbox.isSoftwareApprovalPrompt(sandbox.normalizePrompt(phrase)));
}
check("not approval: approve the email validation step", !sandbox.isSoftwareApprovalPrompt(sandbox.normalizePrompt("approve the email validation step")));
check("not approval: what time is it", !sandbox.isSoftwareApprovalPrompt(sandbox.normalizePrompt("what time is it")));

console.log("\n=== multilingual implementation language detection ===");
// Boundary matching needs the lexeme token verbatim — "питон" matches as a whole
// token, the inflected "питоне" would not (Rust's surface_present behaves the
// same), which is exactly the stricter, regression-proof contract we want.
check("ru питон => python", sandbox.detectSoftwareImplementationLanguage(sandbox.normalizePrompt("напиши инструмент питон")) === "python");
check("zh rust => rust", sandbox.detectSoftwareImplementationLanguage(sandbox.normalizePrompt("用 rust 写一个工具")) === "rust");

if (fail.length) {
  console.log(`\n${fail.length} CHECK(S) FAILED`);
  process.exit(1);
}
console.log("\nALL CHECKS PASSED");
