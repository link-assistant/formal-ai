// Agentic recipe for issue #498: turn Google Trends into multilingual test
// prompts (rust/src/agentic_coding/google_trends_catalog.rs).
//
// The routing predicate is ported exactly. The catalog itself answers every
// generated prompt through the full Formal AI engine
// (`crate::google_trends_catalog::google_trends_catalog`):
// native-only: rust/src/google_trends_catalog.rs (the engine answers 80
// prompts). `renderDocument` returns the committed artifact
// data/meta/google-trends-catalog.lino, which the Rust test
// `committed_google_trends_catalog_is_generated_by_the_recipe` pins
// byte-for-byte to `render_document()`; the final answer reads its counts
// from that same document (the fields `render_catalog` writes).

import { readText } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { trimEnd } from './crate/rust_str.mjs';
import { findChildValue, parseRoot } from './crate/seed_parser.mjs';

/** Mirrors `GOOGLE_TRENDS_CATALOG_PATH` in rust/src/agentic_coding/google_trends_catalog.rs. */
export const GOOGLE_TRENDS_CATALOG_PATH = 'google-trends-catalog.lino';

/** Mirrors `GOOGLE_TRENDS_CATALOG_TASK` in rust/src/agentic_coding/google_trends_catalog.rs. */
export function googleTrendsCatalogTask() {
  return agenticMessage('google_trends_catalog_task');
}

const GOOGLE_TRENDS_KEYWORDS = ['google trends', 'trending searches', 'top searches', 'trends catalog'];

/**
 * Mirrors `fn is_google_trends_catalog_task` in rust/src/agentic_coding/google_trends_catalog.rs.
 * @param {string} prompt
 */
export function isGoogleTrendsCatalogTask(prompt) {
  const lower = prompt.toLowerCase();
  return GOOGLE_TRENDS_KEYWORDS.some((keyword) => lower.includes(keyword))
    && (lower.includes('prompt') || lower.includes('answer') || lower.includes('catalog') || lower.includes('test'));
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/google_trends_catalog.rs (the committed artifact). */
export function renderDocument() {
  return readText(`data/meta/${GOOGLE_TRENDS_CATALOG_PATH}`);
}

/**
 * Mirrors `fn final_answer` in rust/src/agentic_coding/google_trends_catalog.rs.
 * @param {string} document
 */
export function finalAnswer(document) {
  const catalog = parseRoot(renderDocument()).children[0];
  return agenticMessage('google_trends_catalog_final_answer', {
    topic_count: findChildValue(catalog, 'topic_count'),
    geo: findChildValue(catalog, 'geo'),
    prompt_count: findChildValue(catalog, 'prompt_count'),
    answer_count: findChildValue(catalog, 'answered_count'),
    path: GOOGLE_TRENDS_CATALOG_PATH,
    document: trimEnd(document),
  });
}

/** Mirrors `fn verification_command` in rust/src/agentic_coding/google_trends_catalog.rs. */
export function verificationCommand() {
  return `python3 -c p='${GOOGLE_TRENDS_CATALOG_PATH}';s=open(p).read().splitlines();print(len(s));print('\\n'.join(s[:12]))`;
}
