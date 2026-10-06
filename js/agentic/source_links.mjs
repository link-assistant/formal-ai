// The issue-#558 source-links recipe (rust/src/agentic_coding/source_links.rs).
//
// The task predicate is an exact port. The document is native-only: see
// `renderDocument`.

import { agenticMessage } from './messages.mjs';
import { trimEnd } from './crate/rust_str.mjs';

/** Mirrors `const SOURCE_LINKS_PATH`. */
export const SOURCE_LINKS_PATH = 'self-source-links.lino';

const SOURCE_LINKS_KEYWORDS = ['source links', 'source graph', 'source-links', 'recompile'];

/**
 * Mirrors `fn is_source_links_task` in rust/src/agentic_coding/source_links.rs.
 * @param {string} prompt
 */
export function isSourceLinksTask(prompt) {
  const lower = prompt.toLowerCase();
  if (SOURCE_LINKS_KEYWORDS.some((keyword) => lower.includes(keyword))) return true;
  const wholeSource = (lower.includes('entire') || lower.includes('whole') || lower.includes('all')) && lower.includes('source');
  const toLinksAndBack = lower.includes('links') && lower.includes('back');
  return wholeSource && toLinksAndBack;
}

/**
 * Mirrors `fn render_document` in rust/src/agentic_coding/source_links.rs.
 * native-only: rust/src/self_source_links.rs (`owned_file_count`,
 * `owned_total_bytes`, `owned_manifest_content_id`, `SourceLinks::compile` of a
 * representative slice through the meta-language CST/AST engine) needs the
 * whole embedded Rust source tree. The stub renders only the record head.
 */
export function renderDocument() {
  return 'self_source_links\n';
}

/**
 * Mirrors `fn final_answer` in rust/src/agentic_coding/source_links.rs.
 * native-only: the file and slice counts of the stubbed projection are 0.
 */
export function finalAnswer(document) {
  return agenticMessage('source_links_final_answer', { files: 0, slice: 0, path: SOURCE_LINKS_PATH, document: trimEnd(document) });
}
