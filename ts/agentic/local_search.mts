// Meanings-driven, stateful local path discovery (issue #840):
// rust/src/agentic_coding/local_search.rs.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { plainText, rustLines, userRequestText } from './content.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { detect } from './crate/language.mjs';
import { byteOrder, eqIgnoreAsciiCase, isWhitespace, replaceAllLiteral, splitWhitespace, trim, trimMatches,
  trimStart } from './crate/rust_str.mjs';
import { localizedResponse } from './crate/seed.mjs';
import { afterSlot, beforeSlot, mentionsRole, roleWordForms, wordsForRole } from './crate/seed_meanings.mjs';
import { finalAnswer, jsonText, planOne } from './plan.mjs';
import { resultCapability } from './progress.mjs';
import { normalizedPayload, render } from './tool_result.mjs';

const DESKTOP_ROOT = '${FORMAL_AI_DESKTOP_DIR:-$HOME/Desktop}';
const HOME_ROOT = '${FORMAL_AI_HOME_DIR:-$HOME}';
const CURRENT_ROOT = '.';
const PATH_SLOT = '{path}';
const REQUESTED_SLOT = '{requested}';
const ACTUAL_SLOT = '{actual}';
const SCOPE_SLOT = '{scope}';
const STAGE_SLOT = '{stage}';
const ITEMS_SLOT = '{items}';

const ROLE_LOCAL_PATH_SCOPE_DESKTOP = 'local_path_scope_desktop';
const ROLE_LOCAL_PATH_SCOPE_HOME = 'local_path_scope_home';
const ROLE_LOCAL_PATH_SCOPE_CURRENT = 'local_path_scope_current';
const ROLE_LOCAL_PATH_CONTENTS_REQUEST = 'local_path_contents_request';
const ROLE_LOCAL_PATH_TYPE_REQUEST = 'local_path_type_request';
const ROLE_LOCAL_PATH_LIST_ACTION = 'local_path_list_action';
const ROLE_LOCAL_PATH_SEARCH_ACTION = 'local_path_search_action';
const ROLE_LOCAL_PATH_ROUTE_QUESTION = 'local_path_route_question';
const ROLE_LOCAL_PATH_DIRECTORY_KIND = 'local_path_directory_kind';
const ROLE_LOCAL_PATH_FILE_KIND = 'local_path_file_kind';
const ROLE_LOCAL_PATH_QUERY_NOISE = 'local_path_query_noise';

const ALPHANUMERIC = /^[\p{Alphabetic}\p{N}]$/u;
const isAlnum = (character) => character !== undefined && ALPHANUMERIC.test(character);
const lastChar = (text) => Array.from(text).pop();
const firstChar = (text) => Array.from(text)[0];
const charCount = (text) => Array.from(text).length;

/** Split on non-alphanumeric characters, dropping empty tokens. */
function alnumTokens(text) {
  const tokens = [];
  let current = '';
  for (const character of text) {
    if (ALPHANUMERIC.test(character)) current += character;
    else {
      if (current) tokens.push(current);
      current = '';
    }
  }
  if (current) tokens.push(current);
  return tokens;
}

/**
 * Mirrors `fn plan_local_search_step` in rust/src/agentic_coding/local_search.rs.
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planLocalSearchStep(messages, toolNames) {
  const user = [...messages].reverse().find((message) => message.role.toLowerCase() === 'user');
  if (!user) return null;
  const task = userRequestText(user.content);
  const request = requestFor(task);
  if (request === null) return null;
  const language = detect(task);
  if (mentionsRole(ROLE_LOCAL_PATH_ROUTE_QUESTION, normalizePrompt(task))) {
    return finalAnswer(localized('local_search_route_local', language));
  }
  const results = runResults(messages, task);
  if (typeof results === 'string') return finalAnswer(results);
  switch (request.mode) {
    case 'list_scope':
      if (results.length) {
        return finalAnswer(renderListing('local_search_scope_listing', language, request.root, results[0]));
      }
      return run(toolNames, scopeListingCommand(request));
    case 'list_contents':
      return planContents(toolNames, request, results, language);
    default:
      return planFind(toolNames, request, results, language);
  }
}

/**
 * Mirrors `fn narration_for` in rust/src/agentic_coding/local_search.rs:
 * `{subject, intent}` or null.
 * @param {string} prompt
 */
export function narrationFor(prompt) {
  const request = requestFor(prompt);
  if (request === null) return null;
  return {
    subject: request.subject,
    intent: `agentic_action_${request.mode === 'list_scope' ? 'list' : 'find'}_${request.scope}`,
  };
}

/**
 * Mirrors `fn request_for` in rust/src/agentic_coding/local_search.rs:
 * `{scope, root, mode, kind, subject, tokens}` or null. `mode` is 'find' |
 * 'list_scope' | 'list_contents' | 'type'; `kind` is 'directory' | 'file' | null.
 * @param {string} prompt
 */
export function requestFor(prompt) {
  const normalized = normalizePrompt(prompt);
  const fileLiteral = explicitFileLiteral(prompt);
  const targetLiteral = quotedLiteral(prompt) ?? fileLiteral;
  const scopes = [
    ['desktop', DESKTOP_ROOT, ROLE_LOCAL_PATH_SCOPE_DESKTOP],
    ['home', HOME_ROOT, ROLE_LOCAL_PATH_SCOPE_HOME],
    ['current', CURRENT_ROOT, ROLE_LOCAL_PATH_SCOPE_CURRENT],
  ];
  const found = scopes.find(([, , role]) => mentionsRole(role, normalized));
  if (!found) return null;
  const [scope, root, scopeRole] = found;

  const contents = roleMatches(ROLE_LOCAL_PATH_CONTENTS_REQUEST, normalized);
  const typeRequest = roleMatches(ROLE_LOCAL_PATH_TYPE_REQUEST, normalized);
  const list = roleMatches(ROLE_LOCAL_PATH_LIST_ACTION, normalized);
  const find = roleMatches(ROLE_LOCAL_PATH_SEARCH_ACTION, normalized);
  const routeQuestion = roleMatches(ROLE_LOCAL_PATH_ROUTE_QUESTION, normalized);
  if (!contents && !typeRequest && !list && !find && !routeQuestion) return null;

  let mode;
  if (contents) mode = 'list_contents';
  else if (typeRequest) mode = 'type';
  else if (list && !find) mode = 'list_scope';
  else mode = 'find';
  if (scope === 'current' && mode === 'list_scope') return null;

  const kindContext = stripRole(normalized, scopeRole);
  const directory = longestRoleSurface(ROLE_LOCAL_PATH_DIRECTORY_KIND, kindContext);
  const file = longestRoleSurface(ROLE_LOCAL_PATH_FILE_KIND, kindContext);
  let kind;
  if (mode === 'type') kind = null;
  else if (mode === 'list_contents') kind = 'directory';
  else if (directory !== null && file !== null) kind = directory >= file ? 'directory' : 'file';
  else if (directory !== null) kind = 'directory';
  else if (file !== null) kind = 'file';
  else kind = fileLiteral !== null ? 'file' : null;

  let stripped = normalized;
  for (const role of [
    ROLE_LOCAL_PATH_ROUTE_QUESTION, ROLE_LOCAL_PATH_CONTENTS_REQUEST, ROLE_LOCAL_PATH_TYPE_REQUEST,
    ROLE_LOCAL_PATH_SEARCH_ACTION, ROLE_LOCAL_PATH_LIST_ACTION, scopeRole, ROLE_LOCAL_PATH_DIRECTORY_KIND,
    ROLE_LOCAL_PATH_FILE_KIND, ROLE_LOCAL_PATH_QUERY_NOISE,
  ]) {
    stripped = stripRole(stripped, role);
  }
  let tokens = alnumTokens(stripped);
  let subject;
  if (targetLiteral !== null) {
    tokens = alnumTokens(targetLiteral);
    subject = targetLiteral;
  } else {
    subject = tokens.join('-');
  }
  if ((mode === 'find' || mode === 'list_contents' || mode === 'type') && !tokens.length && !routeQuestion) {
    return null;
  }
  return { scope, root, mode, kind, subject, tokens };
}

/** Mirrors `fn role_matches`. */
function roleMatches(role, text) {
  return mentionsRole(role, text) || roleWordForms(role).some((form) => {
    const before = normalizePrompt(beforeSlot(form));
    const after = normalizePrompt(afterSlot(form));
    if (before && after) {
      const position = surfacePosition(text, before);
      return position !== null && surfacePosition(text.slice(position + before.length), after) !== null;
    }
    if (before) return surfacePresent(text, before);
    if (after) return surfacePresent(text, after);
    return false;
  });
}

/** Mirrors `fn surface_position` (non-overlapping `match_indices`). */
function surfacePosition(text, surface) {
  for (let position = text.indexOf(surface); position >= 0 && surface;
    position = text.indexOf(surface, position + surface.length)) {
    const end = position + surface.length;
    const left = position === 0 || !isAlnum(lastChar(text.slice(0, position)));
    const right = end === text.length || !isAlnum(firstChar(text.slice(end)));
    if (left && right) return position;
  }
  if (containsCjk(surface)) {
    const at = text.indexOf(surface);
    return at < 0 ? null : at;
  }
  return null;
}

const FILE_LITERAL_WRAPPERS = new Set(['"', "'", '`', '(', ')', '[', ']', '{', '}', ',', ';', ':']);

/** `Path::file_name` of a slash-separated path, or null. */
function pathFileName(path) {
  const components = path.split('/').filter((part, index) => part !== '' && (part !== '.' || index === 0));
  const last = components[components.length - 1];
  if (last === undefined || last === '..' || last === '.') return null;
  return last;
}

/** `Path::extension`: the text after the file name's last dot (not a leading one), or null. */
function pathExtension(path) {
  const name = pathFileName(path);
  if (name === null) return null;
  const dot = name.lastIndexOf('.');
  if (dot <= 0) return null;
  return name.slice(dot + 1);
}

/** Mirrors `fn explicit_file_literal`. */
function explicitFileLiteral(prompt) {
  for (const raw of splitWhitespace(prompt)) {
    const word = trimMatches(raw, (character) => FILE_LITERAL_WRAPPERS.has(character));
    const extension = pathExtension(word);
    if (extension === null) continue;
    if (extension !== '' && Array.from(extension).every((character) => ALPHANUMERIC.test(character))
      && charCount(extension) <= 10) {
      return (pathFileName(word) ?? word).toLowerCase();
    }
  }
  return null;
}

/** Mirrors `fn quoted_literal`. */
function quotedLiteral(prompt) {
  for (const quote of ["'", '"', '`']) {
    const positions = [];
    for (let index = prompt.indexOf(quote); index >= 0; index = prompt.indexOf(quote, index + 1)) {
      positions.push(index);
    }
    for (let offset = 0; offset < positions.length; offset += 1) {
      const opening = positions[offset];
      const leftBoundary = opening === 0 || !isAlnum(lastChar(prompt.slice(0, opening)));
      if (!leftBoundary) continue;
      for (const closing of positions.slice(offset + 1)) {
        const after = closing + quote.length;
        const rightBoundary = after === prompt.length || !isAlnum(firstChar(prompt.slice(after)));
        const transportWrapper = trim(prompt.slice(0, opening)) === '' && trim(prompt.slice(after)) === '';
        const literal = trim(prompt.slice(opening + quote.length, closing));
        if (rightBoundary && !transportWrapper && literal !== '' && charCount(literal) <= 128
          && Array.from(literal).some((character) => ALPHANUMERIC.test(character))) {
          return literal.toLowerCase();
        }
      }
    }
  }
  return null;
}

/** Mirrors `fn strip_role` (returns the stripped text). */
function stripRole(text, role) {
  const literals = roleWordForms(role)
    .flatMap((form) => [beforeSlot(form), afterSlot(form)])
    .map(normalizePrompt)
    .filter((literal) => literal !== '');
  literals.sort((left, right) => charCount(right) - charCount(left));
  let out = text;
  literals.forEach((literal, index) => {
    if (index > 0 && literals[index - 1] === literal) return;
    out = removeSurface(out, literal);
  });
  return out;
}

/** Mirrors `fn longest_role_surface`: the longest present surface's char count, or null. */
function longestRoleSurface(role, text) {
  const counts = wordsForRole(role).filter((surface) => surfacePresent(text, surface)).map(charCount);
  return counts.length ? Math.max(...counts) : null;
}

/** Mirrors `fn remove_surface` (returns the edited text). */
function removeSurface(text, surface) {
  if (containsCjk(surface)) return replaceAllLiteral(text, surface, ' ');
  let out = text;
  let searchFrom = 0;
  for (;;) {
    const start = out.indexOf(surface, searchFrom);
    if (start < 0 || !surface) return out;
    const end = start + surface.length;
    const left = start === 0 || isWhitespace(lastChar(out.slice(0, start)));
    const right = end === out.length || isWhitespace(firstChar(out.slice(end)));
    if (left && right) {
      out = `${out.slice(0, start)} ${out.slice(end)}`;
      searchFrom = start + 1;
    } else {
      searchFrom = end;
    }
  }
}

/** Mirrors `fn surface_present` in rust/src/agentic_coding/local_search.rs. */
function surfacePresent(text, surface) {
  if (containsCjk(surface)) return text.includes(surface);
  return text === surface || text.startsWith(`${surface} `) || text.endsWith(` ${surface}`)
    || text.includes(` ${surface} `);
}

/** Mirrors `fn contains_cjk` in rust/src/agentic_coding/local_search.rs (U+3400..=U+9FFF). */
function containsCjk(text) {
  for (const character of text) {
    const code = character.codePointAt(0);
    if (code >= 0x3400 && code <= 0x9fff) return true;
  }
  return false;
}

/** Mirrors `fn plan_find`. */
function planFind(toolNames, request, results, language) {
  if (!results.length) return run(toolNames, exactCommand(request));
  if (trim(results[0]) !== '') {
    return answerForCandidates(toolNames, request, results[0], null, language, results[1]);
  }
  if (results.length === 1) return run(toolNames, substringCommand(request));
  if (trim(results[1]) !== '') {
    return answerForCandidates(toolNames, request, results[1], 'widened', language, results[2]);
  }
  if (results.length === 2) return run(toolNames, inventoryCommand(request, false));
  return answerForCandidates(toolNames, request, results[2], 'inventory', language, results[3]);
}

/** Mirrors `fn answer_for_candidates`. */
function answerForCandidates(toolNames, request, output, widened, language, metadata) {
  const candidate = bestCandidate(output, request);
  if (candidate === null) return finalAnswer(renderNotFound(request, language));
  if (request.mode === 'type') {
    if (metadata !== undefined) return finalAnswer(renderType(request, candidate, metadata, language));
    return run(toolNames, metadataCommand(candidate));
  }
  if (widened === 'inventory' && request.kind !== null) {
    if (metadata === undefined) return run(toolNames, metadataCommand(candidate));
    const kindMatches = request.kind === 'directory'
      ? trimStart(metadata).startsWith('d')
      : trimStart(metadata).startsWith('-');
    if (!kindMatches) {
      const intent = request.kind === 'directory'
        ? 'local_search_mismatch_requested_directory'
        : 'local_search_mismatch_requested_file';
      return finalAnswer(fill(localized(intent, language), [
        [PATH_SLOT, candidate], [REQUESTED_SLOT, request.subject], [ACTUAL_SLOT, basename(candidate)],
        [SCOPE_SLOT, request.root],
      ]));
    }
  }
  const actual = basename(candidate);
  const exact = eqIgnoreAsciiCase(actual, request.subject);
  let intent = 'local_search_found_closest';
  if (exact) {
    if (request.kind === 'directory') intent = 'local_search_found_directory';
    else if (request.kind === 'file') intent = 'local_search_found_file';
    else intent = 'local_search_found_path';
  }
  return finalAnswer(fill(localized(intent, language), [
    [PATH_SLOT, candidate], [REQUESTED_SLOT, request.subject], [ACTUAL_SLOT, actual],
    [SCOPE_SLOT, request.root], [STAGE_SLOT, widened ?? 'exact'],
  ]));
}

/** Chained `str::replace` calls. */
function fill(text, pairs) {
  return pairs.reduce((out, [slot, value]) => replaceAllLiteral(out, slot, value), text);
}

/** Mirrors `fn plan_contents`. */
function planContents(toolNames, request, results, language) {
  if (!results.length) return run(toolNames, exactCommand(request));
  if (results.length === 1 && trim(results[0]) === '') return run(toolNames, substringCommand(request));
  if (trim(results[0]) === '' && trim(results[1]) === '' && results.length === 2) {
    return run(toolNames, inventoryCommand(request, true));
  }
  let candidateResult;
  let contentsIndex;
  if (trim(results[0]) !== '') [candidateResult, contentsIndex] = [results[0], 1];
  else if (trim(results[1]) !== '') [candidateResult, contentsIndex] = [results[1], 2];
  else [candidateResult, contentsIndex] = [results[2], 3];
  const candidate = bestCandidate(candidateResult, request);
  if (candidate === null) return finalAnswer(renderNotFound(request, language));
  if (contentsIndex < results.length) {
    return finalAnswer(renderListing('local_search_contents_listing', language, candidate, results[contentsIndex]));
  }
  return run(toolNames, contentsCommand(candidate));
}

/** Mirrors `fn run`. */
function run(toolNames, command) {
  const tool = toolFor(toolNames, Capability.Run) ?? null;
  return tool === null ? null : planOne(tool, jsonText({ command }));
}

/** Mirrors `fn exact_command`. */
function exactCommand(request) {
  return findCommand(request, request.subject, false);
}

/** Mirrors `fn substring_command`. */
function substringCommand(request) {
  const token = request.tokens.find((candidate) => charCount(candidate) >= 4) ?? request.tokens[0] ?? '';
  return findCommand(request, token, true);
}

/** Mirrors `fn find_command`. */
function findCommand(request, name, substring) {
  const pattern = substring ? `*${name}*` : name;
  return `find "${request.root}"${kindPredicate(request.kind)} -iname ${shellQuote(pattern)} -print`;
}

/** Mirrors `fn inventory_command`. */
function inventoryCommand(request, honorKind) {
  const predicate = honorKind ? kindPredicate(request.kind) : '';
  return `find "${request.root}" -mindepth 1 -maxdepth 3${predicate} -print`;
}

/** Mirrors `fn scope_listing_command`. */
function scopeListingCommand(request) {
  return `find "${request.root}" -mindepth 1 -maxdepth 1${kindPredicate(request.kind)} -print`;
}

/** Mirrors `fn contents_command`. */
function contentsCommand(path) {
  return `find ${shellQuote(path)} -mindepth 1 -maxdepth 1 -print`;
}

/** Mirrors `fn metadata_command`. */
function metadataCommand(path) {
  return `ls -ld -- ${shellQuote(path)}`;
}

/** Mirrors `const fn kind_predicate`. */
function kindPredicate(kind) {
  if (kind === 'directory') return ' -type d';
  if (kind === 'file') return ' -type f';
  return '';
}

/** Mirrors `fn run_results`: the payloads, or the rendered failure (a string). */
function runResults(messages, task) {
  let currentTurn = 0;
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (messages[index].role.toLowerCase() === 'user') {
      currentTurn = index + 1;
      break;
    }
  }
  const results = [];
  for (let index = currentTurn; index < messages.length; index += 1) {
    const message = messages[index];
    if (message.role.toLowerCase() !== 'tool') continue;
    if (resultCapability(messages, index) !== Capability.Run) continue;
    const raw = plainText(message.content);
    const payload = normalizedPayload(raw);
    if (payload === null || payload === undefined) return render('shell', raw, task);
    results.push(payload);
  }
  return results;
}

/** Mirrors `fn best_candidate`: highest similarity, ties to the byte-smallest path. */
function bestCandidate(output, request) {
  let best = null;
  for (const path of rustLines(output).map(trim).filter((line) => line !== '')) {
    const score = similarity(path, request.tokens);
    if (score <= 0) continue;
    if (best === null || score > best[0] || (score === best[0] && byteOrder(path, best[1]) <= 0)) {
      best = [score, path];
    }
  }
  return best === null ? null : best[1];
}

/** Mirrors `fn similarity`. */
function similarity(path, requested) {
  const actual = new Set(alnumTokens(basename(path)).map((token) => token.toLowerCase()));
  const wanted = new Set(requested.map((token) => token.toLowerCase()));
  let shared = 0;
  for (const token of wanted) if (actual.has(token)) shared += 1;
  const different = (wanted.size - shared) + (actual.size - shared);
  return shared * 4 - different;
}

/** Mirrors `fn basename`. */
function basename(path) {
  return pathFileName(path) ?? path;
}

/** Mirrors `fn render_listing`. */
function renderListing(intent, language, scope, output) {
  return fill(localized(intent, language), [[SCOPE_SLOT, scope], [ITEMS_SLOT, trim(output)]]);
}

/** Mirrors `fn render_not_found`. */
function renderNotFound(request, language) {
  return fill(localized('local_search_not_found', language), [
    [REQUESTED_SLOT, request.subject], [SCOPE_SLOT, request.root],
  ]);
}

/** Mirrors `fn render_type`. */
function renderType(request, candidate, metadata, language) {
  const intent = trimStart(metadata).startsWith('d') ? 'local_search_type_directory' : 'local_search_type_file';
  return fill(localized(intent, language), [
    [PATH_SLOT, candidate], [REQUESTED_SLOT, request.subject], [SCOPE_SLOT, request.root],
  ]);
}

/** Mirrors `fn localized`. */
function localized(intent, language) {
  return localizedResponse(intent, language) ?? '';
}

/** Mirrors `fn shell_quote`. */
function shellQuote(value) {
  return `'${replaceAllLiteral(value, "'", "'\\''")}'`;
}
