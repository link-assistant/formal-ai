// Requirement extraction in the browser worker (R1188-U20).
//
// The twin of js/agentic/crate/requirement_extraction.mjs (which a classic
// worker cannot import) and of rust/src/agentic_coding/requirement_extraction.rs.
// An issue body is split into units: list items, and the sentences of every
// other prose line. Fenced code, block quotes, tables and headings are not
// requirements. A unit is a requirement when it carries a seeded obligation
// cue, opens with a seeded directive verb, is a task-list checkbox, or sits
// under a heading that names requirements. A unit whose content words an
// earlier kept unit already holds restates it and is dropped. The vocabulary
// is data/seed/meanings-requirement-extraction.lino, read through the worker
// lexicon (formal_ai_worker_meaning_lexicon.js); nothing here names a language.
// rust/tests/web/requirement-listing-route.test.mjs holds this twin equal to
// the crate module over the whole benchmark corpus.

/** The roles whose words mark a requirement, and the closed-class words. */
const REQUIREMENT_EXTRACTION_ROLES = Object.freeze({
  obligation: "requirement_obligation_cue",
  directive: "requirement_directive_verb",
  section: "requirement_section_heading",
  functionWord: "statement_function_word",
});

/** A content word has at least this many letters. */
const REQUIREMENT_CONTENT_WORD_LENGTH = 3;

/** Words are compared by this many leading letters, so inflections meet. */
const REQUIREMENT_STEM_LENGTH = 6;

/** Two units whose content words overlap this much say the same thing. */
const REQUIREMENT_RESTATEMENT_OVERLAP = 0.8;

/** The Markdown fences that open and close a code block. */
const REQUIREMENT_FENCES = Object.freeze(["```", "~~~"]);

const requirementRoleWordCache = new Map();

/**
 * The lowercase words of `text`, punctuation removed, joined by single spaces.
 * @param {string} text
 * @returns {string}
 */
function requirementNormalizeUnit(text) {
  return String(text)
    .toLowerCase()
    .replace(/[^\p{L}\p{M}\p{N}\s'-]+/gu, " ")
    .replace(/\s+/gu, " ")
    .trim();
}

/**
 * The list item of `line`: its text and whether it is a task-list checkbox.
 * @param {string} line
 * @returns {{text: string, checkbox: boolean}|null}
 */
function requirementListItemOf(line) {
  const marker = /^\s*(?:[-*+•]|\d+[.)])\s+(\[[ xX]\]\s+)?/u.exec(line);
  if (!marker) return null;
  return {
    text: line.slice(marker[0].length).trim(),
    checkbox: marker[1] !== undefined,
  };
}

/**
 * The sentences of a prose line: Latin stops end one before a space,
 * full-width and danda stops always.
 * @param {string} line
 * @returns {string[]}
 */
function requirementSentencesOf(line) {
  return line
    .split(/(?<=[.!?])\s+|(?<=[。！？।])\s*/u)
    .map((sentence) => sentence.trim())
    .filter(Boolean);
}

/**
 * The heading level of `line` (`#` = 1), or 0 when it is no heading.
 * @param {string} line
 * @returns {number}
 */
function requirementHeadingLevel(line) {
  const heading = /^(#{1,6})\s/u.exec(line);
  return heading ? heading[1].length : 0;
}

/**
 * The normalized, distinct words of `role`, read once.
 * @param {string} role
 * @returns {string[]}
 */
function requirementRoleWords(role) {
  if (!requirementRoleWordCache.has(role)) {
    const words = meaningsWithRole(role)
      .flatMap((meaning) => meaning.words)
      .map(requirementNormalizeUnit)
      .filter(Boolean);
    requirementRoleWordCache.set(role, [...new Set(words)]);
  }
  return requirementRoleWordCache.get(role);
}

/**
 * Whether the heading `line` names a section of requirements.
 * @param {string} line
 * @returns {boolean}
 */
function requirementNamesSection(line) {
  const normalized = requirementNormalizeUnit(line);
  return requirementRoleWords(REQUIREMENT_EXTRACTION_ROLES.section)
    .some((word) => surfacePresent(normalized, word));
}

/**
 * The candidate units of an issue body, in order; `structural` marks a unit
 * listed as required by a checkbox or a requirements heading.
 * @param {string} text
 * @returns {{text: string, structural: boolean}[]}
 */
function requirementUnits(text) {
  const units = [];
  let inFence = false;
  let sectionLevel = 0;
  for (const raw of String(text).split(/\r?\n/u)) {
    const line = raw.trim();
    if (REQUIREMENT_FENCES.some((fence) => line.startsWith(fence))) {
      inFence = !inFence;
      continue;
    }
    if (inFence || line === "" || line.startsWith(">") || line.startsWith("|")) continue;
    const level = requirementHeadingLevel(line);
    if (level > 0) {
      if (sectionLevel > 0 && level <= sectionLevel) sectionLevel = 0;
      if (sectionLevel === 0 && requirementNamesSection(line)) sectionLevel = level;
      continue;
    }
    const item = requirementListItemOf(raw);
    if (item !== null) {
      if (item.text !== "") {
        units.push({ text: item.text, structural: item.checkbox || sectionLevel > 0 });
      }
      continue;
    }
    for (const sentence of requirementSentencesOf(line)) {
      units.push({ text: sentence, structural: sectionLevel > 0 });
    }
  }
  return units;
}

/**
 * Whether the sentence or list item `unit` states a requirement by its words:
 * an obligation cue anywhere, or a directive verb opening it.
 * @param {string} unit
 * @returns {boolean}
 */
function requirementStatedByWords(unit) {
  const normalized = requirementNormalizeUnit(unit);
  const obligation = requirementRoleWords(REQUIREMENT_EXTRACTION_ROLES.obligation)
    .some((cue) => surfacePresent(normalized, cue));
  if (obligation) return true;
  return requirementRoleWords(REQUIREMENT_EXTRACTION_ROLES.directive).some((verb) => {
    if (containsCjk(verb)) return normalized.startsWith(verb);
    return normalized === verb || normalized.startsWith(`${verb} `);
  });
}

/**
 * The content words of `unit` (CJK text counts each character).
 * @param {string} unit
 * @returns {Set<string>}
 */
function requirementContentWords(unit) {
  const normalized = requirementNormalizeUnit(unit);
  if (containsCjk(normalized)) return new Set(Array.from(normalized.replace(/\s+/gu, "")));
  const functionWords = new Set(requirementRoleWords(REQUIREMENT_EXTRACTION_ROLES.functionWord));
  return new Set(
    normalized
      .split(" ")
      .filter((word) => Array.from(word).length >= REQUIREMENT_CONTENT_WORD_LENGTH)
      .filter((word) => !functionWords.has(word))
      .map((word) => Array.from(word).slice(0, REQUIREMENT_STEM_LENGTH).join("")),
  );
}

/**
 * The share of the smaller word set that the other one also holds.
 * @param {Set<string>} left
 * @param {Set<string>} right
 * @returns {number}
 */
function requirementOverlap(left, right) {
  const smaller = Math.min(left.size, right.size);
  if (smaller === 0) return 0;
  let shared = 0;
  for (const word of left) if (right.has(word)) shared += 1;
  return shared / smaller;
}

/**
 * The requirements `text` states, each once, in order.
 * @param {string} text
 * @returns {string[]}
 */
function extractRequirements(text) {
  const kept = [];
  for (const unit of requirementUnits(text)) {
    if (!unit.structural && !requirementStatedByWords(unit.text)) continue;
    const words = requirementContentWords(unit.text);
    if (words.size === 0) continue;
    const restated = kept.some((earlier) => {
      return requirementOverlap(earlier.words, words) >= REQUIREMENT_RESTATEMENT_OVERLAP;
    });
    if (restated) continue;
    kept.push({ text: unit.text, words });
  }
  return kept.map((requirement) => requirement.text);
}
