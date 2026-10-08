// Insert quoted members into a source list a request names
// (rust/src/agentic_coding/structured_edit.rs).
//
// The source scan works on UTF-8 bytes, as Rust's does, so delimiter and
// char-literal offsets agree byte for byte; request-side offsets are
// JavaScript string indices (only their order is compared).

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { latestResult, sourceFromReadResult } from './code_artifact.mjs';
import { renderSeededChange, renderSeededOutcome } from './code_task.mjs';
import { insertMembersViaLinks } from './link_edit_rules.mjs';
import { finalAnswer, jsonText, planOne, writeArguments } from './plan.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';
import { resolveRequirementTarget } from './requirement_resolution.mjs';
import { quotedSegmentSpans, unwrapTransportQuotes } from './crate/normal_markov.mjs';
import { mentionsRole } from './write_lexicon.mjs';
import { isAlphanumeric, isAsciiAlphanumeric, isAsciiDigit, isWhitespace, trimEndMatches, trimMatches } from './write_str.mjs';

const MAX_MEMBER_LENGTH = 96;
const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** `str::get(start..end)` over UTF-8 bytes: the text, or null off a char boundary. */
function byteSlice(bytes, start, end = bytes.length) {
  const boundary = (at) => at === bytes.length || (at < bytes.length && (bytes[at] & 0xc0) !== 0x80);
  if (start > end || end > bytes.length || !boundary(start) || !boundary(end)) return null;
  return decoder.decode(bytes.subarray(start, end));
}

/**
 * Mirrors `fn plan_structured_edit_step`.
 * @param {string} rawTask
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planStructuredEditStep(rawTask, messages, toolNames) {
  const task = unwrapTransportQuotes(rawTask);
  const edit = memberInsertion(task);
  if (!edit) return null;
  const currentTurn = messages.slice(evidenceWindowStart(messages));
  const read = latestResult(currentTurn, Capability.Read);
  const source = read === null ? '' : sourceFromReadResult(read);
  if (source === '') {
    const readTool = toolFor(toolNames, Capability.Read);
    return readTool ? planOne(readTool, readArguments(edit.target)) : null;
  }
  const result = insertMembers(source, edit);
  if (!result) return null;
  const [updated, inserted] = result;
  // A read that already lists every member is itself the observation: a run
  // resumed after a client compaction answers at once instead of spending
  // another full-file observation that pushes it over the threshold again.
  if (!inserted.length && latestResult(currentTurn, Capability.Write) === null) {
    const rendered = renderSeededChange('coding_member_already_present', task, edit.target, [['{members}', quotedList(edit.values)]]);
    return rendered === null ? null : finalAnswer(rendered);
  }
  // Members the file already lists need no write: after a client compaction
  // the run re-reads the file it already changed, and rewriting it in full
  // pushed the session straight back over the compaction threshold (the
  // issue #1028 ladder looped read -> write until its budget ran out).
  if (inserted.length && latestResult(currentTurn, Capability.Write) === null) {
    const writeTool = toolFor(toolNames, Capability.Write);
    return writeTool ? planOne(writeTool, writeArguments(edit.target, updated)) : null;
  }
  const observed = latestResult(currentTurn, Capability.Run);
  if (observed !== null) {
    let rendered;
    if (observed === updated) {
      const [intent, change] = inserted.length
        ? ['coding_member_inserted', quotedList(inserted)]
        : ['coding_member_already_present', quotedList(edit.values)];
      rendered = renderSeededChange(intent, task, edit.target, [['{members}', change]]);
    } else {
      rendered = renderSeededOutcome('coding_workspace_verification_failed', task, edit.target);
    }
    return rendered === null ? null : finalAnswer(rendered);
  }
  const runTool = toolFor(toolNames, Capability.Run);
  return runTool ? planOne(runTool, jsonText({ command: `cat ${edit.target}` })) : null;
}

function wordsJoined(text) {
  const tokens = [];
  let current = '';
  for (const character of text.toLowerCase()) {
    if (isAlphanumeric(character)) current += character;
    else {
      if (current) tokens.push(current);
      current = '';
    }
  }
  if (current) tokens.push(current);
  return tokens.join(' ');
}

function requestsMemberInsertion(normalized) {
  const changesAFile = mentionsRole('file_write_action_cue', normalized) || mentionsRole('file_edit_action_cue', normalized);
  return changesAFile && mentionsRole('coding_member_list_kind', normalized) && mentionsRole('coding_member_add_action', normalized);
}

function memberInsertion(task) {
  if (!requestsMemberInsertion(wordsJoined(task))) return null;
  const values = [];
  const named = [];
  let candidates = [];
  let literalTarget = null;
  const literalSpans = [];
  for (const segment of quotedSegmentSpans(task)) {
    const markedUp = task.slice(segment.start).startsWith('`');
    const path = isWorkspacePath(segment.text) ? segment.text : null;
    if (markedUp) {
      if (path !== null) candidates.push([segment.start, path]);
      else if (validIdentifier(segment.text) && !named.includes(segment.text)) named.push(segment.text);
    } else if (isMemberLiteral(segment.text)) {
      // A value quoted again (a ladder node's `result=` clause repeats it) is
      // still one member.
      if (!values.includes(segment.text)) values.push(segment.text);
      if (literalTarget === null) literalTarget = path;
    }
    literalSpans.push([segment.start, segment.end]);
  }
  if (!values.length) return null;
  const prose = withoutSpans(task, literalSpans);
  candidates = candidates.concat(bareSourcePaths(task, literalSpans));
  candidates = candidates
    .map((entry, order) => [entry, order])
    .sort(([a, ai], [b, bi]) => (Number(!a[1].includes('/')) - Number(!b[1].includes('/'))) || (a[0] - b[0]) || (ai - bi))
    .map(([entry]) => entry);
  let target = candidates.length ? candidates[0][1] : literalTarget;
  if (target === null) {
    const resolved = resolveRequirementTarget(task);
    if (resolved) {
      if (!named.includes(resolved.symbol)) named.push(resolved.symbol);
      target = resolved.module_path;
    }
  }
  if (target === null) return null;
  for (const token of prose.replace(target, () => ' ').split(/[^A-Za-z0-9_]/u)) {
    if (looksLikeADeclaredName(token) && !named.includes(token)) named.push(token);
  }
  return { target, values, named };
}

function insertMembers(source, edit) {
  if (edit.named.length) {
    const viaLinks = insertMembersViaLinks(source, edit.named[0], edit.values);
    if (viaLinks) return viaLinks;
  }
  const bytes = encoder.encode(source);
  const [regions, literals] = scan(bytes);
  const anchor = anchorByMembers(regions, literals, edit.values) ?? anchorByName(source, bytes, regions, literals, edit.named);
  if (!anchor) return null;
  const members = directMembers(literals, anchor.open);
  const absent = edit.values.filter((value) => !members.some((member) => member.value === value));
  if (!absent.length) return [source, absent];
  const last = members[members.length - 1];
  if (!last) return null;
  let separator = ', ';
  if (members.length >= 2) {
    const gap = byteSlice(bytes, members[members.length - 2].end, last.start);
    if (gap !== null && gap !== '') separator = gap;
  }
  const addition = absent.map((value) => `${separator}"${value}"`).join('');
  const head = byteSlice(bytes, 0, last.end);
  const tail = byteSlice(bytes, last.end);
  if (head === null || tail === null) return null;
  return [head + addition + tail, absent];
}

/** Mirrors `fn insert_quoted_members_into_named_list`: `[updated, inserted]` or null. */
export function insertQuotedMembersIntoNamedList(source, declaration, requirement) {
  if (!requestsMemberInsertion(wordsJoined(requirement))) return null;
  const values = quotedSegmentSpans(requirement)
    .filter((segment) => !requirement.slice(segment.start).startsWith('`'))
    .map((segment) => segment.text)
    .filter(isMemberLiteral);
  if (!values.length) return null;
  return insertMembers(source, { target: '', values, named: [declaration] });
}

function quotedList(values) {
  return values.map((value) => `"${value}"`).join(', ');
}

function anchorByMembers(regions, literals, values) {
  let best = null;
  for (const region of regions) {
    const members = directMembers(literals, region.open);
    const held = values.filter((value) => members.some((member) => member.value === value)).length;
    if (held === 0) continue;
    best = better(best, held, region);
  }
  return best ? best[2] : null;
}

function anchorByName(source, bytes, regions, literals, named) {
  for (const name of named) {
    let best = null;
    for (const declaration of occurrences(source, name)) {
      const statement = statementEnd(bytes, declaration);
      for (const region of regions.filter((candidate) => candidate.open >= declaration && candidate.close <= statement)) {
        const held = directMembers(literals, region.open).length;
        if (held === 0) continue;
        best = better(best, held, region);
      }
    }
    if (best) return best[2];
  }
  return null;
}

function better(best, held, region) {
  const span = Math.max(0, region.close - region.open);
  if (best && (best[0] > held || (best[0] === held && best[1] <= span))) return best;
  return [held, span, region];
}

function directMembers(literals, open) {
  return literals.filter((literal) => literal.enclosing === open);
}

const isWordCharacter = (character) => character !== undefined && (isAsciiAlphanumeric(character) || character === '_');

/** Mirrors `fn occurrences`: UTF-8 byte offsets just past each whole-word `name`. */
function occurrences(source, name) {
  const found = [];
  if (name === '') return found;
  let cursor = 0;
  for (let at = source.indexOf(name, cursor); at >= 0; at = source.indexOf(name, cursor)) {
    const end = at + name.length;
    const before = Array.from(source.slice(0, at)).pop();
    const after = Array.from(source.slice(end, end + 2))[0];
    if (!isWordCharacter(before) && !isWordCharacter(after)) found.push(encoder.encode(source.slice(0, end)).length);
    cursor = end;
  }
  return found;
}

const BYTE = Object.freeze({
  slash: 0x2f, star: 0x2a, quote: 0x22, apostrophe: 0x27, backslash: 0x5c, newline: 0x0a, semicolon: 0x3b,
});
const OPENERS = new Map([[0x5b, 0x5d], [0x28, 0x29], [0x7b, 0x7d]]);
const CLOSERS = new Set([0x5d, 0x29, 0x7d]);

function statementEnd(bytes, from) {
  let depth = 0;
  for (let index = from; index < bytes.length; index += 1) {
    const byte = bytes[index];
    if (OPENERS.has(byte)) depth += 1;
    else if (CLOSERS.has(byte)) depth = Math.max(0, depth - 1);
    else if (byte === BYTE.semicolon && depth === 0) return index;
    else if (byte === BYTE.newline && depth === 0 && bytes[index + 1] === BYTE.newline) return index;
  }
  return bytes.length;
}

function indexOfBytes(bytes, needle, from) {
  outer: for (let index = from; index + needle.length <= bytes.length; index += 1) {
    for (let at = 0; at < needle.length; at += 1) if (bytes[index + at] !== needle[at]) continue outer;
    return index;
  }
  return -1;
}

function scan(bytes) {
  const regions = [];
  const literals = [];
  const openStack = [];
  let index = 0;
  while (index < bytes.length) {
    const byte = bytes[index];
    if (byte === BYTE.slash && bytes[index + 1] === BYTE.slash) {
      const newline = bytes.indexOf(BYTE.newline, index);
      index = newline < 0 ? bytes.length : newline;
    } else if (byte === BYTE.slash && bytes[index + 1] === BYTE.star) {
      const close = indexOfBytes(bytes, [BYTE.star, BYTE.slash], index + 2);
      index = close < 0 ? bytes.length : close + 2;
    } else if (byte === BYTE.apostrophe) {
      index = charLiteralEnd(bytes, index);
    } else if (byte === BYTE.quote) {
      const closing = stringLiteralEnd(bytes, index);
      if (closing === null) {
        index += 1;
        continue;
      }
      literals.push({
        start: index,
        end: closing + 1,
        value: byteSlice(bytes, index + 1, closing) ?? '',
        enclosing: openStack.length ? openStack[openStack.length - 1][1] : null,
      });
      index = closing + 1;
    } else if (OPENERS.has(byte)) {
      openStack.push([byte, index]);
      index += 1;
    } else if (CLOSERS.has(byte)) {
      const opened = openStack.pop();
      if (opened && OPENERS.get(opened[0]) === byte) regions.push({ open: opened[1], close: index });
      index += 1;
    } else index += 1;
  }
  const byOpen = new Map();
  for (const region of regions) if (!byOpen.has(region.open)) byOpen.set(region.open, region);
  const unique = [...byOpen.values()].sort((a, b) => (a.open - b.open) || (a.close - b.close));
  literals.sort((a, b) => a.start - b.start);
  return [unique, literals];
}

function stringLiteralEnd(bytes, open) {
  let cursor = open + 1;
  while (cursor < bytes.length) {
    if (bytes[cursor] === BYTE.backslash) cursor += 2;
    else if (bytes[cursor] === BYTE.quote) return cursor;
    else if (bytes[cursor] === BYTE.newline) return null;
    else cursor += 1;
  }
  return null;
}

function charLiteralEnd(bytes, open) {
  for (const width of [2, 3]) {
    if (bytes[open + width] === BYTE.apostrophe) {
      const escaped = bytes[open + 1] === BYTE.backslash;
      if ((width === 2 && !escaped) || (width === 3 && escaped)) return open + width + 1;
    }
  }
  return open + 1;
}

function withoutSpans(text, spans) {
  let kept = '';
  let cursor = 0;
  for (const [start, end] of spans) {
    if (start < cursor) continue;
    kept += `${text.slice(cursor, start)} `;
    cursor = end;
  }
  return kept + text.slice(cursor);
}

function isMemberLiteral(value) {
  return value !== '' && encoder.encode(value).length <= MAX_MEMBER_LENGTH && !value.includes('"') && !value.includes('\\')
    && Array.from(value).every((character) => !isWhitespace(character) || character === ' ');
}

function looksLikeADeclaredName(token) {
  return validIdentifier(token) && token.length > 1
    && (token.includes('_') || !/[a-z]/.test(token) || (/[A-Z]/.test(token) && !/^[A-Z]/.test(token)));
}

function bareSourcePaths(task, literalSpans) {
  const paths = [];
  let cursor = 0;
  for (const [start, end] of [...literalSpans, [task.length, task.length]]) {
    if (cursor <= start) {
      for (const [offset, path] of prosePathTokens(task.slice(cursor, start))) paths.push([cursor + offset, path]);
    }
    cursor = Math.max(end, cursor);
  }
  return paths;
}

function prosePathTokens(prose) {
  const out = [];
  let start = 0;
  const chars = Array.from(prose);
  let chunk = '';
  const flush = () => {
    const isEdge = (character) => !isAsciiAlphanumeric(character) && !'_-./'.includes(character);
    const trimmed = trimMatches(chunk, isEdge);
    const token = trimEndMatches(trimmed, (character) => character === '.');
    if (isWorkspacePath(token)) {
      let lead = 0;
      while (lead < chunk.length && isEdge(chunk[lead])) lead += chunk.codePointAt(lead) > 0xffff ? 2 : 1;
      out.push([start + lead, token]);
    }
    start += chunk.length;
    chunk = '';
  };
  for (const character of chars) {
    chunk += character;
    if (isWhitespace(character) || character === ',') flush();
  }
  if (chunk !== '') flush();
  return out;
}

function isWorkspacePath(token) {
  const dot = token.lastIndexOf('.');
  if (dot < 0) return false;
  const stem = token.slice(0, dot);
  const extension = token.slice(dot + 1);
  return stem !== '' && !token.startsWith('/') && !token.split('/').some((component) => component === '..')
    && extension.length >= 1 && extension.length <= 8
    && /^[A-Za-z0-9]*$/.test(extension) && !Array.from(extension).every(isAsciiDigit)
    && /^[A-Za-z0-9_./-]*$/.test(token);
}

function validIdentifier(identifier) {
  return /^[A-Za-z_][A-Za-z0-9_]*$/.test(identifier);
}

function readArguments(path) {
  return jsonText({ path, filePath: path, file_path: path });
}
