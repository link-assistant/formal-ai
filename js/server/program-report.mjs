// The native execution report of a catalog program answer: rust/src/engine/program_answer.rs
// `execution_report` (`execution_status_phrase`, `execution_output_label`,
// `execution_command_lines`) over rust/src/coding/catalog `ProgramLanguage`
// (`execution_status`, `environment`, `execution_notes`).
//
// The browser worker reports what its Web Worker sandbox could run; the
// native server reports the toolchain run recorded in data/seed/toolchains.lino.
// The worker hands over both its own block and the facts the native report is
// built from (`programExecution`, js/worker/formal_ai_worker_write_program_and_research.js), and this
// module swaps one for the other. The wording lives in
// data/seed/coding-guidance.lino.

import { childValue, parseLino, readRepoFile } from './lino.mjs';
import { serverMessage } from './messages.mjs';

const GUIDANCE_FILE = 'data/seed/coding-guidance.lino';
const TOOLCHAINS_FILE = 'data/seed/toolchains.lino';
const RECORD_PROBE = 'toolchain_probe';
const STATUS_VERIFIED = 'verified';
const STATUS_UNAVAILABLE = 'unavailable';
const STATUS_NOT_PROBED = 'not_probed';
const FALLBACK_LANGUAGE = 'en';
const NOTES_PREFIX = 'program_execution_notes_';
const TEXT_FENCE_OPEN = '```text';
const FENCE = '```';

let probes = null;

function probeRows() {
  if (probes) return probes;
  probes = [];
  const visit = (node) => {
    for (const child of node?.children || []) {
      if (childValue(child, 'record_type') === RECORD_PROBE) probes.push(child);
      else visit(child);
    }
  };
  visit(parseLino(readRepoFile(TOOLCHAINS_FILE)));
  return probes;
}

/** `prerequisite::probe::seed_field`: one field of a language's probe row. */
export function toolchainField(language, field) {
  const row = probeRows().find((node) => childValue(node, 'language') === language);
  return row ? childValue(row, field) : '';
}

/** `ProgramLanguage::execution_status`, as the message-key slug. */
export function executionStatus(language) {
  const status = toolchainField(language, 'execution_status');
  return status === STATUS_VERIFIED || status === STATUS_UNAVAILABLE ? status : STATUS_NOT_PROBED;
}

let guidanceRows;
function guidance(key, language, values = {}) {
  guidanceRows ??= new Map(parseLino(readRepoFile(GUIDANCE_FILE)).children
    .filter((row) => childValue(row, 'record_type') === 'coding_guidance')
    .map((row) => [childValue(row, 'id'), row]));
  const row = guidanceRows.get(key);
  const template = childValue(row, language) || childValue(row, FALLBACK_LANGUAGE);
  return template.replace(/\{([a-z_]+)\}/g, (whole, name) => Object.hasOwn(values, name) ? String(values[name]) : whole);
}

function commandLines(program) {
  const lines = [];
  if (program.checkCommand) lines.push(guidance('execution-check-command', FALLBACK_LANGUAGE, { command: program.checkCommand }));
  lines.push(guidance('execution-run-command', FALLBACK_LANGUAGE, { command: program.runCommand }));
  return lines;
}

/**
 * `ProgramLanguage::execution_notes`: the catalog row's note, else the seed's
 * `program_execution_notes_<slug>` report text.
 */
function executionNotes(language, seedReport) {
  const key = `${NOTES_PREFIX}${language}`;
  const note = serverMessage(key);
  if (note !== key) return note;
  const seeded = typeof seedReport === 'function' ? seedReport(key) : null;
  return seeded ?? key;
}

/**
 * Mirrors rust/src/engine/program_answer.rs `execution_report`.
 * @param {{language: string, checkCommand?: string|null, runCommand: string, output: string, composed?: boolean}} program
 * @param {string} responseLanguage
 * @param {(intent: string) => string|null} [seedReport] the seed's English report text
 * @returns {string}
 */
export function executionReport(program, responseLanguage, seedReport) {
  if (program.rediscoveredFrom) return rediscoveredReport(program, responseLanguage);
  const status = program.composed ? STATUS_NOT_PROBED : executionStatus(program.language);
  const slug = status.replace('_', '-');
  const statusLine = guidance(program.composed ? 'execution-composed-line' : 'execution-status-line', responseLanguage, {
    status: guidance(`execution-status-${slug}`, responseLanguage),
    environment: toolchainField(program.language, 'environment'),
  });
  const outputLabel = guidance(`execution-output-${slug}`, responseLanguage);
  const lines = [
    statusLine,
    ...commandLines(program),
    `${outputLabel}:`,
    TEXT_FENCE_OPEN,
    program.output,
    FENCE,
  ];
  if (!program.composed) lines.push(executionNotes(program.language, seedReport));
  return lines.join('\n');
}

/**
 * The report of a program the documentation route rediscovered and no
 * recorded run verified (issue #1165): its status names the page and the
 * decomposition check, in the response language when the messages carry it,
 * and it borrows no run or note of the language's recorded program. Mirrors
 * the `rediscovered_from` branch of the native `execution_report`.
 * @param {{checkCommand?: string|null, runCommand: string, output: string, rediscoveredFrom: string}} program
 * @param {string} responseLanguage
 * @returns {string}
 */
function rediscoveredReport(program, responseLanguage) {
  const statusLine = guidance('execution-rediscovered', responseLanguage, { page: program.rediscoveredFrom });
  const outputLabel = guidance('execution-output-unavailable', responseLanguage);
  return [statusLine, ...commandLines(program), `${outputLabel}:`, TEXT_FENCE_OPEN, program.output, FENCE].join('\n');
}

/**
 * The answer text with the worker's sandbox block replaced by the native
 * report; unchanged when the answer carries no program execution or the block
 * is no longer in the text.
 * @param {string} answer
 * @param {object|undefined} program the worker's `programExecution`
 * @param {(intent: string) => string|null} [seedReport]
 * @returns {string}
 */
export function nativeProgramAnswer(answer, program, seedReport) {
  if (!program || typeof program.block !== 'string' || !program.block) return answer;
  const at = answer.indexOf(program.block);
  if (at < 0) return answer;
  const report = executionReport(program, String(program.responseLanguage || FALLBACK_LANGUAGE), seedReport);
  return answer.slice(0, at) + report + answer.slice(at + program.block.length);
}
