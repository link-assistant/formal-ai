// Telegram code execution: rust/src/telegram_runtime.rs
// `execute_telegram_code_request` and
// `execute_telegram_code_request_from_environment`, over the execution box
// port in ./execution-box.mjs. Every sentence is a seed response
// (data/seed/multilingual-responses-*.lino), read as the Rust `seed` module does.

import { readdirSync } from 'node:fs';
import path from 'node:path';

import {
  BoxError,
  ExecutionBox,
  backendFromEnvironment,
  backendSlug,
  millis,
  rustDebugOption,
  rustTrimEnd,
} from './execution-box.mjs';
import { EvidenceSource, ObservationKind, observedEvidence } from './execution-evidence.mjs';
import { REPO_ROOT, childValue, childrenNamed, parseLino, readRepoFile } from './lino.mjs';

/** Mirrors rust/src/telegram_runtime.rs `TELEGRAM_EXECUTION_HARD_LIMIT` (ten minutes). */
export const TELEGRAM_EXECUTION_HARD_LIMIT_MS = 10 * 60 * 1000;

/** The production deadline rust/src/telegram_runtime.rs passes (`Duration::from_secs(60)`). */
const TELEGRAM_EXECUTION_DEADLINE_MS = 60 * 1000;

let responseTable = null;

/** Mirrors rust/src/seed `response_for` over every multilingual-responses file. */
export function responseFor(intent, language) {
  if (!responseTable) {
    responseTable = new Map();
    const files = readdirSync(path.join(REPO_ROOT, 'data/seed'))
      .filter((name) => name.startsWith('multilingual-responses') && name.endsWith('.lino'))
      .sort();
    for (const file of files) {
      for (const record of childrenNamed(parseLino(readRepoFile(`data/seed/${file}`)), 'response')) {
        const key = `${childValue(record, 'intent')}\u0000${childValue(record, 'language')}`;
        if (!responseTable.has(key)) responseTable.set(key, childValue(record, 'text'));
      }
    }
  }
  const text = responseTable.get(`${intent}\u0000${language}`);
  return text === undefined ? null : text;
}

/** Mirrors rust/src/seed `split_pipe_list`. */
function splitPipeList(raw) {
  const trimmed = raw.trim();
  if (!trimmed) return [];
  if (trimmed.startsWith('(') && trimmed.endsWith(')')) {
    return [...trimmed.slice(1, -1).matchAll(/"((?:[^"\\]|\\.)*)"|(\S+)/g)].map((match) => match[1] ?? match[2]);
  }
  return trimmed.split('|').map((part) => part.trim()).filter(Boolean);
}

/** Mirrors rust/src/telegram_runtime.rs `requested_python`: the code a prompt asks to run, or null. */
export function requestedPython(prompt, language) {
  const normalized = prompt.toLowerCase();
  const markers = splitPipeList(responseFor('code_execution_request_markers', language) ?? '');
  if (!markers.some((marker) => normalized.includes(marker))) return null;
  const open = prompt.indexOf('```');
  if (open >= 0) {
    const after = prompt.slice(open + 3);
    const body = after.startsWith('python') ? after.slice(6) : after.startsWith('py') ? after.slice(2) : after;
    const close = body.indexOf('```');
    if (close >= 0) {
      const code = body.slice(0, close).trim();
      return code || null;
    }
  }
  const separator = Math.max(prompt.lastIndexOf(':'), prompt.lastIndexOf('：'));
  if (separator < 0) return null;
  const code = prompt.slice(separator + 1).trim();
  return code && code.includes('(') ? code : null;
}

/** Mirrors rust/src/telegram_runtime.rs `initial_iteration_bound`: the largest u64 digit run. */
function initialIterationBound(prompt) {
  const U64_MAX = (1n << 64n) - 1n;
  let best = null;
  for (const [digits] of prompt.matchAll(/[0-9]+/g)) {
    const value = BigInt(digits);
    if (value <= U64_MAX && (best === null || value > best)) best = value;
  }
  return best;
}

/** Mirrors rust/src/telegram_runtime.rs `localized` (`seed::localized_response`). */
export function localizedResponse(intent, language) {
  return responseFor(intent, language) ?? responseFor(intent, 'unknown') ?? responseFor(intent, 'en') ?? intent;
}

/** Mirrors rust/src/telegram_runtime.rs `render`: placeholders filled in order. */
export function render(intent, language, values) {
  let text = localizedResponse(intent, language);
  for (const [name, value] of values) text = text.split(`{${name}}`).join(value);
  return text;
}

/** Mirrors rust/src/event_log.rs `render_fields`. */
function renderFields(fields) {
  return fields.map(([name, value]) => `${name}=${value}`).join(' ');
}

const failed = (language, backend, log) => ({
  kind: 'failed',
  answer: render('code_execution_failed', language, [
    ['backend', backend],
    ['log', log],
  ]),
});

function asBoxError(error) {
  if (error instanceof BoxError) return error;
  throw error;
}

/**
 * Mirrors rust/src/telegram_runtime.rs `execute_telegram_code_request`.
 * Supplying the backend is the permission boundary: null is an honest refusal.
 * @param {string} prompt
 * @param {string} language the `detect_language` slug
 * @param {object|null} backend
 * @param {number} deadlineMs
 * @returns {Promise<{kind: 'not_requested'|'refused'|'observed'|'failed', answer?: string, evidence?: object, ladder?: object|null}>}
 */
export async function executeTelegramCodeRequest(prompt, language, backend, deadlineMs) {
  const code = requestedPython(prompt, language);
  if (code === null) return { kind: 'not_requested' };
  if (backend === null) return { kind: 'refused', answer: localizedResponse('code_execution_refused', language) };
  const slug = backendSlug(backend);
  const policy = { network: 'denied', deadlineMs: Math.min(deadlineMs, TELEGRAM_EXECUTION_HARD_LIMIT_MS) };
  let boxed;
  try {
    boxed = ExecutionBox.open(backend, policy);
  } catch (error) {
    return failed(language, slug, asBoxError(error).debug());
  }

  let ladder = null;
  if (code.includes('{N}')) {
    try {
      ladder = await boxed.halvingLadderWithHardLimit(
        code,
        initialIterationBound(prompt) ?? 1n,
        TELEGRAM_EXECUTION_HARD_LIMIT_MS,
      );
    } catch (error) {
      return failed(language, slug, asBoxError(error).debug());
    }
  }
  if (ladder !== null && ladder.hard_failed) return failed(language, slug, ladder.verbose_log);

  let invocation;
  let observation;
  try {
    invocation = boxed.scriptInvocation();
    observation = await boxed.run(code);
  } catch (error) {
    return failed(language, slug, asBoxError(error).debug());
  }
  const argv = [invocation.program, ...invocation.arguments];
  const evidence = observedEvidence(
    argv.join(' '),
    argv,
    observation.exit_code,
    Buffer.from(observation.partial_output, 'utf8'),
    ObservationKind.CommandExit,
    EvidenceSource.LocalProcess,
  );
  evidence.for_need = 'telegram_code_execution';
  evidence.produced_by = 'telegram_execution_box';

  const elapsedMs = String(millis(observation.elapsedNs));
  const deadline = String(observation.deadlineMs);
  if (observation.timed_out || observation.exit_code !== 0) {
    const log = `${renderFields([
      ['timed_out', String(observation.timed_out)],
      ['elapsed_ms', elapsedMs],
      ['deadline_ms', deadline],
      ['exit', rustDebugOption(observation.exit_code)],
    ])}\n${observation.partial_output}`;
    return failed(language, slug, log);
  }
  const answer = render('code_execution_observed', language, [
    ['backend', slug],
    ['exit', String(observation.exit_code ?? 0)],
    ['elapsed_ms', elapsedMs],
    ['deadline_ms', deadline],
    ['output', rustTrimEnd(observation.partial_output)],
    ['evidence', evidence.evidence_id],
  ]);
  return { kind: 'observed', answer, evidence, ladder };
}

/**
 * Mirrors rust/src/telegram_runtime.rs `execute_telegram_code_request_from_environment`:
 * missing configuration is an honest refusal, invalid configuration an observed failure.
 * @param {string} prompt
 * @param {string} language the `detect_language` slug
 * @param {Record<string, string|undefined>} [env]
 */
export async function executeTelegramCodeRequestFromEnvironment(prompt, language, env = process.env) {
  let backend;
  try {
    backend = backendFromEnvironment(env);
  } catch (error) {
    const boxError = asBoxError(error);
    if (requestedPython(prompt, language) === null) return { kind: 'not_requested' };
    return failed(language, 'configuration', boxError.debug());
  }
  return executeTelegramCodeRequest(prompt, language, backend, TELEGRAM_EXECUTION_DEADLINE_MS);
}
