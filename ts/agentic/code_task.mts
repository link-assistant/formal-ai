import { rustIdentifierIsValid } from './code_task/identifier_domain.mjs';
import { firstRoleMatch as sourceRoleMatch } from './crate/seed_meanings.mjs';
import { sourceDescriptionContract, sourceWhitespaceSupported, sourceRegistrationContract } from './code_task/source_contract.mjs';
import { ownsLiteralBody } from './crate/literal_body_ownership.mjs';
import { FinalDisposition, resolvedFinalAnswer } from './plan.mjs';
// Small Rust source items generated from a coding request
// (rust/src/agentic_coding/code_task.rs).

import { Capability } from './capability.mjs';
import { guardedSourceStep } from './code_task/target_guard.mjs';
import { toolFor } from './capability_router.mjs';
import { latestResult, resultForCommand } from './code_artifact.mjs';
import { composeEditRequest, composeGeneralChangePlan } from './general_planner.mjs';
import { finalAnswer, jsonText, planOne, writeArguments } from './plan.mjs';
import { observedBytesMatch, responseLanguage } from './tool_result.mjs';
import { unwrapTransportQuotes } from './crate/normal_markov.mjs';
import { localizedResponse, responseFor } from './crate/seed.mjs';
import { firstRoleMatch, words } from './crate/seed_meanings.mjs';
import { mentionsRole, roleWordForms, wordsForRole } from './write_lexicon.mjs';
import { isAsciiAlphanumeric, isAsciiDigit, trim, trimEndMatches } from './write_str.mjs';

const RustItemKind = Object.freeze({ Function: 'function', Constant: 'constant', Test: 'test' });

const eqIgnoreAsciiCase = (left, right) => left.replace(/[A-Z]/g, (c) => c.toLowerCase()) === right.replace(/[A-Z]/g, (c) => c.toLowerCase());

/**
 * Mirrors `fn plan_generated_source_step`.
 * @param {string} rawTask
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planGeneratedSourceStep(rawTask, messages, toolNames) {
  const task = unwrapTransportQuotes(rawTask);
  if (composeEditRequest(task) !== null) return null;
  const artifact = rustSourceForTask(task);
  if (!artifact) return null;
  const writeTool = toolFor(toolNames, Capability.Write);
  if (!writeTool) return null;
  let latestUser = -1;
  for (let index = messages.length - 1; index >= 0 && latestUser < 0; index -= 1) {
    if (eqIgnoreAsciiCase(messages[index].role, 'user')) latestUser = index;
  }
  if (latestUser < 0) return null;
  const currentTurn = messages.slice(latestUser + 1);
  const observed = resultForCommand(currentTurn, `cat ${artifact.path}`);
  if (observed !== null) {
    const outcome = observedBytesMatch(observed, artifact.content) ? 'coding_workspace_effect_observed' : 'coding_workspace_verification_failed';
    const rendered = renderSeededOutcome(outcome, task, artifact.path);
    return rendered === null ? null : finalAnswer(rendered);
  }
  if (latestResult(currentTurn, Capability.Write) !== null) {
    const runTool = toolFor(toolNames, Capability.Run);
    if (runTool) return planOne(runTool, jsonText({ command: `cat ${artifact.path}` }));
    const rendered = renderSeededOutcome('coding_workspace_written_unverified', task, artifact.path);
    return rendered === null ? null : finalAnswer(rendered);
  }
  return guardedSourceStep(task, artifact, currentTurn, toolNames)
    ?? planOne(writeTool, writeArguments(artifact.path, artifact.content));
}

/** Mirrors `fn rust_source_for_task`: `{path, content}` or null. */
export function rustSourceForTask(task) {
  const normalized = task.toLowerCase();
  if (!mentionsRole('file_write_action_cue', normalized) && !mentionsRole('program_request', normalized)) return null;
  const path = rustPath(task);
  if (path === null) return null;
  const extraKind = firstRoleMatch('coding_artifact_kind', normalized);
  let kind;
  if (extraKind?.slug === 'coding_constant') kind = RustItemKind.Constant;
  else if (extraKind?.slug === 'coding_test') kind = RustItemKind.Test;
  else if (firstRoleMatch('program_kind', normalized)?.slug === 'function') kind = RustItemKind.Function;
  else return null;
  const name = requestedIdentifier(task, path, kind);
  if (name === null) return null;
  const numbers = numericLiterals(task.replace(path, () => ''));
  const visibility = mentionsRole('coding_visibility', normalized) ? 'pub ' : '';
  let content;
  if (kind === RustItemKind.Function) {
    const value = numbers[numbers.length - 1];
    if (normalized.includes('f64') && mentionsRole('coding_division_action', normalized)) {
      if (value === undefined) return null;
      content = renderRustTemplate('coding_source_function_division', [['{visibility}', visibility], ['{name}', name], ['{divisor}', value]]);
    } else if (mentionsRole('coding_return_action', normalized)) {
      if (value === undefined) return null;
      content = renderRustTemplate('coding_source_function_return', [['{visibility}', visibility], ['{name}', name], ['{value}', value]]);
    } else return null;
  } else if (kind === RustItemKind.Constant) {
    if (!normalized.includes('&str')) return null;
    const value = slotIdentifier(normalized, 'coding_value_slot');
    if (value === null) return null;
    content = renderRustTemplate('coding_source_string_constant', [['{visibility}', visibility], ['{name}', name], ['{value}', value]]);
  } else {
    const [left, right, expected] = numbers;
    if (left === undefined || right === undefined) return null;
    const meaning = firstRoleMatch('arithmetic_operator_word', normalized);
    const operator = meaning ? words(meaning).find((surface) => !/\p{Alphabetic}/u.test(surface)) : undefined;
    content = expected !== undefined && operator !== undefined
      ? renderRustTemplate('coding_source_binary_operation_test', [
        ['{name}', name], ['{left}', left], ['{operator}', operator], ['{right}', right], ['{expected}', expected],
      ])
      : renderRustTemplate('coding_source_equality_test', [['{name}', name], ['{left}', left], ['{right}', right]]);
  }
  return content === null ? null : { path, content };
}

/** Mirrors `fn render_rust_template`. */
export function renderRustTemplate(intent, substitutions) {
  const template = responseFor(intent, 'rust');
  return template === null ? null : renderTemplate(template, substitutions);
}

/** Mirrors `fn render_seeded_outcome`. */
export function renderSeededOutcome(intent, task, path) {
  const template = localizedResponse(intent, responseLanguage(task));
  return template === null || template === undefined ? null : renderTemplate(template, [['{path}', path]]);
}

/** Mirrors `fn render_seeded_change`. */
export function renderSeededChange(intent, task, path, slots) {
  const template = localizedResponse(intent, responseLanguage(task));
  return template === null || template === undefined ? null : renderTemplate(template, [['{path}', path], ...slots]);
}

/** Mirrors `fn render_seeded_list_change`: an explicitly typed list slot. */
export function renderSeededListChange(intent, task, path, slot, values) {
  return renderSeededChange(intent, task, path, [[slot, values]]);
}

// A placeholder the seed sentence wraps in backticks becomes a CommonMark code
// span: a value holding a backtick run gets a longer fence, padded when it
// starts or ends with a backtick, so the inserted text reads back verbatim.
// Mirrors `code_span` in rust/src/agentic_coding/code_task.rs.
function codeSpanItem(value) {
  const longest = Math.max(0, ...(value.match(/`+/g) ?? []).map((run) => run.length));
  if (longest === 0) return `\`${value}\``;
  const fence = '`'.repeat(longest + 1);
  const pad = value.startsWith('`') || value.endsWith('`') ? ' ' : '';
  return `${fence}${pad}${value}${pad}${fence}`;
}

// A scalar payload is fenced whole, even when it contains comma-separated code.
const codeSpan = codeSpanItem;

function renderTemplate(template, substitutions) {
  const values = substitutions.map(([slot, value]) => {
    const quoted = Array.isArray(value) ? value.map(codeSpanItem).join(', ') : codeSpan(value);
    return [slot, Array.isArray(value) ? quoted : value, quoted];
  });
  let rendered = '';
  let cursor = 0;
  while (cursor < template.length) {
    const found = values.map(([slot, plain, quoted]) => {
      const fenced = '`' + slot + '`';
      if (template.startsWith(fenced, cursor)) return [fenced.length, quoted];
      return template.startsWith(slot, cursor) ? [slot.length, plain] : null;
    }).find((value) => value !== null);
    if (found) { rendered += found[1]; cursor += found[0]; }
    else { rendered += template[cursor]; cursor += 1; }
  }
  return rendered;
}

const isPathCharacter = (character) => isAsciiAlphanumeric(character) || '_-./'.includes(character);

function rustPath(task) {
  const found = task.indexOf('.rs');
  if (found < 0) return null;
  const end = found + 3;
  let start = end;
  while (start > 0 && isPathCharacter(task[start - 1])) start -= 1;
  if (start === end) start = 0;
  const path = task.slice(start, end);
  return path !== '' && !path.startsWith('/') && !path.split('/').some((component) => component === '..') ? path : null;
}

function requestedIdentifier(task, path, kind) {
  const normalized = task.replace(path, () => ' ').toLowerCase();
  if (kind !== RustItemKind.Constant) {
    const name = slotIdentifier(normalized, 'coding_name_slot');
    if (name !== null && validIdentifier(name)) return name;
  }
  if (kind === RustItemKind.Function) {
    const meaning = firstRoleMatch('program_kind', normalized);
    for (const surface of meaning === null ? [] : words(meaning)) {
      const escaped = surface.replace(/[.*+?^\x24{}()|[\]\\]/g, '\\$&');
      const expression = new RegExp('(?:^|[^A-Za-z_0-9])' + escaped + '[ \u0009-\u000D]+([A-Za-z_][A-Za-z_0-9]*)', 'iu');
      const captured = expression.exec(task);
      if (captured !== null && validIdentifier(captured[1])) return captured[1];
    }
  }
  let candidates = identifierTokens(task.replace(path, () => '')).filter(validIdentifier);
  if (kind === RustItemKind.Constant) {
    candidates = candidates.filter((candidate) => candidate.includes('_') && /^[A-Z_]+$/.test(candidate));
  } else {
    candidates = candidates.filter((candidate) => candidate.includes('_'));
  }
  return candidates[0] ?? null;
}

function slotIdentifier(normalized, role) {
  for (const form of roleWordForms(role)) {
    let found;
    if (form.slot === 'prefix') {
      const prefix = trim(form.before).toLowerCase();
      const at = normalized.indexOf(prefix);
      found = at < 0 ? undefined : identifierTokens(normalized.slice(at + prefix.length))[0];
    } else if (form.slot === 'suffix') {
      const suffix = trim(form.after).toLowerCase();
      const at = normalized.indexOf(suffix);
      found = at < 0 ? undefined : identifierTokens(normalized.slice(0, at)).pop();
    } else if (form.slot === 'circumfix') {
      const before = trim(form.before).toLowerCase();
      const after = trim(form.after).toLowerCase();
      const at = normalized.indexOf(before);
      if (at >= 0) {
        const remainder = normalized.slice(at + before.length);
        const end = remainder.indexOf(after);
        found = end < 0 ? undefined : identifierTokens(remainder.slice(0, end))[0];
      }
    }
    if (found !== undefined) return found;
  }
  return null;
}

function identifierTokens(text) {
  return text.split(/[^A-Za-z0-9_]/u).filter((token) => token !== '');
}

function validIdentifier(identifier) { return rustIdentifierIsValid(identifier); }

function numericLiterals(text) {
  const chars = Array.from(text);
  const values = [];
  let index = 0;
  while (index < chars.length) {
    const character = chars[index];
    if (!isAsciiDigit(character) || (index > 0 && (isAsciiAlphanumeric(chars[index - 1]) || chars[index - 1] === "_"))) {
      index += 1;
      continue;
    }
    let endIndex = index + 1;
    while (endIndex < chars.length && (isAsciiDigit(chars[endIndex]) || chars[endIndex] === '.')) endIndex += 1;
    if (endIndex < chars.length && (isAsciiAlphanumeric(chars[endIndex]) || chars[endIndex] === "_")) {
      index = endIndex + 1;
      continue;
    }
    const signedStart = index > 0 && chars[index - 1] === '-' && (index < 2 || /[ \u0009-\u000D]/u.test(chars[index - 2])) ? index - 1 : index;
    const value = trimEndMatches(chars.slice(signedStart, endIndex).join(''), (c) => c === '.');
    if (value !== '') values.push(value);
    index = endIndex;
  }
  return values;
}

/** Mirrors plan_verified_generated_source_step: pre-mask dispatch requires full source contract and preserves byte ownership. */
export function planVerifiedGeneratedSourceStep(rawTask, messages, toolNames) {
  const task = unwrapTransportQuotes(rawTask);
  if (composeEditRequest(task) !== null) return null;
  const literal = composeGeneralChangePlan(task);
  if (literal?.mode === 'literal_file' && ownsLiteralBody(task, literal.content)) return null;
  const compound = sourceRegistrationContract(task, rustSourceForTask);
  if (compound !== null) return compound.wholeRequestConsumed ? null : resolvedFinalAnswer(
    renderSeededOutcome('coding-source-authoring-contract-missing', task, ''),
    FinalDisposition.Gap, 'source-registration-goal-coverage-unbound');
  const artifact = rustSourceForTask(task);
  if (artifact === null) {
    const body = literal?.mode === 'literal_file' ? literal.content.toLowerCase() : '';
    if (sourceRoleMatch('program_language_alias', body)?.slug === 'program_language_rust'
      && sourceRoleMatch('program_kind', body)?.slug === 'function') return resolvedFinalAnswer(
        renderSeededOutcome('coding-source-authoring-contract-missing', task, ''),
        FinalDisposition.Gap, 'source-description-goal-coverage-unbound');
    return null;
  }
  if (!sourceWhitespaceSupported(rawTask) || sourceDescriptionContract(task, artifact) === null) return resolvedFinalAnswer(
    renderSeededOutcome('coding-source-authoring-contract-missing', task, ''),
    FinalDisposition.Gap, 'source-description-goal-coverage-unbound');
  return planGeneratedSourceStep(task, messages, toolNames);
}

/** Mirrors verified_source_description: whole-request evidence before semantic dispatch. */
export function verifiedSourceDescription(rawTask) {
  const task = unwrapTransportQuotes(rawTask);
  if (composeEditRequest(task) !== null) return null;
  const literal = composeGeneralChangePlan(task);
  if (literal?.mode === 'literal_file' && ownsLiteralBody(task, literal.content)) return null;
  const compound = sourceRegistrationContract(task, rustSourceForTask);
  if (compound !== null) return compound;
  if (!sourceWhitespaceSupported(rawTask)) return null;
  const artifact = rustSourceForTask(task);
  return artifact === null ? null : sourceDescriptionContract(task, artifact);
}
