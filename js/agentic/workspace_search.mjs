// Workspace content search (PR #1188 T90, gap G12): "Find all usages of add",
// "Where is add used in this project?", "Grep for add" ask for the places a
// name or a literal appears inside the workspace's files. The seeded
// `workspace_content_search_form` slot forms (every registered language) read
// the searched pattern out of the request; the arm greps it -- the client's
// grep tool when advertised, else `grep -rn` through the shell -- and answers
// with the file:line hits, or a seeded not-found naming the pattern and the
// scope. It runs ahead of the file-name locate arm and of web search.
// rust/src/agentic_coding/workspace_search.rs.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { clientWorkingDirectory, plainText } from './content.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { parseRoot } from './crate/seed_parser.mjs';
import { detect } from './crate/language.mjs';
import { isWhitespace, splitWhitespace, trimEndMatches, trimMatches } from './crate/rust_str.mjs';
import { renderResponse } from './crate/seed.mjs';
import { cached, childValue, childrenNamed, readText } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { afterSlot, beforeSlot, containsCjk, mentionsRole, mentionsRoleRaw, roleWordForms, wordsForRole } from './crate/seed_meanings.mjs';
import { finalAnswer, jsonText, planOne } from './plan.mjs';
import { looksLikeFilePath, safeRelativePath } from './write_request.mjs';
import { resultCapability } from './progress.mjs';
import { normalizedPayload, render } from './tool_result.mjs';

const ROLE_SEARCH_FORM = 'workspace_content_search_form';
const ROLE_SEARCH_NOISE = 'workspace_content_search_noise';
const ROLE_WEB_MEDIUM = 'web_medium';
const EDIT_ROLES = ['file_edit_action_cue', 'coding_text_remove_action', 'coding_identifier_rename_action'];
const CURRENT_SCOPE = '.';
const SHELL_INTENTS_FILE = 'data/seed/shell-intents.lino';
const NOISE_SKIP_LIMIT = 3;
const PATTERN_CHAR_LIMIT = 128;
const QUOTE_PAIRS = [['"', '"'], ["'", "'"], ['`', '`'], ['«', '»'], ['“', '”'], ['‘', '’']];
const WRAPPERS = new Set(['.', ',', ';', ':', '!', '?', '¿', '¡', '(', ')', '[', ']', '{', '}', '"', "'", '`',
  '«', '»', '“', '”', '‘', '’', '。', '，', '？', '！', '：', '；']);
const IDENTIFIER = /^[\p{Alphabetic}_$][\p{Alphabetic}\p{N}_$]*$/u;
const ALPHANUMERIC = /[\p{Alphabetic}\p{N}]/u;

const isSpace = (character) => isWhitespace(character);
const isCjk = (character) => containsCjk(character);
const isWrapper = (character) => WRAPPERS.has(character);

/** One char per char: a lowercase that keeps every offset (`char::to_lowercase` of one char). */
export function lowerChars(chars) {
  return chars.map((character) => {
    const lower = character.toLowerCase();
    return Array.from(lower).length === 1 ? lower : character;
  });
}

/** Does `needle` (chars) start at `at` in `haystack` (chars), on word boundaries? */
function startsAt(haystack, needle, at) {
  if (at < 0 || at + needle.length > haystack.length) return false;
  for (let index = 0; index < needle.length; index += 1) {
    if (haystack[at + index] !== needle[index]) return false;
  }
  const first = needle[0];
  const last = needle[needle.length - 1];
  const leftOpen = isCjk(first) || at === 0 || !ALPHANUMERIC.test(haystack[at - 1]);
  const end = at + needle.length;
  const rightOpen = isCjk(last) || end === haystack.length || !ALPHANUMERIC.test(haystack[end]);
  return leftOpen && rightOpen;
}

/** The quote pair a char opens, or null. */
function opening(character) {
  return QUOTE_PAIRS.find(([open]) => open === character) ?? null;
}

/** The quote pair a char closes, or null. */
function closing(character) {
  return QUOTE_PAIRS.find(([, close]) => close === character) ?? null;
}

/**
 * The token that starts at `at` (after spaces): `{text, end, quoted}`, or null.
 * A token stops at whitespace and at CJK text; a quoted literal runs to its closer.
 */
function tokenAfter(chars, at) {
  let start = at;
  while (start < chars.length && isSpace(chars[start])) start += 1;
  if (start >= chars.length) return null;
  const pair = opening(chars[start]);
  if (pair !== null) {
    const close = chars.indexOf(pair[1], start + 1);
    if (close > start + 1) return { text: chars.slice(start + 1, close).join(''), end: close + 1, quoted: true };
  }
  let end = start;
  while (end < chars.length && !isSpace(chars[end]) && !isCjk(chars[end])) end += 1;
  if (end === start) return null;
  return { text: chars.slice(start, end).join(''), end, quoted: false };
}

/** The token that ends at `at` (before spaces): `{text, start, quoted}`, or null. */
function tokenBefore(chars, at) {
  let end = at;
  while (end > 0 && isSpace(chars[end - 1])) end -= 1;
  if (end <= 0) return null;
  const pair = closing(chars[end - 1]);
  if (pair !== null) {
    const open = chars.lastIndexOf(pair[0], end - 2);
    if (open >= 0 && open < end - 2) return { text: chars.slice(open + 1, end - 1).join(''), start: open, quoted: true };
  }
  let start = end;
  while (start > 0 && !isSpace(chars[start - 1]) && !isCjk(chars[start - 1])) start -= 1;
  if (start === end) return null;
  return { text: chars.slice(start, end).join(''), start, quoted: false };
}

/** A token read as a pattern: wrappers peeled, noise and wordless tokens refused. */
function patternOf(token) {
  if (token === null) return null;
  const text = token.quoted ? token.text : trimMatches(token.text, isWrapper);
  if (text === '' || !ALPHANUMERIC.test(text) || Array.from(text).length > PATTERN_CHAR_LIMIT) return null;
  return text;
}

/** Is a token a seeded noise word standing in the slot? */
function isNoise(token) {
  if (token === null || token.quoted) return false;
  const word = normalizePrompt(trimMatches(token.text, isWrapper));
  return wordsForRole(ROLE_SEARCH_NOISE).includes(word);
}

/** The pattern after `at`, skipping noise words. */
function patternAfter(chars, at) {
  let cursor = at;
  for (let skipped = 0; skipped <= NOISE_SKIP_LIMIT; skipped += 1) {
    const token = tokenAfter(chars, cursor);
    if (token === null) return null;
    if (!isNoise(token)) return { pattern: patternOf(token), end: token.end };
    cursor = token.end;
  }
  return null;
}

/** The pattern before `at`, skipping noise words. */
function patternBefore(chars, at) {
  let cursor = at;
  for (let skipped = 0; skipped <= NOISE_SKIP_LIMIT; skipped += 1) {
    const token = tokenBefore(chars, cursor);
    if (token === null) return null;
    if (!isNoise(token)) return { pattern: patternOf(token), start: token.start };
    cursor = token.start;
  }
  return null;
}

/** Every offset `needle` starts at in `haystack`, on word boundaries. */
function offsetsOf(haystack, needle) {
  const offsets = [];
  for (let at = 0; at + needle.length <= haystack.length; at += 1) {
    if (startsAt(haystack, needle, at)) offsets.push(at);
  }
  return offsets;
}

/** The pattern one slot form reads out of the request: `{pattern, span}` or null. */
function formMatch(chars, lower, form) {
  const before = Array.from(beforeSlot(form).trim().toLowerCase());
  const after = Array.from(afterSlot(form).trim().toLowerCase());
  if (before.length) {
    for (const at of offsetsOf(lower, before)) {
      const found = patternAfter(chars, at + before.length);
      if (found === null || found.pattern === null) continue;
      if (after.length) {
        let next = found.end;
        while (next < lower.length && isSpace(lower[next])) next += 1;
        if (!startsAt(lower, after, next)) continue;
        return { pattern: found.pattern, span: [at, next + after.length] };
      }
      return { pattern: found.pattern, span: [at, found.end] };
    }
    return null;
  }
  if (!after.length) return null;
  for (const at of offsetsOf(lower, after)) {
    const found = patternBefore(chars, at);
    if (found !== null && found.pattern !== null) return { pattern: found.pattern, span: [found.start, at + after.length] };
  }
  return null;
}

/** A path-shaped token outside the matched form: the searched scope. */
function scopeOf(chars, span) {
  const outside = [...chars.slice(0, span[0]), ' ', ...chars.slice(span[1])].join('');
  for (const raw of splitWhitespace(outside)) {
    const word = trimMatches(raw, isWrapper);
    if (word.includes(':') || !(word.includes('/') || (looksLikeFilePath(word) && safeRelativePath(word)))) continue;
    const scope = trimEndMatches(word, (character) => character === '/');
    if (scope !== '') return scope;
  }
  return CURRENT_SCOPE;
}

/**
 * Mirrors `fn content_search_for` in rust/src/agentic_coding/workspace_search.rs:
 * `{pattern, scope, identifier}` for a workspace content-search request, or null.
 * @param {string} task
 */
export function contentSearchFor(task) {
  if (mentionsRoleRaw(ROLE_WEB_MEDIUM, ` ${normalizePrompt(task)} `)) return null;
  const chars = Array.from(task);
  const lower = lowerChars(chars);
  let best = null;
  for (const form of roleWordForms(ROLE_SEARCH_FORM)) {
    const found = formMatch(chars, lower, form);
    if (found === null) continue;
    const weight = Array.from(form.text).length;
    if (best === null || weight > best.weight) best = { ...found, weight };
  }
  if (best === null) return null;
  // A request that changes what it finds (rename, replace, remove the uses of
  // a name) is an edit, not a search: the pattern itself may be such a word.
  const rest = normalizePrompt(task.split(best.pattern).join(' '));
  if (EDIT_ROLES.some((role) => mentionsRole(role, rest))) return null;
  return { pattern: best.pattern, scope: scopeOf(chars, best.span), identifier: IDENTIFIER.test(best.pattern) };
}

/** A regular expression matching `literal` exactly. */
function escapedRegex(literal) {
  return literal.replace(/[\\^$.*+?()[\]{}|/]/gu, (character) => `\\${character}`);
}

/** Mirrors `fn shell_quote` in rust/src/agentic_coding/git_commit.rs. */
function shellQuote(text) {
  return `'${text.split("'").join("'\\''")}'`;
}

/** The grep tool arguments for a search. */
function grepArguments(search) {
  const pattern = search.identifier ? `\\b${escapedRegex(search.pattern)}\\b` : escapedRegex(search.pattern);
  return jsonText({ path: search.scope, pattern });
}

/** The seeded `content_search` group of data/seed/shell-intents.lino. */
function contentSearchSeed() {
  return cached('workspace-content-search', () => {
    const root = parseRoot(readText(SHELL_INTENTS_FILE)).children[0];
    return childrenNamed(root, 'content_search')[0] ?? null;
  });
}

/** The seeded shell command for a search (whole-word for an identifier, fixed text for a literal). */
function shellCommand(search) {
  const seed = contentSearchSeed();
  const mode = childValue(seed, search.identifier ? 'identifier_mode' : 'literal_mode');
  return [['{mode}', mode], ['{pattern}', shellQuote(search.pattern)], ['{scope}', shellQuote(search.scope)]]
    .reduce((command, [slot, value]) => command.split(slot).join(value), childValue(seed, 'command'));
}

/** The search results of the current turn: `{capability, raw}` of each grep or run result. */
function turnResults(messages) {
  let currentTurn = 0;
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (messages[index].role.toLowerCase() === 'user') {
      currentTurn = index + 1;
      break;
    }
  }
  const results = [];
  for (let index = currentTurn; index < messages.length; index += 1) {
    if (messages[index].role.toLowerCase() !== 'tool') continue;
    const capability = resultCapability(messages, index);
    if (capability === Capability.Grep || capability === Capability.Run) {
      results.push({ capability, raw: plainText(messages[index].content) });
    }
  }
  return results;
}

/** A hit path relative to the working directory. */
function relativePath(path, root) {
  let relative = path;
  if (root !== null && relative.startsWith(`${root}/`)) relative = relative.slice(root.length + 1);
  while (relative.startsWith('./')) relative = relative.slice(2);
  return relative;
}

/** `digits:text` after a hit's line label: `[digits, text]`, or null. */
function numberedText(rest) {
  const colon = rest.indexOf(':');
  const number = colon < 0 ? '' : rest.slice(0, colon);
  if (number === '' || !/^[0-9]+$/u.test(number)) return null;
  return [number, rest.slice(colon + 1).trim()];
}

/** `path:digits:text` of one `grep -rn` line: `[path, digits, text]`, or null. */
function shellHit(line) {
  for (let colon = line.indexOf(':', 1); colon > 0; colon = line.indexOf(':', colon + 1)) {
    const numbered = numberedText(line.slice(colon + 1));
    if (numbered !== null) return [line.slice(0, colon), ...numbered];
  }
  return null;
}

/** The `[path, line, text]` hits of a grep tool result (grouped by file) or `grep -rn` output. */
function hitsOf(payload, capability, root) {
  const hits = [];
  let file = null;
  for (const line of payload.split('\n')) {
    if (capability === Capability.Grep) {
      const quoted = line.trimStart();
      if (quoted !== line) {
        const space = quoted.indexOf(' ');
        const numbered = space < 0 ? null : numberedText(quoted.slice(space + 1));
        if (numbered !== null && file !== null) hits.push([file, ...numbered]);
      } else if (line.endsWith(':')) {
        file = relativePath(line.slice(0, -1), root);
      }
      continue;
    }
    const hit = shellHit(line);
    if (hit !== null) hits.push([relativePath(hit[0], root), hit[1], hit[2]]);
  }
  return hits;
}

/** The seeded answer for the observed hits. */
function answerFor(search, hits, language) {
  const values = [['pattern', search.pattern], ['scope', search.scope]];
  if (!hits.length) return renderResponse('workspace_search_none', language, values)
    ?? renderResponse('workspace_search_none', 'en', values);
  const listed = hits.map(([path, line, text]) => `- \`${path}:${line}\`: ${text}`).join('\n');
  const all = [...values, ['count', String(hits.length)], ['hits', listed]];
  return renderResponse('workspace_search_hits', language, all) ?? renderResponse('workspace_search_hits', 'en', all);
}

/**
 * Mirrors `fn plan_workspace_search_step` in rust/src/agentic_coding/workspace_search.rs.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planWorkspaceSearchStep(task, messages, toolNames) {
  const search = contentSearchFor(task);
  if (search === null) return null;
  const results = turnResults(messages);
  if (results.length) {
    const { capability, raw } = results[results.length - 1];
    const language = detect(task);
    if (raw.trim() === agenticMessage('file_read_no_files_found', {})) return finalAnswer(answerFor(search, [], language));
    const payload = normalizedPayload(raw);
    if (payload === null || payload === undefined) return finalAnswer(render('grep', raw, task));
    return finalAnswer(answerFor(search, hitsOf(payload, capability, clientWorkingDirectory(messages)), language));
  }
  const grep = toolFor(toolNames, Capability.Grep);
  if (grep !== null) return planOne(grep, grepArguments(search));
  const shell = toolFor(toolNames, Capability.Run);
  return shell === null ? null : planOne(shell, jsonText({ command: shellCommand(search) }));
}
