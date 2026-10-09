// The language-independent meaning lexicon: a port of rust/src/seed/meanings.rs
// (`Lexicon`) and its loader rust/src/seed/meanings/parse.rs.
//
// The worker realm parses a differently ordered file set (it skips
// learned-request-openers.lino and sorts by name), and declaration order is
// priority for `first_role_match` / `words_for_role`, so this module parses
// `MEANING_FILES` itself, in the order rust/src/seed/embedded_registry.rs
// lists them. A meaning is `{slug, defined_by, roles, wikidata, lexemes:
// [{language, words: [{text, action}]}]}`.

import { cached, childrenNamed, parseLino, readText } from '../host.mjs';

/** The one inventory of the seed files and the lexicons that read them. */
const SEED_REGISTRY = 'data/meta/seed-registry.lino';

/** `MEANING_FILES`: every seed the registry gives the `meaning` lexicon, in the
 * Mirrors `const MEANING_FILES` in rust/src/seed/embedded_registry.rs.
 * registry's (name) order, as rust/src/seed/embedded_registry.rs lists them. */
export function meaningFiles() {
  return cached('meaning-files', () => childrenNamed(parseLino(readText(SEED_REGISTRY)), 'seed')
    .filter((seed) => childrenNamed(seed, 'lexicon').some((lexicon) => lexicon.id === 'meaning'))
    .map((seed) => `data/seed/${seed.id}.lino`));
}

const CANONICAL_TARGETS = new Map([
  ['reference_action', 'reference-action'], ['link_action', 'link-action'],
  ['any_of_reference', 'any-of-reference'], ['any_of_link', 'any-of-link'],
  ['repeatable_from_zero', 'repeatable-from-zero'], ['zero_or_more', 'zero-or-more'],
  ['point_at', 'point-at'], ['or_else', 'or-else'], ['is_identity', 'is-identity'],
  ['is_a_kind_of', 'is-a-kind-of'], ['held_by', 'held-by'], ['together_with', 'together-with'],
  ['self_equation', 'self-equation'], ['one_symbol_one_meaning', 'one-symbol-one-meaning'],
  ['sense_split', 'sense-split'], ['bank_river', 'bank-river'], ['bank_money', 'bank-money'],
]);

const findChildValue = (node, name) => (node.children || []).find((child) => child.name === name)?.value ?? '';

/** Mirrors `fn definition_targets` in rust/src/seed/meanings/parse.rs. */
function definitionTargets(raw) {
  return raw.split(/[\s()[\],]/u).filter(Boolean).map((target) => CANONICAL_TARGETS.get(target) ?? target);
}

/** Mirrors `decode_codepoints` in rust/src/seed/parser.rs. */
function decodeCodepoints(raw) {
  let out = '';
  for (const part of raw.split(/\s+/u).filter(Boolean)) {
    const hex = part.startsWith('0x') || part.startsWith('0X');
    const parsed = hex ? parseInt(part.slice(2), 16) : parseInt(part, 10);
    if (Number.isFinite(parsed) && parsed <= 0x10ffff && !(parsed >= 0xd800 && parsed <= 0xdfff)) {
      out += String.fromCodePoint(parsed);
    }
  }
  return out;
}

/** Mirrors `fn surface_text`. */
function surfaceText(node) {
  const text = findChildValue(node, 'text');
  if (text) return text;
  const codepoints = findChildValue(node, 'codepoints');
  return codepoints ? decodeCodepoints(codepoints) : node.value;
}

/** Mirrors `fn parse_word_form` (the fields the planner reads). */
function parseWordForm(node) {
  return { text: surfaceText(node), action: findChildValue(node, 'action') };
}

/** Mirrors `fn parse_meaning`. */
function parseMeaning(node) {
  const slug = node.name === 'meaning' ? node.value : node.name;
  const meaning = { slug, defined_by: [], roles: [], wikidata: '', lexemes: [] };
  if (node.name !== 'meaning' && node.value) meaning.defined_by.push(...definitionTargets(node.value));
  for (const child of node.children || []) {
    if (child.name === 'defined_by' || child.name === 'defined-by') {
      meaning.defined_by.push(...definitionTargets(child.value));
    } else if (child.name === 'grounded-in' || child.name === 'wikidata') {
      meaning.wikidata = child.value;
    } else if (child.name === 'role') {
      meaning.roles.push(child.value);
    } else if (child.name === 'lexeme') {
      const explicit = findChildValue(child, 'language');
      meaning.lexemes.push({
        language: explicit || child.value,
        words: (child.children || [])
          .filter((word) => word.name === 'word' || word.name === 'surface')
          .map(parseWordForm),
      });
    } else if (child.name === 'surface') {
      meaning.lexemes.push({ language: findChildValue(child, 'language'), words: [parseWordForm(child)] });
    }
  }
  return meaning;
}

/** Mirrors `fn parse_lexicon_text`: parse an explicit source without changing the cached lexicon. */
export function parseLexiconText(source) {
  const root = parseLino(source);
  const containers = (root.children || []).filter((child) => child.name === 'meanings');
  const sources = containers.length ? containers : [root];
  const meanings = [];
  for (const container of sources) {
    for (const node of container.children || []) {
      if (node.name === 'meaning' || node.name !== 'meanings') meanings.push(parseMeaning(node));
    }
  }
  return meanings;
}

/** Mirrors `fn lexicon` / `fn parse_lexicon`: every meaning in declaration order. */
export function lexicon() {
  return cached('meaning-lexicon', () => parseLexiconText(meaningFiles().map(readText).join('\n')));
}

/** Mirrors `crate::coding::contains_cjk`. @param {string} text */
export function containsCjk(text) {
  for (const character of text) {
    const cp = character.codePointAt(0);
    if ((cp >= 0x3400 && cp <= 0x4dbf) || (cp >= 0x4e00 && cp <= 0x9fff) || (cp >= 0xf900 && cp <= 0xfaff)
      || (cp >= 0x3040 && cp <= 0x30ff) || (cp >= 0x3100 && cp <= 0x312f)) return true;
  }
  return false;
}

/** Mirrors `fn surface_present` in rust/src/seed/meanings.rs. */
export function surfacePresent(normalized, expected) {
  if (!expected) return false;
  if (containsCjk(expected)) return normalized.includes(expected);
  return normalized === expected
    || normalized.startsWith(`${expected} `)
    || normalized.endsWith(` ${expected}`)
    || normalized.includes(` ${expected} `);
}

/** Mirrors `Meaning::has_role`. */
export const hasRole = (meaning, role) => meaning.roles.includes(role);

/** Mirrors `Meaning::words`. */
export function words(meaning) {
  return meaning.lexemes.flatMap((lexeme) => lexeme.words.map((word) => word.text));
}

/** Mirrors `Meaning::word_forms`. */
export function wordForms(meaning) {
  return meaning.lexemes.flatMap((lexeme) => lexeme.words);
}

/** Mirrors `Meaning::evidenced_in`. */
export function evidencedIn(meaning, normalized) {
  return words(meaning).some((word) => surfacePresent(normalized, word));
}

/** Mirrors `Meaning::word_in`. */
export function wordIn(meaning, language) {
  return meaning.lexemes.find((lexeme) => lexeme.language === language)?.words[0]?.text ?? null;
}

/** Mirrors `Meaning::mentions_in_languages_raw`. */
export function mentionsInLanguagesRaw(meaning, normalized, languages) {
  return meaning.lexemes
    .filter((lexeme) => languages.includes(lexeme.language))
    .some((lexeme) => lexeme.words.some((word) => word.text && normalized.includes(word.text)));
}

/** Mirrors `WordForm::slot`: 'bare' | 'prefix' | 'suffix' | 'circumfix'. */
export function slotOf(form) {
  const at = form.text.indexOf('…');
  if (at < 0) return 'bare';
  const before = at > 0;
  const after = at + 1 < form.text.length;
  if (before && after) return 'circumfix';
  if (before) return 'prefix';
  if (after) return 'suffix';
  return 'bare';
}

/** Mirrors `WordForm::before_slot`. */
export function beforeSlot(form) {
  const at = form.text.indexOf('…');
  return at < 0 ? form.text : form.text.slice(0, at);
}

/** Mirrors `WordForm::after_slot`. */
export function afterSlot(form) {
  const at = form.text.indexOf('…');
  return at < 0 ? '' : form.text.slice(at + 1);
}

/** Mirrors `Lexicon::meaning`. */
export function meaning(slug) {
  return lexicon().find((candidate) => candidate.slug === slug) ?? null;
}

/** Mirrors `Lexicon::meanings_with_role`. */
export function meaningsWithRole(role) {
  return lexicon().filter((candidate) => hasRole(candidate, role));
}

/** Mirrors `Lexicon::role_word_forms`. */
export function roleWordForms(role) {
  return meaningsWithRole(role).flatMap(wordForms);
}

/** Mirrors `Lexicon::words_for_role`. */
export function wordsForRole(role) {
  const out = [];
  for (const candidate of meaningsWithRole(role)) {
    for (const word of words(candidate)) if (!out.includes(word)) out.push(word);
  }
  return out;
}

/** Mirrors `Lexicon::words_for_role_in_languages`. */
export function wordsForRoleInLanguages(role, languages) {
  const out = [];
  for (const candidate of meaningsWithRole(role)) {
    for (const lexeme of candidate.lexemes) {
      if (!languages.includes(lexeme.language)) continue;
      for (const word of lexeme.words) if (!out.includes(word.text)) out.push(word.text);
    }
  }
  return out;
}

/** Mirrors `Lexicon::mentions_role`. */
export function mentionsRole(role, normalized) {
  return meaningsWithRole(role).some((candidate) => evidencedIn(candidate, normalized));
}

/** Mirrors `Lexicon::mentions_role_spelled`. */
export function mentionsRoleSpelled(role, normalized) {
  return meaningsWithRole(role).some((candidate) =>
    words(candidate).filter((word) => /\p{Alphabetic}/u.test(word)).some((word) => surfacePresent(normalized, word)));
}

/** Mirrors `Lexicon::mentions_role_raw`. */
export function mentionsRoleRaw(role, normalized) {
  return meaningsWithRole(role).some((candidate) => words(candidate).some((word) => normalized.includes(word)));
}

const PHRASAL_VERB_OBJECT_LIMIT = 6;

/** Mirrors `fn separated_surface_present`. */
function separatedSurfacePresent(tokens, surface) {
  const parts = surface.split(/\s+/u).filter(Boolean);
  if (parts.length !== 2) return false;
  const [verb, particle] = parts;
  return tokens.some((word, index) =>
    word === verb && tokens.slice(index + 2, index + 2 + PHRASAL_VERB_OBJECT_LIMIT).includes(particle));
}

/** Mirrors `Lexicon::mentions_role_separated`. */
export function mentionsRoleSeparated(role, normalized) {
  const tokens = normalized.split(/\s+/u).filter(Boolean);
  return meaningsWithRole(role)
    .filter((candidate) => candidate.defined_by.includes('action'))
    .flatMap(words)
    .some((surface) => separatedSurfacePresent(tokens, surface));
}

/** Mirrors `Lexicon::first_role_language`. */
export function firstRoleLanguage(role, normalized, priority) {
  return priority.find((language) => meaningsWithRole(role).some((candidate) =>
    candidate.lexemes
      .filter((lexeme) => lexeme.language === language)
      .some((lexeme) => lexeme.words.some((word) => normalized.includes(word.text))))) ?? null;
}

/** Mirrors `Lexicon::first_role_match`. */
export function firstRoleMatch(role, normalized) {
  return meaningsWithRole(role).find((candidate) => evidencedIn(candidate, normalized)) ?? null;
}

/** Mirrors `Lexicon::first_role_match_in_languages_raw`. */
export function firstRoleMatchInLanguagesRaw(role, normalized, languages) {
  return meaningsWithRole(role).find((candidate) => mentionsInLanguagesRaw(candidate, normalized, languages)) ?? null;
}

/** Mirrors `Lexicon::mentions_role_in_languages_raw`. */
export function mentionsRoleInLanguagesRaw(role, normalized, languages) {
  return meaningsWithRole(role).some((candidate) => mentionsInLanguagesRaw(candidate, normalized, languages));
}

function lexemeLists(candidate, language, surface, action = null) {
  return candidate.lexemes
    .filter((lexeme) => lexeme.language === language)
    .some((lexeme) => lexeme.words.some((word) => word.text === surface && (action === null || word.action === action)));
}

/** Mirrors `Lexicon::role_surface_translation`. */
export function roleSurfaceTranslation(role, source, target, surface) {
  const found = meaningsWithRole(role).find((candidate) => lexemeLists(candidate, source, surface));
  return found ? wordIn(found, target) : null;
}

/** Mirrors `Lexicon::role_lists_surface`. */
export function roleListsSurface(role, language, surface) {
  return meaningsWithRole(role).some((candidate) => lexemeLists(candidate, language, surface));
}

/** Mirrors `Lexicon::role_action_surface_translation`. */
export function roleActionSurfaceTranslation(role, action, source, target, surface) {
  const found = meaningsWithRole(role).find((candidate) => lexemeLists(candidate, source, surface, action));
  return found ? wordIn(found, target) : null;
}

/** Mirrors `Lexicon::meaning_by_wikidata`. */
export function meaningByWikidata(id) {
  return lexicon().find((candidate) => candidate.wikidata === id) ?? null;
}
