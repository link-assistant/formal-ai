// The `crate::seed::lexicon()` readers the write-side modules call
// (rust/src/seed/meanings.rs `Lexicon::role_word_forms`, `words_for_role`,
// `mentions_role`, `mentions_role_raw`, `WordForm::slot/before_slot/after_slot`),
// read through the booted worker realm's port of the same lexicon
// (js/worker/formal_ai_worker_13.js). Agent "write" helper; agent "core" owns
// the general crate/seed_meanings.mjs.

import { realm } from './host.mjs';

const SLOT_MARKER = '…';

/**
 * Mirrors `Lexicon::role_word_forms`: `{text, slot, before, after}` per form,
 * `slot` one of 'bare' | 'prefix' | 'suffix' | 'circumfix'.
 * @param {string} role
 */
export function roleWordForms(role) {
  return realm().roleWordForms(role).map((form) => wordForm(String(form.text)));
}

/** Mirrors `WordForm::slot`, `before_slot`, `after_slot` for `text`. */
export function wordForm(text) {
  const index = text.indexOf(SLOT_MARKER);
  if (index < 0) return { text, slot: 'bare', before: text, after: '' };
  const before = text.slice(0, index);
  const after = text.slice(index + SLOT_MARKER.length);
  const slot = before && after ? 'circumfix' : before ? 'prefix' : after ? 'suffix' : 'bare';
  return { text, slot, before, after };
}

/** Mirrors `Lexicon::words_for_role`. @param {string} role @returns {Array<string>} */
export function wordsForRole(role) {
  return Array.from(realm().wordsForRole(role), String);
}

/** Mirrors `Lexicon::mentions_role`. */
export function mentionsRole(role, normalized) {
  return Boolean(realm().lexiconMentionsRole(role, normalized));
}

/** Mirrors `Lexicon::mentions_role_raw` (`str::contains`, empty words included). */
export function mentionsRoleRaw(role, normalized) {
  return Array.from(realm().meaningsWithRole(role)).some((meaning) =>
    Array.from(meaning.words).some((word) => normalized.includes(String(word))));
}

/** Mirrors `lexicon().meaning(slug).map(Meaning::words)`: [] when absent. */
export function meaningWords(slug) {
  return Array.from(realm().wordsForMeaning(slug), String);
}

/** Mirrors `lexicon().meaning(slug).is_some_and(|m| m.evidenced_in(normalized))`. */
export function meaningEvidencedIn(slug, normalized) {
  const meaning = realm().findMeaning(slug);
  return Boolean(meaning) && Boolean(realm().meaningEvidencedIn(meaning, normalized));
}

/** Mirrors `Lexicon::words_for_role_in_languages`. */
export function wordsForRoleInLanguages(role, languages) {
  return Array.from(realm().wordsForRoleInLanguages(role, languages), String);
}
