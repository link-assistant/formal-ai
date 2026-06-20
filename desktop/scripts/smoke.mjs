import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopDir = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(desktopDir, "..");

function read(relativePath) {
  return fs.readFileSync(path.join(desktopDir, relativePath), "utf8");
}

function readRepo(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function requireIncludes(label, text, snippets) {
  for (const snippet of snippets) {
    if (!text.includes(snippet)) {
      throw new Error(`${label} is missing ${snippet}`);
    }
  }
}

const manifest = JSON.parse(read("package.json"));
for (const script of ["dev", "build", "build:linux", "build:mac", "build:win", "smoke", "test"]) {
  if (!manifest.scripts || !manifest.scripts[script]) {
    throw new Error(`desktop package is missing npm run ${script}`);
  }
}
for (const [script, command] of Object.entries(manifest.scripts || {})) {
  if (command.includes("--config package.json")) {
    throw new Error(
      `desktop npm run ${script} must let electron-builder read the package.json build key`
    );
  }
}
if (!Array.isArray(manifest.build.files) || !manifest.build.files.includes("lib/**")) {
  throw new Error("desktop build must bundle lib/** (tool-router / memory-sync)");
}

requireIncludes("main.cjs", read("main.cjs"), [
  "BrowserWindow",
  "contextIsolation: true",
  "nodeIntegration: false",
  "formalAiDesktop:getStatus",
  "formal-ai",
  // R3/R4: the local server is opt-in (in-process is the default).
  "serverModeRequested",
  // R5d (D2): permission-gated tool routing through the local process / sandbox.
  "formalAiDesktop:invokeTool",
  "formalAiDesktop:setToolGrants",
  "formalAiDesktop:runAgentProvider",
  "createToolRouter",
  "createAgentProvider",
  "dockerIsAvailable",
  // R5c (D1): local-database sync.
  "formalAiDesktop:syncMemory",
  "createMemorySync",
  // Issue #438 (follow-up): one-click start/stop of the prepared containers.
  "formalAiDesktop:serviceStatus",
  "formalAiDesktop:startService",
  "formalAiDesktop:installAgentEnvironment",
  "formalAiDesktop:stopService",
  "createServiceControl",
  // Issue #515: entering Agent / Full Auto mode starts or reuses the local API.
  "formalAiDesktop:ensureAgentServer",
  "createLocalServerManager",
]);
requireIncludes("preload.cjs", read("preload.cjs"), [
  "contextBridge",
  "FormalAiDesktop",
  "getStatus",
  "ensureAgentServer",
  "invokeTool",
  "setToolGrants",
  "runAgentProvider",
  "syncMemory",
  "serviceStatus",
  "startService",
  "installAgentEnvironment",
  "stopService",
]);

// R5d (D2): the tool router defaults to deny, runs shell on the host by
// default, and routes code-exec to box-dind.
requireIncludes("lib/tool-router.cjs", read("lib/tool-router.cjs"), [
  "createToolRouter",
  "konard/box-dind",
  "explicit-permission",
  "default-deny",
  "http_fetch",
  "code_exec",
  "runOnHost",
  "host-shell",
]);
// R5c (D1): the memory-sync client speaks the Links-Notation memory endpoints.
requireIncludes("lib/memory-sync.cjs", read("lib/memory-sync.cjs"), [
  "createMemorySync",
  "/v1/memory/since",
  "/v1/memory/import",
]);
// Issue #438 (follow-up): the service-control module manages both prepared
// containers (Telegram bot + OpenAI-compatible server) behind one runner.
requireIncludes("lib/service-control.cjs", read("lib/service-control.cjs"), [
  "createServiceControl",
  "formal-ai-telegram",
  "formal-ai-server",
  "formal-ai-agent",
  "installAgentEnvironment",
  "agent --version",
  "start-agent --help",
  "TELEGRAM_BOT_TOKEN",
  "serve",
]);
// Issue #517 / E5: the prepared image bundles the local server binary, the Agent
// CLI, and agent-commander so the desktop install health check can verify all
// three inside the container.
requireIncludes("Dockerfile", readRepo("Dockerfile"), [
  "konard/box-dind:2.1.1",
  "apt-get install -y --no-install-recommends nodejs",
  "node --version",
  "@link-assistant/agent",
  "agent-commander",
  "agent --version",
  "start-agent --help",
]);
// Issue #515: the local-server manager owns start/reuse health logic behind an
// injectable interface so the Electron main process can be tested without
// loading Electron.
requireIncludes("lib/local-server.cjs", read("lib/local-server.cjs"), [
  "createLocalServerManager",
  "FORMAL_AI_DESKTOP_SERVER",
  "requestHealth",
  "startApiProcess",
  "/v1/chat/completions",
  "/v1/graph",
  "agentProvider",
  "local-openai-compatible",
]);
// Issue #516: the desktop agent execution seam is default-in-process and can be
// switched to the agent-commander provider without spawning host agent CLIs.
requireIncludes("lib/agent-provider.cjs", read("lib/agent-provider.cjs"), [
  "createAgentProvider",
  "in-process",
  "commander",
  "start-agent",
  "agent-commander",
  "directHostSpawnViolations",
]);
// Issue #518 / E6: agent CLI NDJSON transcripts render through the same answer,
// reasoning-step, and tool-call shape as ordinary chat responses.
requireIncludes("lib/agent-chat-adapter.cjs", read("lib/agent-chat-adapter.cjs"), [
  "agentEventsToChatAnswer",
  "tool_use",
  "tool_result",
  "agent_cli_turn",
]);

console.log("formal-ai desktop smoke checks passed");
