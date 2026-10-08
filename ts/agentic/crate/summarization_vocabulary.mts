// `crate::summarization::vocabulary` (rust/src/summarization/vocabulary.rs):
// the seed-driven word lists shared by statement merging and the identifier
// rung. Three roles in `data/seed/meanings-statement-merge.lino` carry the
// vocabulary; nothing is hardcoded here. The lists are cached per installed
// host, like the Rust `OnceLock`s.
//

import { cached } from '../host.mjs';
import { containsCjk, wordsForRole } from './seed_meanings.mjs';
import { multilingualResponses } from './seed.mjs';

/** Mirrors `ROLE_STATEMENT_FUNCTION_WORD` in rust/src/seed/roles/tooling.rs. */
export const ROLE_STATEMENT_FUNCTION_WORD = 'statement_function_word';
/** Mirrors `ROLE_STATEMENT_NEGATION_CUE` in rust/src/seed/roles/tooling.rs. */
export const ROLE_STATEMENT_NEGATION_CUE = 'statement_negation_cue';
/** Mirrors `ROLE_IDENTIFIER_RESERVED_WORD` in rust/src/seed/roles/program.rs. */
export const ROLE_IDENTIFIER_RESERVED_WORD = 'identifier_reserved_word';

/** Mirrors `fn words_for` in rust/src/summarization/vocabulary.rs: the role's words, lowercased. */
function wordsFor(role) {
  return wordsForRole(role).map((word) => word.toLowerCase());
}

/**
 * Mirrors `fn function_words` in rust/src/summarization/vocabulary.rs: articles,
 * prepositions, copulas and coordinators dropped from terms and identifiers.
 * @returns {Array<string>}
 */
export function functionWords() {
  return cached('summarization-function-words', () => wordsFor(ROLE_STATEMENT_FUNCTION_WORD));
}

/**
 * Mirrors `fn negation_cues` in rust/src/summarization/vocabulary.rs.
 * @returns {Array<string>}
 */
export function negationCues() {
  return cached('summarization-negation-cues', () => wordsFor(ROLE_STATEMENT_NEGATION_CUE));
}

/**
 * Mirrors `fn reserved_words` in rust/src/summarization/vocabulary.rs.
 * @returns {Array<string>}
 */
export function reservedWords() {
  return cached('summarization-reserved-words', () => wordsFor(ROLE_IDENTIFIER_RESERVED_WORD));
}

/**
 * Mirrors `fn is_reserved_word` in rust/src/summarization/vocabulary.rs: an
 * exact (case-sensitive) match against the seed's reserved words.
 * @param {string} candidate
 */
export function isReservedWord(candidate) {
  return wordsForRole(ROLE_IDENTIFIER_RESERVED_WORD).some((word) => word === candidate);
}

/** Mirrors `fn flush` in rust/src/summarization/vocabulary.rs. */
function flush(buffer, out) {
  let start = 0;
  let end = buffer.length;
  while (start < end && buffer[start] === "'") start += 1;
  while (end > start && buffer[end - 1] === "'") end -= 1;
  const token = buffer.slice(start, end);
  if (token !== '') out.push(token);
}

/**
 * Mirrors `fn tokenize` in rust/src/summarization/vocabulary.rs: lowercased
 * alphanumeric runs, with the apostrophe kept inside a token and typographic
 * apostrophes normalized to ASCII.
 * @param {string} text
 * @returns {Array<string>}
 */
export function tokenize(text) {
  const out = [];
  let buffer = '';
  for (const character of text) {
    if (/^[\p{Alphabetic}\p{N}]$/u.test(character)) {
      buffer += character.toLowerCase();
    } else if (character === "'" || character === '’') {
      buffer += "'";
    } else {
      flush(buffer, out);
      buffer = '';
    }
  }
  flush(buffer, out);
  return out;
}

/**
 * Mirrors `fn cjk_words_longest_first` in rust/src/summarization/vocabulary.rs:
 * the CJK members of the vocabulary, longest first, ties in declaration order.
 */
function cjkWordsLongestFirst(vocabulary) {
  return vocabulary
    .filter((word) => containsCjk(word))
    .map((word, index) => ({ word, index, length: Array.from(word).length }))
    .sort((left, right) => (right.length - left.length) || (left.index - right.index))
    .map((entry) => entry.word);
}

/**
 * Mirrors `fn strip_words` in rust/src/summarization/vocabulary.rs: every token
 * equal to a vocabulary word removed; a CJK token loses vocabulary words as
 * substrings (longest first) and is dropped if nothing is left.
 * @param {Array<string>} tokens
 * @param {Array<string>} vocabulary
 * @returns {Array<string>}
 */
export function stripWords(tokens, vocabulary) {
  const cjk = cjkWordsLongestFirst(vocabulary);
  const out = [];
  for (const token of tokens) {
    if (vocabulary.some((word) => word === token)) continue;
    if (containsCjk(token)) {
      let stripped = token;
      for (const word of cjk) stripped = stripped.split(word).join('');
      if (stripped !== '') out.push(stripped);
      continue;
    }
    out.push(token);
  }
  return out;
}

/**
 * Mirrors `fn mentions_word` in rust/src/summarization/vocabulary.rs: does any
 * token carry a vocabulary word, whole or (for CJK) as a substring?
 * @param {Array<string>} tokens
 * @param {Array<string>} vocabulary
 */
export function mentionsWord(tokens, vocabulary) {
  const cjk = cjkWordsLongestFirst(vocabulary);
  return tokens.some((token) => vocabulary.some((word) => word === token)
    || (containsCjk(token) && cjk.some((word) => token.includes(word))));
}

/**
 * Mirrors `fn response_template` in rust/src/summarization/vocabulary.rs: the
 * seed response text for an intent and language slug.
 */
function responseTemplate(intent, language) {
  const found = multilingualResponses().find((record) => record.intent === intent && record.language === language);
  return found === undefined ? null : found.text;
}

/**
 * Mirrors `fn rendered_response` in rust/src/summarization/vocabulary.rs: a
 * seed response template for `language` (English when it has none, empty when
 * neither exists) with every placeholder replaced.
 * @param {string} intent
 * @param {string} language a language slug
 * @param {Array<[string, string]>} values `[placeholder, value]` pairs, applied in order
 */
export function renderedResponse(intent, language, values) {
  const template = responseTemplate(intent, language) ?? responseTemplate(intent, 'en') ?? '';
  return values.reduce((rendered, [placeholder, value]) => rendered.split(placeholder).join(value), template);
}
