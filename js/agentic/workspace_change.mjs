// Grounded workspace rewrites and composite module changes
// (rust/src/agentic_coding/workspace_change.rs).

import { Capability } from './capability.mjs';
import { classifyTool, toolFor } from './capability_router.mjs';
import { sourceFromAgentReadResult, sourceFromReadResult } from './code_artifact.mjs';
import { renderRustTemplate, renderSeededChange, renderSeededOutcome, rustSourceForTask } from './code_task.mjs';
import { plainText } from './content.mjs';
import { composeEditRequest } from './general_planner.mjs';
import { editArguments } from './intent_router.mjs';
import { finalAnswer, jsonText, planOne, writeArguments } from './plan.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';
import { composePositionalInsert, unescapeProseNewlines, unquotedPathTokens } from './positional_edit.mjs';
import { bareSurfaces, cleanCueToken, cleanPathToken, looksLikeFilePath, safeRelativePath, tokens } from './write_request.mjs';
import { commandArgument, failureMessage } from './tool_result.mjs';
import { quotedSegmentSpans, quotedSegments, unwrapTransportQuotes } from './crate/normal_markov.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { correctedSpelling } from './crate/spelling.mjs';
import {
  RewriteScope, executeScopedWorkspaceRewrite, isIdentifierWord, wordScopedMatches,
} from './crate/workspace_change_learning.mjs';
import { mentionsRole, wordsForRole } from './write_lexicon.mjs';
import { isAsciiAlphanumeric, lines, matchIndices, splitWhitespace, trim } from './write_str.mjs';

const eqIgnoreAsciiCase = (left, right) => left.replace(/[A-Z]/g, (c) => c.toLowerCase()) === right.replace(/[A-Z]/g, (c) => c.toLowerCase());
const finalOrNull = (text) => (text === null ? null : finalAnswer(text));

const statedIntent = (rewrite) => rewrite.intent ?? (rewrite.renaming ? 'coding_identifier_renamed' : 'coding_text_replaced');
const statedSlots = (rewrite) => rewrite.slots ?? [['{old}', rewrite.pattern], ['{new}', rewrite.replacement]];

/**
 * Mirrors `fn plan_workspace_change_step`.
 * @param {string} rawTask
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planWorkspaceChangeStep(rawTask, messages, toolNames) {
  const task = unwrapTransportQuotes(rawTask);
  const currentTurn = messages.slice(evidenceWindowStart(messages));
  const change = compositeModuleChange(task);
  if (change) return planCompositeStep(task, currentTurn, toolNames, change);
  const rewrite = groundedRewrite(task);
  if (rewrite) return planRewriteStep(task, currentTurn, toolNames, rewrite);
  const computed = groundedEndInsertion(task) ?? groundedRemoval(task) ?? groundedSetting(task) ?? groundedTypoFix(task);
  return computed ? planComputedChangeStep(task, currentTurn, toolNames, computed) : null;
}

/** Mirrors `fn is_verification_failure_answer`. */
export function isVerificationFailureAnswer(rawTask, answer) {
  const task = unwrapTransportQuotes(rawTask);
  const edit = composeEditRequest(task);
  const targets = [...(edit ? [edit[0]] : []), ...rustPaths(task)];
  return targets.some((target) => renderSeededOutcome('coding_workspace_verification_failed', task, target) === answer);
}

function failed(task, target) {
  return finalOrNull(renderSeededOutcome('coding_workspace_verification_failed', task, target));
}

function planWithTool(toolNames, capability, args) {
  const tool = toolFor(toolNames, capability);
  return tool ? planOne(tool, args) : null;
}

function planRewriteStep(task, currentTurn, toolNames, rewrite) {
  const read = resultForPath(currentTurn, Capability.Read, rewrite.target, null);
  if (read === null) return planWithTool(toolNames, Capability.Read, readArguments(rewrite.target));
  const source = sourceFromReadResult(read);
  const updated = rewrittenSource(source, rewrite);
  if (updated === null) return failed(task, rewrite.target);
  const occurrences = rewrite.scope === RewriteScope.Substring
    ? matchIndices(source, rewrite.pattern).length
    : wordScopedMatches(source, rewrite.pattern).length;
  const verified = { target: rewrite.target, expected: updated, intent: statedIntent(rewrite), slots: statedSlots(rewrite) };
  if (occurrences === 1) {
    const tool = toolFor(toolNames, Capability.Edit);
    const edit = matchIndices(source, rewrite.pattern).length === 1
      ? [rewrite.pattern, rewrite.replacement]
      : changedLinesEdit(source, updated);
    if (tool && edit) {
      if (resultForEdit(currentTurn, rewrite.target, edit[0], edit[1]) === null) {
        return planOne(tool, editArguments(rewrite.target, edit[0], edit[1]));
      }
      return planDigestVerification(task, currentTurn, toolNames, verified);
    }
  } else if (toolFor(toolNames, Capability.Edit)) {
    const command = repeatedIdentifierRewriteCommand(rewrite);
    const tool = command === null ? null : toolFor(toolNames, Capability.Run);
    if (tool) {
      if (resultForCommand(currentTurn, command) === null) return planOne(tool, jsonText({ command }));
      return planDigestVerification(task, currentTurn, toolNames, verified);
    }
  }
  if (resultForPath(currentTurn, Capability.Write, rewrite.target, updated) === null) {
    return planWithTool(toolNames, Capability.Write, writeArguments(rewrite.target, updated));
  }
  const command = `cat ${rewrite.target}`;
  const observed = resultForCommand(currentTurn, command);
  if (observed === null) return planWithTool(toolNames, Capability.Run, jsonText({ command }));
  if (observed !== updated) return failed(task, rewrite.target);
  return finalOrNull(renderSeededChange(statedIntent(rewrite), task, rewrite.target, statedSlots(rewrite)));
}

function planCompositeStep(task, currentTurn, toolNames, change) {
  if (resultForPath(currentTurn, Capability.Write, change.source_path, change.source) === null) {
    return planWithTool(toolNames, Capability.Write, writeArguments(change.source_path, change.source));
  }
  const sourceCommand = `cat ${change.source_path}`;
  const observedSource = resultForCommand(currentTurn, sourceCommand);
  if (observedSource === null) return planWithTool(toolNames, Capability.Run, jsonText({ command: sourceCommand }));
  if (observedSource !== change.source) return failed(task, change.source_path);
  const read = resultForPath(currentTurn, Capability.Read, change.registration_path, null);
  if (read === null) return planWithTool(toolNames, Capability.Read, readArguments(change.registration_path));
  const current = sourceFromReadResult(read);
  const updated = insertRegistration(current, change.registration);
  const registered = `\`${trim(change.registration)}\``;
  const members = [['{members}', registered]];
  if (updated === current) {
    return finalOrNull(renderSeededChange('coding_member_already_present', task, change.registration_path, members));
  }
  const compact = compactRegistrationEdit(current, change.registration);
  const editTool = compact ? toolFor(toolNames, Capability.Edit) : null;
  if (compact && editTool) {
    const [old, next] = compact;
    if (resultForEdit(currentTurn, change.registration_path, old, next) === null) {
      return planOne(editTool, editArguments(change.registration_path, old, next));
    }
    return planDigestVerification(task, currentTurn, toolNames, {
      target: change.registration_path, expected: updated, intent: 'coding_member_inserted', slots: members,
    });
  }
  if (resultForPath(currentTurn, Capability.Write, change.registration_path, updated) === null) {
    return planWithTool(toolNames, Capability.Write, writeArguments(change.registration_path, updated));
  }
  const registrationCommand = `cat ${change.registration_path}`;
  const observed = resultForCommand(currentTurn, registrationCommand);
  if (observed === null) return planWithTool(toolNames, Capability.Run, jsonText({ command: registrationCommand }));
  if (observed !== updated) return failed(task, change.registration_path);
  return finalOrNull(renderSeededChange('coding_member_inserted', task, change.registration_path, members));
}

/**
 * The file after `rewrite`, or null when it cannot apply. A positional
 * insertion replaces its one anchor occurrence; an anchor found zero or
 * several times does not say where the line goes.
 */
function rewrittenSource(source, rewrite) {
  if (rewrite.unique) {
    return matchIndices(source, rewrite.pattern).length === 1
      ? source.replace(rewrite.pattern, () => rewrite.replacement)
      : null;
  }
  const execution = executeScopedWorkspaceRewrite(source, rewrite.pattern, rewrite.replacement, rewrite.scope);
  return execution.ok ? execution.ok.output : null;
}

/**
 * The smallest run of whole lines holding every change from `source` to
 * `updated`, as an edit pair, when that run occurs once in `source` — the
 * edit tool refuses an `oldString` it finds twice, and a bare word can sit
 * inside a longer one (`smal` in `small`).
 */
function changedLinesEdit(source, updated) {
  const MAX_EDIT_BYTES = 4096;
  let prefix = 0;
  while (prefix < source.length && prefix < updated.length && source[prefix] === updated[prefix]) prefix += 1;
  let suffix = 0;
  while (suffix < source.length - prefix && suffix < updated.length - prefix
    && source[source.length - 1 - suffix] === updated[updated.length - 1 - suffix]) suffix += 1;
  const start = source.lastIndexOf('\n', prefix - 1) + 1;
  const lineEnd = source.indexOf('\n', source.length - suffix);
  const end = lineEnd < 0 ? source.length : lineEnd;
  const old = source.slice(start, end);
  const next = updated.slice(start, updated.length - (source.length - end));
  if (old === '' || new TextEncoder().encode(old).length > MAX_EDIT_BYTES) return null;
  return matchIndices(source, old).length === 1 ? [old, next] : null;
}

/**
 * `Insert 'x' after the line 'y' in f`: the anchor line, replaced once by the
 * anchor beside the new line, stated by the position meaning that placed it.
 */
function groundedPositionalInsert(task) {
  const insert = composePositionalInsert(task);
  if (!insert) return null;
  const [target, anchor, replacement] = insert;
  const after = replacement.startsWith(`${anchor}\n`);
  const inserted = after ? replacement.slice(anchor.length + 1) : replacement.slice(0, -(anchor.length + 1));
  return {
    target, pattern: anchor, replacement, scope: RewriteScope.Substring, unique: true,
    intent: after ? 'file_edit_position_after' : 'file_edit_position_before',
    slots: [['{new}', inserted], ['{anchor}', anchor]],
  };
}

function groundedRewrite(task) {
  const positional = groundedPositionalInsert(task);
  if (positional) return positional;
  const edit = composeEditRequest(task);
  if (!edit) return null;
  const [target, oldClause, newClause] = edit;
  const renaming = mentionsRole('coding_identifier_rename_action', task.toLowerCase());
  // The edit request unescapes `\n` / `\t` in its literals, so the quoted
  // segments are compared unescaped too: a multi-line replacement written with
  // `\n` was otherwise not "quoted", and the general fallback wrote its new
  // text over the whole file (PR #1188 sub-agent gap 3).
  const quoted = quotedSegments(task).map(unescapeProseNewlines);
  let old;
  let next;
  if (quoted.includes(oldClause) && quoted.includes(newClause)) {
    [old, next] = [oldClause, newClause];
  } else if (renaming) {
    old = identifierTokens(oldClause).pop();
    next = identifierTokens(newClause)[0];
    if (old === undefined || next === undefined) return null;
  } else return null;
  // A word replaced by a longer word that contains it (`smal` -> `small`) is
  // only safe word by word: a substring rewrite would never terminate.
  const words = isIdentifierWord(old) && isIdentifierWord(next);
  const scope = words && (renaming || next.includes(old)) ? RewriteScope.Word : RewriteScope.Substring;
  if (old === '' || old === next || (scope === RewriteScope.Substring && next.includes(old))) return null;
  return { target, pattern: old, replacement: next, scope, renaming: renaming && scope === RewriteScope.Word };
}

/**
 * The one workspace path a request names and its one quoted segment that is
 * not that path: `{target, text}` or null.
 */
function quotedPayloadAndPath(task) {
  const segments = quotedSegmentSpans(task);
  const inside = (token) => segments.some((segment) => token.start >= segment.start && token.end <= segment.end
    && cleanPathToken(token.text) !== segment.text);
  const paths = [...new Set(tokens(task).filter((token) => !inside(token)).map((token) => cleanPathToken(token.text))
    .filter((path) => looksLikeFilePath(path) && safeRelativePath(path)))];
  if (paths.length !== 1) return null;
  const payloads = segments.filter((segment) => segment.text !== paths[0] && trim(segment.text) !== '');
  return payloads.length === 1 ? { target: paths[0], text: payloads[0].text } : null;
}

/**
 * An additive edit at one end of a named file (`append 'x' to notes.txt`,
 * `prepend "x" to a.md`), positioned by the seeded `file_edit_position_end` /
 * `file_edit_position_start` meanings. A missing file is created, as `>>`
 * would.
 */
function groundedEndInsertion(task) {
  const lowered = task.toLowerCase();
  const atEnd = mentionsRole('file_edit_position_end', lowered);
  if (atEnd === mentionsRole('file_edit_position_start', lowered)) return null;
  const named = quotedPayloadAndPath(task) ?? blankLineAndPath(task) ?? linePayloadAndPath(task);
  if (!named) return null;
  return {
    target: named.target,
    compute: (source) => insertedAtEnd(source, unescapeProseNewlines(named.text), atEnd),
    edit: (source, updated) => (source === '' ? null : compactEndEdit(source, updated, atEnd)),
    // An empty line has no text to quote back; it is stated in its own words.
    intent: named.text === '' ? 'file_edit_blank_line' : atEnd ? 'file_edit_position_end' : 'file_edit_position_start',
    slots: [['{new}', named.text]],
  };
}

/**
 * `Delete the line 'x' from notes.txt`, `remove "TODO" from a.md`: the seeded
 * `coding_text_remove_action` with one quoted payload and one path. Every
 * line that is exactly the payload goes; otherwise its one occurrence inside
 * a line does. A missing file, or a payload found nowhere or more than once
 * inside lines, is not a removal anyone can verify.
 */
function groundedRemoval(task) {
  const lowered = task.toLowerCase();
  if (!mentionsRole('coding_text_remove_action', lowered) || mentionsRole('coding_member_add_action', lowered)) return null;
  const named = quotedPayloadAndPath(task);
  if (!named) return null;
  return {
    target: named.target,
    compute: (source, missing) => (missing ? null : removedLiteral(source, named.text)),
    edit: changedLinesEdit,
    intent: 'coding_text_remove',
    slots: [['{old}', named.text]],
  };
}

/**
 * `Change the value of "debug" to true in config.json`: the seeded
 * `config_value_lead` names a setting, the edit request's old clause names
 * its key (quoted, or its last identifier) and the new clause its value. The
 * one line assigning that key (`"debug": …`, `debug: …`, `debug = …`) gets
 * the new value; a key assigned nowhere or more than once is not a change
 * anyone can verify.
 */
function groundedSetting(task) {
  if (!mentionsRole('config_value_lead', task.toLowerCase())) return null;
  const edit = composeEditRequest(task);
  if (!edit) return null;
  const [target, oldClause, value] = edit;
  const quoted = quotedSegments(oldClause);
  const key = quoted.length === 1 ? quoted[0] : identifierTokens(oldClause).pop();
  if (key === undefined || trim(value) === '') return null;
  return {
    target,
    compute: (source, missing) => (missing ? null : assignedSetting(source, key, trim(value), target)),
    edit: changedLinesEdit,
    intent: 'setting',
    slots: [['{old}', key], ['{new}', trim(value)]],
  };
}

const escapeRegExp = (text) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
/** A value every config format writes bare: a boolean, null or a number. */
const isBareLiteral = (value) => /^(true|false|null|-?\d+(\.\d+)?)$/u.test(value);

function assignedSetting(source, key, value, target) {
  const pattern = new RegExp(`^(\\s*(["']?)${escapeRegExp(key)}\\2\\s*[:=]\\s*)(.*?)(\\s*,?\\s*)$`, 'u');
  const lines = source.split('\n');
  const matches = lines.map((line, index) => [index, pattern.exec(line)]).filter(([, match]) => match !== null);
  if (matches.length !== 1) return null;
  const [index, match] = matches[0];
  const [, head, , old, tail] = match;
  let written = value;
  const quote = /^["']/u.exec(old)?.[0];
  if (quote && !isBareLiteral(value)) written = `${quote}${value}${quote}`;
  else if (!quote && target.endsWith('.json') && !isBareLiteral(value) && !/^["'[{]/u.test(value)) written = JSON.stringify(value);
  if (written === old) return null;
  lines[index] = `${head}${written}${tail}`;
  return lines.join('\n');
}

/**
 * `Fix the typo 'smal' in README.md`: the seeded `typo_fix_lead` with one
 * quoted word and one path, and no stated correction. The correction is
 * discovered (`correctedSpelling`), and the change is the word-scoped
 * replacement a stated correction would have made.
 */
function groundedTypoFix(task) {
  if (!mentionsRole('typo_fix_lead', task.toLowerCase())) return null;
  const named = quotedPayloadAndPath(task);
  if (!named || !isIdentifierWord(named.text)) return null;
  const correction = correctedSpelling(named.text);
  if (correction === null) return null;
  return {
    target: named.target,
    compute: (source, missing) => {
      if (missing) return null;
      const execution = executeScopedWorkspaceRewrite(source, named.text, correction, RewriteScope.Word);
      return execution.ok ? execution.ok.output : null;
    },
    edit: changedLinesEdit,
    intent: 'coding_text_replaced',
    slots: [['{old}', named.text], ['{new}', correction]],
  };
}

function removedLiteral(source, text) {
  const kept = source.split(/(?<=\n)/u).filter((line) => line.replace(/\r?\n$/u, '') !== text);
  const withoutLines = kept.join('');
  if (withoutLines !== source) return withoutLines;
  return matchIndices(source, text).length === 1 ? source.replace(text, '') : null;
}

/** The file after the insertion; a file without a final newline keeps none. */
function insertedAtEnd(source, text, atEnd) {
  if (source === '') return `${text}\n`;
  if (!atEnd) return `${text}\n${source}`;
  return source.endsWith('\n') ? `${source}${text}\n` : `${source}\n${text}`;
}

/**
 * The smallest unique run of whole lines at the insertion end, and what it
 * becomes, so the edit tool carries the change rather than the whole file.
 */
function compactEndEdit(source, updated, atEnd) {
  const MAX_ANCHOR_BYTES = 4096;
  const breaks = matchIndices(source, '\n');
  const cuts = atEnd
    ? [0, ...breaks.map((index) => index + 1).filter((start) => start < source.length)].reverse()
    : [...breaks.map((index) => index + 1), source.length];
  for (const cut of cuts) {
    const anchor = atEnd ? source.slice(cut) : source.slice(0, cut);
    if (trim(anchor) === '') continue;
    if (new TextEncoder().encode(anchor).length > MAX_ANCHOR_BYTES) break;
    if (matchIndices(source, anchor).length !== 1) continue;
    return [anchor, atEnd ? updated.slice(cut) : updated.slice(0, updated.length - (source.length - cut))];
  }
  return null;
}

/**
 * A change whose new bytes are computed from the file: read it, compute, send
 * the smallest unique edit (or the whole file), check the digest, and state
 * the change from the seeded response for its intent.
 */
function planComputedChangeStep(task, currentTurn, toolNames, change) {
  const { target, intent, slots } = change;
  const read = resultForPath(currentTurn, Capability.Read, target, null);
  if (read === null) return planWithTool(toolNames, Capability.Read, readArguments(target));
  // A read that came back as the client's file block is the file, whatever
  // its text says: a ledger that quotes "Error:" lines is not a missing file.
  const missing = sourceFromAgentReadResult(read) === null && failureMessage(read, false, true) !== null;
  const source = missing ? '' : sourceFromReadResult(read);
  const updated = change.compute(source, missing);
  if (updated === null || updated === source) return failed(task, target);
  const compact = change.edit(source, updated);
  const editTool = compact ? toolFor(toolNames, Capability.Edit) : null;
  if (compact && editTool) {
    const [old, next] = compact;
    if (resultForEdit(currentTurn, target, old, next) === null) return planOne(editTool, editArguments(target, old, next));
    return planDigestVerification(task, currentTurn, toolNames, { target, expected: updated, intent, slots });
  }
  if (resultForPath(currentTurn, Capability.Write, target, updated) === null) {
    return planWithTool(toolNames, Capability.Write, writeArguments(target, updated));
  }
  return planDigestVerification(task, currentTurn, toolNames, { target, expected: updated, intent, slots });
}

function compositeModuleChange(task) {
  if (!mentionsRole('coding_module_registration_action', task.toLowerCase())) return null;
  const generated = rustSourceForTask(task);
  if (!generated) return null;
  const registrationPath = rustPaths(task).find((path) => path !== generated.path);
  if (registrationPath === undefined) return null;
  const file = generated.path.split('/').pop();
  if (!file.endsWith('.rs')) return null;
  const module = file.slice(0, -3);
  if (!validIdentifier(module)) return null;
  const registration = renderRustTemplate('coding_source_module_registration', [['{module}', module]]);
  if (registration === null) return null;
  return { source_path: generated.path, source: generated.content, registration_path: registrationPath, registration };
}

function insertRegistration(source, registration) {
  if (lines(source).some((line) => trim(line) === trim(registration))) return source;
  let updated = source;
  if (updated !== '' && !updated.endsWith('\n')) updated += '\n';
  return updated + registration;
}

function compactRegistrationEdit(source, registration) {
  const MAX_SUFFIX_BYTES = 4096;
  const starts = [0, ...matchIndices(source, '\n').map((index) => index + 1).filter((start) => start < source.length)];
  for (const start of starts.reverse()) {
    const suffix = source.slice(start);
    if (new TextEncoder().encode(suffix).length > MAX_SUFFIX_BYTES) break;
    if (matchIndices(source, suffix).length !== 1) continue;
    const replacement = `${suffix.endsWith('\n') ? suffix : `${suffix}\n`}${registration}`;
    return [suffix, replacement];
  }
  return null;
}

function repeatedIdentifierRewriteCommand(rewrite) {
  if (!shellSafeIdentifier(rewrite.pattern) || !shellSafeIdentifier(rewrite.replacement)) return null;
  const pattern = rewrite.scope === RewriteScope.Substring ? rewrite.pattern : `\\b${rewrite.pattern}\\b`;
  return ['perl', '-pi', '-e', `'s/${pattern}/${rewrite.replacement}/g'`, '--', rewrite.target].join(' ');
}

function shellSafeIdentifier(identifier) {
  return /^[A-Za-z0-9_]+$/.test(identifier);
}

function identifierTokens(text) {
  return text.split(/[^A-Za-z0-9_]/u).filter(validIdentifier);
}

function validIdentifier(identifier) {
  return /^[A-Za-z_][A-Za-z0-9_]*$/.test(identifier) && !wordsForRole('identifier_reserved_word').includes(identifier);
}

const isPathCharacter = (character) => isAsciiAlphanumeric(character) || '_-./'.includes(character);

function rustPaths(task) {
  const paths = [];
  for (const suffix of matchIndices(task, '.rs')) {
    const end = suffix + 3;
    let start = end;
    while (start > 0 && isPathCharacter(task[start - 1])) start -= 1;
    const path = task.slice(start, end);
    if (path !== '' && !path.startsWith('/') && !path.split('/').some((component) => component === '..') && !paths.includes(path)) {
      paths.push(path);
    }
  }
  return paths;
}

function parseObject(text) {
  try {
    const value = JSON.parse(text);
    return value !== null && typeof value === 'object' && !Array.isArray(value) ? value : null;
  } catch {
    return null;
  }
}

const stringField = (value, key) => (typeof value[key] === 'string' ? value[key] : null);

function resultForPath(messages, capability, path, expectedContent) {
  return matchingResult(messages, (name, args) => {
    if (classifyTool(name) !== capability) return false;
    const value = parseObject(args);
    if (!value) return false;
    return argumentMatchesPath(value, path) && (expectedContent === null || stringField(value, 'content') === expectedContent);
  });
}

function resultForEdit(messages, path, old, next) {
  return matchingResult(messages, (name, args) => {
    if (classifyTool(name) !== Capability.Edit) return false;
    const value = parseObject(args);
    return value !== null && argumentMatchesPath(value, path)
      && ['oldString', 'old_string', 'old_str', 'old'].some((key) => stringField(value, key) === old)
      && ['newString', 'new_string', 'new_str', 'new'].some((key) => stringField(value, key) === next);
  });
}

function argumentMatchesPath(value, expected) {
  return ['path', 'filePath', 'file_path'].some((key) => {
    const observed = stringField(value, key);
    return observed !== null && workspacePathMatches(expected, observed);
  });
}

function components(path) {
  return path.split('/').filter((part, index) => part !== '' && (part !== '.' || index === 0));
}

function workspacePathMatches(expected, observed) {
  if (observed === expected) return true;
  if (expected.startsWith('/') || !observed.startsWith('/')) return false;
  const want = components(expected);
  const have = components(observed);
  return want.length <= have.length && want.every((part, index) => have[have.length - want.length + index] === part);
}

function resultForCommand(messages, command) {
  return matchingResult(messages, (name, args) => classifyTool(name) === Capability.Run && commandArgument(args) === command);
}

function planDigestVerification(task, currentTurn, toolNames, change) {
  const command = `sha256sum -- ${change.target}`;
  const observed = resultForCommand(currentTurn, command);
  if (observed === null) return planWithTool(toolNames, Capability.Run, jsonText({ command }));
  if (splitWhitespace(observed)[0] !== sha256Hex(change.expected)) return failed(task, change.target);
  return finalOrNull(renderSeededChange(change.intent, task, change.target, change.slots));
}

function matchingResult(messages, matches) {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (!eqIgnoreAsciiCase(message.role, 'tool')) continue;
    const id = message.tool_call_id;
    if (id === null || id === undefined) continue;
    let call = null;
    for (let prior = index - 1; prior >= 0 && call === null; prior -= 1) {
      const calls = messages[prior].tool_calls || [];
      for (let at = calls.length - 1; at >= 0; at -= 1) {
        if (calls[at].id === id) {
          call = calls[at];
          break;
        }
      }
    }
    if (call === null) continue;
    if (matches(call.function.name, call.function.arguments)) return plainText(message.content);
  }
  return null;
}

function readArguments(path) {
  return jsonText({ path, filePath: path, file_path: path });
}

/**
 * `Append the line third to notes.txt`: no quoted text, one path, and the
 * seeded `file_edit_line_lead`; the line is the words after the lead up to
 * the last destination cue before the path (`go to bed to notes.txt` keeps
 * `go to bed`). A request without a lead is declined, never guessed.
 */

/**
 * `Append an empty line to notes.txt`: the seeded `file_edit_blank_line` and
 * one named path, with no quoted text, adds one empty line at the named end.
 */
function linePayloadAndPath(task) {
  if (quotedSegmentSpans(task).length > 0) return null;
  const toks = tokens(task);
  const pathIndices = toks.map((token, index) => [index, cleanPathToken(token.text)])
    .filter(([, path]) => looksLikeFilePath(path) && safeRelativePath(path));
  if (new Set(pathIndices.map(([, path]) => path)).size !== 1) return null;
  const [pathIndex, target] = pathIndices[0];
  const leads = wordsForRole('file_edit_line_lead').map((lead) => splitWhitespace(lead.toLowerCase()))
    .filter((words) => words.length > 0).sort((left, right) => right.length - left.length);
  let leadEnd = -1;
  for (let index = 0; index < pathIndex && leadEnd < 0; index += 1) {
    const lead = leads.find((words) => words.every((word, offset) => index + offset < pathIndex
      && cleanCueToken(toks[index + offset].text) === word));
    if (lead) leadEnd = index + lead.length;
  }
  if (leadEnd < 0) return null;
  const destinations = bareSurfaces('file_write_destination_cue');
  let end = -1;
  for (let index = leadEnd; index < pathIndex; index += 1) {
    if (destinations.includes(cleanCueToken(toks[index].text))) end = index;
  }
  if (end <= leadEnd) return null;
  return { target, text: task.slice(toks[leadEnd].start, toks[end - 1].end) };
}

function blankLineAndPath(task) {
  if (quotedSegmentSpans(task).length > 0 || !mentionsRole('file_edit_blank_line', task.toLowerCase())) return null;
  const paths = [...new Set(unquotedPathTokens(task).map((token) => cleanPathToken(token.text))
    .filter((path) => looksLikeFilePath(path) && safeRelativePath(path)))];
  return paths.length === 1 ? { target: paths[0], text: '' } : null;
}
