#!/usr/bin/env node
// `formal-ai agent <action>` (rust/src/cli_orchestration.rs): the CLI wiring of
// the permission-gated external agent orchestration (issue #703). The actions,
// flags, defaults, outputs and error strings mirror `AgentAction`'s
// `run_external_action`:
//
//   node js/agentic/orchestration_cli.mjs run --cli codex --task "..." --workspace DIR
//   node js/agentic/orchestration_cli.mjs dispatch --cli codex,claude --compare ...
//   node js/agentic/orchestration_cli.mjs resume --parent SESSION --task ... --disproved-claim ... --evidence ...
//   node js/agentic/orchestration_cli.mjs synthesize SESSION... [--response-language ru] [--translation-session SESSION]
//   node js/agentic/orchestration_cli.mjs learn SESSION...
//   node js/agentic/orchestration_cli.mjs replay SESSION
//
// Rust's `main` prints a returned error as `Error: {error:?}` and exits 1; the
// same line and code are produced here. The controller program a Formal AI
// target runs (`formal-ai with --orchestration ...`) is `current_exe()` in Rust;
// here it is `$FORMAL_AI_CONTROLLER_PROGRAM` or `formal-ai` on `PATH`.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { installHost } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { learnClientContracts, learningLinksNotation } from './crate/client_contract_learning.mjs';
import {
  AgentSynthesisError, applyVerifiedTranslation, extractAgentResult, observeOrchestrationSession, synthesizeSessions,
} from './crate/orchestration_analysis.mjs';
import { dispatchAgents, dispatchConfig } from './crate/orchestration_dispatch.mjs';
import { DispatchError } from './crate/orchestration_dispatch_error.mjs';
import { grantFor } from './crate/orchestration_permission.mjs';
import { ReplayError } from './crate/orchestration_replay.mjs';
import {
  AgentRunError, DEFAULT_BASE_URL, DEFAULT_MODEL, DEFAULT_TIME_LIMIT_SECONDS, agentCommand, agentRunConfig,
  resumeAgent, runAgent, sessionPassed, sessionSha256, verificationCommand,
} from './crate/orchestration_runner.mjs';
import { readSession, writeSession } from './crate/orchestration_session_file.mjs';
import { IoError, ioErrorFrom } from './crate/orchestration_workspace.mjs';
import { clientIntegrations } from './crate/seed_client_integrations.mjs';

const TASK_PLACEHOLDER = '{task}';
const SESSION_ID_PLACEHOLDER = '{session_id}';

/** A `Box<dyn Error>` from a string: `Debug` quotes it. */
class CliError extends Error {
  /** @param {string} message @param {string} [debug] */
  constructor(message, debug = JSON.stringify(message)) {
    super(message);
    this.debug = debug;
  }
}

/** A clap usage error: printed as `error: ...`, exit code 2. */
class UsageError extends Error {}

/** A clap-style placeholder: `--max-depth` is `MAX_DEPTH`. */
const placeholder = (flag) => flag.slice(2).toUpperCase().replaceAll('-', '_');

const TARGETS = { 'formal-ai': 'formal_ai', vendor: 'vendor' };

/** The flag tables of each action: `kind` is value, bool, list (repeated) or csv (repeated, comma-delimited). */
const COMMON_RUN = {
  '--timeout-seconds': { kind: 'value', default: String(DEFAULT_TIME_LIMIT_SECONDS), number: 'u64' },
  '--session': { kind: 'value' },
  '--command': { kind: 'value' },
  '--allow-agent-command': { kind: 'list' },
  '--allow-command': { kind: 'list' },
  '--verify': { kind: 'list' },
};
const ACTIONS = {
  run: {
    flags: {
      '--cli': { kind: 'value', required: true },
      '--task': { kind: 'value', required: true },
      '--workspace': { kind: 'value', required: true },
      '--model': { kind: 'value', default: DEFAULT_MODEL },
      '--base-url': { kind: 'value', default: DEFAULT_BASE_URL },
      '--target': { kind: 'value', default: 'formal-ai', choices: Object.keys(TARGETS) },
      ...COMMON_RUN,
    },
    positional: null,
  },
  dispatch: {
    flags: {
      '--cli': { kind: 'csv', required: true },
      '--task': { kind: 'value', required: true },
      '--workspace': { kind: 'value', required: true },
      '--compare': { kind: 'bool' },
      '--incremental': { kind: 'bool', conflicts: '--compare' },
      '--output-dir': { kind: 'value' },
      '--pull-request': { kind: 'value', requires: '--incremental' },
      '--model': { kind: 'value', default: DEFAULT_MODEL },
      '--base-url': { kind: 'value', default: DEFAULT_BASE_URL },
      '--target': { kind: 'value', default: 'formal-ai', choices: Object.keys(TARGETS) },
      '--timeout-seconds': COMMON_RUN['--timeout-seconds'],
      '--command': { kind: 'list' },
      '--allow-agent-command': { kind: 'list' },
      '--allow-command': { kind: 'list' },
      '--verify': { kind: 'list' },
      '--max-depth': { kind: 'value', default: '3', number: 'u8' },
      '--synthesize': { kind: 'bool' },
      '--response-language': { kind: 'value', default: 'en' },
      '--translation-session': { kind: 'value', requires: '--synthesize' },
    },
    positional: null,
  },
  resume: {
    flags: {
      '--parent': { kind: 'value', required: true },
      '--task': { kind: 'value', required: true },
      '--workspace': { kind: 'value', required: true },
      '--disproved-claim': { kind: 'value', required: true },
      '--evidence': { kind: 'value', required: true },
      ...COMMON_RUN,
    },
    positional: null,
  },
  synthesize: {
    flags: {
      '--response-language': { kind: 'value', default: 'en' },
      '--translation-session': { kind: 'value' },
    },
    positional: { name: 'SESSION_JSON', multiple: true },
  },
  learn: { flags: {}, positional: { name: 'SESSION_JSON', multiple: true } },
  replay: { flags: {}, positional: { name: 'SESSION_JSON', multiple: false } },
};

/** Parse `argv` against an action's flag table (clap's `--flag value` and `--flag=value`). */
function parseArguments(action, argv) {
  const { flags, positional } = ACTIONS[action];
  const values = new Map();
  const positionals = [];
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (!token.startsWith('--')) {
      positionals.push(token);
      continue;
    }
    const equals = token.indexOf('=');
    const name = equals < 0 ? token : token.slice(0, equals);
    const spec = flags[name];
    if (!spec) throw new UsageError(`unexpected argument '${name}' found`);
    if (spec.kind === 'bool') {
      values.set(name, true);
      continue;
    }
    let value;
    if (equals >= 0) value = token.slice(equals + 1);
    else if (index + 1 < argv.length) value = argv[++index];
    else throw new UsageError(agenticMessage('orchestration_cli_value_required', { flag: name, value: placeholder(name) }));
    if (spec.choices && !spec.choices.includes(value)) {
      throw new UsageError(`invalid value '${value}' for '${name} <${name.slice(2).toUpperCase().replaceAll('-', '_')}>'\n  [possible values: ${spec.choices.join(', ')}]`);
    }
    if (spec.number && !/^[0-9]+$/.test(value)) {
      throw new UsageError(agenticMessage('orchestration_cli_invalid_digit', { value, flag: name, name: placeholder(name) }));
    }
    if (spec.number === 'u8' && Number(value) > 255) {
      throw new UsageError(`invalid value '${value}' for '${name} <MAX_DEPTH>': number too large to fit in target type`);
    }
    if (spec.kind === 'list') values.set(name, [...(values.get(name) ?? []), value]);
    else if (spec.kind === 'csv') values.set(name, [...(values.get(name) ?? []), ...value.split(',')]);
    else values.set(name, value);
  }
  for (const [name, spec] of Object.entries(flags)) {
    if (spec.required && !values.has(name)) throw new UsageError(agenticMessage('orchestration_cli_required_missing', { argument: `${name} <${placeholder(name)}>` }));
    if (spec.conflicts && values.has(name) && values.has(spec.conflicts)) throw new UsageError(agenticMessage('orchestration_cli_conflict', { flag: name, other: spec.conflicts }));
    if (spec.requires && values.has(name) && !values.has(spec.requires)) throw new UsageError(agenticMessage('orchestration_cli_required_missing', { argument: spec.requires }));
    if (!values.has(name) && spec.default !== undefined) values.set(name, spec.default);
  }
  if (positional === null && positionals.length > 0) throw new UsageError(`unexpected argument '${positionals[0]}' found`);
  if (positional !== null) {
    if (positionals.length === 0) throw new UsageError(agenticMessage('orchestration_cli_required_missing', { argument: `<${positional.name}>` }));
    if (!positional.multiple && positionals.length > 1) throw new UsageError(`unexpected argument '${positionals[1]}' found`);
  }
  return { values, positionals };
}

/** `serde_json::from_str::<Vec<String>>`: the argv a JSON string array encodes. */
function parseJsonArgv(value) {
  let parsed;
  try {
    parsed = JSON.parse(value);
  } catch (error) {
    throw new CliError(error.message, `Error(${JSON.stringify(error.message)}, line: 1, column: 0)`);
  }
  if (!Array.isArray(parsed) || !parsed.every((item) => typeof item === 'string')) {
    const message = agenticMessage('orchestration_cli_invalid_argv');
    throw new CliError(message, `Error(${JSON.stringify(message)}, line: 1, column: 0)`);
  }
  return parsed;
}

/** Mirrors `fn parse_verification` in rust/src/cli_orchestration.rs. */
function parseVerification(values) {
  return values.map((value) => {
    const argv = parseJsonArgv(value);
    if (argv.length === 0) throw new CliError('empty_verification_argv');
    return verificationCommand(argv[0], argv.slice(1));
  });
}

/** Mirrors `fn parse_agent_command` in rust/src/cli_orchestration.rs. */
function parseAgentCommand(value, sessionId) {
  const argv = parseJsonArgv(value);
  if (argv.length === 0) throw new CliError('empty_agent_argv');
  const [program, ...rest] = argv;
  if (!rest.some((argument) => argument.includes(TASK_PLACEHOLDER))) throw new CliError('missing_task_placeholder');
  if (sessionId !== null && !rest.some((argument) => argument.includes(SESSION_ID_PLACEHOLDER))) {
    throw new CliError('missing_session_id_placeholder');
  }
  return agentCommand(program, rest.map((argument) => (sessionId === null ? argument : argument.split(SESSION_ID_PLACEHOLDER).join(sessionId))));
}

/** Mirrors `fn parse_agent_commands` in rust/src/cli_orchestration.rs. */
function parseAgentCommands(values) {
  const commands = new Map();
  for (const value of values) {
    const at = value.indexOf('=');
    if (at < 0) throw new CliError('invalid_agent_command_mapping');
    const cli = value.slice(0, at);
    if (cli === '' || commands.has(cli)) throw new CliError('invalid_agent_command_mapping');
    commands.set(cli, parseAgentCommand(value.slice(at + 1), null));
  }
  return commands;
}

/** Mirrors `fn ensure_parent` in rust/src/cli_orchestration.rs. */
function ensureParent(file) {
  const parent = path.dirname(file);
  if (parent !== '' && parent !== '.') fs.mkdirSync(parent, { recursive: true });
}

/** The controller program a Formal AI target launches (`std::env::current_exe` in Rust). */
const controllerProgram = () => process.env.FORMAL_AI_CONTROLLER_PROGRAM || 'formal-ai';

const prettyJson = (value) => JSON.stringify(value, null, 2);

/** The canonical workspace (`Path::canonicalize`); a failure is the `io::Error` Rust would print. */
function canonicalize(target) {
  try {
    return fs.realpathSync(target);
  } catch (error) {
    throw new CliError(ioErrorFrom(error).message, ioErrorFrom(error).debug);
  }
}

/** Mirrors `fn run_one` in rust/src/cli_orchestration.rs. */
async function runOne(values, io) {
  const workspace = canonicalize(values.get('--workspace'));
  const config = agentRunConfig(values.get('--cli'), values.get('--task'), workspace);
  config.permission = grantFor(workspace);
  config.model = values.get('--model');
  config.base_url = values.get('--base-url');
  config.target = TARGETS[values.get('--target')];
  config.timeout_ms = Number(values.get('--timeout-seconds')) * 1000;
  config.controller_program = controllerProgram();
  config.allowlisted_agent_commands = new Set(values.get('--allow-agent-command') ?? []);
  config.allowlisted_commands = new Set(values.get('--allow-command') ?? []);
  config.verification = parseVerification(values.get('--verify') ?? []);
  config.command_override = values.has('--command') ? parseAgentCommand(values.get('--command'), null) : null;
  const session = await runAgent(config);
  if (values.has('--session')) {
    ensureParent(values.get('--session'));
    writeSession(values.get('--session'), session);
  }
  io.stdout(`${prettyJson(session)}\n`);
  if (!sessionPassed(session)) throw new CliError('agent_run_failed');
}

/** Install the host the seed readers and the lexicon-driven splitter need. */
async function installRuntimeHost(withWorker) {
  const { parseLino, readRepoFile } = await import('../server/lino.mjs');
  if (!withWorker) {
    installHost({ readText: readRepoFile, parseLino, realm: {} });
    return;
  }
  const { installNodeHost } = await import('./node-host.mjs');
  const { WorkerHost } = await import('../server/worker-host.mjs');
  await installNodeHost(new WorkerHost());
}

/** Mirrors `fn run_dispatch` in rust/src/cli_orchestration.rs. */
async function runDispatch(values, io) {
  const workspace = canonicalize(values.get('--workspace'));
  const config = dispatchConfig(values.get('--task'), workspace, values.get('--cli'));
  config.mode = values.has('--compare') ? 'compare' : values.has('--incremental') ? 'incremental' : 'decompose';
  if (values.has('--output-dir')) config.output_dir = values.get('--output-dir');
  config.model = values.get('--model');
  config.base_url = values.get('--base-url');
  config.target = TARGETS[values.get('--target')];
  config.timeout_ms = Number(values.get('--timeout-seconds')) * 1000;
  config.permission = grantFor(workspace);
  config.controller_program = controllerProgram();
  config.allowlisted_agent_commands = new Set(values.get('--allow-agent-command') ?? []);
  config.allowlisted_commands = new Set(values.get('--allow-command') ?? []);
  config.verification = parseVerification(values.get('--verify') ?? []);
  config.command_overrides = parseAgentCommands(values.get('--command') ?? []);
  config.max_depth = Number(values.get('--max-depth'));
  config.pull_request = values.has('--pull-request') ? values.get('--pull-request') : null;
  const report = await dispatchAgents(config);
  if (values.has('--synthesize')) {
    const synthesis = await synthesizeSessions(report.sessions, values.get('--response-language'));
    if (values.has('--translation-session')) {
      const translation = readSession(values.get('--translation-session'));
      applyVerifiedTranslation(synthesis, extractAgentResult(translation.stdout), sessionSha256(translation));
    }
    io.stdout(`${prettyJson({ dispatch: report, synthesis })}\n`);
  } else {
    io.stdout(`${prettyJson(report)}\n`);
  }
  let passed;
  if (report.mode === 'compare') passed = report.ledger.winner !== null;
  // A failure is the input of the incremental mode, not its verdict: only the
  // root task ending up solved counts, however many attempts that took.
  else if (report.mode === 'incremental') passed = Boolean(report.incremental?.solved);
  else passed = report.sessions.every(sessionPassed);
  if (!passed) throw new CliError('agent_dispatch_failed');
}

/** Mirrors `fn run_resume` in rust/src/cli_orchestration.rs. */
async function runResume(values, io) {
  const parent = readSession(values.get('--parent'));
  const workspace = canonicalize(values.get('--workspace'));
  const request = {
    cli: parent.cli,
    claim: values.get('--disproved-claim'),
    evidence: values.get('--evidence'),
    source_session_sha256: sessionSha256(parent),
  };
  const config = agentRunConfig(parent.cli, values.get('--task'), workspace);
  config.permission = grantFor(workspace);
  config.timeout_ms = Number(values.get('--timeout-seconds')) * 1000;
  config.controller_program = controllerProgram();
  config.allowlisted_agent_commands = new Set(values.get('--allow-agent-command') ?? []);
  config.allowlisted_commands = new Set(values.get('--allow-command') ?? []);
  config.verification = parseVerification(values.get('--verify') ?? []);
  config.command_override = values.has('--command')
    ? parseAgentCommand(values.get('--command'), parent.native_session ? parent.native_session.id : null)
    : null;
  const session = await resumeAgent(parent, request, config);
  if (values.has('--session')) {
    ensureParent(values.get('--session'));
    writeSession(values.get('--session'), session);
  }
  io.stdout(`${prettyJson(session)}\n`);
  if (!sessionPassed(session)) throw new CliError('agent_resume_failed');
}

/** Mirrors `fn run_synthesis` in rust/src/cli_orchestration.rs. */
async function runSynthesis(values, positionals, io) {
  const sessions = positionals.map((file) => readSession(file));
  const report = await synthesizeSessions(sessions, values.get('--response-language'));
  if (values.has('--translation-session')) {
    const translation = readSession(values.get('--translation-session'));
    applyVerifiedTranslation(report, extractAgentResult(translation.stdout), sessionSha256(translation));
  }
  io.stdout(`${prettyJson(report)}\n`);
}

/** Mirrors `fn run_learning` in rust/src/cli_orchestration.rs. */
function runLearning(positionals, io) {
  const observations = positionals.map((file) => observeOrchestrationSession(readSession(file), file));
  io.stdout(`${learningLinksNotation(learnClientContracts(observations, clientIntegrations()))}\n`);
}

/** The `Debug` rendering `main` prints after `Error: ` for a returned error. */
function debugOf(error) {
  if (error instanceof CliError || error instanceof AgentRunError || error instanceof DispatchError || error instanceof IoError) return error.debug;
  if (error instanceof ReplayError) return error.code === 'io' ? `Io(${JSON.stringify(error.detail)})` : error.message;
  if (error instanceof AgentSynthesisError) return error.debug;
  return error?.message ?? String(error);
}

/**
 * Mirrors `fn run_external_action` in rust/src/cli_orchestration.rs.
 * @param {string[]} argv the arguments after `agent`: the action, then its flags
 * @param {{stdout: (text: string) => void, stderr: (text: string) => void}} io
 * @returns {Promise<number>} the process exit code
 */
export async function runOrchestrationCli(argv, io) {
  const [action, ...rest] = argv;
  try {
    await installRuntimeHost(false);
    if (!Object.hasOwn(ACTIONS, action ?? '')) {
      throw new UsageError(action === undefined ? agenticMessage('orchestration_cli_subcommand_required') : `unrecognized subcommand '${action}'`);
    }
    const { values, positionals } = parseArguments(action, rest);
    if (action === 'dispatch' && !values.has('--compare')) await installRuntimeHost(true);
    if (action === 'run') await runOne(values, io);
    else if (action === 'dispatch') await runDispatch(values, io);
    else if (action === 'resume') await runResume(values, io);
    else if (action === 'synthesize') await runSynthesis(values, positionals, io);
    else if (action === 'learn') runLearning(positionals, io);
    else io.stdout(`${prettyJson(readSession(positionals[0]))}\n`);
    return 0;
  } catch (error) {
    if (error instanceof UsageError) {
      io.stderr(`error: ${error.message}\n`);
      return 2;
    }
    io.stderr(`Error: ${debugOf(error)}\n`);
    return 1;
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  runOrchestrationCli(process.argv.slice(2), {
    stdout: (text) => process.stdout.write(text),
    stderr: (text) => process.stderr.write(text),
  }).then((code) => {
    process.exitCode = code;
  });
}
