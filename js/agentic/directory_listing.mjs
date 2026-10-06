// Recognise a prose request to list the files in the current place: a port of
// rust/src/agentic_coding/directory_listing.rs.

import { shellIntentVocabulary } from './crate/seed_shell_intents.mjs';
import { isAlphanumeric } from './crate/rust_str.mjs';

/** Mirrors `fn asks_for_directory_listing`. */
export function asksForDirectoryListing(prompt) {
  return composesAListingRequest(prompt.toLowerCase(), shellIntentVocabulary().directory_listing);
}

/** Mirrors `fn composes_a_listing_request`. */
function composesAListingRequest(prompt, vocabulary) {
  const mentionsAny = (parts) => parts.some((part) => mentions(prompt, part));
  return (mentionsAny(vocabulary.verbs) || mentionsAny(vocabulary.questions))
    && mentionsAny(vocabulary.objects)
    && mentionsAny(vocabulary.scopes);
}

/** Mirrors `const fn is_unspaced_script`. */
function isUnspacedScript(character) {
  const cp = character.codePointAt(0);
  return (cp >= 0x3400 && cp <= 0x9fff) || (cp >= 0xf900 && cp <= 0xfaff);
}

/** Mirrors `fn mentions`. */
function mentions(text, phrase) {
  if (!phrase) return false;
  if (Array.from(phrase).some(isUnspacedScript)) return text.includes(phrase);
  let searched = 0;
  for (;;) {
    const start = text.indexOf(phrase, searched);
    if (start < 0) return false;
    const end = start + phrase.length;
    const before = Array.from(text.slice(0, start)).pop();
    const after = Array.from(text.slice(end))[0];
    if (!(before !== undefined && isAlphanumeric(before)) && !(after !== undefined && isAlphanumeric(after))) return true;
    searched = start + Array.from(phrase)[0].length;
  }
}
