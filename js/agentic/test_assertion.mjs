// Adding one assertion to a test file (PR #1188 G13): "Add an assertion that
// add(2, 2) equals 4 to m.test.mjs." A request that adds the seeded
// `coding_assertion_kind` concept to one named test file, and states a call,
// the seeded `coding_assertion_equality_cue` and a value, is written in the
// file's own assertion form -- the form of its last assertion line, from the
// language's `assertion_styles` in data/meta/function-test-contracts.lino
// (`assert.equal` or `expect(…).toBe`, `assert … ==` or `self.assertEqual`) --
// on the line after it at its indentation, and the file is then run with its
// seeded runner. The JavaScript twin of rust/src/agentic_coding/test_assertion.rs.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { renderSeededChange, renderSeededOutcome } from './code_task.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { mentionsRole, wordsForRole } from './crate/seed_meanings.mjs';
import { cached, readText } from './host.mjs';
import { editArguments } from './intent_router.mjs';
import { extensionLanguage, pathsIn, readSource } from './module_function.mjs';
import { finalAnswer, jsonText, planOne, writeArguments } from './plan.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';
import { testFileCommand } from './test_file_runner.mjs';
import { failureMessage, render } from './tool_result.mjs';
import { changedLinesEdit, readArguments, resultForCommand, resultForEdit, resultForPath } from './workspace_change.mjs';
import { findChildValue, parseLinoRoot } from './write_lino.mjs';
import { bareSurfaces, cleanCueToken, cleanPathToken, tokens } from './write_request.mjs';

const VALUE_EDGES = /^[\s.,;:!?。，！？]+|[\s.,;:!?。，！？]+$/gu;
const STOP_ROLES = ['file_edit_new_lead_cue', 'file_edit_target_cue', 'file_write_destination_cue', 'file_edit_joiner_cue'];
const CUE_ROLES = ['coding_assertion_equality_cue', 'coding_expectation_cue'];
const CLAUSE_END = /[,;，；]$/u;
const WORKSPACE_TEST_COMMAND = 'formal-ai:workspace-test';

/** The seeded `assertion_styles` record of `language`, or null. */
function stylesRecord(language) {
  const root = cached('test_assertion:styles',
    () => parseLinoRoot(readText('data/meta/function-test-contracts.lino') ?? '').children[0] ?? null);
  return (root?.children || []).find((node) => node.name === 'assertion_styles' && node.id === language) ?? null;
}

/** Mirrors `fn assertion_styles`: the seeded assertion forms of `language`. */
function assertionStyles(language) {
  return (stylesRecord(language)?.children || []).filter((node) => node.name === 'style')
    .map((node) => String(node.id ?? node.value ?? ''));
}

/**
 * Mirrors `fn new_test_file`: a new test file of `language` at `path` holding
 * the one `assertion` of the function `name`, from the seeded `new_file`
 * template, the module named by the seeded `test_name` pattern; or null.
 */
function newTestFile(language, path, name, assertion) {
  const record = stylesRecord(language);
  const [prefix, suffix = null] = findChildValue(record, 'test_name').split('{stem}');
  const file = path.slice(path.lastIndexOf('/') + 1);
  if (suffix === null || file.length <= prefix.length + suffix.length || !file.startsWith(prefix) || !file.endsWith(suffix)) {
    return null;
  }
  const stem = file.slice(prefix.length, file.length - suffix.length);
  const template = findChildValue(record, 'new_file');
  return template === '' ? null
    : template.split('{stem}').join(stem).split('{name}').join(name).split('{assertion}').join(assertion);
}

/**
 * Mirrors `fn assertion_request`: `{path, language, actual, expected}` for a
 * request that adds an assertion of a stated call and value to one named
 * test file whose language has assertion forms, or null.
 * @param {string} task
 */
export function assertionRequest(task) {
  const normalized = normalizePrompt(task).toLowerCase();
  // `Add an assertion that … to f`, or `Create a test in f that …` (G25).
  const adds = mentionsRole('coding_member_add_action', normalized) && mentionsRole('coding_assertion_kind', normalized);
  const writes = mentionsRole('coding_request_verb', normalized) && mentionsRole('coding_test_artifact_kind', normalized);
  if (!adds && !writes) return null;
  const paths = pathsIn(task);
  if (paths.length !== 1) return null;
  const [path] = paths;
  const language = extensionLanguage(path);
  if (language === null || assertionStyles(language).length === 0) return null;
  const lowered = task.toLowerCase().length === task.length ? task.toLowerCase() : task;
  let cue = null;
  for (const surface of CUE_ROLES.flatMap(wordsForRole)) {
    const at = lowered.indexOf(surface.toLowerCase());
    if (at >= 0 && (cue === null || at < cue[0])) cue = [at, at + surface.length];
  }
  if (cue === null) return null;
  const actual = lastCall(task.slice(0, cue[0]));
  if (actual === null) return null;
  const stops = STOP_ROLES.flatMap(bareSurfaces);
  const value = [];
  for (const token of tokens(task).filter((candidate) => candidate.start >= cue[1])) {
    if (cleanPathToken(token.text) === path || stops.includes(cleanCueToken(token.text))) break;
    value.push(token);
    if (CLAUSE_END.test(token.text)) break;
  }
  if (value.length === 0) return null;
  const expected = task.slice(value[0].start, value.at(-1).end).replace(VALUE_EDGES, '');
  return expected === '' ? null : { path, language, actual, expected };
}

/**
 * Mirrors `fn last_call`: the last call written in `text` the way a request
 * states one (`add(2, 2)`): a name, then parentheses that hold no
 * parentheses; or null.
 */
function lastCall(text) {
  const named = (character) => /^[A-Za-z0-9_$.]$/u.test(character);
  let last = null;
  for (let open = text.indexOf('('); open >= 0; open = text.indexOf('(', open + 1)) {
    const close = text.indexOf(')', open + 1);
    if (close < 0 || text.slice(open + 1, close).includes('(')) continue;
    const end = text.slice(0, open).trimEnd().length;
    let start = end;
    while (start > 0 && named(text[start - 1])) start -= 1;
    while (start < end && /^[0-9.]$/u.test(text[start])) start += 1;
    if (start < end) last = text.slice(start, close + 1);
  }
  return last;
}

/**
 * Mirrors `fn last_assertion`: `[index, style, indentation]` of the file's
 * last line written in one of `styles` (the longest lead when several fit),
 * or null.
 */
function lastAssertion(source, styles) {
  const lines = source.split('\n');
  for (let index = lines.length - 1; index >= 0; index -= 1) {
    const line = lines[index].endsWith('\r') ? lines[index].slice(0, -1) : lines[index];
    const text = line.trim();
    let best = null;
    for (const style of styles) {
      const [head, rest = ''] = style.split('{actual}');
      const [middle, tail = ''] = rest.split('{expected}');
      const fits = text.startsWith(head) && text.endsWith(tail)
        && text.length >= head.length + middle.length + tail.length
        && text.slice(head.length, text.length - tail.length).includes(middle);
      if (fits && (best === null || head.length > best.head)) best = { style, head: head.length };
    }
    if (best !== null) return [index, best.style, line.slice(0, line.length - line.trimStart().length)];
  }
  return null;
}

/**
 * Mirrors `fn plan_test_assertion_step`: read the test file, add the
 * assertion in its own form, run the file, and state both.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planTestAssertionStep(task, messages, toolNames) {
  const request = assertionRequest(task);
  if (request === null) return null;
  const currentTurn = messages.slice(evidenceWindowStart(messages));
  const source = readSource(currentTurn, request.path);
  if (source === null) {
    const read = toolFor(toolNames, Capability.Read);
    return read === null ? null : planOne(read, readArguments(request.path));
  }
  if (source === '') return planNewTestFile(task, request, currentTurn, toolNames);
  const found = lastAssertion(source, assertionStyles(request.language));
  if (found === null) return null;
  const [index, style, indentation] = found;
  const assertion = style.split('{actual}').join(request.actual).split('{expected}').join(request.expected);
  const lines = source.split('\n');
  const updated = [...lines.slice(0, index + 1), `${indentation}${assertion}`, ...lines.slice(index + 1)].join('\n');
  const edit = changedLinesEdit(source, updated);
  const editTool = toolFor(toolNames, Capability.Edit);
  if (edit === null || editTool === null) return null;
  const edited = resultForEdit(currentTurn, request.path, edit[0], edit[1]);
  if (edited === null) return planOne(editTool, editArguments(request.path, edit[0], edit[1]));
  if (failureMessage(edited, false, true) !== null) {
    const failed = renderSeededOutcome('coding_workspace_verification_failed', task, request.path);
    return failed === null ? null : finalAnswer(failed);
  }
  return planRunStep(task, request.path, currentTurn, toolNames, 'test_assertion_added', assertion);
}

/**
 * Mirrors `fn plan_new_test_file`: a missing test file is written whole, in
 * the language's new-file form around the one assertion (PR #1188 G25), then
 * run.
 */
function planNewTestFile(task, request, currentTurn, toolNames) {
  const [style] = assertionStyles(request.language);
  const assertion = style.split('{actual}').join(request.actual).split('{expected}').join(request.expected);
  const name = request.actual.slice(0, request.actual.indexOf('(')).trim();
  const content = newTestFile(request.language, request.path, name, assertion);
  const writeTool = toolFor(toolNames, Capability.Write);
  if (content === null || writeTool === null) return null;
  const written = resultForPath(currentTurn, Capability.Write, request.path, content);
  if (written === null) return planOne(writeTool, writeArguments(request.path, content));
  if (failureMessage(written, false, true) !== null) {
    const failed = renderSeededOutcome('coding_workspace_verification_failed', task, request.path);
    return failed === null ? null : finalAnswer(failed);
  }
  return planRunStep(task, request.path, currentTurn, toolNames, 'test_file_written', assertion);
}

/**
 * Mirrors `fn plan_run_step`: run the test file with its seeded runner, then
 * state the change by `intent` and the run.
 */
function planRunStep(task, path, currentTurn, toolNames, intent, assertion) {
  const command = testFileCommand(WORKSPACE_TEST_COMMAND, path);
  const shell = toolFor(toolNames, Capability.Run);
  if (command === null || shell === null) return null;
  const raw = resultForCommand(currentTurn, command);
  if (raw === null) return planOne(shell, jsonText({ command }));
  const stated = renderSeededChange(intent, task, path, [['{assertion}', assertion], ['{command}', command]]);
  return stated === null ? null : finalAnswer(`${stated}\n\n${render(command, raw, task)}`);
}
