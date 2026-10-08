// `crate::orchestration::runner` (rust/src/orchestration/runner.rs): run one
// registered (or explicitly granted) external agent CLI inside a bounded
// workspace, record every process and filesystem effect as a SHA-256 event
// chain, and never retry a process implicitly. The seed registry is the same
// data/seed/client-integrations.lino file `formal-ai with` reads.
//
// Rust drives the process through a dedicated Tokio thread; JavaScript spawns it
// in its own process group and awaits it, so a run is a Promise. A timeout kills
// the whole group (`SIGKILL`) and records exit code 137, the `128 + signal`
// code command-stream reports.

import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import { constants } from 'node:os';
import path from 'node:path';

import { agenticMessage } from '../messages.mjs';
import { detect } from './language.mjs';
import { isJsonObject, jsonValueStream, sortedValues } from './orchestration_json_stream.mjs';
import { noPermission, permits } from './orchestration_permission.mjs';
import { sessionSha256 as replaySessionSha256, SESSION_SCHEMA } from './orchestration_replay.mjs';
import { IoError, changes, ioError, ioErrorFrom, snapshot } from './orchestration_workspace.mjs';
import { clientIntegrations } from './seed_client_integrations.mjs';
import { localizedResponse } from './seed.mjs';

export const DEFAULT_MODEL = 'formal-ai';
export const DEFAULT_BASE_URL = 'http://127.0.0.1:8080';
/** `crate::research_learning::DEFAULT_RESEARCH_TIME_LIMIT_SECONDS`. */
export const DEFAULT_TIME_LIMIT_SECONDS = 60 * 60;
const VERIFICATION_TASK_ENV = 'FORMAL_AI_VERIFICATION_TASK';
const NATIVE_SESSION_PREFIX = 'formal-ai: orchestration-session-json:';
const PIPE_DRAIN_GRACE_MS = 250;

/** Mirrors `enum AgentRunError` in rust/src/orchestration/runner.rs; `message` is its `Display`, `debug` its `Debug`. */
export class AgentRunError extends Error {
  /** @param {string} variant @param {string|IoError|null} [detail] */
  constructor(variant, detail = null) {
    super(AgentRunError.display(variant, detail));
    this.variant = variant;
    this.detail = detail;
    this.debug = AgentRunError.debugOf(variant, detail);
  }

  static display(variant, detail) {
    switch (variant) {
      case 'unsupported_cli': return `unsupported_cli:${detail}`;
      case 'workspace': return `workspace:${detail.message}`;
      case 'command_not_allowlisted': return `command_not_allowlisted:${detail}`;
      case 'agent_command_not_allowlisted': return `agent_command_not_allowlisted:${detail}`;
      case 'seed_contract_unavailable': return `seed_contract_unavailable:${detail}`;
      case 'process': return `process:${detail.message}`;
      default: return variant;
    }
  }

  static debugOf(variant, detail) {
    const name = variant.split('_').map((part) => part[0].toUpperCase() + part.slice(1)).join('');
    if (detail === null) return name;
    return `${name}(${detail instanceof IoError ? detail.debug : JSON.stringify(detail)})`;
  }
}

/**
 * Mirrors `AgentCommand::new` in rust/src/orchestration/runner.rs (with `.arg` / `.env` as plain fields).
 * @param {string} program
 * @param {string[]} [args]
 * @param {Record<string, string>} [env]
 */
export function agentCommand(program, args = [], env = {}) {
  return { program: String(program), args: [...args], env: { ...env } };
}

/** Mirrors `VerificationCommand::new` in rust/src/orchestration/runner.rs. */
export function verificationCommand(program, args = []) {
  return { program: String(program), args: [...args] };
}

/**
 * Mirrors `AgentRunConfig::new`; `timeout_ms` is the `Duration` in milliseconds.
 * @param {string} cli @param {string} task @param {string} workspace
 */
export function agentRunConfig(cli, task, workspace) {
  return {
    cli,
    task,
    workspace: String(workspace),
    model: DEFAULT_MODEL,
    base_url: DEFAULT_BASE_URL,
    target: 'formal_ai',
    timeout_ms: DEFAULT_TIME_LIMIT_SECONDS * 1000,
    permission: noPermission(),
    allowlisted_agent_commands: new Set(),
    allowlisted_commands: new Set(),
    verification: [],
    controller_program: 'formal-ai',
    orchestration_home: null,
    command_override: null,
    continuation: null,
  };
}

/** Mirrors `AgentRunConfig::with_permission` in rust/src/orchestration/runner.rs. */
export function withPermission(config, permission) {
  return { ...config, permission };
}

/** Mirrors `AgentRunConfig::with_command` in rust/src/orchestration/runner.rs. */
export function withCommand(config, command) {
  return { ...config, command_override: command };
}

/** Mirrors `AgentSession::passed` in rust/src/orchestration/runner.rs. */
export function sessionPassed(session) {
  return session.status === 'succeeded' && session.verification.every((result) => result.passed);
}

/** Mirrors `AgentSession::diff_size` in rust/src/orchestration/runner.rs. */
export function sessionDiffSize(session) {
  return session.changes.reduce((total, change) => total + change.bytes_changed, 0);
}

/** Mirrors `fn session_sha256` in rust/src/orchestration/runner.rs. */
export function sessionSha256(session) {
  return replaySessionSha256(session);
}

/** Mirrors `struct EventChain` in rust/src/orchestration/runner.rs and `EventChain::push`. */
function eventChain() {
  const events = [];
  return {
    events,
    push(kind, detail) {
      const sequence = events.length;
      const previous = events.length ? events[events.length - 1].sha256 : '0'.repeat(64);
      const payload = `${sequence}\0${kind}\0${detail}\0${previous}`;
      events.push({
        sequence,
        kind,
        detail,
        previous_sha256: previous,
        sha256: createHash('sha256').update(payload).digest('hex'),
      });
    },
  };
}

/** Mirrors `fn validate_verification` in rust/src/orchestration/runner.rs. */
function validateVerification(config) {
  for (const command of config.verification) {
    if (!config.allowlisted_commands.has(command.program)) throw new AgentRunError('command_not_allowlisted', command.program);
  }
}

/** Mirrors `the canonicalize / is_dir prologue of fn run_agent` in rust/src/orchestration/runner.rs. The canonical workspace, refused unless it is an existing directory. */
function canonicalWorkspace(config) {
  let workspace;
  try {
    workspace = fs.realpathSync(config.workspace);
  } catch (error) {
    throw new AgentRunError('workspace', ioErrorFrom(error));
  }
  if (!fs.statSync(workspace).isDirectory()) {
    throw new AgentRunError('workspace', ioError('InvalidInput', 'workspace_not_directory'));
  }
  return workspace;
}

/** Mirrors `fn build_vendor_command` in rust/src/orchestration/runner.rs. */
function buildVendorCommand(config, integration) {
  const fromEnv = integration.command_env ? process.env[integration.command_env] : undefined;
  const program = fromEnv ? fromEnv : integration.command;
  const invocation = integration.invocation;
  let args = [...invocation.prepend_args];
  if (invocation.mode_arg_position === 'before_invocation') args.push(...invocation.non_interactive_args);
  args.push(...(invocation.vendor_orchestration_args.length === 0 ? invocation.orchestration_args : invocation.vendor_orchestration_args));
  const modelArg = invocation.vendor_model_arg === '' ? invocation.model_arg : invocation.vendor_model_arg;
  if (modelArg !== '' && config.model !== '') {
    if (invocation.model_arg_position === 'after_first_arg' && args.length > 0) {
      args = [args[0], modelArg, config.model, ...args.slice(1)];
    } else {
      args = [modelArg, config.model, ...args];
    }
  }
  if (config.continuation) {
    args.push(...invocation.resume_args.map((argument) => argument.split('{session_id}').join(config.continuation.native_session_id)));
  }
  if (invocation.mode_arg_position !== 'before_invocation') args.push(...invocation.non_interactive_args);
  args.push(config.task);
  return agentCommand(program, args);
}

/** Mirrors `fn build_command` in rust/src/orchestration/runner.rs. */
function buildCommand(config, integration) {
  if (config.target !== 'formal_ai') return buildVendorCommand(config, integration);
  const home = config.orchestration_home
    ?? path.join(config.workspace, '.formal-ai-orchestration', 'native-sessions', integration.id);
  const args = [
    'with', '--orchestration', '--base-url', config.base_url, '--model', config.model, '--non-interactive',
    '--orchestration-home', home,
  ];
  if (config.continuation) args.push('--orchestration-resume', config.continuation.native_session_id);
  args.push(integration.id, config.task);
  return agentCommand(config.controller_program, args);
}

/** Mirrors `fn find_session_id` in rust/src/orchestration/runner.rs. */
function findSessionId(value) {
  if (isJsonObject(value)) {
    for (const key of ['sessionId', 'session_id', 'thread_id']) {
      if (typeof value[key] === 'string') return value[key];
    }
    for (const nested of sortedValues(value)) {
      const found = findSessionId(nested);
      if (found !== null) return found;
    }
    return null;
  }
  if (!Array.isArray(value)) return null;
  for (const item of value) {
    const found = findSessionId(item);
    if (found !== null) return found;
  }
  return null;
}

/** Mirrors `serde_json::from_str::<NativeAgentSession>` in rust/src/orchestration/runner.rs. `serde_json::from_str::<NativeAgentSession>`: both fields strings, or null. */
function nativeSessionFrom(json) {
  let value;
  try {
    value = JSON.parse(json);
  } catch {
    return null;
  }
  if (!isJsonObject(value) || typeof value.id !== 'string' || typeof value.resume_command !== 'string') return null;
  return { id: value.id, resume_command: value.resume_command };
}

/** Mirrors `fn parse_native_session` in rust/src/orchestration/runner.rs. */
export function parseNativeSession(stderr, stdout, integration) {
  for (const line of stderr.split(/\r?\n/u)) {
    const trimmed = line.trim();
    if (!trimmed.startsWith(NATIVE_SESSION_PREFIX)) continue;
    const evidence = nativeSessionFrom(trimmed.slice(NATIVE_SESSION_PREFIX.length));
    if (evidence !== null) return evidence;
  }
  if (!integration || integration.invocation.resume_command === '') return null;
  for (const entry of jsonValueStream(stdout)) {
    if (entry.error) return null;
    const id = findSessionId(entry.value);
    if (id !== null) {
      return { id, resume_command: integration.invocation.resume_command.split('{session_id}').join(id) };
    }
  }
  return null;
}

/** Mirrors `which::which_in (executable-file test)` in rust/src/orchestration/runner.rs. Whether `file` is an executable regular file. */
function executable(file) {
  try {
    if (!fs.statSync(file).isFile()) return false;
    fs.accessSync(file, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

/**
 * Mirrors `fn ensure_program_available` in rust/src/orchestration/runner.rs (`which::which_in`): the program must
 * resolve in the command's `PATH` (or the process `PATH`), relative to `workspace`.
 */
function ensureProgramAvailable(command, workspace) {
  const searchPath = command.env.PATH ?? process.env.PATH ?? '';
  const extensions = process.platform === 'win32' ? ['', ...(process.env.PATHEXT ?? '.EXE;.CMD;.BAT').split(';')] : [''];
  const candidates = command.program.includes('/') || command.program.includes(path.sep)
    ? [path.resolve(workspace, command.program)]
    : searchPath.split(path.delimiter).filter(Boolean).map((directory) => path.resolve(workspace, directory, command.program));
  const found = candidates.some((candidate) => extensions.some((extension) => executable(candidate + extension)));
  if (!found) throw new AgentRunError('process', ioError('NotFound', agenticMessage('orchestration_binary_not_found')));
}

/** The `128 + signal` exit code command-stream reports for a signalled child. */
const signalExitCode = (signal) => 128 + (constants.signals[signal] ?? 0);

/**
 * Mirrors `fn execute` in rust/src/orchestration/runner.rs / `fn collect_command_stream`: run `command` in
 * `workspace` with a hard deadline, killing the whole process group on timeout.
 * @returns {Promise<{exit_code: number|null, stdout: string, stderr: string, timed_out: boolean}>}
 */
async function execute(command, workspace, timeoutMs) {
  ensureProgramAvailable(command, workspace);
  const env = { ...process.env, ...command.env, PWD: workspace };
  const child = spawn(command.program, command.args, {
    cwd: workspace,
    env,
    detached: process.platform !== 'win32',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  const stdout = [];
  const stderr = [];
  child.stdout.on('data', (chunk) => stdout.push(chunk));
  child.stderr.on('data', (chunk) => stderr.push(chunk));
  let timedOut = false;
  const killGroup = () => {
    try {
      if (process.platform === 'win32') child.kill('SIGKILL');
      else process.kill(-child.pid, 'SIGKILL');
    } catch {
      try {
        child.kill('SIGKILL');
      } catch {
        // already gone
      }
    }
  };
  const deadline = setTimeout(() => {
    timedOut = true;
    killGroup();
  }, timeoutMs);
  const outcome = await new Promise((resolve, reject) => {
    let exit = null;
    let drain = null;
    child.on('error', (error) => {
      clearTimeout(deadline);
      reject(new AgentRunError('process', ioErrorFrom(error)));
    });
    child.on('exit', (code, signal) => {
      exit = { code, signal };
      drain = setTimeout(() => {
        child.stdout.destroy();
        child.stderr.destroy();
      }, PIPE_DRAIN_GRACE_MS);
    });
    child.on('close', (code, signal) => {
      clearTimeout(drain);
      resolve(exit ?? { code, signal });
    });
  });
  clearTimeout(deadline);
  let exitCode = outcome.code;
  if (exitCode === null && outcome.signal) exitCode = signalExitCode(outcome.signal);
  return {
    exit_code: exitCode,
    stdout: Buffer.concat(stdout).toString('utf8'),
    stderr: Buffer.concat(stderr).toString('utf8'),
    timed_out: timedOut,
  };
}

/** Mirrors `fn run_verification` in rust/src/orchestration/runner.rs. */
async function runVerification(config, workspace, events) {
  const results = [];
  for (const verification of config.verification) {
    events.push('verification_started', verification.program);
    const command = agentCommand(verification.program, verification.args, { [VERIFICATION_TASK_ENV]: config.task });
    const output = await execute(command, workspace, config.timeout_ms);
    const passed = !output.timed_out && output.exit_code === 0;
    events.push(output.timed_out ? 'verification_timed_out' : passed ? 'verification_passed' : 'verification_failed', verification.program);
    results.push({
      program: verification.program,
      args: verification.args,
      exit_code: output.exit_code,
      stdout: output.stdout,
      stderr: output.stderr,
      timed_out: output.timed_out,
      passed,
    });
  }
  return results;
}

/**
 * Mirrors `fn run_agent` in rust/src/orchestration/runner.rs: run the adapter
 * and return the recorded session. Refusals throw an `AgentRunError`.
 * @param {ReturnType<typeof agentRunConfig>} config
 * @returns {Promise<object>} the `AgentSession`
 */
export async function runAgent(config) {
  if (!permits(config.permission, config.workspace)) throw new AgentRunError('permission_denied');
  validateVerification(config);
  const workspace = canonicalWorkspace(config);
  const integration = clientIntegrations().find((entry) => entry.id === config.cli || entry.aliases.includes(config.cli)) ?? null;
  if (integration && integration.verification.surface !== 'cli') throw new AgentRunError('unsupported_cli', config.cli);
  if (!integration && !config.command_override) throw new AgentRunError('unsupported_cli', config.cli);
  if (config.command_override && !config.allowlisted_agent_commands.has(config.command_override.program)) {
    throw new AgentRunError('agent_command_not_allowlisted', config.command_override.program);
  }
  let before;
  try {
    before = snapshot(workspace);
  } catch (error) {
    throw new AgentRunError('workspace', ioErrorFrom(error));
  }
  const command = config.command_override
    ? { ...config.command_override, args: config.command_override.args.map((argument) => argument.split('{task}').join(config.task)) }
    : buildCommand(config, integration);
  const adapterId = integration ? integration.id : config.cli;
  const processName = config.command_override ? command.program : integration.command;
  const events = eventChain();
  events.push('permission_granted', '.');
  if (config.command_override) events.push('custom_adapter_granted', processName);
  events.push('adapter_selected', adapterId);
  if (config.continuation) events.push('native_session_resumed', adapterId);
  events.push('process_started', processName);
  const started = Date.now();
  const output = await execute(command, workspace, config.timeout_ms);
  const wallTimeMs = Date.now() - started;
  let status;
  if (output.timed_out) {
    events.push('process_timed_out', adapterId);
    status = 'timed_out';
  } else if (output.exit_code === 0) {
    events.push('process_succeeded', adapterId);
    status = 'succeeded';
  } else {
    events.push('process_failed', adapterId);
    status = 'failed';
  }
  const verification = await runVerification(config, workspace, events);
  let after;
  try {
    after = snapshot(workspace);
  } catch (error) {
    throw new AgentRunError('workspace', ioErrorFrom(error));
  }
  const effects = changes(before, after);
  for (const effect of effects) events.push('workspace_effect', effect.path);
  const nativeSession = parseNativeSession(output.stderr, output.stdout, config.command_override ? null : integration);
  const session = {
    schema: SESSION_SCHEMA,
    cli: adapterId,
    target: config.target,
    task: config.task,
    model: config.model,
    base_url: config.base_url,
    workspace: '.',
    program: command.program,
    args: command.args,
    status,
    exit_code: output.exit_code,
    wall_time_ms: wallTimeMs,
    stdout: output.stdout,
    stderr: output.stderr,
    changes: effects,
    verification,
    events: events.events,
  };
  if (nativeSession) session.native_session = nativeSession;
  if (config.continuation) session.continuation = config.continuation;
  return session;
}

/**
 * Mirrors `fn verify_workspace` in rust/src/orchestration/runner.rs: verify
 * already-composed workspace effects without invoking another agent.
 * @returns {Promise<object>} the `AgentSession` of the composed verifier
 */
export async function verifyWorkspace(config) {
  if (!permits(config.permission, config.workspace)) throw new AgentRunError('permission_denied');
  validateVerification(config);
  const workspace = canonicalWorkspace(config);
  const events = eventChain();
  events.push('permission_granted', '.');
  events.push('composition_verification_started', config.task);
  const started = Date.now();
  const verification = await runVerification(config, workspace, events);
  const wallTimeMs = Date.now() - started;
  const passed = verification.every((result) => result.passed);
  events.push(passed ? 'composition_verification_passed' : 'composition_verification_failed', config.task);
  return {
    schema: SESSION_SCHEMA,
    cli: 'composed-verifier',
    target: config.target,
    task: config.task,
    model: config.model,
    base_url: config.base_url,
    workspace: '.',
    program: 'verification-only',
    args: [],
    status: passed ? 'succeeded' : 'failed',
    exit_code: passed ? 0 : 1,
    wall_time_ms: wallTimeMs,
    stdout: '',
    stderr: '',
    changes: [],
    verification,
    events: events.events,
  };
}

/**
 * Mirrors `fn resume_agent` in rust/src/orchestration/runner.rs: continue the
 * exact native session that produced a disproved claim. A mismatch fails
 * closed instead of silently starting a fresh model turn.
 * @param {object} parent the parent `AgentSession`
 * @param {{cli: string, claim: string, evidence: string, source_session_sha256: string}} request
 * @param {ReturnType<typeof agentRunConfig>} config
 */
export async function resumeAgent(parent, request, config) {
  const parentDigest = sessionSha256(parent);
  if (parent.cli !== request.cli || parentDigest !== request.source_session_sha256) throw new AgentRunError('continuation_mismatch');
  const native = parent.native_session;
  if (!native) throw new AgentRunError('native_session_unavailable');
  const resumed = { ...config, cli: parent.cli, target: parent.target, model: parent.model, base_url: parent.base_url };
  const template = localizedResponse('orchestration_correction_prompt', detect(resumed.task));
  if (template === null) throw new AgentRunError('seed_contract_unavailable', 'orchestration_correction_prompt');
  resumed.task = template
    .split('{task}').join(resumed.task)
    .split('{claim}').join(request.claim)
    .split('{evidence}').join(request.evidence);
  resumed.continuation = {
    parent_session_sha256: parentDigest,
    native_session_id: native.id,
    disproved_claim: request.claim,
    evidence: request.evidence,
  };
  const session = await runAgent(resumed);
  if (session.native_session && session.native_session.id !== native.id) throw new AgentRunError('continuation_mismatch');
  return session.native_session ? session : withNativeSession(session, native);
}

/** Mirrors `the `session.native_session = Some(native.clone())` step of fn resume_agent` in rust/src/orchestration/runner.rs. The session with `native_session` placed where the canonical rendering puts it (before `continuation`). */
function withNativeSession(session, native) {
  const { continuation, ...rest } = session;
  return continuation === undefined ? { ...rest, native_session: native } : { ...rest, native_session: native, continuation };
}
