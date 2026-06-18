"use strict";

// AgentProvider seam for issue #516. The commander implementation invokes the
// `agent-commander` CLI (`start-agent`) rather than native host agent binaries.

const childProcess = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const DEFAULT_PROVIDER_TYPE = "in-process";
const COMMANDER_PROVIDER_TYPE = "commander";
const DEFAULT_COMMANDER_COMMAND = "start-agent";
const DEFAULT_COMMANDER_TOOL = "agent";
const DEFAULT_AGENT_CONTAINER = "formal-ai-agent";
const DEFAULT_MODEL = "formal-symbolic-production";
const HOST_AGENT_BINARIES = Object.freeze(["agent", "claude", "codex"]);
const PROVIDER_TYPES = Object.freeze([DEFAULT_PROVIDER_TYPE, COMMANDER_PROVIDER_TYPE]);
const READ_ONLY_DESKTOP_TOOLS = Object.freeze(["http_fetch", "url_navigate", "read_local_file"]);
const WRITE_CAPABLE_DESKTOP_TOOLS = Object.freeze(["eval_js", "code_exec", "shell"]);
const HOST_SUBSCRIPTION_ENV_KEYS = Object.freeze([
  "ANTHROPIC_API_KEY",
  "CLAUDE_API_KEY",
  "CODEX_API_KEY",
  "GEMINI_API_KEY",
  "GOOGLE_API_KEY",
  "OPENAI_API_KEY",
  "OPENAI_API_BASE",
  "OPENAI_BASE_URL",
  "FORMAL_AI_API_BEARER_TOKEN",
  "FORMAL_AI_HTTP_BEARER_TOKEN",
  "FORMAL_AI_API_TOKEN",
]);
const SAFE_COMMANDER_ENV_KEYS = Object.freeze([
  "PATH",
  "Path",
  "HOME",
  "USERPROFILE",
  "TMPDIR",
  "TMP",
  "TEMP",
  "SystemRoot",
  "WINDIR",
]);

function normalizeProviderType(value) {
  const type = String(value || "").trim().toLowerCase();
  return PROVIDER_TYPES.includes(type) ? type : DEFAULT_PROVIDER_TYPE;
}

function normalizeMode(value) {
  const mode = String(value || "").trim();
  return mode === "fullAuto" || mode === "agent" || mode === "chat" ? mode : "agent";
}

function openAiBaseUrlFrom(value) {
  const raw = String(value || "").trim();
  if (!raw) {
    return "";
  }
  return raw.replace(/\/+$/, "").endsWith("/v1") ? raw.replace(/\/+$/, "") : `${raw.replace(/\/+$/, "")}/v1`;
}

function providerConfigFrom(request = {}) {
  const explicit = request.agentProvider && typeof request.agentProvider === "object"
    ? request.agentProvider
    : {};
  const apiBase = String(request.apiBase || explicit.apiBase || "").replace(/\/+$/, "");
  const openAiBaseUrl = openAiBaseUrlFrom(request.openAiBaseUrl || explicit.openAiBaseUrl || apiBase);
  return {
    apiBase,
    openAiBaseUrl,
    model: String(request.model || explicit.model || DEFAULT_MODEL),
  };
}

function commandFromRequest(request = {}) {
  return String(
    request.command ||
      (request.input && request.input.command) ||
      (request.tool === "shell" && request.prompt) ||
      "",
  ).trim();
}

function promptFromRequest(request = {}) {
  const prompt = String(request.prompt || request.task || "").trim();
  if (prompt) {
    return prompt;
  }
  const command = commandFromRequest(request);
  if (!command) {
    return "";
  }
  return `Run the read-only terminal command and return its output:\n\n${command}`;
}

function firstCommandToken(command) {
  const trimmed = String(command || "").trim();
  if (!trimmed) {
    return "";
  }
  const token = trimmed.split(/\s+/)[0].replace(/^["']|["']$/g, "");
  return path.basename(token);
}

function isReadOnlyShellCommand(command) {
  const program = firstCommandToken(command);
  if (!program) {
    return false;
  }
  if (program === "git") {
    const action = String(command || "").trim().split(/\s+/)[1] || "";
    return ["diff", "log", "status", "show"].includes(action);
  }
  return [
    "cat",
    "file",
    "find",
    "grep",
    "head",
    "ls",
    "pwd",
    "rg",
    "stat",
    "tail",
    "wc",
  ].includes(program);
}

function spawnAndCollect(command, args, options = {}) {
  const spawn = options.spawn || childProcess.spawn;
  return new Promise((resolve) => {
    let child = null;
    try {
      child = spawn(command, args, {
        cwd: options.cwd || process.cwd(),
        env: options.env || process.env,
        stdio: ["ignore", "pipe", "pipe"],
      });
    } catch (error) {
      resolve({
        code: 1,
        stdout: "",
        stderr: error && error.message ? error.message : String(error),
      });
      return;
    }

    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.once("error", (error) => {
      resolve({
        code: 1,
        stdout,
        stderr: error && error.message ? error.message : String(error),
      });
    });
    child.once("exit", (code) => {
      resolve({ code: typeof code === "number" ? code : 1, stdout, stderr });
    });
  });
}

function normalizeRunnerResult(result = {}) {
  const code = typeof result.code === "number"
    ? result.code
    : typeof result.exitCode === "number"
      ? result.exitCode
      : 0;
  return {
    code,
    stdout: String(result.stdout || result.output || ""),
    stderr: String(result.stderr || ""),
  };
}

function packagedFormalAiPath(options = {}) {
  const platform = options.platform || process.platform;
  const resourcesPath = options.resourcesPath || process.resourcesPath || "";
  const binary = platform === "win32" ? "formal-ai.exe" : "formal-ai";
  return resourcesPath ? path.join(resourcesPath, "bin", binary) : "";
}

function inProcessEnvironment(options = {}) {
  const source = options.env || process.env;
  const env = { ...source };
  for (const key of HOST_SUBSCRIPTION_ENV_KEYS) {
    delete env[key];
  }
  env.FORMAL_AI_AGENT_PROVIDER = DEFAULT_PROVIDER_TYPE;
  return env;
}

function inProcessAgentCandidates(options = {}) {
  const repoRoot = options.workingDirectory || path.resolve(__dirname, "..", "..");
  const env = options.env || process.env;
  const existsSync = options.existsSync || fs.existsSync;
  const candidates = [];

  if (options.formalAiCommand) {
    candidates.push({
      command: String(options.formalAiCommand),
      args: ["agent"],
      cwd: repoRoot,
      label: "configured formal-ai",
    });
  }

  if (env.FORMAL_AI_DESKTOP_BINARY) {
    candidates.push({
      command: env.FORMAL_AI_DESKTOP_BINARY,
      args: ["agent"],
      cwd: repoRoot,
      label: "FORMAL_AI_DESKTOP_BINARY",
    });
  }

  const packaged = packagedFormalAiPath(options);
  if (packaged && existsSync(packaged)) {
    candidates.push({
      command: packaged,
      args: ["agent"],
      cwd: repoRoot,
      label: "bundled formal-ai",
    });
  }

  if (existsSync(path.join(repoRoot, "Cargo.toml"))) {
    candidates.push({
      command: "cargo",
      args: ["run", "--", "agent"],
      cwd: repoRoot,
      label: "cargo run",
    });
  }

  candidates.push({
    command: "formal-ai",
    args: ["agent"],
    cwd: repoRoot,
    label: "formal-ai on PATH",
  });
  return candidates;
}

async function runFormalAiAgentTask(task, request = {}, options = {}) {
  const runner = options.processRunner || spawnAndCollect;
  const transcript = request.transcript === true || options.transcript === true;
  let last = null;
  for (const candidate of inProcessAgentCandidates(options)) {
    const args = [...candidate.args, "--task", task];
    if (transcript) {
      args.push("--transcript");
    }
    const result = normalizeRunnerResult(
      await runner(candidate.command, args, {
        cwd: candidate.cwd,
        env: inProcessEnvironment(options),
      }),
    );
    if (result.code === 0) {
      return {
        ok: true,
        finalAnswer: result.stdout.trim(),
        transcript: transcript ? result.stdout : "",
        runner: candidate.label,
      };
    }
    last = { candidate, result };
  }
  return {
    ok: false,
    finalAnswer: "",
    transcript: "",
    runner: last && last.candidate ? last.candidate.label : "",
    exitCode: last && last.result ? last.result.code : 1,
    stderr: last && last.result ? last.result.stderr : "",
    reason: last && last.result
      ? `${last.candidate.label} failed: ${last.result.stderr || last.result.stdout || "unknown error"}`
      : "no formal-ai agent runner is available",
  };
}

function createInProcessProvider(options = {}) {
  const toolRouter = options.toolRouter || null;
  const invokeTool =
    options.invokeTool ||
    (toolRouter && typeof toolRouter.invoke === "function"
      ? (request) => toolRouter.invoke(request)
      : null);
  const runAgentTask =
    options.runAgentTask || ((task, request) => runFormalAiAgentTask(task, request, options));
  const provider = {
    type: DEFAULT_PROVIDER_TYPE,
    status() {
      return {
        type: DEFAULT_PROVIDER_TYPE,
        hermetic: true,
        supportsReadOnlyCommands: Boolean(invokeTool),
        supportsAgenticLoop: true,
      };
    },
    async run(request = {}) {
      const command = commandFromRequest(request);
      if (command) {
        if (typeof invokeTool !== "function") {
          return {
            ok: false,
            provider: DEFAULT_PROVIDER_TYPE,
            status: "unavailable",
            executed: false,
            reason: "in-process shell runner is unavailable",
          };
        }
        const result = await invokeTool({ tool: "shell", input: { command } });
        return {
          ...result,
          provider: DEFAULT_PROVIDER_TYPE,
          command,
          events: [
            {
              type: "tool_result",
              tool: "shell",
              command,
              body: result && result.body ? String(result.body) : "",
            },
          ],
        };
      }

      const task = promptFromRequest(request);
      if (!task) {
        return {
          ok: false,
          provider: DEFAULT_PROVIDER_TYPE,
          status: "unsupported",
          executed: false,
          reason: "in-process agentic loop runner is unavailable",
        };
      }
      const outcome = await runAgentTask(task, request);
      if (outcome && outcome.ok === false) {
        return {
          ok: false,
          provider: DEFAULT_PROVIDER_TYPE,
          status: "error",
          executed: false,
          task,
          body: "",
          transcript: String(outcome.transcript || ""),
          exitCode: outcome.exitCode,
          stderr: String(outcome.stderr || ""),
          reason: String(outcome.reason || "in-process agentic loop failed"),
          events: [],
        };
      }
      return {
        ok: true,
        provider: DEFAULT_PROVIDER_TYPE,
        status: "ok",
        executed: true,
        task,
        body: outcome && outcome.finalAnswer ? String(outcome.finalAnswer) : String(outcome || ""),
        transcript: outcome && outcome.transcript ? String(outcome.transcript) : "",
        runner: outcome && outcome.runner ? String(outcome.runner) : "",
        events: [],
      };
    },
  };
  return provider;
}

function commanderEnvironment(request = {}, options = {}) {
  const provider = providerConfigFrom(request);
  const source = options.env || process.env;
  const env = {};
  for (const key of SAFE_COMMANDER_ENV_KEYS) {
    if (Object.hasOwn(source, key)) {
      env[key] = source[key];
    }
  }
  if (provider.openAiBaseUrl) {
    env.OPENAI_BASE_URL = provider.openAiBaseUrl;
    env.OPENAI_API_BASE = provider.openAiBaseUrl;
    env.FORMAL_AI_OPENAI_BASE_URL = provider.openAiBaseUrl;
  }
  env.OPENAI_API_KEY = String(options.apiKey || request.apiKey || "formal-ai-local");
  env.FORMAL_AI_AGENT_PROVIDER = COMMANDER_PROVIDER_TYPE;
  return env;
}

function commanderToolEnvArgs(request = {}, options = {}) {
  const provider = providerConfigFrom(request);
  const apiKey = String(options.apiKey || request.apiKey || "formal-ai-local");
  const entries = [];
  if (provider.openAiBaseUrl) {
    entries.push(["OPENAI_BASE_URL", provider.openAiBaseUrl]);
    entries.push(["OPENAI_API_BASE", provider.openAiBaseUrl]);
    entries.push(["FORMAL_AI_OPENAI_BASE_URL", provider.openAiBaseUrl]);
  }
  entries.push(["OPENAI_API_KEY", apiKey]);
  return entries.flatMap(([key, value]) => ["--tool-env", `${key}=${value}`]);
}

function grantedDesktopTools(grants) {
  if (!grants || typeof grants !== "object") {
    return [];
  }
  if (grants.all === true) {
    return [...READ_ONLY_DESKTOP_TOOLS, ...WRITE_CAPABLE_DESKTOP_TOOLS];
  }
  return [...READ_ONLY_DESKTOP_TOOLS, ...WRITE_CAPABLE_DESKTOP_TOOLS]
    .filter((tool) => grants[tool] === true);
}

function restrictionFromDesktopGrants(grants) {
  if (!grants || typeof grants !== "object") {
    return "";
  }
  const granted = grantedDesktopTools(grants);
  if (granted.length === 0) {
    return "plan-only";
  }
  if (granted.some((tool) => WRITE_CAPABLE_DESKTOP_TOOLS.includes(tool))) {
    return "approve-each";
  }
  return "read-only";
}

function commanderRestrictionArgs(request = {}, options = {}) {
  const tool = String(request.commanderTool || options.commanderTool || DEFAULT_COMMANDER_TOOL);
  const grantRestriction = restrictionFromDesktopGrants(request.grants);
  if (request.planOnly === true) {
    return ["--plan-only"];
  }
  if (grantRestriction === "plan-only") {
    return ["--plan-only"];
  }
  if (
    request.readOnly === true ||
    grantRestriction === "read-only" ||
    isReadOnlyShellCommand(commandFromRequest(request))
  ) {
    // agent-commander currently rejects `--tool agent --read-only`; until the
    // upstream Agent permission gap is closed, the desktop gate remains the
    // read-only enforcement layer for the default agent backend.
    if (tool === DEFAULT_COMMANDER_TOOL) {
      return ["--approve-each"];
    }
    return ["--read-only"];
  }
  if (grantRestriction === "approve-each") {
    return ["--approve-each"];
  }
  if (normalizeMode(request.mode) === "agent") {
    return ["--approve-each"];
  }
  return [];
}

function buildCommanderArgs(request = {}, options = {}) {
  const provider = providerConfigFrom(request);
  const prompt = promptFromRequest(request);
  const workingDirectory = String(
    request.workingDirectory || options.workingDirectory || process.cwd(),
  );
  const tool = String(request.commanderTool || options.commanderTool || DEFAULT_COMMANDER_TOOL);
  const containerName = String(
    request.containerName || options.containerName || DEFAULT_AGENT_CONTAINER,
  );
  const args = [
    "--tool",
    tool,
    "--working-directory",
    workingDirectory,
    "--prompt",
    prompt,
    "--model",
    provider.model,
    "--isolation",
    "docker",
    "--container-name",
    containerName,
    ...commanderRestrictionArgs(request, options),
    ...commanderToolEnvArgs(request, options),
  ];
  return args;
}

function createCommanderProvider(options = {}) {
  const runner = options.processRunner || spawnAndCollect;
  const command = String(options.commanderCommand || DEFAULT_COMMANDER_COMMAND);
  return {
    type: COMMANDER_PROVIDER_TYPE,
    status() {
      return {
        type: COMMANDER_PROVIDER_TYPE,
        commanderCommand: command,
        defaultTool: String(options.commanderTool || DEFAULT_COMMANDER_TOOL),
        isolation: "docker",
      };
    },
    async run(request = {}) {
      const args = buildCommanderArgs(request, options);
      const result = normalizeRunnerResult(
        await runner(command, args, {
          cwd: options.workingDirectory || request.workingDirectory || process.cwd(),
          env: commanderEnvironment(request, options),
        }),
      );
      return {
        ok: result.code === 0,
        provider: COMMANDER_PROVIDER_TYPE,
        status: result.code === 0 ? "ok" : "error",
        executed: result.code === 0,
        command,
        args,
        exitCode: result.code,
        body: result.stdout,
        stderr: result.stderr,
        events: parseNdjsonEvents(result.stdout),
      };
    },
  };
}

function parseNdjsonEvents(text) {
  const events = [];
  for (const line of String(text || "").split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed) {
      continue;
    }
    try {
      events.push(JSON.parse(trimmed));
    } catch (_error) {
      events.push({ type: "text", text: trimmed });
    }
  }
  return events;
}

function createAgentProvider(options = {}) {
  const type = normalizeProviderType(options.type || process.env.FORMAL_AI_AGENT_PROVIDER);
  if (type === COMMANDER_PROVIDER_TYPE) {
    return createCommanderProvider(options);
  }
  return createInProcessProvider(options);
}

function directHostSpawnViolations(text) {
  const source = String(text || "");
  const patterns = HOST_AGENT_BINARIES.flatMap((binary) => [
    new RegExp(`\\b(?:spawn|spawnSync|execFile|execFileSync)\\s*\\(\\s*["'\`]${binary}["'\`]`, "g"),
    new RegExp(`\\b(?:exec|execSync)\\s*\\(\\s*["'\`]${binary}(?:\\s|["'\`])`, "g"),
  ]);
  const violations = [];
  for (const pattern of patterns) {
    for (const match of source.matchAll(pattern)) {
      violations.push({ index: match.index || 0, text: match[0] });
    }
  }
  return violations;
}

module.exports = {
  DEFAULT_PROVIDER_TYPE,
  COMMANDER_PROVIDER_TYPE,
  DEFAULT_COMMANDER_COMMAND,
  DEFAULT_COMMANDER_TOOL,
  DEFAULT_AGENT_CONTAINER,
  DEFAULT_MODEL,
  HOST_AGENT_BINARIES,
  READ_ONLY_DESKTOP_TOOLS,
  WRITE_CAPABLE_DESKTOP_TOOLS,
  normalizeProviderType,
  createAgentProvider,
  createInProcessProvider,
  createCommanderProvider,
  buildCommanderArgs,
  commanderRestrictionArgs,
  restrictionFromDesktopGrants,
  commanderEnvironment,
  packagedFormalAiPath,
  inProcessEnvironment,
  inProcessAgentCandidates,
  runFormalAiAgentTask,
  SAFE_COMMANDER_ENV_KEYS,
  HOST_SUBSCRIPTION_ENV_KEYS,
  isReadOnlyShellCommand,
  directHostSpawnViolations,
};
