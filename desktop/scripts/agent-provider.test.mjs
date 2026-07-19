import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { createToolRouter } = require("../lib/tool-router.cjs");
const {
  buildCommanderArgs,
  commanderControllerOptions,
  createAgentProvider,
  directHostSpawnViolations,
  isReadOnlyShellCommand,
  restrictionFromDesktopGrants,
} = require("../lib/agent-provider.cjs");
const {
  parseNdjsonEvents,
  agentEventsToChatAnswer,
} = require("../lib/agent-chat-adapter.cjs");

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopDir = path.resolve(scriptDir, "..");
const ignoredStaticGuardDirs = new Set(["node_modules", "dist", "release"]);

function collectDesktopSourceFiles(dir) {
  const files = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (!ignoredStaticGuardDirs.has(entry.name)) {
        files.push(...collectDesktopSourceFiles(path.join(dir, entry.name)));
      }
      continue;
    }
    if (entry.isFile() && [".cjs", ".js", ".mjs"].includes(path.extname(entry.name))) {
      files.push(path.join(dir, entry.name));
    }
  }
  return files;
}

test("agent NDJSON events map onto the existing chat answer contract", () => {
  const fixture = fs.readFileSync(
    path.join(scriptDir, "fixtures", "issue-518-agent.ndjson"),
    "utf8",
  );
  const events = parseNdjsonEvents(fixture);
  const answer = agentEventsToChatAnswer(events, {
    provider: "commander",
    status: "ok",
  });

  assert.equal(events.length, 6);
  assert.equal(answer.intent, "agent_cli_error");
  assert.equal(answer.source, "agent_provider");
  assert.match(answer.content, /I will inspect the home directory/);
  assert.match(answer.content, /Desktop and Documents are present/);
  assert.match(answer.content, /Network lookup skipped in fixture/);
  assert.ok(answer.evidence.includes("agent_provider:ndjson"));
  assert.ok(answer.evidence.includes("agent_events:6"));
  assert.ok(answer.evidence.includes("provider:commander"));

  const stepNames = answer.steps.map((step) => step.step);
  assert.ok(stepNames.includes("agent_text"));
  assert.ok(stepNames.includes("agent_tool_start"));
  assert.ok(stepNames.includes("agent_tool_result"));
  assert.ok(stepNames.includes("agent_error"));

  assert.equal(answer.toolCalls.length, 1);
  assert.equal(answer.toolCalls[0].tool, "shell");
  assert.equal(answer.toolCalls[0].inputs.command, "ls ~");
  assert.match(answer.toolCalls[0].outputs.stdout, /Desktop/);
  assert.equal(answer.toolCalls[0].outputs.exitCode, 0);
});

test("Agent lifecycle logs stay out of chat while nested text parts render", () => {
  const answer = agentEventsToChatAnswer([
    { type: "config", message: "Agent configuration resolved" },
    { type: "log", message: "loading provider" },
    { type: "text", part: { type: "text", text: "visible answer" } },
    { type: "session_idle", message: "done" },
  ]);

  assert.equal(answer.content, "visible answer");
  assert.doesNotMatch(answer.content, /configuration|loading|done/);
});

test("the default desktop agent provider is the hermetic in-process provider", () => {
  const provider = createAgentProvider({});
  assert.equal(provider.type, "in-process");
  assert.equal(provider.status().hermetic, true);
});

test("in-process provider executes a granted read-only shell command through the local gate", async () => {
  const calls = [];
  const router = createToolRouter({
    dockerAvailable: () => {
      throw new Error("read-only shell command must not probe Docker by default");
    },
    runOnHost: async (spec) => {
      calls.push(spec);
      return {
        exitCode: 0,
        output: "Desktop\nDocuments\n",
        stdout: "Desktop\nDocuments\n",
        stderr: "",
        logPath: "/tmp/agent.log",
      };
    },
    runInSandbox: async () => {
      throw new Error("read-only shell command must not run in Docker by default");
    },
  });
  router.setGrants({ shell: true });

  const provider = createAgentProvider({ type: "in-process", toolRouter: router });
  const result = await provider.run({
    mode: "agent",
    prompt: "Run `ls ~` in the terminal",
    command: "ls ~",
  });

  assert.equal(result.ok, true);
  assert.equal(result.provider, "in-process");
  assert.equal(result.executed, true);
  assert.equal(result.command, "ls ~");
  assert.equal(result.servedBy, "host-shell");
  assert.equal(calls.length, 1);
  assert.equal(calls[0].tool, "shell");
  assert.equal(calls[0].command, "ls ~");
  assert.equal(result.events[0].type, "tool_result");
  assert.equal(result.answer.intent, "agent_cli_turn");
  assert.match(result.answer.content, /Desktop/);
  assert.equal(result.answer.toolCalls[0].tool, "shell");
});

test("in-process provider drives the existing formal-ai agentic loop for tasks", async () => {
  const calls = [];
  const provider = createAgentProvider({
    type: "in-process",
    formalAiCommand: "/opt/formal-ai/bin/formal-ai",
    workingDirectory: "/workspace",
    env: {
      PATH: "/usr/bin",
      OPENAI_API_KEY: "host-openai-token",
      ANTHROPIC_API_KEY: "host-anthropic-token",
    },
    existsSync: () => false,
    processRunner: async (command, args, options) => {
      calls.push({ command, args, options });
      return {
        code: 0,
        stdout: "finished offline agentic task\n",
        stderr: "",
      };
    },
  });

  const result = await provider.run({
    mode: "agent",
    prompt: "Formalize a short text into Links Notation",
  });

  assert.equal(result.ok, true);
  assert.equal(result.provider, "in-process");
  assert.equal(result.runner, "configured formal-ai");
  assert.equal(result.body, "finished offline agentic task");
  assert.equal(result.answer.intent, "agent_cli_turn");
  assert.equal(result.answer.content, "finished offline agentic task");
  assert.equal(calls.length, 1);
  assert.equal(calls[0].command, "/opt/formal-ai/bin/formal-ai");
  assert.deepEqual(calls[0].args, [
    "agent",
    "--task",
    "Formalize a short text into Links Notation",
  ]);
  assert.equal(calls[0].options.cwd, "/workspace");
  assert.equal(calls[0].options.env.FORMAL_AI_AGENT_PROVIDER, "in-process");
  assert.equal(Object.hasOwn(calls[0].options.env, "OPENAI_API_KEY"), false);
  assert.equal(Object.hasOwn(calls[0].options.env, "ANTHROPIC_API_KEY"), false);
});

test("commander provider streams the org-owned agent backend through the agent-commander JS API", async () => {
  const calls = [];
  const streamed = [];
  const provider = createAgentProvider({
    type: "commander",
    workingDirectory: "/workspace",
    onEvent: (event) => streamed.push(event),
    agentFactory: (controllerOptions) => {
      calls.push({ controllerOptions });
      return {
        async start(startOptions) {
          calls.push({ startOptions });
          startOptions.onMessage({ type: "assistant", content: "ok" });
        },
        async stop() {
          calls.push({ stop: true });
          return {
            exitCode: 0,
            output: { plain: '{"type":"assistant","content":"ok"}\n', parsed: [] },
            sessionId: "agent-session-1",
            metadata: { sessionId: "agent-session-1" },
          };
        },
      };
    },
  });

  const result = await provider.run({
    mode: "agent",
    prompt: "Run `ls ~` in the terminal",
    command: "ls ~",
    agentProvider: {
      apiBase: "http://127.0.0.1:19191",
      model: "formal-ai",
    },
    sessionKey: "conversation-1",
  });

  assert.equal(result.ok, true);
  assert.equal(result.provider, "commander");
  assert.equal(calls[0].controllerOptions.tool, "agent");
  assert.equal(calls[0].controllerOptions.workingDirectory, "/workspace");
  assert.equal(calls[0].controllerOptions.isolation, "none");
  assert.equal(calls[0].controllerOptions.model, "formalai/formal-ai");
  assert.equal(calls[0].controllerOptions.readOnly, true);
  assert.equal(calls[1].startOptions.attached, false);
  assert.equal(calls[2].stop, true);
  const agentConfig = JSON.parse(calls[0].controllerOptions.toolOptions.extraEnv.LINK_ASSISTANT_AGENT_CONFIG_CONTENT);
  assert.equal(agentConfig.provider.formalai.options.baseURL, "http://127.0.0.1:19191/api/openai/v1");
  assert.deepEqual(streamed, [{ type: "assistant", content: "ok" }]);
  assert.deepEqual(result.events, [{ type: "assistant", content: "ok" }]);
  assert.equal(result.sessionId, "agent-session-1");
  assert.equal(result.answer.intent, "agent_cli_turn");
  assert.equal(result.answer.content, "ok");
});

test("commander controller options point Codex and Claude at the same local server", () => {
  const codex = commanderControllerOptions({
    commanderTool: "codex",
    prompt: "hello",
    agentProvider: { apiBase: "http://127.0.0.1:19191", model: "formal-ai" },
  });
  assert.equal(codex.isolation, "none");
  assert.ok(codex.toolOptions.extraArgs.includes('model_providers.formalai.base_url="http://127.0.0.1:19191/api/openai/v1"'));
  assert.equal(codex.toolOptions.extraEnv.FORMAL_AI_API_KEY, "formal-ai-local");

  const writableCodex = commanderControllerOptions({
    commanderTool: "codex",
    prompt: "edit a file",
    grants: { shell: true },
    agentProvider: { apiBase: "http://127.0.0.1:19191", model: "formal-ai" },
  });
  assert.equal(writableCodex.approveEach, false);
  assert.equal(writableCodex.toolOptions.sandboxMode, "workspace-write");
  assert.equal(writableCodex.toolOptions.approvalMode, "never");

  const claude = commanderControllerOptions({
    commanderTool: "claude",
    prompt: "hello",
    agentProvider: { apiBase: "http://127.0.0.1:19191", model: "formal-ai" },
  });
  assert.equal(claude.toolOptions.extraEnv.ANTHROPIC_BASE_URL, "http://127.0.0.1:19191/api/anthropic");
  assert.equal(claude.toolOptions.extraEnv.ANTHROPIC_API_KEY, "formal-ai-local");
});

test("commander provider maps non-read-only agent mode to approve-each", () => {
  const args = buildCommanderArgs({
    mode: "agent",
    prompt: "Create a file",
    command: "touch demo.txt",
    agentProvider: { apiBase: "http://127.0.0.1:19999" },
  });
  assert.ok(args.includes("--approve-each"));
  assert.ok(!args.includes("--read-only"));
});

test("commander provider maps read-only mode for all backends to read-only", () => {
  const agentArgs = buildCommanderArgs({
    mode: "agent",
    prompt: "List files",
    command: "ls ~",
    agentProvider: { apiBase: "http://127.0.0.1:19999" },
  });
  assert.ok(agentArgs.includes("--read-only"));
  assert.ok(!agentArgs.includes("--approve-each"));

  const args = buildCommanderArgs({
    mode: "agent",
    commanderTool: "test-tool",
    prompt: "List files",
    command: "ls ~",
    agentProvider: { apiBase: "http://127.0.0.1:19999" },
  });
  assert.ok(args.includes("--read-only"));
  assert.ok(!args.includes("--approve-each"));
});

test("commander provider maps desktop grants to coarse execution flags", () => {
  assert.equal(restrictionFromDesktopGrants(), "");
  assert.equal(restrictionFromDesktopGrants({ all: false }), "read-only");
  assert.equal(restrictionFromDesktopGrants({ read_local_file: true }), "read-only");
  assert.equal(restrictionFromDesktopGrants({ shell: true }), "approve-each");
  assert.equal(restrictionFromDesktopGrants({ all: true }), "approve-each");

  const readOnlyArgs = buildCommanderArgs({
    mode: "agent",
    prompt: "Create a file",
    grants: { all: false, shell: false },
  });
  assert.ok(readOnlyArgs.includes("--read-only"));
  assert.ok(!readOnlyArgs.includes("--approve-each"));
});

test("read-only shell command detection covers the first terminal journey", () => {
  assert.equal(isReadOnlyShellCommand("ls ~"), true);
  assert.equal(isReadOnlyShellCommand("pwd"), true);
  assert.equal(isReadOnlyShellCommand("git status --short"), true);
  assert.equal(isReadOnlyShellCommand("rm -rf demo"), false);
});

test("desktop code does not spawn host agent, claude, or codex binaries directly", () => {
  const violations = [];
  for (const filePath of collectDesktopSourceFiles(desktopDir)) {
    const relative = path.relative(desktopDir, filePath);
    const text = fs.readFileSync(filePath, "utf8");
    for (const violation of directHostSpawnViolations(text)) {
      violations.push(`${relative}:${violation.index}:${violation.text}`);
    }
  }
  assert.deepEqual(violations, []);
});
