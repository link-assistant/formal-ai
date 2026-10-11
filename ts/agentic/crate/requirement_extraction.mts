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

import { cached } from '../host.mjs';
import { containsCjk, roleWordForms, surfacePresent } from './seed_meanings.mjs';

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
const headingLevel = (line) => /^(#{1,6})\s/u.exec(line)?.[1].length ?? 0;

/**
 * The candidate units of an issue body, in order. A unit is listed as
 * required by its structure when it is a task-list checkbox or sits under a
 * heading that names requirements.
 */
export function requirementUnits(text) {
  const units = [];
  let inFence = false;
  let sectionLevel = 0;
  for (const raw of text.split(/\r?\n/u)) {
    const line = raw.trim();
    if (FENCES.some((fence) => line.startsWith(fence))) {
      inFence = !inFence;
      continue;
    }
    if (inFence || line === '' || line.startsWith('>') || line.startsWith('|')) continue;
    const level = headingLevel(line);
    if (level > 0) {
      if (sectionLevel > 0 && level <= sectionLevel) sectionLevel = 0;
      if (sectionLevel === 0 && namesRequirementSection(line)) sectionLevel = level;
      continue;
    }
    const item = listItemOf(raw);
    if (item !== null) {
      if (item.text !== '') units.push({ text: item.text, structural: item.checkbox || sectionLevel > 0 });
      continue;
    }
    for (const sentence of sentencesOf(line)) units.push({ text: sentence, structural: sectionLevel > 0 });
  }
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

/** Whether the heading `line` names a section of requirements. */
export function namesRequirementSection(line) {
  const normalized = normalizeUnit(line);
  return wordsOfRole(SECTION_ROLE).some((word) => surfacePresent(normalized, word));
}

/** Whether the sentence or list item `unit` states a requirement by its words. */
export function isRequirement(unit) {
  return statesObligation(unit) || opensWithDirective(unit);
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
