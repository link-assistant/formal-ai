// Agentic recipe for issues #498 + #558: route the Google Trends learning
// frontier through the human-gated self-improvement loop
// (rust/src/agentic_coding/google_trends_learning.rs).
//
// The routing predicate is ported exactly, including the seed-declared
// learn-from-source directive (crate/seed_learning_sources.mjs). The report
// itself replays every catalog prompt through the engine
// (`crate::google_trends_learning::trending_learning_report`):
// native-only: rust/src/google_trends_learning.rs (an engine replay of the
// catalog). `renderDocument` returns the committed artifact
// data/meta/google-trends-learning.lino, pinned byte-for-byte to
// `render_document()` by the Rust test
// `committed_google_trends_learning_report_is_generated_by_the_recipe`; the
// final answer reads its counts from the fields `links_notation` writes.

import { readText } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { trimEnd } from './crate/rust_str.mjs';
import { learningSources, matchDirective } from './crate/seed_learning_sources.mjs';
import { findChildValue, parseRoot } from './crate/seed_parser.mjs';

/** Mirrors `GOOGLE_TRENDS_LEARNING_PATH` in rust/src/agentic_coding/google_trends_learning.rs. */
export const GOOGLE_TRENDS_LEARNING_PATH = 'google-trends-learning.lino';

/** Mirrors `GOOGLE_TRENDS_LEARNING_TASK` in rust/src/agentic_coding/google_trends_learning.rs. */
export function googleTrendsLearningTask() {
  return agenticMessage('google_trends_learning_task');
}

/** Mirrors `LEARNING_CAPABILITY` in rust/src/agentic_coding/google_trends_learning.rs. */
const LEARNING_CAPABILITY = 'google_trends_learning';

const GOOGLE_TRENDS_KEYWORDS = ['google trends', 'trending search'];

/**
 * Mirrors `fn is_google_trends_learning_task` in rust/src/agentic_coding/google_trends_learning.rs.
 * @param {string} prompt
 */
export function isGoogleTrendsLearningTask(prompt) {
  const lower = prompt.toLowerCase();
  if (isLearnFromSourceDirective(lower)) return true;
  const cannotResolve = lower.includes('resolve')
    && (lower.includes('cannot') || lower.includes("can't") || lower.includes('not yet'));
  return GOOGLE_TRENDS_KEYWORDS.some((keyword) => lower.includes(keyword))
    && (agenticMessage('google_trends_learning_frontier_cues').split('|').some((cue) => lower.includes(cue)) || cannotResolve);
}

/** Mirrors `fn is_learn_from_source_directive` in rust/src/agentic_coding/google_trends_learning.rs. */
function isLearnFromSourceDirective(lowercased) {
  return matchDirective(learningSources(), lowercased)?.capability === LEARNING_CAPABILITY;
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/google_trends_learning.rs (the committed artifact). */
export function renderDocument() {
  return readText(`data/meta/${GOOGLE_TRENDS_LEARNING_PATH}`);
}

/**
 * Mirrors `fn final_answer` in rust/src/agentic_coding/google_trends_learning.rs.
 * @param {string} document
 */
export function finalAnswer(document) {
  const report = parseRoot(renderDocument()).children[0];
  return agenticMessage('google_trends_learning_final_answer', {
    total: findChildValue(report, 'total_prompts'),
    handled: findChildValue(report, 'handled_by_engine'),
    frontier: findChildValue(report, 'learning_frontier'),
    proposals: findChildValue(report, 'learning_run_proposals'),
    adopted: findChildValue(report, 'learning_run_adopted'),
    path: GOOGLE_TRENDS_LEARNING_PATH,
    document: trimEnd(document),
  });
}

/** Mirrors `fn verification_command` in rust/src/agentic_coding/google_trends_learning.rs. */
export function verificationCommand() {
  return `python3 -c p='${GOOGLE_TRENDS_LEARNING_PATH}';s=open(p).read().splitlines();print(len(s));print('\\n'.join(s[:14]))`;
}
