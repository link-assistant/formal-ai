// Seed-driven language detection: a port of rust/src/language.rs.
//
// Every rule lives in data/seed/language-detection.lino and the language
// ledger in data/seed/languages.lino, exactly the files the Rust module
// embeds. A `Language` is its slug string; `Language::Unknown` is 'unknown'.
// The forced response language (issue #556) is a per-solve native slot the
// planner never sets, so `detect` here never honours one.

import { lower as canonicalUnaryLower } from './rust_str.mjs';
import { cached, readText } from '../host.mjs';
import { rustLines } from '../content.mjs';

export const UNKNOWN = 'unknown';
export const ENGLISH = 'en';

const ALPHABETIC = /\p{Alphabetic}/u;
const isAlphabetic = (character) => ALPHABETIC.test(character);

/** Mirrors `fn unquote` in rust/src/language.rs. */
function unquote(value) {
  return value.length >= 2 && value.startsWith('"') && value.endsWith('"') ? value.slice(1, -1) : value;
}

/** Rust `str::split_once(' ')`. */
function splitOnce(text, separator) {
  const at = text.indexOf(separator);
  return at < 0 ? null : [text.slice(0, at), text.slice(at + separator.length)];
}

/** Mirrors `fn parse_codepoint`. */
function parseCodepoint(value) {
  const digits = value.startsWith('0x') || value.startsWith('0X') ? value.slice(2) : null;
  if (digits === null || !/^[0-9a-fA-F]+$/.test(digits)) return 0;
  return parseInt(digits, 16);
}

/** Mirrors `fn parse_quoted_list`. @param {string} value */
function parseQuotedList(value) {
  const items = [];
  let rest = value;
  for (let open = rest.indexOf('"'); open >= 0; open = rest.indexOf('"')) {
    const after = rest.slice(open + 1);
    const close = after.indexOf('"');
    if (close < 0) break;
    items.push(after.slice(0, close));
    rest = after.slice(close + 1);
  }
  return items;
}

/** Mirrors `fn parse_rules` (cached like the Rust `OnceLock`). */
function rules() {
  return cached('language-detection-rules', () => {
    const out = [];
    for (const line of rustLines(readText('data/seed/language-detection.lino'))) {
      const trimmed = line.trimStart();
      if (!trimmed) continue;
      const [key, value] = splitOnce(trimmed, ' ') ?? [trimmed, ''];
      if (key === 'rule') {
        out.push({ language: '', script: '', start: 0, end: 0, alphabetic_only: false, fallback: false, markers: [] });
        continue;
      }
      const rule = out[out.length - 1];
      if (!rule) continue;
      if (key === 'language') rule.language = unquote(value);
      else if (key === 'script') rule.script = unquote(value);
      else if (key === 'label' && !rule.script) rule.script = unquote(value);
      else if (key === 'start') rule.start = parseCodepoint(value);
      else if (key === 'end') rule.end = parseCodepoint(value);
      else if (key === 'alphabetic-only') rule.alphabetic_only = value === 'yes';
      else if (key === 'fallback') rule.fallback = value === 'yes';
      else if (key === 'markers') rule.markers = parseQuotedList(value);
    }
    return out.filter((rule) => rule.language);
  });
}

function ruleClaims(rule, character) {
  const codepoint = character.codePointAt(0);
  return codepoint >= rule.start && codepoint <= rule.end && (!rule.alphabetic_only || isAlphabetic(character));
}

/** Mirrors `fn from_slug`: the slug when the registry declares it, else null. */
export function languageFromSlug(slug) {
  return rules().find((rule) => rule.language === slug)?.language ?? null;
}

/** Mirrors `fn registered_languages`. */
export function registeredLanguages() {
  return rules().map((rule) => rule.language);
}

/** Mirrors `fn fallback_language`. */
export function fallbackLanguage() {
  return fallbackOf(rules());
}

function ledgerLines() {
  return cached('language-ledger-lines', () => rustLines(readText('data/seed/languages.lino')));
}

/** Mirrors `fn uses_postpositions` in rust/src/language.rs. */
export function usesPostpositions(slug) {
  let current = null;
  for (const line of ledgerLines()) {
    const split = splitOnce(line.trimStart(), ' ');
    if (!split) continue;
    const [key, value] = split;
    if (key === 'language') current = unquote(value);
    else if (key === 'adposition' && current === slug) return unquote(value) === 'postposition';
  }
  return false;
}

/** Mirrors `fn language_name` in rust/src/language.rs. */
export function languageName(slug) {
  let current = null;
  for (const line of ledgerLines()) {
    const trimmed = line.trimStart();
    const [key, value] = splitOnce(trimmed, ' ') ?? [trimmed, ''];
    if (key === 'language') current = unquote(value);
    else if (key === 'name' && current === slug) return unquote(value);
  }
  return null;
}

/** Mirrors `fn surface_matches_language` in rust/src/language.rs. */
export function surfaceMatchesLanguage(surface, slug) {
  const all = rules();
  const rule = all.find((candidate) => candidate.language === slug);
  if (!rule) return false;
  return all
    .filter((candidate) => candidate.script === rule.script)
    .some((candidate) => Array.from(surface).some((character) => ruleClaims(candidate, character)));
}

function fallbackOf(all) {
  return all.find((rule) => rule.fallback)?.language ?? ENGLISH;
}

function fallbackScriptOf(all) {
  return all.find((rule) => rule.fallback)?.script ?? '';
}

/** Mirrors `fn default_language_of`. */
function defaultLanguageOf(all, script) {
  const rule = all.find((candidate) => candidate.script === script && candidate.fallback)
    ?? all.find((candidate) => candidate.script === script);
  return rule ? rule.language : fallbackOf(all);
}

/** Mirrors `struct ScriptCounts` + `fn count_scripts`. */
function countScripts(prompt, all) {
  const counts = { counts: [], other: 0, first: null };
  for (const character of prompt) {
    const matched = all.find((rule) => ruleClaims(rule, character));
    if (matched) {
      const entry = counts.counts.find(([name]) => name === matched.script);
      if (entry) entry[1] += 1;
      else counts.counts.push([matched.script, 1]);
      if (counts.first === null) counts.first = matched.script;
    } else if (isAlphabetic(character)) {
      counts.other += 1;
      if (counts.first === null) counts.first = '';
    }
  }
  return counts;
}

const countOf = (counts, script) => counts.counts.find(([name]) => name === script)?.[1] ?? 0;
const maxExcluding = (counts, script) =>
  counts.counts.filter(([name]) => name !== script).reduce((max, [, count]) => Math.max(max, count), 0);
const total = (counts) => counts.counts.reduce((sum, [, count]) => sum + count, 0) + counts.other;

const WHITE_SPACE = /\p{White_Space}/u;
const isWhitespace = (character) => WHITE_SPACE.test(character);

/** The code point that ends just before UTF-16 index `at`, or undefined. */
function charBefore(text, at) {
  if (at <= 0) return undefined;
  const low = text.charCodeAt(at - 1);
  return at >= 2 && low >= 0xdc00 && low <= 0xdfff ? text.slice(at - 2, at) : text[at - 1];
}

/** The code point that starts at UTF-16 index `at`, or undefined. */
function charAt(text, at) {
  const codepoint = text.codePointAt(at);
  return codepoint === undefined ? undefined : String.fromCodePoint(codepoint);
}

/**
 * Mirrors `fn in_identifier`: whether the whitespace-delimited token around
 * `at` is a path or an identifier (`_`, `/`, `\`, or a `.` before a letter or
 * a digit), whose words are no language's (PR #1188 G97).
 */
function inIdentifier(text, at) {
  let start = at;
  while (start > 0 && !isWhitespace(text[start - 1])) start -= 1;
  let end = at;
  while (end < text.length && !isWhitespace(text[end])) end += 1;
  return identifierToken(text.slice(start, end));
}

/** Mirrors `fn identifier_token`. */
function identifierToken(token) {
  return /[_/\\]|\.[\p{Alphabetic}\p{N}]/u.test(token);
}

/** Mirrors `fn marker_count`: the occurrences of `marker` that count. */
function markerCount(normalized, marker, all, fallbackScript, limit) {
  const first = Array.from(marker)[0];
  if (first === undefined) return 0;
  const wholeWord = all.some((rule) => rule.script === fallbackScript && ruleClaims(rule, first));
  let found = 0;
  for (let from = 0; found < limit;) {
    const at = normalized.indexOf(marker, from);
    if (at < 0) break;
    from = at + marker.length;
    if (wholeWord) {
      const previous = charBefore(normalized, at);
      const next = charAt(normalized, from);
      const wordStart = previous === undefined || !isAlphabetic(previous);
      const wordEnd = next === undefined || !isAlphabetic(next);
      if (!wordStart || !wordEnd || inIdentifier(normalized, at)) continue;
    }
    found += 1;
  }
  return found;
}

/** Mirrors `fn marker_present`. */
function markerPresent(normalized, marker, all, fallbackScript) {
  return markerCount(normalized, marker, all, fallbackScript, 1) > 0;
}

/** Mirrors `fn marker_language`. */
function markerLanguage(prompt, all, counts, fallbackScript) {
  const normalized = prompt.toLowerCase();
  let best = null;
  for (const rule of all.filter((candidate) => candidate.markers.length)) {
    const count = countOf(counts, rule.script);
    if (count === 0) continue;
    const contested = all.some((other) =>
      other.script !== rule.script && other.script !== fallbackScript && countOf(counts, other.script) > 0);
    if (contested) continue;
    if (!rule.markers.some((marker) => markerPresent(normalized, marker.toLowerCase(), all, fallbackScript))) continue;
    if (best && count <= best[0]) continue;
    best = [count, rule.language];
  }
  return best ? best[1] : null;
}

/** Mirrors `fn detect_with`. */
function detectWith(prompt, all) {
  const fallback = fallbackOf(all);
  const fallbackScript = fallbackScriptOf(all);
  const counts = countScripts(prompt, all);
  if (total(counts) === 0) return fallback;
  const fallbackCount = countOf(counts, fallbackScript);
  if (counts.other > fallbackCount && counts.other >= maxExcluding(counts, fallbackScript)) return UNKNOWN;
  if (fallbackCount > 0) {
    const language = markerLanguage(prompt, all, counts, fallbackScript);
    if (language !== null) return language;
    const first = counts.first;
    if (first !== null && first !== '' && first !== fallbackScript) {
      const rival = counts.counts
        .filter(([name]) => name !== fallbackScript && name !== first)
        .reduce((max, [, count]) => Math.max(max, count), 0);
      if (countOf(counts, first) >= rival) return defaultLanguageOf(all, first);
    }
  }
  for (const rule of all.filter((candidate) => !candidate.fallback)) {
    const count = countOf(counts, rule.script);
    if (count > 0 && count >= maxExcluding(counts, rule.script)) return defaultLanguageOf(all, rule.script);
  }
  return fallback;
}

/**
 * Mirrors `fn detect` in rust/src/language.rs: the dominant language's slug,
 * or 'unknown'. The forced-language slot is never set on the planner path.
 * @param {string} prompt
 * @returns {string}
 */
export function detect(prompt) {
  return detectWith(String(prompt), rules());
}

/**
 * Mirrors `PROSE_SCRIPT_WEIGHT`: in prose, a registered script other than the
 * fallback's wins once its letters reach a sixth of the fallback script's.
 * Writers of other languages borrow fallback-script names and terms freely,
 * and a fallback-script word spends several letters where an ideographic
 * script spends one, so the fallback script must clearly outnumber a rival
 * script to win (issue #1557).
 */
const PROSE_SCRIPT_WEIGHT = 6;

/**
 * Mirrors `SHARED_SCRIPT_SHARE`: a language sharing the fallback script wins
 * prose once its votes reach a tenth of the fallback-script words, and a lone
 * vote must be half of them, so one marker or one accented name cannot outvote
 * a document or a short sentence (issue #1557).
 */
const SHARED_SCRIPT_SHARE = 10;

/** Mirrors `QUOTATIONS`: an opening quotation mark and its closing marks. */
const QUOTATIONS = [['"', '"'], ['\u201C', '\u201D'], ['\u00AB', '\u00BB'], ['\u201E', '\u201C\u201D']];

/** Mirrors `QUOTATION_OPENERS_AFTER`: punctuation a quotation may open after. */
const QUOTATION_OPENERS_AFTER = '([{|';

/** Mirrors `fn without_code_spans`: drop each closed `code` span of a line. */
function withoutCodeSpans(line) {
  const parts = line.split('`');
  return parts.filter((_, index) => index % 2 === 0 || index === parts.length - 1).join(' ');
}

/**
 * Mirrors `fn without_quotations`: drop each quotation that opens a token and
 * closes on the same line. A quote mark inside a token (`alt="Image"`) is
 * markup, not a quotation.
 */
function withoutQuotations(line) {
  const characters = Array.from(line);
  const kept = [];
  for (let index = 0; index < characters.length; index += 1) {
    const character = characters[index];
    const pair = QUOTATIONS.find(([open]) => open === character);
    const previous = characters[index - 1];
    if (pair && (previous === undefined || isWhitespace(previous) || QUOTATION_OPENERS_AFTER.includes(previous))) {
      const close = characters.findIndex((candidate, at) => at > index && pair[1].includes(candidate));
      if (close > index) {
        kept.push(' ');
        index = close;
        continue;
      }
    }
    kept.push(character);
  }
  return kept.join('');
}

/** Mirrors `fn registry_list`: the quoted items of the registry's first `key` line. */
function registryList(key) {
  for (const line of rustLines(readText('data/seed/language-detection.lino'))) {
    const [name, value] = splitOnce(line.trimStart(), ' ') ?? ['', ''];
    if (name === key) return parseQuotedList(value);
  }
  return [];
}

/** Mirrors `fn prose_noise`: a URL, a path, an identifier or a camelCase name. */
function proseNoise(token, links) {
  const lowered = token.toLowerCase();
  return links.some((link) => lowered.includes(link))
    || identifierToken(token)
    || /\p{Lowercase}\p{Uppercase}/u.test(token);
}

/**
 * Mirrors `fn prose_of`: the prose of a document — its lines without fenced
 * code blocks, block quotations, code spans, quotations, URLs and identifiers.
 */
export function proseOf(text) {
  const lines = [];
  const fences = registryList('fence-markers');
  const links = registryList('link-markers');
  let fenced = false;
  for (const line of rustLines(String(text))) {
    const trimmed = line.replace(/^\p{White_Space}+/u, '');
    if (fences.some((fence) => trimmed.startsWith(fence))) {
      fenced = !fenced;
      continue;
    }
    if (fenced || trimmed.startsWith('>')) continue;
    const tokens = withoutQuotations(withoutCodeSpans(line))
      .split(/\p{White_Space}+/u)
      .filter((token) => token && !proseNoise(token, links));
    if (tokens.length) lines.push(tokens.join(' '));
  }
  return lines.join('\n');
}

/** Mirrors `fn shared_script_language`. */
function sharedScriptLanguage(prose, all, fallbackScript) {
  const normalized = prose.toLowerCase();
  const words = normalized
    .split(/[^\p{Alphabetic}]+/u)
    .filter((word) => Array.from(word).some((character) =>
      all.some((rule) => rule.script === fallbackScript && ruleClaims(rule, character))));
  let best = null;
  for (const rule of all.filter((candidate) => candidate.script === fallbackScript && !candidate.fallback)) {
    const markers = rule.markers.reduce((sum, marker) =>
      sum + markerCount(normalized, marker.toLowerCase(), all, fallbackScript, Infinity), 0);
    const letters = words.filter((word) => Array.from(word).some((character) => ruleClaims(rule, character))).length;
    const votes = markers + letters;
    // A lone vote (one marker or one accented loanword such as "café")
    // decides only a text it is at least half of.
    const lone = votes === 1 && votes * 2 < words.length;
    if (votes === 0 || lone || votes * SHARED_SCRIPT_SHARE < words.length) continue;
    if (best && votes <= best[0]) continue;
    best = [votes, rule.language];
  }
  return best ? best[1] : null;
}

/** Mirrors `fn prose_language`. */
function proseLanguage(prose, all) {
  const fallbackScript = fallbackScriptOf(all);
  const counts = countScripts(prose, all);
  if (total(counts) === 0) return null;
  const fallbackCount = countOf(counts, fallbackScript);
  if (counts.other > fallbackCount && counts.other >= maxExcluding(counts, fallbackScript)) return UNKNOWN;
  let rival = null;
  for (const rule of all) {
    if (rule.script === fallbackScript) continue;
    const count = countOf(counts, rule.script);
    if (count > 0 && (rival === null || count > rival[0])) rival = [count, rule.script];
  }
  if (rival && rival[0] * PROSE_SCRIPT_WEIGHT >= fallbackCount) return defaultLanguageOf(all, rival[1]);
  return sharedScriptLanguage(prose, all, fallbackScript) ?? fallbackOf(all);
}

/**
 * Mirrors `fn detect_prose` in rust/src/language.rs: the language of a
 * document decided by the proportion of its prose, or null when the document
 * holds no prose letters. Code, quotations, URLs and identifiers do not vote,
 * so a quoted phrase or a single marker cannot outvote the rest (issue #1557).
 * @param {string} text
 * @returns {string | null}
 */
export function detectProse(text) {
  return proseLanguage(proseOf(text), rules());
}

/** Mirrors `fn language_for_concept_slug` in rust/src/language.rs. */
export function languageForConceptSlug(conceptSlug) {
  if (!conceptSlug.startsWith('language_')) return null;
  const name = conceptSlug.slice('language_'.length);
  const rule = rules().find((candidate) => {
    const declared = languageName(candidate.language);
    if (declared === null) return false;
    const left = Array.from(declared);
    const right = Array.from(name);
    return new TextEncoder().encode(declared).length === new TextEncoder().encode(name).length
      && left.every((character, index) => right[index] !== undefined
        && character.replace(/[A-Z]/g, canonicalUnaryLower) === right[index]);
  });
  return rule ? rule.language : null;
}
