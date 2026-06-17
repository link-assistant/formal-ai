// Issue #513 (visible fix for #511): verify the browser worker recognizes a
// terminal-command request and returns an `agent_suggestion` intent instead of
// the `unknown` fallback, matching the Rust solver (`src/solver_terminal.rs`).
//
// Run with: node experiments/issue-513-terminal-intent-parity.mjs

import { readFileSync } from "node:fs";
import vm from "node:vm";

const source = readFileSync(
  new URL("../src/web/formal_ai_worker.js", import.meta.url),
  "utf8",
);

const sandbox = {
  self: { location: { search: "" } },
  importScripts: () => {
    throw new Error("no importScripts in node harness");
  },
  postMessage: () => {},
  console,
  TextEncoder,
  TextDecoder,
  WebAssembly,
  fetch: () => Promise.reject(new Error("offline")),
  setTimeout,
  clearTimeout,
};
sandbox.globalThis = sandbox;
vm.createContext(sandbox);
vm.runInContext(source, sandbox, { filename: "formal_ai_worker.js" });

let failures = 0;
function check(label, condition, detail) {
  if (condition) {
    console.log(`ok   ${label}`);
  } else {
    failures += 1;
    console.error(`FAIL ${label}${detail ? `\n     ${detail}` : ""}`);
  }
}

const { tryTerminalCommand, detectTerminalCommand } = sandbox;

// Detection unit checks (mirrors the Rust module's unit tests).
check(
  "detects ru terminal request",
  detectTerminalCommand("Выполни `ls ~` в терминале") === "ls ~",
  detectTerminalCommand("Выполни `ls ~` в терминале"),
);
check(
  "detects en terminal request",
  detectTerminalCommand("run `ls ~` in terminal") === "ls ~",
  detectTerminalCommand("run `ls ~` in terminal"),
);
check(
  "detects leading shell token",
  detectTerminalCommand("git status") === "git status",
  detectTerminalCommand("git status"),
);
check(
  "ignores plain prose",
  detectTerminalCommand("run a marathon next year") === null,
  detectTerminalCommand("run a marathon next year"),
);

// Handler return-shape checks.
const ru = tryTerminalCommand("Выполни `ls ~` в терминале", "ru", { agentMode: false });
check("ru intent is agent_suggestion", ru && ru.intent === "agent_suggestion", ru && ru.intent);
check("ru names the command", ru && ru.content.includes("ls ~"), ru && ru.content);
check(
  "ru offers to switch + grant shell when agent off",
  ru && ru.content.includes("shell") && ru.content.includes("Agent"),
  ru && ru.content,
);

const en = tryTerminalCommand("run `ls ~` in terminal", "en", { agentMode: false });
check("en intent is agent_suggestion", en && en.intent === "agent_suggestion", en && en.intent);
check(
  "en explains agent mode",
  en && en.content.toLowerCase().includes("agent mode"),
  en && en.content,
);

const onState = tryTerminalCommand("run `ls ~` in terminal", "en", { agentMode: true });
check(
  "agent-on variant mentions grant rather than switching",
  onState && onState.content.includes("Agent mode is on"),
  onState && onState.content,
);

const miss = tryTerminalCommand("what is the capital of France?", "en", {});
check("plain question returns null", miss === null, JSON.stringify(miss));

if (failures > 0) {
  console.error(`\n${failures} check(s) failed`);
  process.exit(1);
}
console.log("\nAll issue-513 worker terminal-intent checks passed");
