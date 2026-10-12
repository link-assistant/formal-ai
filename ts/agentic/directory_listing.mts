// Recognise a prose request to list the files in the current place: a port of
// rust/src/agentic_coding/directory_listing.rs.

import { shellIntentVocabulary } from './crate/seed_shell_intents.mjs';
import { isAlphanumeric, splitWhitespace, trimEndMatches, trimMatches } from './crate/rust_str.mjs';
import { wordsForRole } from './crate/seed_meanings.mjs';
import { isProseWord } from './shell_command_policy.mjs';

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

// The directory a listing request names (PR #1188 T98, gap G29): "List the
// files in src." listed the workspace root. The operand is the path-shaped
// word right after a seeded place preposition (`statement_place_preposition`,
// "in src", "в src"), unless that word is prose or part of a seeded scope
// phrase ("in this directory", "in the workspace") or a seeded function word
// (`request_function_word`: the article of "en la carpeta actual" is no
// directory, PR #1188 CIFIX2); `.` otherwise.
const LISTED_SCOPE_ROLES = ['local_path_scope_current', 'local_path_scope_desktop', 'local_path_scope_home',
  'capability_container_scope', 'capability_workspace_scope'];
const FUNCTION_WORD_ROLE = 'request_function_word';
const PLACE_ROLE = 'statement_place_preposition';
const OPERAND_WRAPPERS = '`"\',;:!?()[]{}';
const CURRENT_DIRECTORY = '.';

/** A relative path a listing may name: no flag, no parent step, no URL (the operand characters admit no colon). */
function isDirectoryOperand(token) {
  return token !== '' && !token.startsWith('-')
    && !token.split('/').includes('..') && /^[A-Za-z0-9._/-]+$/u.test(token);
}

/**
 * Mirrors `fn listed_directory` in rust/src/agentic_coding/directory_listing.rs.
 * @param {string} task
 * @returns {string}
 */
export function listedDirectory(task) {
  const excluded = new Set([...LISTED_SCOPE_ROLES, FUNCTION_WORD_ROLE].flatMap((role) => wordsForRole(role))
    .flatMap((surface) => splitWhitespace(surface.toLowerCase())));
  const places = wordsForRole(PLACE_ROLE).map((surface) => surface.toLowerCase());
  const words = splitWhitespace(task).map((word) =>
    trimEndMatches(trimMatches(word, (character) => OPERAND_WRAPPERS.includes(character)), (character) => character === '.'));
  for (let index = 0; index + 1 < words.length; index += 1) {
    if (!places.includes(words[index].toLowerCase())) continue;
    const candidate = trimEndMatches(words[index + 1], (character) => character === '/');
    if (isDirectoryOperand(candidate) && !isProseWord(candidate) && !excluded.has(candidate.toLowerCase())) {
      return candidate;
    }
  }
  return CURRENT_DIRECTORY;
}
