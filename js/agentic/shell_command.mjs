// Resolve a user turn into the concrete shell command the agentic loop should
// run: a port of rust/src/agentic_coding/shell_command.rs.

import {
  commandSpan, governsCommandsRatherThanRequestingOne, isProseWord, namedShellCommandInSentence,
  normalizeCommandWord, sentenceSpans, shellQuotesPaired, statesACommandPolicy,
} from './shell_command_policy.mjs';
import { asksForDirectoryListing } from './directory_listing.mjs';
import { withoutPathWords } from './workspace_line_operation.mjs';
import { isDottedNumber, trimTrailingSentenceDot } from './file_path_shape.mjs';
import { requestBlocks } from './stated_request.mjs';
import { testFileCommand } from './test_file_runner.mjs';
import { currentDirectory, isFile } from './host.mjs';
import { effectIsDeclared, shellIntentVocabulary } from './crate/seed_shell_intents.mjs';
import { terminalCommandVocabulary } from './crate/seed_terminal_commands.mjs';
import { asksAQuestion, callerContextVocabulary, copulaIn } from './crate/seed_caller_context.mjs';
import { agenticToolCapabilities } from './crate/seed_agentic_tool_capabilities.mjs';
import { evidencedIn, meaning, mentionsRole, roleWordForms, wordsForRole } from './crate/seed_meanings.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { containsWordSequence } from './crate/solver_handlers_benchmark_prompts.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { webSearchQueryFor } from './crate/solver_handlers_web_search.mjs';
import {
  byteFind, charCount, compareTuples, eqIgnoreAsciiCase, isAlphanumeric, isWhitespace, maxByKey, minByKey,
  sliceFromByte, splitWhitespace, trim, trimEndMatches, trimMatches, trimStart, utf8Len,
} from './crate/rust_str.mjs';

const REPORT_ISSUE_ACTION = 'formal-ai:report-issue';
const ROLE_CODING_SEARCH_LITERAL_QUERY = 'coding_search_literal_query';
const ROLE_CODING_SEARCH_SUBJECT_KIND = 'coding_search_subject_kind';
const ROLE_HELLO_WORLD_REFERENCE = 'hello_world_reference';
const ROLE_PROGRAM_KIND = 'program_kind';
const ROLE_PROGRAM_REQUEST = 'program_request';

const firstWord = (text) => splitWhitespace(text)[0];

/** Mirrors `fn shell_command_for_task`. */
export function shellCommandForTask(task) {
  const prompt = stripBalancedOuterQuotes(trim(task));
  if (governsCommandsRatherThanRequestingOne(prompt)) return null;
  const vocab = terminalCommandVocabulary();
  const intentVocab = shellIntentVocabulary();
  const intent = intentShellCommand(prompt, intentVocab);
  const listing = asksForDirectoryListing(prompt);
  const webSearch = webSearchQueryFor(prompt) !== null;
  const semantic = listing ? 'ls' : intent;
  const command = prefixedShellCommand(prompt, vocab);
  if (command !== null) {
    const firstToken = firstWord(command);
    const first = firstToken === undefined ? null : normalizeCommandWord(firstToken);
    const namesKnownCommand = first !== null && vocab.shell_tokens.includes(first);
    const semanticFirst = semantic === null ? undefined : firstWord(semantic);
    const namesSemanticCommand = semantic !== null
      && (semanticFirst === undefined ? null : normalizeCommandWord(semanticFirst)) === first;
    if (namesSemanticCommand) return semantic;
    if (namesKnownCommand || (semantic === null && !webSearch)) return command;
  }
  return semantic ?? namedShellCommand(prompt, vocab) ?? bareShellCommand(prompt, vocab);
}

/** Mirrors `fn semantic_shell_command_for_task`. */
export function semanticShellCommandForTask(prompt) {
  return intentShellCommand(prompt, shellIntentVocabulary());
}

/** Mirrors `fn strip_balanced_outer_quotes`. */
function stripBalancedOuterQuotes(prompt) {
  for (const quote of ['"', '\'', '`']) {
    if (prompt.startsWith(quote) && prompt.endsWith(quote) && utf8Len(prompt) >= 2) return prompt.slice(1, -1);
  }
  return prompt;
}

/** Mirrors `fn prefixed_shell_command`. */
function prefixedShellCommand(raw, vocab) {
  const prompt = trim(raw);
  const lower = prompt.toLowerCase();
  const prefix = maxByKey(vocab.passthrough_prefixes.filter((candidate) => prefixBoundary(lower, candidate)), charCount);
  if (prefix === undefined) return null;
  const tail = sliceFromByte(prompt, utf8Len(prefix));
  if (tail === null) return null;
  let remainder = commandSpan(trimStart(tail));
  if (remainder === null) return null;
  remainder = trimCommandSentenceEnd(stripBalancedOuterQuotes(remainder));
  const named = commandNamedInProse(remainder, vocab);
  if (named !== null) return named;
  return remainder && !readsAsProse(remainder, vocab) ? remainder : null;
}

/**
 * Mirrors `fn trim_command_sentence_end`: the sentence's full stop and the
 * quotes it wrapped around the command, peeled from the last token to a
 * fixpoint — `Run python3 greet.py.` runs `python3 greet.py`, and
 * `` Run `ls -la`. `` runs `ls -la`, while `cd ..` and `ls .` keep their dots.
 */
function trimCommandSentenceEnd(remainder) {
  let current = remainder;
  for (;;) {
    const unquoted = stripBalancedOuterQuotes(trim(current));
    const lastSpace = /\s(?=\S*$)/u.exec(unquoted);
    const split = lastSpace === null ? 0 : lastSpace.index + lastSpace[0].length;
    const next = unquoted.slice(0, split) + trimTrailingSentenceDot(unquoted.slice(split));
    if (next === current) return current;
    current = next;
  }
}

const SHELL_QUOTING_AND_METACHARACTERS = ['"', '\'', '`', '|', '&', ';', '<', '>', '$', '(', ')', '{', '}'];

/** Mirrors `fn command_named_in_prose`. */
function commandNamedInProse(remainder, vocab) {
  if (SHELL_QUOTING_AND_METACHARACTERS.some((character) => remainder.includes(character))) return null;
  const words = splitWhitespace(remainder);
  const kept = words.filter((word) => !namesACommand(word, vocab));
  if (kept.length === words.length || kept.length === 0) return null;
  const start = kept.findIndex((word) => !isProseWord(word));
  if (start < 0) return null;
  const command = [kept[start]];
  for (const word of kept.slice(start + 1)) {
    if (isProseWord(word)) break;
    command.push(word);
  }
  return command.join(' ');
}

/** Mirrors `fn names_a_command`. */
function namesACommand(word, vocab) {
  const normalized = trimMatches(word, (character) => !isAlphanumeric(character)).toLowerCase();
  return normalized !== '' && vocab.command_nouns.includes(normalized);
}

/** Mirrors `fn reads_as_prose`. */
function readsAsProse(remainder, vocab) {
  const words = splitWhitespace(remainder);
  const first = words[0];
  if (first === undefined) return true;
  if (first.includes('://')) return true;
  if (vocab.shell_tokens.includes(normalizeCommandWord(first))) return false;
  return isProseWord(first) || words.slice(1).filter(isProseWord).length >= 2;
}

/** Mirrors `fn explicit_passthrough_command`. */
export function explicitPassthroughCommand(task) {
  const prompt = stripBalancedOuterQuotes(trim(task));
  if (governsCommandsRatherThanRequestingOne(prompt)) return null;
  const vocab = terminalCommandVocabulary();
  const command = prefixedShellCommand(prompt, vocab);
  if (command === null) return null;
  const first = normalizeCommandWord(firstWord(command) ?? '');
  const semantic = asksForDirectoryListing(prompt) ? 'ls' : intentShellCommand(prompt, shellIntentVocabulary());
  if ((semantic !== null || webSearchQueryFor(prompt) !== null) && !vocab.shell_tokens.includes(first)) return null;
  if (semantic !== null && normalizeCommandWord(firstWord(semantic) ?? '') === first
    && !SHELL_QUOTING_AND_METACHARACTERS.some((character) => command.includes(character))
    && splitWhitespace(command).slice(1).some(isProseWord)) return semantic;
  return shellQuotesPaired(command) ? command : null;
}

/** Mirrors `fn bare_shell_command`. */
function bareShellCommand(prompt, vocab) {
  const words = splitWhitespace(prompt);
  if (!words.length) return null;
  const command = normalizeCommandWord(words[0]);
  if (webSearchQueryFor(prompt) !== null) return null;
  const isKnown = vocab.bare_shell_tokens.includes(command);
  return isKnown && words.slice(1).every((word) => !isProseWord(word)) ? prompt : null;
}

/** Mirrors `fn prefix_boundary`. */
function prefixBoundary(prompt, prefix) {
  if (!prompt.startsWith(prefix)) return false;
  const rest = sliceFromByte(prompt, utf8Len(prefix));
  const next = rest === null ? undefined : Array.from(rest)[0];
  return next !== undefined && (isWhitespace(next) || next === ':');
}

/** Mirrors `fn code_search_query_for_task`. */
export function codeSearchQueryForTask(prompt) {
  const vocab = shellIntentVocabulary();
  for (const block of requestBlocks(prompt)) {
    const lower = block.toLowerCase();
    const cue = longestSearchCue(lower, vocab);
    if (cue === null) continue;
    const query = literalCodeSearchQuery(lower)
      ?? shapedCodeSearchToken(block)
      ?? adjacentCodeSearchToken(block, lower)
      ?? localSearchQuery(block, cue, vocab);
    if (query !== null) return query;
  }
  return null;
}

/** Mirrors `fn code_shaped_query`. */
export function codeShapedQuery(text) {
  const normalized = text.toLowerCase();
  return literalCodeSearchQuery(normalized) ?? shapedCodeSearchToken(text) ?? adjacentCodeSearchToken(text, normalized);
}

/** Mirrors `fn longest_search_cue`. */
function longestSearchCue(normalized, vocab) {
  const intentCues = vocab.intents.filter((intent) => intent.command === 'rg').flatMap((intent) => intent.cues);
  const capabilityCues = agenticToolCapabilities().filter((capability) => capability.id === 'grep')
    .flatMap((capability) => capability.cues);
  const cue = maxByKey([...intentCues, ...capabilityCues].filter((candidate) => normalized.includes(candidate)), charCount);
  return cue ?? null;
}

/** Mirrors `fn literal_code_search_query`. */
function literalCodeSearchQuery(normalized) {
  const form = roleWordForms(ROLE_CODING_SEARCH_LITERAL_QUERY).find((candidate) => normalized.includes(candidate.text.toLowerCase()));
  return form && form.action ? form.action : null;
}

/** Mirrors `fn shaped_code_search_token`. */
function shapedCodeSearchToken(prompt) {
  const scored = [];
  for (const token of searchTokens(prompt)) {
    const interiorUppercase = Array.from(token).slice(1).some((character) => /^[A-Z]$/.test(character));
    const score = (token.includes('.') ? 4 : 0) + (token.includes('_') ? 4 : 0) + (interiorUppercase ? 3 : 0)
      + (token.includes('-') ? 2 : 0);
    if (score > 0 && !token.includes('/') && !isDottedNumber(token)) scored.push([score, utf8Len(token), token]);
  }
  const best = maxByKey(scored, ([score, length]) => [score, length], compareTuples);
  return best === undefined ? null : best[2].replaceAll('-', '_');
}

/** Mirrors `fn adjacent_code_search_token` (UTF-8 byte offsets, as Rust measures them). */
function adjacentCodeSearchToken(prompt, normalized) {
  const tokens = searchTokensWithOffsets(prompt);
  const candidates = [];
  for (const form of roleWordForms(ROLE_CODING_SEARCH_SUBJECT_KIND)) {
    const surface = form.text.toLowerCase();
    const start = byteFind(normalized, surface);
    if (start < 0) continue;
    const end = start + utf8Len(surface);
    const near = [];
    for (const [token, tokenStart, tokenEnd] of tokens) {
      if (!(tokenEnd <= start || tokenStart >= end)) continue;
      const distance = tokenEnd <= start ? start - tokenEnd : tokenStart - end;
      if (validSearchIdentifier(token)) near.push([distance, tokenStart, token]);
    }
    const best = minByKey(near, ([distance, offset]) => [distance, offset], compareTuples);
    if (best !== undefined) candidates.push(best);
  }
  const best = minByKey(candidates, ([distance, offset]) => [distance, offset], compareTuples);
  return best === undefined ? null : best[2];
}

/** Mirrors `fn search_tokens`. */
export function searchTokens(text) {
  return text.split(/[^0-9A-Za-z_.:-]/u)
    .map((token) => trimMatches(token, (character) => character === '.' || character === '-'))
    .filter(Boolean);
}

/** Mirrors `fn search_tokens_with_offsets`: `[token, startByte, endByte]`. */
function searchTokensWithOffsets(text) {
  const separator = (character) => !/^[0-9A-Za-z_]$/.test(character);
  const chunks = [];
  let chunk = '';
  for (const character of text) {
    chunk += character;
    if (separator(character)) {
      chunks.push(chunk);
      chunk = '';
    }
  }
  if (chunk) chunks.push(chunk);
  const out = [];
  let offset = 0;
  for (const part of chunks) {
    const start = offset;
    offset += utf8Len(part);
    const token = trimEndMatches(part, separator);
    if (token) out.push([token, start, start + utf8Len(token)]);
  }
  return out;
}

/** Mirrors `fn valid_search_identifier`. */
export function validSearchIdentifier(token) {
  const chars = Array.from(token);
  return chars.length > 0
    && /^[A-Za-z_]$/.test(chars[0])
    && chars.slice(1).every((character) => /^[0-9A-Za-z_]$/.test(character))
    && !wordsForRole(ROLE_CODING_SEARCH_SUBJECT_KIND).some((surface) => eqIgnoreAsciiCase(surface, token));
}

const MOOD_ENDS = new Set(['.', '!', '?', ';', '\n', '。', '！', '？', '；']);

/** Mirrors `fn sentences_with_mood`: `[text, interrogative]` pairs. */
function sentencesWithMood(lower) {
  const out = [];
  let start = 0;
  let index = 0;
  for (const character of lower) {
    const at = index;
    index += character.length;
    if (!MOOD_ENDS.has(character)) continue;
    if (character === '.') {
      const next = Array.from(lower.slice(index))[0];
      if (next !== undefined && isAlphanumeric(next)) continue;
    }
    const text = trim(lower.slice(start, at));
    if (text) out.push([text, character === '?' || character === '？']);
    start = index;
  }
  const tail = trim(lower.slice(start));
  if (tail) out.push([tail, false]);
  return out;
}

/** Mirrors `fn requesting_sentences`. */
function requestingSentences(sentences, cues) {
  return sentences
    .filter(([sentence, interrogative]) => !cues.some((cue) =>
      sentence.includes(cue) && isFactStatement(sentence, interrogative, cue)))
    .map(([sentence]) => sentence);
}

/** Mirrors `fn carries_authoring_task`. */
export function carriesAuthoringTask(sentence) {
  return mentionsRole(ROLE_PROGRAM_REQUEST, sentence)
    && [ROLE_PROGRAM_KIND, ROLE_HELLO_WORLD_REFERENCE].some((role) => mentionsRole(role, sentence));
}

/** Mirrors `fn strip_subject_lead`. */
function stripSubjectLead(sentence) {
  for (const lead of callerContextVocabulary().subject_leads) {
    if (sentence.startsWith(lead) && sentence.slice(lead.length).startsWith(' ')) {
      return trimStart(sentence.slice(lead.length));
    }
  }
  return sentence;
}

/** Mirrors `fn labels_a_value`. */
function labelsAValue(sentence, cue) {
  const headEnd = byteFind(sentence.toLowerCase(), cue);
  if (headEnd < 0) return false;
  const tail = sliceFromByte(sentence, headEnd + utf8Len(cue));
  if (tail === null) return false;
  const rest = trimStart(tail);
  if (!rest.startsWith(':') && !rest.startsWith('：')) return false;
  const words = splitWhitespace(trim(rest.slice(1)));
  if (!words.length) return false;
  return words.length === 1 && (words[0].startsWith('/') || words[0].startsWith('~/'));
}

/** Mirrors `fn is_fact_statement`. */
function isFactStatement(sentence, interrogative, cue) {
  if (interrogative || asksAQuestion(sentence)) return false;
  if (labelsAValue(sentence, cue)) return true;
  let rest = null;
  if (sentence.startsWith(cue)) rest = sentence.slice(cue.length);
  else {
    const stripped = stripSubjectLead(sentence);
    if (stripped.startsWith(cue)) rest = stripped.slice(cue.length);
  }
  if (rest === null) return false;
  rest = trimMatches(rest, (character) => isWhitespace(character) || ',:：，-'.includes(character));
  const words = splitWhitespace(rest);
  if (!words.length) return false;
  const head = words[0];
  const last = words[words.length - 1];
  const copula = copulaIn(head);
  if (copula !== null) return words.length > 1 || charCount(head) > charCount(copula);
  return words.length > 1 && copulaIn(last) !== null;
}

/** Mirrors `fn intent_shell_command`. */
function intentShellCommand(prompt, vocab) {
  const lower = prompt.toLowerCase();
  // A request about text inside a file carries its payload in quotes: the
  // words there are data, never the command it asks for (PR #1188 G32:
  // `Add the line '- run tests' …` ran the workspace tests).
  const insideAFile = editsInsideAFile(prompt);
  const matched = matchedIntentCue(insideAFile ? outsideQuotedSegments(prompt).toLowerCase() : lower, vocab);
  if (matched === null) return null;
  const [intent, cue] = matched;
  // A destructive intent (`destructive true` in the seed) never reads a
  // request about text inside a file as a request to delete the file.
  if (intent.destructive && insideAFile) return null;
  // Nor does it read a move or copy of text inside a file (`Move lines 2-3 of
  // a.md to the end of b.md`) as a move or copy of the file (PR #1188 G85).
  if (intent.argument === 'two_paths' && insideAFile) return null;
  const withArgument = (argument) => (argument === null ? null : `${intent.command} ${argument}`);
  switch (intent.argument) {
    case 'none':
      // A named test file runs with its own runtime (PR #1188 T91).
      return testFileCommand(intent.command, pathArgument(prompt)) ?? resolveShellCommand(intent.command, vocab);
    case 'path':
      return withArgument(pathArgument(prompt));
    case 'name_lead':
      return withArgument(nameLeadArgument(prompt, vocab.name_leads));
    case 'one_path':
      return withArgument(pathArguments(prompt, cue, vocab, 1));
    case 'two_paths':
      return withArgument(pathArguments(prompt, cue, vocab, 2));
    case 'remainder': {
      const argument = remainderArgument(prompt, lower, cue);
      return argument === null ? null : `${intent.command} --fixed-strings -- '${argument}' .`;
    }
    default: {
      const argument = localSearchQuery(prompt, cue, vocab);
      return argument === null ? null : `${intent.command} --fixed-strings -- '${argument}' .`;
    }
  }
}

/**
 * Mirrors `fn sentence_carries_cue`: the cue is in the sentence as written,
 * or once both drop the seeded filler words a speaker inserts inside a cue
 * phrase (`show me git status` carries `show git status`).
 */
function sentenceCarriesCue(sentence, cue, fillers) {
  // Whole words only (CJK aside): `move` is not inside `removed` (PR #1188 G66).
  if (containsWordSequence(sentence, cue)) return true;
  const withoutFillers = (text) => splitWhitespace(text).filter((word) => !fillers.includes(word)).join(' ');
  const bare = withoutFillers(cue);
  return bare !== '' && containsWordSequence(withoutFillers(sentence), bare);
}

/** Mirrors `fn matched_intent_cue`: `[intent, cue]` or null. */
function matchedIntentCue(lower, vocab) {
  // Path operands and quoted payloads supply data, never operation cues.
  lower = outsideQuotedSegments(withoutPathWords(lower));
  const moods = sentencesWithMood(lower);
  const cues = vocab.intents.flatMap((intent) => intent.cues);
  const requesting = requestingSentences(moods, cues);
  if (carriesAuthoringTask(lower)) return null;
  const pairs = [];
  for (const intent of vocab.intents) {
    if (intent.command === REPORT_ISSUE_ACTION) continue;
    for (const cue of intent.cues) {
      if (!requesting.some((sentence) => sentenceCarriesCue(sentence, cue, vocab.cue_fillers))) continue;
      if (intent.argument === 'search_query' && !vocab.local_search_scopes.some((scope) => lower.includes(scope))) continue;
      pairs.push([intent, cue]);
    }
  }
  return maxByKey(pairs, ([, cue]) => charCount(cue)) ?? null;
}

/** The request without its quoted segments. */
function outsideQuotedSegments(prompt) {
  let outside = '';
  let cursor = 0;
  for (const segment of quotedSegmentSpans(prompt)) {
    if (segment.start < cursor) continue;
    outside += `${prompt.slice(cursor, segment.start)} `;
    cursor = segment.end;
  }
  return outside + prompt.slice(cursor);
}

/**
 * Mirrors `fn edits_inside_a_file`: the request is about text inside a file —
 * it quotes a payload that is not a path, or names a line (the seeded `line`
 * meaning) or another unit of text (`file_text_unit`: word, phrase,
 * occurrence, text, in every registered language) outside its quotes
 * (PR #1188: `t.md से drop शब्द हटाओ।` ran `rm t.md`).
 */
export function editsInsideAFile(prompt) {
  const quoted = quotedSegmentSpans(prompt).map((segment) => trim(segment.text)).filter((text) => text !== '');
  if (quoted.some((text) => !looksLikeAPath(text))) return true;
  const outside = normalizePrompt(outsideQuotedSegments(withoutPathWords(prompt))).toLowerCase();
  const line = meaning('line');
  return (line !== null && line !== undefined && evidencedIn(line, outside)) || mentionsRole('file_text_unit', outside);
}

/**
 * Mirrors `fn refused_destructive_edit`: when the request's seeded intent is
 * destructive but the request edits inside a file, the path it names (or
 * '') so the planner can decline honestly instead of composing the command;
 * null otherwise.
 */
export function refusedDestructiveEdit(task) {
  const prompt = stripBalancedOuterQuotes(trim(task));
  const matched = matchedIntentCue(prompt.toLowerCase(), shellIntentVocabulary());
  if (matched === null || !matched[0].destructive || !editsInsideAFile(prompt)) return null;
  const written = splitWhitespace(outsideQuotedSegments(prompt))
    .map((token) => trimMatches(token, (character) => '`"\'()[]{}<>,;:!?।。'.includes(character)))
    .map((token) => trimTrailingSentenceDot(token))
    .find((token) => looksLikeAPath(token));
  return written ?? pathArguments(prompt, matched[1], shellIntentVocabulary(), 1) ?? '';
}

/** Mirrors `fn names_mutating_shell_intent`. */
export function namesMutatingShellIntent(prompt) {
  const matched = matchedIntentCue(prompt.toLowerCase(), shellIntentVocabulary());
  return matched !== null && effectIsDeclared(matched[0].effect);
}

/** Mirrors `fn path_arguments`. */
function pathArguments(prompt, cue, vocab, count) {
  const lower = prompt.toLowerCase();
  const anchored = namesAPathObject(cue, vocab);
  const start = byteFind(lower, cue);
  const afterCue = start < 0 ? '' : (sliceFromByte(prompt, start + utf8Len(cue)) ?? '');
  let args = collectPathArguments(afterCue, '', vocab, count, anchored);
  if (args.length < count) args = collectPathArguments(prompt, cue, vocab, count, anchored);
  return args.length === count ? args.join(' ') : null;
}

/** Mirrors `fn names_a_path_object`. */
function namesAPathObject(cue, vocab) {
  return vocab.path_objects.some((object) => cue.includes(object));
}

/** Mirrors `fn looks_like_a_path`. */
function looksLikeAPath(token) {
  if (isDottedNumber(token)) return false;
  if (token.startsWith('~') || token.includes('/')) return true;
  const dot = token.lastIndexOf('.');
  if (dot < 0) return false;
  const stem = token.slice(0, dot);
  const extension = token.slice(dot + 1);
  return stem !== '' && extension !== '' && Array.from(extension).every(isAlphanumeric);
}

/** Mirrors `fn collect_path_arguments`. */
function collectPathArguments(text, rawCue, vocab, count, anchored) {
  const cue = rawCue.toLowerCase();
  const cueParts = splitWhitespace(cue);
  const args = [];
  for (const word of splitWhitespace(text)) {
    // A quoted operand closes before the full stop: `'b.txt'.` (PR #1188 G37).
    const quote = (character) => '`"\',;:!?«»‘’“”'.includes(character);
    const candidate = trimMatches(trimTrailingSentenceDot(trimMatches(word, quote)), quote);
    const normalized = candidate.toLowerCase();
    if (!candidate || !isSafePath(candidate) || (!anchored && !looksLikeAPath(candidate))
      || cueParts.includes(normalized) || vocab.argument_noise.includes(normalized)) continue;
    args.push(candidate);
    if (args.length === count) break;
  }
  return args;
}

const QUERY_CHARACTER = (character) => isAlphanumeric(character) || '_-.:'.includes(character);

/** Mirrors `fn local_search_query`. */
function localSearchQuery(prompt, cue, vocab) {
  const cueWords = splitWhitespace(cue).map((word) => word.toLowerCase());
  const scopeWords = vocab.local_search_scopes.flatMap((scope) => splitWhitespace(scope).map((word) => word.toLowerCase()));
  const query = splitWhitespace(prompt)
    .map((word) => trimMatches(word, (character) => !QUERY_CHARACTER(character)))
    .filter(Boolean)
    .filter((word) => {
      const lower = word.toLowerCase();
      return !cueWords.includes(lower) && !scopeWords.includes(lower) && !vocab.argument_noise.includes(lower);
    })
    .join(' ');
  return query && Array.from(query).every((character) => QUERY_CHARACTER(character) || isWhitespace(character)) ? query : null;
}

/** Mirrors `fn resolve_shell_command`. */
function resolveShellCommand(command, vocab) {
  if (command.startsWith('formal-ai:workspace-')) {
    return workspaceCommand(command.slice('formal-ai:workspace-'.length), vocab);
  }
  return command;
}

/**
 * Mirrors `fn workspace_command`: the first workspace whose marker file exists
 * in the server's working directory (`Path::new(marker).is_file()`).
 */
function workspaceCommand(action, vocab) {
  const cwd = currentDirectory();
  const commands = vocab.workspace_commands.find((candidate) =>
    isFile(cwd && !candidate.marker.startsWith('/') ? `${cwd}/${candidate.marker}` : candidate.marker));
  if (!commands) return null;
  if (action === 'test') return commands.test;
  if (action === 'install') return commands.install;
  if (action === 'build') return commands.build;
  return null;
}

/** Mirrors `fn remainder_argument`. */
function remainderArgument(prompt, lower, cue) {
  const start = byteFind(lower, cue);
  if (start < 0) return null;
  const tail = sliceFromByte(prompt, start + utf8Len(cue));
  if (tail === null) return null;
  const remainder = trim(tail);
  return remainder && Array.from(remainder).every((character) => QUERY_CHARACTER(character) || isWhitespace(character))
    ? remainder : null;
}

/** Mirrors `fn path_argument`. */
function pathArgument(prompt) {
  for (const word of prompt.split(/\p{White_Space}/u)) {
    const token = trimEndMatches(trimMatches(word, (character) => '`"\',;?'.includes(character)),
      (character) => character === '.' || character === '!');
    const interiorDot = trimMatches(token, (character) => character === '.').includes('.');
    if (interiorDot && !token.includes('://') && isSafePath(token)) return token;
  }
  return null;
}

/** Mirrors `fn name_lead_argument`. */
function nameLeadArgument(prompt, nameLeads) {
  const words = splitWhitespace(prompt);
  const leadIndex = words.findIndex((word) =>
    nameLeads.includes(trimMatches(word, (character) => !isAlphanumeric(character)).toLowerCase()));
  if (leadIndex < 0 || words[leadIndex + 1] === undefined) return null;
  const name = trimMatches(words[leadIndex + 1], (character) => '`"\',;.!?:'.includes(character));
  return name && isSafePath(name) ? name : null;
}

/** Mirrors `fn is_safe_path`. */
function isSafePath(token) {
  let body = token.startsWith('~/') ? token.slice(2) : token;
  body = body.startsWith('/') ? body.slice(1) : body;
  return !token.startsWith('-')
    && body !== ''
    && !body.split('/').some((part) => part === '..' || part === '')
    && Array.from(token).every((character) => isAlphanumeric(character) || '/._-~'.includes(character));
}

/** Mirrors `fn named_shell_command`. */
function namedShellCommand(prompt, vocab) {
  for (const sentence of sentenceSpans(prompt)) {
    if (statesACommandPolicy(sentence)) continue;
    const command = namedShellCommandInSentence(sentence, vocab);
    if (command !== null) return command;
  }
  return null;
}
