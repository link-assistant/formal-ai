// Composing the document a request describes, instead of writing its
// description (issue #1066): rust/src/agentic_coding/note_composition.rs.

import { plainText, rustLines } from './content.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { eqIgnoreAsciiCase, splitWhitespace, trim, trimEndMatches, trimStartMatches } from './crate/rust_str.mjs';
import { mentionsRole, wordsForRole } from './crate/seed_meanings.mjs';
import { agenticMessage } from './messages.mjs';
import { finalAnswer } from './plan.mjs';
import { sentences } from './shell_command_policy.mjs';
import { firstContentLeadEnd } from './write_request.mjs';

const ROLE_DOCUMENT_COMPOSITION_ACTION = 'document_composition_action';
const ROLE_COMPOSED_DOCUMENT_KIND = 'composed_document_kind';
const ROLE_CLAUSE_CONTINUATION_MARKER = 'clause_continuation_marker';

/**
 * Mirrors `fn plan_note_composition_step` in rust/src/agentic_coding/note_composition.rs.
 * @param {string} task
 * @param {Array<object>} messages
 */
export function planNoteCompositionStep(task, messages) {
  const specification = parseSpecification(task);
  return specification === null ? null : finalAnswer(compose(specification, messages));
}

/**
 * Mirrors `fn composed_document_specification_span` in
 * rust/src/agentic_coding/note_composition.rs: the specifying sentence's span
 * exactly as `sentences` (./shell_command_policy.mjs) reports it (UTF-16
 * offsets), or null.
 * @param {string} task
 */
export function composedDocumentSpecificationSpan(task) {
  const specification = parseSpecification(task);
  return specification === null ? null : specification.span;
}

/** Mirrors `fn parse_specification`: `{request, span, parts}` or null. */
function parseSpecification(task) {
  for (const sentence of sentences(task)) {
    const normalized = normalizePrompt(sentence.text);
    if (!mentionsRole(ROLE_DOCUMENT_COMPOSITION_ACTION, normalized)
      || !mentionsRole(ROLE_COMPOSED_DOCUMENT_KIND, normalized)) continue;
    const parts = enumeratedParts(sentence.text);
    if (parts === null) continue;
    return { request: trim(sentence.text), span: sentence.span, parts };
  }
  return null;
}

/** Mirrors `fn enumerated_parts`. */
function enumeratedParts(sentence) {
  const lead = firstContentLeadEnd(sentence.toLowerCase());
  if (lead === null || lead === undefined) return null;
  const end = lead[1];
  if (end > sentence.length) return null;
  const separators = wordsForRole(ROLE_CLAUSE_CONTINUATION_MARKER);
  const parts = sentence.slice(end)
    .split(',')
    .flatMap((span) => splitOnSeparators(span, separators))
    .map(trimmedPart)
    .filter((part) => part !== '');
  return parts.length >= 2 ? parts : null;
}

/** Mirrors `fn trimmed_part`. */
function trimmedPart(part) {
  const head = trimStartMatches(trim(part), (character) => [':', '-', '—', '–'].includes(character));
  return trim(trimEndMatches(head, (character) => character === '.'));
}

/** Mirrors `fn split_on_separators`. */
function splitOnSeparators(span, separators) {
  const parts = [''];
  for (const word of splitWhitespace(span)) {
    if (separators.some((separator) => eqIgnoreAsciiCase(word, separator))) {
      parts.push('');
      continue;
    }
    const current = parts[parts.length - 1];
    parts[parts.length - 1] = current ? `${current} ${word}` : word;
  }
  return parts;
}

/** Mirrors `fn compose`. */
function compose(specification, messages) {
  const observed = observations(messages);
  let note = `${specification.request}\n\n${agenticMessage('note_composition_requested_parts')}\n`;
  for (const part of specification.parts) note += `- ${part}\n`;
  note += `\n${agenticMessage('note_composition_observed_heading')}\n`;
  if (!observed.length) note += `- ${agenticMessage('note_composition_nothing_observed')}\n`;
  else for (const observation of observed) note += `- ${observation}\n`;
  if (!observed.length) note += `\n${agenticMessage('note_composition_unbacked')}\n`;
  return note;
}

/** Mirrors `fn observations`. */
function observations(messages) {
  let turn = 0;
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (messages[index].role.toLowerCase() === 'user') {
      turn = index + 1;
      break;
    }
  }
  return messages.slice(turn)
    .filter((message) => message.role.toLowerCase() === 'tool')
    .map((message) => {
      const name = message.name ?? 'tool';
      const head = rustLines(plainText(message.content)).find((line) => trim(line) !== '') ?? '';
      return `${name}: ${trim(head)}`;
    });
}
