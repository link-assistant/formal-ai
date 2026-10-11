// Script-aware sentence and clause segmentation with exact UTF-8 byte spans
// (rust/src/formalization/segment.rs). Terminators, openings, clause
// separators and script ranges come from data/seed/sentence-punctuation.lino.

import { cached, parseLino, readText } from '../host.mjs';
import { isNumeric, isWhitespace, trimEnd, utf8Len } from './rust_str.mjs';

/** Mirrors `SENTENCE_PUNCTUATION_LINO` (the repository path it embeds). */
export const SENTENCE_PUNCTUATION_PATH = 'data/seed/sentence-punctuation.lino';

/** Mirrors `enum Script` (`Script::slug` values). */
export const Script = Object.freeze({
  Latin: 'latin', Cyrillic: 'cyrillic', Devanagari: 'devanagari', Han: 'han', Other: 'other',
});

/** Mirrors `Script::from_slug`. */
function scriptFromSlug(slug) {
  return Object.values(Script).includes(slug) && slug !== Script.Other ? slug : Script.Other;
}

const unquote = (value) => String(value ?? '').replace(/^"+|"+$/g, '');
const firstChar = (value) => (value ? Array.from(value)[0] : undefined);

/** Mirrors `fn block`: `"0041-024F"` as an inclusive code-point range. */
function block(value) {
  const at = value.indexOf('-');
  if (at < 0) return null;
  const low = value.slice(0, at).trim();
  const high = value.slice(at + 1).trim();
  if (!/^[0-9A-Fa-f]+$/.test(low) || !/^[0-9A-Fa-f]+$/.test(high)) return null;
  return [parseInt(low, 16), parseInt(high, 16)];
}

/** Mirrors `fn punctuation`: every declared script's punctuation. */
function punctuation() {
  return cached('formalization-sentence-punctuation', () => {
    const root = parseLino(readText(SENTENCE_PUNCTUATION_PATH));
    const container = root?.name === 'sentence_punctuation' ? root : (root.children || []).find((node) => node.name === 'sentence_punctuation');
    if (!container) return [];
    const values = (node, name) => (node.children || []).filter((child) => child.name === name).map((child) => unquote(child.id ?? child.value));
    return (container.children || []).filter((node) => node.name === 'script').map((node) => ({
      script: scriptFromSlug(unquote(node.id ?? node.value)),
      ranges: values(node, 'range').map(block).filter((range) => range !== null),
      terminators: values(node, 'terminator').map(firstChar).filter((value) => value !== undefined),
      openings: values(node, 'opening').map(firstChar).filter((value) => value !== undefined),
      clause_separators: values(node, 'clause_separator').map(firstChar).filter((value) => value !== undefined),
    }));
  });
}

/** Mirrors `fn script_of`. @param {string} character */
export function scriptOf(character) {
  const code = character.codePointAt(0);
  const declared = punctuation().find((entry) => entry.ranges.some(([low, high]) => code >= low && code <= high));
  return declared ? declared.script : Script.Other;
}

const isTerminator = (character) => punctuation().some((entry) => entry.terminators.includes(character));
const isOpening = (character) => punctuation().some((entry) => entry.openings.includes(character));

/** Mirrors `fn dominant_script` (`max_by_key` keeps the last maximum). */
function dominantScript(text) {
  const counts = [];
  for (const character of text) {
    const script = scriptOf(character);
    if (script === Script.Other) continue;
    const entry = counts.find(([seen]) => seen === script);
    if (entry) entry[1] += 1;
    else counts.push([script, 1]);
  }
  let best = null;
  for (const entry of counts) if (best === null || entry[1] >= best[1]) best = entry;
  return best === null ? Script.Other : best[0];
}

/**
 * The characters of `text` with their UTF-8 byte offsets (Rust `char_indices`).
 * Rust built-in `str::char_indices`.
 * @param {string} text
 * @returns {Array<[number, string]>}
 */
export function charIndices(text) {
  const out = [];
  let offset = 0;
  for (const character of text) {
    out.push([offset, character]);
    offset += utf8Len(character);
  }
  return out;
}

/** Rust built-in `&text[start..end]`: byte slicing over UTF-8 byte offsets. */
export function byteSlice(text, start, end) {
  return new TextDecoder().decode(new TextEncoder().encode(text).slice(start, end));
}

/**
 * Mirrors `fn sentences`: `{text, start, end, script}` segments with byte spans.
 * @param {string} text
 */
export function sentences(text) {
  const out = [];
  let start = null;
  const characters = charIndices(text);
  characters.forEach(([offset, character], index) => {
    if (start === null) {
      if (isWhitespace(character)) return;
      start = offset;
    }
    if (!isTerminator(character) || isOpening(character)) return;
    const previous = index > 0 ? characters[index - 1][1] : undefined;
    const next = index + 1 < characters.length ? characters[index + 1][1] : undefined;
    if (previous !== undefined && isNumeric(previous) && next !== undefined && isNumeric(next)) return;
    const end = offset + utf8Len(character);
    const begin = start;
    start = null;
    pushSegment(text, begin, end, out);
  });
  if (start !== null) pushSegment(text, start, utf8Len(text), out);
  return out;
}

function pushSegment(text, start, end, out) {
  const trimmed = trimEnd(byteSlice(text, start, end));
  if (!trimmed) return;
  out.push({ text: trimmed, start, end: start + utf8Len(trimmed), script: dominantScript(trimmed) });
}

/**
 * Mirrors `fn clauses`: a sentence split at its script's clause separators,
 * spans absolute in the original text.
 * @param {{text: string, start: number, end: number, script: string}} sentence
 */
export function clauses(sentence) {
  const separators = punctuation().filter((entry) => entry.script === sentence.script).flatMap((entry) => entry.clause_separators);
  const out = [];
  let start = null;
  for (const [offset, character] of charIndices(sentence.text)) {
    if (start === null) {
      if (isWhitespace(character)) continue;
      start = offset;
    }
    if (!separators.includes(character)) continue;
    const begin = start;
    start = null;
    pushClause(sentence, begin, offset, out);
  }
  if (start !== null) pushClause(sentence, start, utf8Len(sentence.text), out);
  if (!out.length) out.push({ ...sentence });
  return out;
}

function pushClause(sentence, start, end, out) {
  const trimmed = trimEnd(byteSlice(sentence.text, start, end));
  if (!trimmed) return;
  out.push({
    text: trimmed,
    start: sentence.start + start,
    end: sentence.start + start + utf8Len(trimmed),
    script: sentence.script,
  });
}
