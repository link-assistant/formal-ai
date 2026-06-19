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

test("the default desktop agent provider is the hermetic in-process provider", () => {
  const provider = createAgentProvider({});
  assert.equal(provider.type, "in-process");
  assert.equal(provider.status().hermetic, true);
});

test("in-process provider executes a granted read-only shell command through the local gate", async () => {
  const calls = [];
  const router = createToolRouter({
    dockerAvailable: () => true,
    runInSandbox: async (spec) => {
      calls.push(spec);
      return { exitCode: 0, output: "Desktop\nDocuments\n", logPath: "/tmp/agent.log" };
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
  assert.equal(result.servedBy, "box-dind");
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

test("commander provider defaults to the org-owned agent backend through agent-commander", async () => {
  const calls = [];
  const provider = createAgentProvider({
    type: "commander",
    commanderCommand: "start-agent",
    containerName: "formal-ai-agent-test",
    workingDirectory: "/workspace",
    env: {
      PATH: "/usr/bin",
      ANTHROPIC_API_KEY: "host-anthropic-token",
      OPENAI_API_KEY: "host-openai-token",
    },
    processRunner: async (command, args, options) => {
      calls.push({ command, args, options });
      return {
        code: 0,
        stdout: '{"type":"assistant","content":"ok"}\n',
        stderr: "",
      };
    },
  });

  const result = await provider.run({
    mode: "agent",
    prompt: "Run `ls ~` in the terminal",
    command: "ls ~",
    agentProvider: {
      apiBase: "http://127.0.0.1:19191",
      openAiBaseUrl: "http://127.0.0.1:19191/v1",
      model: "formal-symbolic-production",
    },
  });

  assert.equal(result.ok, true);
  assert.equal(result.provider, "commander");
  assert.equal(calls.length, 1);
  assert.equal(calls[0].command, "start-agent");
  assert.deepEqual(calls[0].args.slice(0, 4), ["--tool", "agent", "--working-directory", "/workspace"]);
  assert.ok(!calls[0].args.includes("--read-only"), "--tool agent rejects --read-only upstream");
  assert.ok(calls[0].args.includes("--approve-each"), "desktop gate enforces read-only for the agent backend");
  assert.ok(calls[0].args.includes("--isolation"));
  assert.ok(calls[0].args.includes("docker"));
  assert.ok(calls[0].args.includes("--container-name"));
  assert.ok(calls[0].args.includes("formal-ai-agent-test"));
  assert.ok(calls[0].args.includes("OPENAI_BASE_URL=http://127.0.0.1:19191/v1"));
  assert.equal(calls[0].options.env.OPENAI_BASE_URL, "http://127.0.0.1:19191/v1");
  assert.equal(calls[0].options.env.OPENAI_API_KEY, "formal-ai-local");
  assert.equal(Object.hasOwn(calls[0].options.env, "ANTHROPIC_API_KEY"), false);
  assert.deepEqual(result.events, [{ type: "assistant", content: "ok" }]);
  assert.equal(result.answer.intent, "agent_cli_turn");
  assert.equal(result.answer.content, "ok");
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

test("commander provider maps read-only mode for non-agent backends to read-only", () => {
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
  assert.equal(restrictionFromDesktopGrants({ all: false }), "plan-only");
  assert.equal(restrictionFromDesktopGrants({ read_local_file: true }), "read-only");
  assert.equal(restrictionFromDesktopGrants({ shell: true }), "approve-each");
  assert.equal(restrictionFromDesktopGrants({ all: true }), "approve-each");

  const planOnlyArgs = buildCommanderArgs({
    mode: "agent",
    prompt: "Create a file",
    grants: { all: false, shell: false },
  });
  assert.ok(planOnlyArgs.includes("--plan-only"));
  assert.ok(!planOnlyArgs.includes("--approve-each"));
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
