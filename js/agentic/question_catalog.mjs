// Eleventh agentic recipe - generate every possible question and answer it
// (issue #527; rust/src/agentic_coding/question_catalog.rs).
//
// The routing predicate is ported exactly. The catalog enumerates questions
// through `crate::question_generation` and answers the meaningful ones with
// the deterministic engine:
// native-only: rust/src/question_generation.rs + crate::engine (the
// enumeration and engine answers). `renderDocument` returns the committed
// artifact data/meta/question-catalog.lino, pinned byte-for-byte to
// `render_document()` by the Rust test
// `committed_catalog_document_is_generated_by_the_recipe`; the final answer
// and `catalog()` read the same document.

import { readText } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { splitWhitespace, toAsciiLowercase, trimEnd } from './crate/rust_str.mjs';
import { findChildValue, parseRoot } from './crate/seed_parser.mjs';

/** Mirrors `QUESTION_CATALOG_PATH` in rust/src/agentic_coding/question_catalog.rs. */
export const QUESTION_CATALOG_PATH = 'question-catalog.lino';

/** Mirrors `QUESTION_CATALOG_TASK` in rust/src/agentic_coding/question_catalog.rs. */
export function questionCatalogTask() {
  return agenticMessage('question_catalog_task');
}

/** Mirrors `QUESTION_CATALOG_KEYWORDS` (data/meta/agentic-messages.lino `question_catalog_keywords`). */
const questionCatalogKeywords = () => agenticMessage('question_catalog_keywords').split('|');

/**
 * Mirrors `fn is_question_catalog_task` in rust/src/agentic_coding/question_catalog.rs.
 * @param {string} prompt
 */
export function isQuestionCatalogTask(prompt) {
  const lower = prompt.toLowerCase();
  return questionCatalogKeywords().some((keyword) => lower.includes(keyword))
    || ((lower.includes('generate') || lower.includes('enumerate')) && lower.includes('question') && lower.includes('answer'));
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/question_catalog.rs (the committed artifact). */
export function renderDocument() {
  return readText(`data/meta/${QUESTION_CATALOG_PATH}`);
}

/**
 * Mirrors `fn catalog` in rust/src/agentic_coding/question_catalog.rs, read
 * back from the rendered document: `{vocabulary_size, candidates, answered}`.
 */
export function catalog() {
  const root = parseRoot(renderDocument()).children[0];
  const children = root.children || [];
  return {
    vocabulary_size: Number(findChildValue(root, 'vocabulary_size')),
    candidates: children.filter((node) => node.name === 'candidate').map((node) => ({
      text: findChildValue(node, 'text'),
      word_count: Number(findChildValue(node, 'word_count')),
      grammar: findChildValue(node, 'grammar'),
      logical_meaning: findChildValue(node, 'logical_meaning'),
      class: findChildValue(node, 'class'),
    })),
    answered: children.filter((node) => node.name === 'answered').map((node) => ({
      question: findChildValue(node, 'question'),
      intent: findChildValue(node, 'intent'),
      confidence: Number(findChildValue(node, 'confidence')),
      answer: findChildValue(node, 'answer'),
    })),
  };
}

/** Mirrors `fn normalise_question` in rust/src/agentic_coding/question_catalog.rs. */
const normaliseQuestion = (question) => toAsciiLowercase(splitWhitespace(question).join(' '));

/**
 * Mirrors `QuestionCatalog::answer_for`: the recalled answer or null.
 * @param {ReturnType<typeof catalog>} questionCatalog
 * @param {string} question
 */
export function answerFor(questionCatalog, question) {
  const needle = normaliseQuestion(question);
  return questionCatalog.answered.find((answered) => normaliseQuestion(answered.question) === needle) ?? null;
}

/**
 * Mirrors `fn final_answer` in rust/src/agentic_coding/question_catalog.rs.
 * @param {string} document
 */
export function finalAnswer(document) {
  const root = parseRoot(renderDocument()).children[0];
  return agenticMessage('question_catalog_final_answer', {
    vocabulary: findChildValue(root, 'vocabulary_size'),
    candidates: findChildValue(root, 'candidate_count'),
    answers: findChildValue(root, 'answered_count'),
    path: QUESTION_CATALOG_PATH,
    document: trimEnd(document),
  });
}
