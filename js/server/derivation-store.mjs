// The white-box derivation record on the JavaScript server (issue #1184,
// R1184-9): the twin of rust/src/derivation.rs `finalize_answer`, which every
// native solve ends with, and of rust/src/cli_explain.rs `run_explain`, the
// `formal-ai explain <answer-id> [--format text|links]` request.
//
// The record, its path and the miss wording all come from
// js/agentic/crate/derivation.mjs (`recordFor`, `storePath`, `missMessage`
// over the seed's `derivation_record_missing` text); this module only supplies
// the `node:fs` io the crate module takes injected and the working directory
// the Rust server writes under (`std::env::current_dir`).

import { mkdirSync, readFileSync, realpathSync, writeFileSync } from 'node:fs';

import {
  DERIVATIONS_DIR,
  RENDER_EMIT_KIND,
  answerDerivationId,
  load,
  missMessage,
  persist,
  recordFor,
  explainText,
  toLino,
} from '../agentic/crate/derivation.mjs';
import { hasHost, installHost } from '../agentic/host.mjs';
import { installNodeHost } from '../agentic/node-host.mjs';
import { parseLino, readRepoFile } from './lino.mjs';

/** The `node:fs` io `persist` / `load` take (`std::fs` in the Rust twin). */
export const nodeDerivationIo = {
  readText: (file) => readFileSync(file, 'utf8'),
  writeText: (file, text) => writeFileSync(file, text),
  createDirAll: (directory) => mkdirSync(directory, { recursive: true }),
};

/** The `--format` values of `ExplainFormat`. */
const EXPLAIN_FORMATS = new Set(['text', 'links']);

/**
 * Mirrors `fn finalize_answer`: append this answer's `render:emit` event,
 * project the record from the log, link it, append its Links Notation and
 * persist it under `<root>/data/cache/derivations/<answer id>.lino`.
 * @param {object} symbolic the `SymbolicAnswer` built from the worker result
 * @param {Array<{kind: string, payload: string}>} events the native log in append order
 * @param {string} root the directory the record is filed under
 * @param {object} [io] the injected filesystem
 */
export function finalizeAnswer(symbolic, events, root, io = nodeDerivationIo) {
  const answerId = answerDerivationId(symbolic.answer);
  const log = [...events, { kind: RENDER_EMIT_KIND, payload: `answer_id=${answerId};format=text` }];
  const record = recordFor(log, answerId);
  const evidence = [...symbolic.evidence_links, `derivation:${answerId}`];
  const persisted = persist(record, root, io);
  evidence.push(persisted.ok
    ? `${DERIVATIONS_DIR}/${answerId}.lino`
    : `derivation:persistence_failed:${persisted.error}`);
  return { ...symbolic, evidence_links: evidence, links_notation: `${symbolic.links_notation}\n${toLino(record)}` };
}

/**
 * `finalize_answer` on the server solve path: the crate module reads the
 * derivation schema and the seed through the agentic host, installed over
 * the booted worker when no host is installed yet.
 * @param {{worker: object, derivationRoot?: string}} ctx
 * @param {object} symbolic
 * @param {object} result the worker's `solve` value
 */
export async function finalizeServerAnswer(ctx, symbolic, result) {
  if (!hasHost()) await installNodeHost(ctx.worker);
  const observedEvents = symbolic?.solver_events ?? result?.solverEvents;
  const events = Array.isArray(observedEvents)
    ? observedEvents.map((event) => ({ kind: String(event.kind), payload: String(event.payload ?? '') }))
    : [];
  return finalizeAnswer(symbolic, events, ctx.derivationRoot ?? process.cwd());
}

/** `cli_paths::resolve_root(None, ".")`: the canonical working directory. */
function resolveRoot(cwd) {
  try {
    return realpathSync(cwd);
  } catch {
    return cwd;
  }
}

/** Rust's `{:?}` of the error string `main` returns. @param {string} text */
function debugString(text) {
  const escaped = text.replaceAll('\\', '\\\\').replaceAll('"', '\\"')
    .replaceAll('\n', '\\n').replaceAll('\r', '\\r').replaceAll('\t', '\\t');
  return `"${escaped}"`;
}

/**
 * Mirrors `fn run_explain` for `explain <answer-id> [--format text|links]`:
 * `{ code, stdout, stderr }`, a miss being the seed's
 * `derivation_record_missing` text as `main`'s `Error: "…"` with exit 1.
 * @param {Array<string>} argv the arguments after `explain`
 * @param {{cwd?: string, io?: object}} [options]
 */
export function runExplain(argv, { cwd = process.cwd(), io = nodeDerivationIo } = {}) {
  let answerId = null;
  let format = 'text';
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--format') format = argv[++index] ?? '';
    else if (arg.startsWith('--format=')) format = arg.slice('--format='.length);
    else if (answerId === null) answerId = arg;
    else return { code: 2, stdout: '', stderr: `unsupported argument: ${arg}\n` };
  }
  if (answerId === null || !EXPLAIN_FORMATS.has(format)) {
    return { code: 2, stdout: '', stderr: `unsupported argument: ${answerId === null ? 'explain' : format}\n` };
  }
  if (!hasHost()) installHost({ readText: readRepoFile, parseLino });
  const root = resolveRoot(cwd);
  const derivation = load(root, answerId, io);
  if (derivation === null) return { code: 1, stdout: '', stderr: `Error: ${debugString(missMessage(root, answerId))}\n` };
  return { code: 0, stdout: format === 'links' ? toLino(derivation) : explainText(derivation), stderr: '' };
}
