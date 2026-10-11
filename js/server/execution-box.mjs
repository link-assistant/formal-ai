// Where code actually runs, and the honest report of what happened:
// rust/src/execution_box/mod.rs and rust/src/execution_box/invocation.rs.
//
// Ported: the backends configuration can select (`host_sandbox`, the
// `start-command` runner pair, `box:<image>` and `box-language:<language>`),
// `ExecutionBox::open`, `run`, `script_invocation`, the deadline-bounded
// process observation with partial output, and the descending-N ladder.
// Not ported, because no configuration reaches them from the server: the
// SWE-bench image and per-conversation container backends
// (rust/src/execution_box/conversation.rs, container.rs), `run_command`,
// `detach`, input-file transport and the browser runtime.

import { spawn, spawnSync } from 'node:child_process';
import { appendFileSync, mkdirSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { childValue, parseLino, readRepoFile } from './lino.mjs';
import { serverMessage } from './messages.mjs';

/** Mirrors rust/src/execution_box/mod.rs `BACKEND_ENV`. */
export const BACKEND_ENV = 'FORMAL_AI_EXECUTION_BACKEND';
/** Mirrors rust/src/execution_box/mod.rs `START_ISOLATION_ENV`. */
export const START_ISOLATION_ENV = 'FORMAL_AI_START_ISOLATION';
/** Mirrors rust/src/execution_box/mod.rs `START_RUNNER_ENV`. */
export const START_RUNNER_ENV = 'FORMAL_AI_START_RUNNER';

const HOST_INTERPRETER = 'python3';
const SCRIPT_FILE = '__formal_ai_run.py';
const COMMAND_LOG = '__formal_ai_commands.log';
const CONTAINER_RUNNER =
  'mkdir -p /tmp/formal-ai && tar -xzf - -C /tmp/formal-ai && cd /tmp/formal-ai && exec "$@"';
const STARTUP_FLOOR_NS = 250_000_000n;
const READ_BUFFER = 1024;
const COMMAND_NOT_FOUND_EXIT = 127;
const NS_PER_MS = 1_000_000n;

// ---------------------------------------------------------------- Rust renderings

const WHITE_SPACE = /^\p{White_Space}+|\p{White_Space}+$/gu;
const TRAILING_WHITE_SPACE = /\p{White_Space}+$/u;

/** Rust `str::trim` (Unicode `White_Space`). @param {string} text */
export function rustTrim(text) {
  return text.replace(WHITE_SPACE, '');
}

/** Rust `str::trim_end`. @param {string} text */
export function rustTrimEnd(text) {
  return text.replace(TRAILING_WHITE_SPACE, '');
}

const DEBUG_ESCAPED = /[\p{Cc}\p{Cf}\p{Cs}\p{Co}\p{Cn}\p{Zl}\p{Zp}\p{Zs}\p{Grapheme_Extend}]/u;
const DEBUG_SHORT = { '\t': '\\t', '\r': '\\r', '\n': '\\n', '\\': '\\\\', '"': '\\"', '\0': '\\0' };

/**
 * Rust `<str as Debug>::fmt`: quoted, with `char::escape_debug` escapes for
 * control, format, separator and grapheme-extending characters.
 * @param {string} text
 * @returns {string}
 */
export function rustDebugString(text) {
  let out = '"';
  for (const character of text) {
    if (Object.prototype.hasOwnProperty.call(DEBUG_SHORT, character)) out += DEBUG_SHORT[character];
    else if (character !== ' ' && DEBUG_ESCAPED.test(character)) {
      out += `\\u{${character.codePointAt(0).toString(16)}}`;
    } else out += character;
  }
  return `${out}"`;
}

/** Rust `{:?}` of `Option<i64>`. @param {number|null} value */
export function rustDebugOption(value) {
  return value === null ? 'None' : `Some(${value})`;
}

/** Rust `Duration::as_millis` of a nanosecond count. @param {bigint} nanos */
function millis(nanos) {
  return nanos / NS_PER_MS;
}

/**
 * Rust `<io::Error as Display>::fmt` for an OS error: `{description} (os error {code})`.
 * @param {NodeJS.ErrnoException} error
 * @returns {string}
 */
export function ioErrorText(error) {
  const code = error?.code ? os.constants.errno[error.code] : undefined;
  const key = `execution_os_error_${error?.code}`;
  const description = serverMessage(key);
  if (code === undefined || description === key) return String(error?.message ?? error);
  return serverMessage('execution_os_error', { description, code });
}

// ---------------------------------------------------------------- errors

/** Mirrors rust/src/execution_box/mod.rs `BoxError`; `debug()` is its `{:?}`. */
export class BoxError extends Error {
  /** @param {string} variant @param {Record<string, string>} [fields] */
  constructor(variant, fields = {}) {
    super(variant);
    this.variant = variant;
    this.fields = fields;
  }

  /** `format!("{error:?}")`. */
  debug() {
    const entries = Object.entries(this.fields);
    if (!entries.length) return this.variant;
    return `${this.variant} { ${entries.map(([name, value]) => `${name}: ${rustDebugString(value)}`).join(', ')} }`;
  }
}

const invalidConfiguration = (detail) => new BoxError('InvalidConfiguration', { detail });
const observedError = (detail) => new BoxError('Observed', { detail });

// ---------------------------------------------------------------- backends

/**
 * Mirrors rust/src/execution_box/mod.rs `ExecutionBackend::slug`.
 * @param {{kind: string}} backend
 */
export function backendSlug(backend) {
  switch (backend.kind) {
    case 'host_sandbox':
      return 'host_sandbox';
    case 'start_runner':
      return ['start_runner', backend.isolation].join(':');
    case 'box':
      return ['box', backend.image].join(':');
    default:
      return [backend.kind, backend.name ?? ''].join(':');
  }
}

/** Mirrors rust/src/execution_box/mod.rs `ExecutionBackend::needs_container_runtime`. */
function needsContainerRuntime(backend) {
  return backend.kind === 'box';
}

/** Mirrors rust/src/execution_box/invocation.rs `validate_image`. */
function validateImage(image) {
  if (image && !image.startsWith('-') && /^[A-Za-z0-9/._:@-]+$/.test(image)) return;
  throw invalidConfiguration(serverMessage('execution_unsafe_image'));
}

/** Mirrors rust/src/execution_box/invocation.rs `prefixed_invocation`. */
function prefixedInvocation(runner, prefix, program, args) {
  return { program: runner, arguments: [...prefix, program, ...args] };
}

/** Mirrors rust/src/execution_box/mod.rs `NetworkPolicy::container_flags`. */
function containerFlags(network) {
  return network === 'denied' ? ['--network', 'none'] : [];
}

/** Mirrors rust/src/execution_box/mod.rs `disposable_container_invocation`. */
export function disposableContainerInvocation(image, network, program, args) {
  return {
    program: 'docker',
    arguments: ['run', '--rm', '-i', ...containerFlags(network), image, 'sh', '-lc', CONTAINER_RUNNER, 'formal-ai', program, ...args],
  };
}

/**
 * Mirrors the `shell-words` 1.1 crate's `split`, which
 * rust/src/execution_box/mod.rs `backend_from_configuration` parses the runner with.
 * @param {string} text
 * @returns {Array<string>}
 */
export function shellWordsSplit(text) {
  const words = [];
  let word = '';
  let state = 'delimiter';
  const characters = [...text];
  for (let index = 0; ; index += 1) {
    const c = index < characters.length ? characters[index] : null;
    const blank = c === '\t' || c === ' ' || c === '\n';
    switch (state) {
      case 'delimiter':
        if (c === null) return words;
        if (c === "'") state = 'single';
        else if (c === '"') state = 'double';
        else if (c === '\\') state = 'backslash';
        else if (blank) state = 'delimiter';
        else if (c === '#') state = 'comment';
        else {
          word += c;
          state = 'unquoted';
        }
        break;
      case 'backslash':
      case 'unquoted_backslash':
        if (c === null) {
          words.push(`${word}\\`);
          return words;
        }
        if (c === '\n') state = state === 'backslash' ? 'delimiter' : 'unquoted';
        else {
          word += c;
          state = 'unquoted';
        }
        break;
      case 'unquoted':
        if (c === null) {
          words.push(word);
          return words;
        }
        if (c === "'") state = 'single';
        else if (c === '"') state = 'double';
        else if (c === '\\') state = 'unquoted_backslash';
        else if (blank) {
          words.push(word);
          word = '';
          state = 'delimiter';
        } else word += c;
        break;
      case 'single':
        if (c === null) throw invalidConfiguration(serverMessage('execution_missing_closing_quote'));
        if (c === "'") state = 'unquoted';
        else word += c;
        break;
      case 'double':
        if (c === null) throw invalidConfiguration(serverMessage('execution_missing_closing_quote'));
        if (c === '"') state = 'unquoted';
        else if (c === '\\') state = 'double_backslash';
        else word += c;
        break;
      case 'double_backslash':
        if (c === null) throw invalidConfiguration(serverMessage('execution_missing_closing_quote'));
        if (c !== '\n') word += c === '$' || c === '`' || c === '"' || c === '\\' ? c : `\\${c}`;
        state = 'double';
        break;
      default:
        // comment
        if (c === null) return words;
        if (c === '\n') state = 'delimiter';
    }
  }
}

let boxContract = null;

/** Mirrors rust/src/box_language_projects.rs `box_language_contract` (the fields configuration reads). */
function boxLanguageContract() {
  if (boxContract) return boxContract;
  const records = parseLino(readRepoFile('data/meta/box-language-projects.lino')).children;
  const typed = (type) => records.filter((node) => childValue(node, 'record_type') === type);
  const header = typed('box_language_project_contract')[0];
  boxContract = {
    image_tag: childValue(header, 'image_tag'),
    projects: typed('box_language_project').map((node) => ({
      language: childValue(node, 'language'),
      image: childValue(node, 'image'),
    })),
    deferred: typed('box_language_project_deferred').map((node) => ({
      language: childValue(node, 'language'),
      reason: childValue(node, 'reason'),
    })),
  };
  return boxContract;
}

const present = (value) => {
  const trimmed = value === undefined || value === null ? null : rustTrim(value);
  return trimmed ? trimmed : null;
};

/**
 * Mirrors rust/src/execution_box/mod.rs `backend_from_configuration`. Throws
 * `BoxError`; returns null when nothing is configured.
 * @param {string|null|undefined} startIsolation
 * @param {string|null|undefined} startRunner
 * @param {string|null|undefined} backend
 */
export function backendFromConfiguration(startIsolation, startRunner, backend) {
  const isolation = present(startIsolation);
  const runner = present(startRunner);
  if (isolation !== null && runner !== null) {
    if (isolation !== 'docker') {
      throw invalidConfiguration([serverMessage('execution_unsupported_start_isolation'), isolation].join(': '));
    }
    const [program, ...args] = shellWordsSplit(runner);
    if (program === undefined) throw invalidConfiguration(serverMessage('execution_start_runner_empty'));
    return { kind: 'start_runner', program, arguments: args, isolation };
  }
  if (isolation !== null || runner !== null) {
    throw invalidConfiguration(serverMessage('execution_runner_pair_required'));
  }
  const selected = present(backend);
  if (selected === null) return null;
  if (selected === 'host_sandbox') return { kind: 'host_sandbox' };
  if (selected.startsWith('box-language:') && selected.length > 'box-language:'.length) {
    const language = selected.slice('box-language:'.length);
    const contract = boxLanguageContract();
    const project = contract.projects.find((entry) => entry.language === language);
    if (project) return { kind: 'box', image: `${project.image}:${contract.image_tag}` };
    const deferred = contract.deferred.find((entry) => entry.language === language);
    if (deferred) throw invalidConfiguration(deferred.reason);
    throw invalidConfiguration([serverMessage('execution_no_box_language_contract'), language].join(': '));
  }
  if (selected.startsWith('box:') && selected.length > 'box:'.length) {
    const image = selected.slice('box:'.length);
    validateImage(image);
    return { kind: 'box', image };
  }
  throw invalidConfiguration([serverMessage('execution_unknown_backend'), selected].join(': '));
}

/**
 * Mirrors rust/src/execution_box/mod.rs `backend_from_environment`.
 * @param {Record<string, string|undefined>} [env]
 */
export function backendFromEnvironment(env = process.env) {
  return backendFromConfiguration(env[START_ISOLATION_ENV], env[START_RUNNER_ENV], env[BACKEND_ENV]);
}

// ---------------------------------------------------------------- probes

let startupNs = null;

/** Mirrors rust/src/execution_box/mod.rs `interpreter_startup`, measured once. */
function interpreterStartup() {
  if (startupNs !== null) return startupNs;
  const started = process.hrtime.bigint();
  const observed = spawnSync(HOST_INTERPRETER, ['-c', 'pass'], { stdio: 'ignore' });
  startupNs = observed.error ? STARTUP_FLOOR_NS : process.hrtime.bigint() - started;
  return startupNs;
}

/**
 * Mirrors rust/src/prerequisite/probe.rs `probe_command` for `docker info`,
 * returning the verdict slug. A fresh workspace holds no workspace-scoped
 * toolchains, so `workspace_path` is the ambient `PATH`.
 */
function probeDocker(workspace) {
  const observed = spawnSync('docker', ['info'], {
    cwd: workspace,
    env: { ...process.env, PATH: process.env.PATH ?? '' },
    encoding: 'utf8',
  });
  if (observed.error) return observed.error.code === 'ENOENT' ? 'missing' : 'unusable';
  const stderr = observed.stderr ?? '';
  if (observed.status === COMMAND_NOT_FOUND_EXIT || stderr.includes(serverMessage('execution_command_not_found'))) {
    return 'missing';
  }
  return observed.status === 0 ? 'present' : 'unusable';
}

/** Mirrors rust/src/execution_box/invocation.rs `archive_workspace`. */
function archiveWorkspace(workspace) {
  const output = spawnSync('tar', ['-czf', '-', '-C', workspace, '.'], { maxBuffer: Infinity });
  if (output.error) throw observedError(ioErrorText(output.error));
  if (output.status !== 0) throw observedError(output.stderr.toString('utf8'));
  return output.stdout;
}

// ---------------------------------------------------------------- the box

let nextBox = 0;

/** Mirrors rust/src/execution_box/mod.rs `ExecutionBox`. */
export class ExecutionBox {
  /**
   * Mirrors rust/src/execution_box/mod.rs `ExecutionBox::open` (and `open_in`).
   * @param {object} backend
   * @param {{network: 'denied'|'required', deadlineMs: number}} policy
   */
  static open(backend, policy) {
    const unique = nextBox;
    nextBox += 1;
    const workspace = path.join(
      os.tmpdir(),
      'formal-ai-execution-box',
      backendSlug(backend).replace(/[/:]/g, '_'),
      `${process.pid}-${unique}`,
    );
    try {
      mkdirSync(workspace, { recursive: true });
    } catch (error) {
      throw observedError(ioErrorText(error));
    }
    if (backend.kind === 'box') validateImage(backend.image);
    if (needsContainerRuntime(backend)) {
      if (!rustTrim(process.env[BACKEND_ENV] ?? '')) {
        throw new BoxError('BackendNotConfigured', { backend: backendSlug(backend) });
      }
      const verdict = probeDocker(workspace);
      if (verdict !== 'present') throw new BoxError('NoDaemon', { detail: [backendSlug(backend), verdict].join(':') });
    }
    return new ExecutionBox(backend, policy.network, policy.deadlineMs, workspace);
  }

  constructor(backend, network, deadlineMs, workspace) {
    this.backend = backend;
    this.network = network;
    this.deadlineMs = deadlineMs;
    this.workspace = workspace;
  }

  /** Mirrors rust/src/execution_box/mod.rs `ExecutionBox::script_invocation`. */
  scriptInvocation() {
    const scriptArgument = path.join(this.workspace, SCRIPT_FILE);
    switch (this.backend.kind) {
      case 'host_sandbox':
        return { program: HOST_INTERPRETER, arguments: [scriptArgument] };
      case 'start_runner':
        return prefixedInvocation(this.backend.program, this.backend.arguments, HOST_INTERPRETER, ['-']);
      case 'box':
        return disposableContainerInvocation(this.backend.image, this.network, HOST_INTERPRETER, [SCRIPT_FILE]);
      default:
        throw observedError([serverMessage('execution_browser_runtime_unavailable'), this.backend.name ?? ''].join(': '));
    }
  }

  /**
   * Mirrors rust/src/execution_box/mod.rs `ExecutionBox::run` with no input files.
   * @param {string} script
   * @returns {Promise<{exit_code: number|null, timed_out: boolean, elapsedNs: bigint, deadlineMs: number, partial_output: string}>}
   */
  async run(script) {
    const scriptPath = path.join(this.workspace, SCRIPT_FILE);
    try {
      writeFileSync(scriptPath, script);
    } catch (error) {
      throw observedError(ioErrorText(error));
    }
    this.recordCommand(script);
    switch (this.backend.kind) {
      case 'host_sandbox':
        return this.observeInvocation({ program: HOST_INTERPRETER, arguments: [scriptPath] }, null);
      case 'start_runner':
        return this.observeInvocation(
          prefixedInvocation(this.backend.program, this.backend.arguments, HOST_INTERPRETER, ['-']),
          Buffer.from(script, 'utf8'),
        );
      case 'box': {
        const archive = archiveWorkspace(this.workspace);
        const invocation = disposableContainerInvocation(this.backend.image, this.network, HOST_INTERPRETER, [SCRIPT_FILE]);
        return this.observeInvocation(invocation, archive);
      }
      default:
        throw observedError([serverMessage('execution_browser_runtime_unavailable'), this.backend.name ?? ''].join(': '));
    }
  }

  /**
   * Mirrors rust/src/execution_box/mod.rs `ExecutionBox::observe_invocation`:
   * spawn exact argv, stream both pipes into one buffer as they arrive, and
   * kill the process at the deadline (widened by the measured start-up).
   */
  async observeInvocation(invocation, stdin) {
    const env = { ...process.env };
    if (this.network === 'denied' && this.backend.kind === 'host_sandbox') env.FORMAL_AI_NETWORK = 'denied';
    const started = process.hrtime.bigint();
    const startup = interpreterStartup();
    const bound = BigInt(this.deadlineMs) * NS_PER_MS + (startup > STARTUP_FLOOR_NS ? startup : STARTUP_FLOOR_NS);
    const child = spawn(invocation.program, invocation.arguments, {
      cwd: this.workspace,
      env,
      stdio: [stdin === null ? 'ignore' : 'pipe', 'pipe', 'pipe'],
    });
    let observed = '';
    const collect = (chunk) => {
      for (let offset = 0; offset < chunk.length; offset += READ_BUFFER) {
        observed += chunk.subarray(offset, offset + READ_BUFFER).toString('utf8');
      }
    };
    child.on('error', () => {});
    child.stdin?.on('error', () => {});
    child.stdout.on('data', collect);
    child.stderr.on('data', collect);
    const closed = new Promise((resolve) => child.once('close', resolve));
    let exitInfo = null;
    const exited = new Promise((resolve) => {
      child.once('exit', (code) => {
        exitInfo = { code, at: process.hrtime.bigint() };
        resolve();
      });
    });
    const spawnError = await new Promise((resolve) => {
      child.once('spawn', () => resolve(null));
      child.once('error', resolve);
    });
    if (spawnError) throw observedError(ioErrorText(spawnError));

    if (stdin !== null) {
      const writeError = await new Promise((resolve) => {
        child.stdin.once('error', resolve);
        child.stdin.end(stdin, () => resolve(null));
      });
      if (writeError) {
        child.kill('SIGKILL');
        await exited;
        throw observedError(ioErrorText(writeError));
      }
    }

    let timedOut = false;
    const remainingNs = bound - (process.hrtime.bigint() - started);
    const timer = setTimeout(
      () => {
        if (exitInfo !== null) return;
        timedOut = true;
        child.kill('SIGKILL');
      },
      Number(remainingNs > 0n ? (remainingNs + NS_PER_MS - 1n) / NS_PER_MS : 0n),
    );
    await exited;
    clearTimeout(timer);
    const elapsedNs = exitInfo.at - started;
    await closed;
    return {
      exit_code: timedOut ? null : exitInfo.code,
      timed_out: timedOut,
      elapsedNs,
      deadlineMs: this.deadlineMs,
      partial_output: observed,
    };
  }

  /**
   * Mirrors rust/src/execution_box/mod.rs `ExecutionBox::halving_ladder_with_hard_limit`.
   * @param {string} script uses the explicit `{N}` placeholder
   * @param {bigint} startN
   * @param {number} hardLimitMs
   */
  async halvingLadderWithHardLimit(script, startN, hardLimitMs) {
    const rungs = [];
    let verboseLog = '';
    const started = process.hrtime.bigint();
    let n = startN;
    let hardFailed = false;
    for (;;) {
      if (process.hrtime.bigint() - started >= BigInt(hardLimitMs) * NS_PER_MS) {
        hardFailed = true;
        break;
      }
      const observation = await this.run(script.split('{N}').join(n.toString()));
      rungs.push({ n, timed_out: observation.timed_out, elapsedNs: observation.elapsedNs });
      verboseLog += `N=${n} timed_out=${observation.timed_out} elapsed_ms=${millis(observation.elapsedNs)} deadline_ms=${observation.deadlineMs} exit=${rustDebugOption(observation.exit_code)} output=${rustDebugString(observation.partial_output)}\n`;
      if (!observation.timed_out || n === 0n || rungs.length >= 64) break;
      n /= 2n;
    }
    return { rungs, hard_failed: hardFailed, verbose_log: verboseLog };
  }

  /** Mirrors rust/src/execution_box/mod.rs `ExecutionBox::record_command`. */
  recordCommand(script) {
    try {
      appendFileSync(path.join(this.workspace, COMMAND_LOG), `${script.replace(/\n/g, '\\n')}\n`);
    } catch {
      // The replay log is best effort, as natively.
    }
  }
}

export { millis };
