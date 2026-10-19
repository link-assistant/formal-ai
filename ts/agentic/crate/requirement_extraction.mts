// Requirement extraction (R1188-U20): the exact requirements an issue states.
//
// An issue body is split into units: list items, and the sentences of every
// other prose line. Fenced code, block quotes, tables and headings are not
// requirements. A unit is a requirement when it carries a seeded obligation
// cue (`must`, `нужно`, `必须`, …), opens with a seeded directive verb (`Add`,
// `Исправь`, `添加`, …), is a task-list checkbox, or sits under a heading that
// names requirements (`Acceptance criteria`, `Требования`, …). A unit whose content words another kept unit
// already holds restates it and is dropped. The vocabulary lives in
// data/seed/meanings-requirement-extraction.lino; nothing here names a
// language.

import { cached, childrenNamed, parseLino, readText } from '../host.mjs';
import { containsCjk, meaningsWithRole, roleWordForms, surfacePresent } from './seed_meanings.mjs';
import { quoteFault, quotedSegmentSpans } from './normal_markov.mjs';

/** The role whose words mark a unit as stating an obligation. */
const OBLIGATION_ROLE = 'requirement_obligation_cue';

/** The role whose words, opening a unit, mark it as an instruction. */
const DIRECTIVE_ROLE = 'requirement_directive_verb';

/** The role whose words, in a heading, mark the section below as requirements. */
const SECTION_ROLE = 'requirement_section_heading';

/** The role of words that carry no content of their own. */
const FUNCTION_WORD_ROLE = 'statement_function_word';

/** A content word has at least this many letters. */
const CONTENT_WORD_LENGTH = 3;

/** Words are compared by this many leading letters, so inflections meet. */
const STEM_LENGTH = 6;

/** Two units whose content words overlap this much say the same thing. */
const RESTATEMENT_OVERLAP = 0.8;

/** The Markdown fences that open and close a code block. */
const FENCES = ['```', '~~~'];

/** The lowercase words of `text`, punctuation removed, joined by single spaces. */
export function normalizeUnit(text) {
  return text.toLowerCase().replace(/[^\p{L}\p{M}\p{N}\s'-]+/gu, ' ').replace(/\s+/gu, ' ').trim();
}

/** The list item of `line`: its text and whether it is a task-list checkbox, or null. */
export function listItemOf(line) {
  const marker = /^\s*(?:[-*+•]|\d+[.)])\s+(\[[ xX]\]\s+)?/u.exec(line);
  return marker ? { text: line.slice(marker[0].length).trim(), checkbox: marker[1] !== undefined } : null;
}

/** The sentences of a prose line: Latin stops end one before a space, full-width and danda stops always. */
export function sentencesOf(line) {
  return line.split(/(?<=[.!?])\s+|(?<=[。！？।])\s*/u).map((sentence) => sentence.trim()).filter(Boolean);
}

/** The heading level of `line` (`#` = 1), or 0 when it is no heading. */
const headingLevel = (line) => {
  const atx = /^(#{1,6})\s/u.exec(line)?.[1].length ?? 0;
  if (atx) return atx;
  for (const marker of ['**', '__']) {
    if (!line.startsWith(marker) || !line.endsWith(marker)) continue;
    const title = line.slice(marker.length, -marker.length).trim();
    if (title && !/[.!?。！？।]$/u.test(title) && !isRequirement(title)) return 2;
  }
  return 0;
};

/**
 * The candidate units of an issue body, in order. A unit is listed as
 * required by its structure when it is a task-list checkbox or sits under a
 * heading that names requirements.
 */
export function requirementUnits(text) {
  const units = [];
  let inFence = false, sectionLevel = 0, pending = null;
  const flush = () => {
    if (pending === null) return;
    const texts = pending.item ? [pending.text] : sentencesOf(pending.text);
    for (const text of texts) if (text) units.push({text, structural: pending.structural});
    pending = null;
  };
  for (const raw of text.split(/\r?\n/u)) {
    const line = raw.trim();
    if (FENCES.some((fence) => line.startsWith(fence))) {
      flush(); inFence = !inFence; continue;
    }
    if (inFence || line === '' || line.startsWith('>') || line.startsWith('|')) {
      flush(); continue;
    }
    const level = headingLevel(line);
    if (level > 0) {
      flush();
      if (sectionLevel > 0 && level <= sectionLevel) sectionLevel = 0;
      if (sectionLevel === 0 && namesRequirementSection(line)) sectionLevel = level;
      continue;
    }
    const item = listItemOf(raw);
    if (item !== null) {
      flush(); pending = {text: item.text, item: true, structural: item.checkbox || sectionLevel > 0};
    } else if (pending !== null) {
      pending.text += ' ' + line;
    } else {
      pending = {text: line, item: false, structural: sectionLevel > 0};
    }
    if (raw.endsWith('  ') || raw.endsWith('\\')) flush();
  }
  flush();
  return units;
}

/** The normalized, distinct words of `role`, read once. */
const wordsOfRole = (role) => cached(`requirement-words-${role}`, () =>
  [...new Set(roleWordForms(role).map((form) => normalizeUnit(form.text)).filter(Boolean))]);

/** Whether `unit` carries an obligation cue anywhere in it. */
export function statesObligation(unit) {
  const normalized = normalizeUnit(unit);
  return wordsOfRole(OBLIGATION_ROLE).some((cue) => surfacePresent(normalized, cue));
}

/** Whether `unit` opens with a directive verb. */
export function opensWithDirective(unit) {
  const normalized = normalizeUnit(unit);
  return wordsOfRole(DIRECTIVE_ROLE).some((verb) =>
    (containsCjk(verb) ? normalized.startsWith(verb) : normalized === verb || normalized.startsWith(`${verb} `)));
}

/** Directive positions are grammar metadata; lexemes remain the existing role vocabulary. */
function terminalDirectiveWords() {
  return cached('requirement-terminal-directives', () => {
    const root = parseLino(readText('data/seed/meanings-requirement-extraction.lino'));
    const containers = childrenNamed(root, 'meanings');
    const nodes = (containers.length ? containers : [root]).flatMap((container) => container.children || []);
    const languages = new Set(nodes
      .filter((node) => childrenNamed(node, 'role').some((role) => role.value === DIRECTIVE_ROLE))
      .flatMap((node) => childrenNamed(node, 'directive-position'))
      .filter((position) => position.value === 'terminal')
      .flatMap((position) => childrenNamed(position, 'language').map((language) => language.value)));
    return meaningsWithRole(DIRECTIVE_ROLE).flatMap((meaning) => meaning.lexemes)
      .filter((lexeme) => languages.has(lexeme.language))
      .flatMap((lexeme) => lexeme.words.map((word) => normalizeUnit(word.text))).filter(Boolean);
  });
}

/** Only unquoted terminal directives are new evidence; preserve the original unit verbatim. */
export function endsWithDirective(unit) {
  if (quoteFault(unit) !== null) return false;
  let instruction = unit;
  for (const span of quotedSegmentSpans(unit).slice().reverse()) {
    instruction = instruction.slice(0, span.start) + ' '.repeat(span.end - span.start) + instruction.slice(span.end);
  }
  const normalized = normalizeUnit(instruction);
  if (wordsOfRole('source_attribution_marker').some((word) => surfacePresent(normalized, word))) return false;
  return terminalDirectiveWords().some((verb) => normalized === verb || normalized.endsWith(` ${verb}`));
}

/** Whether the heading `line` names a section of requirements. */
export function namesRequirementSection(line) {
  const normalized = normalizeUnit(line);
  return wordsOfRole(SECTION_ROLE).some((word) => surfacePresent(normalized, word));
}

/** Whether the sentence or list item `unit` states a requirement by its words. */
export function isRequirement(unit) {
  return statesObligation(unit) || opensWithDirective(unit) || endsWithDirective(unit);
}

/** The content words of `unit` (CJK text counts each character). */
export function contentWords(unit) {
  const normalized = normalizeUnit(unit);
  if (containsCjk(normalized)) return new Set(Array.from(normalized.replace(/\s+/gu, '')));
  const functionWords = new Set(wordsOfRole(FUNCTION_WORD_ROLE));
  return new Set(normalized.split(' ')
    .filter((word) => Array.from(word).length >= CONTENT_WORD_LENGTH && !functionWords.has(word))
    .map((word) => Array.from(word).slice(0, STEM_LENGTH).join('')));
}

/** The share of the smaller word set that the other one also holds. */
export function overlap(left, right) {
  const smaller = Math.min(left.size, right.size);
  if (smaller === 0) return 0;
  let shared = 0;
  for (const word of left) if (right.has(word)) shared += 1;
  return shared / smaller;
}

/** The requirements `text` states, each once, in order. */
export function extractRequirements(text) {
  const kept = [];
  for (const unit of requirementUnits(text)) {
    if (!unit.structural && !isRequirement(unit.text)) continue;
    const words = contentWords(unit.text);
    if (words.size === 0 || kept.some((earlier) => overlap(earlier.words, words) >= RESTATEMENT_OVERLAP)) continue;
    kept.push({ text: unit.text, words });
  }
  return kept.map((requirement) => requirement.text);
}
