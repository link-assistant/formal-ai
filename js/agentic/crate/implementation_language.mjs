// The implementation-language modifier, `"in <language>"`
// (rust/src/implementation_language.rs: `is_known`, `requested`,
// `requested_in`, `without_modifier`, `without_trailing_known_modifier`,
// `modifier_span`), with `crate::knowledge::CodingOracle::knows_language`
// (rust/src/knowledge.rs) answered by the worker realm's twin
// `codingOracleKnowsLanguage` behind the same seed `bootstrap` gate.

import { cached, childValue, childrenNamed, parseLino, readText, realm } from '../host.mjs';
import { normalizePrompt } from './engine.mjs';
import { programLanguageByAlias, programLanguageBySlug } from './coding_catalog.mjs';
import { wordsForRole, wordsForRoleInLanguages } from '../write_lexicon.mjs';
import { charIn, splitWhitespace, trim, trimEndMatches } from '../write_str.mjs';

const prepositionSurfaces = () => cached('implementation-language-prepositions', () =>
  headInitialSurfaces('implementation_language_preposition'));
const languageNounSurfaces = () => cached('implementation-language-nouns', () =>
  headInitialSurfaces('implementation_language_noun'));
const functionWords = () => cached('statement-function-words', () =>
  wordsForRole('statement_function_word').map((word) => word.toLowerCase()));

function headInitialSurfaces(role) {
  return wordsForRoleInLanguages(role, ['en', 'ru']).map((word) => word.toLowerCase());
}

const isPreposition = (token) => prepositionSurfaces().includes(token);
const isLanguageNoun = (token) => languageNounSurfaces().includes(token);
const isFunctionWord = (token) => functionWords().includes(token);

/** Mirrors `fn bootstrap_cache_active` in rust/src/knowledge.rs. */
function bootstrapCacheActive() {
  return cached('program-cache-bootstrap', () => {
    const root = parseLino(readText('data/seed/program-cache-policy.lino'));
    const policy = root.children[0];
    const bootstrap = childrenNamed(policy, 'bootstrap')[0];
    return Boolean(bootstrap) && childValue(bootstrap, 'active') === 'true';
  });
}

/** Mirrors `CodingOracle::knows_language`. */
export function oracleKnowsLanguage(language) {
  return bootstrapCacheActive() && Boolean(realm().codingOracleKnowsLanguage(language));
}

/** Mirrors `fn is_known`. */
export function isKnown(language) {
  return programLanguageBySlug(language) !== null || oracleKnowsLanguage(language);
}

const couldNameALanguage = (token) => /\p{Alphabetic}/u.test(token);

/** Mirrors `fn modifier_span`: `{start, end, name}` or null. */
function modifierSpan(tokens) {
  for (let start = 0; start < tokens.length; start += 1) {
    if (!isPreposition(tokens[start])) continue;
    let cursor = start + 1;
    let namedByNoun = false;
    let skippedFunctionWord = false;
    while (cursor < tokens.length) {
      const next = tokens[cursor];
      if (isLanguageNoun(next)) {
        namedByNoun = true;
        cursor += 1;
        continue;
      }
      if (isFunctionWord(next) && !isKnown(next)) {
        skippedFunctionWord = true;
        cursor += 1;
        continue;
      }
      break;
    }
    const candidate = tokens[cursor];
    if (candidate === undefined) continue;
    let end = cursor + 1;
    if (end < tokens.length && isLanguageNoun(tokens[end])) {
      namedByNoun = true;
      end += 1;
    }
    if (!couldNameALanguage(candidate)) continue;
    const phraseFinal = tokens.slice(end).every(isFunctionWord);
    const accepted = isKnown(candidate) || namedByNoun || (!skippedFunctionWord && phraseFinal);
    if (!accepted) continue;
    return { start, end, name: candidate };
  }
  return null;
}

/** Mirrors `fn requested`. @param {string} normalized @returns {string|null} */
export function requested(normalized) {
  const tokens = splitWhitespace(normalized);
  const span = modifierSpan(tokens);
  if (span && isKnown(span.name)) {
    const target = programLanguageByAlias(normalized);
    if (target && target.framework_of === span.name) return target.slug;
    return span.name;
  }
  const language = programLanguageByAlias(normalized);
  if (language) return language.slug;
  return span ? span.name : null;
}

/** Mirrors `fn requested_in`. */
export function requestedIn(text) {
  return requested(normalizePrompt(text));
}

function normalizedWords(text) {
  const words = splitWhitespace(text);
  return [words, words.map((word) => normalizePrompt(word))];
}

/** Mirrors `fn without_modifier`. @returns {string|null} */
export function withoutModifier(text) {
  const [words, tokens] = normalizedWords(text);
  const span = modifierSpan(tokens);
  if (!span) return null;
  const kept = words.filter((_, index) => index < span.start || index >= span.end);
  return kept.length ? kept.join(' ') : null;
}

/** Mirrors `fn without_trailing_known_modifier`. @returns {string|null} */
export function withoutTrailingKnownModifier(text) {
  const [words, tokens] = normalizedWords(text);
  const span = modifierSpan(tokens);
  if (!span || span.end !== tokens.length || !isKnown(span.name)) return null;
  const kept = trim(trimEndMatches(trim(words.slice(0, span.start).join(' ')), charIn(',;:-—')));
  return kept ? kept : null;
}
