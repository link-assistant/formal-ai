// The issue-#558 source-links recipe (rust/src/agentic_coding/source_links.rs).
//
// The task predicate, the slice choice and the `entire_source` header (file
// count, total bytes, manifest content id) are exact ports over the owned
// manifest the host enumerates from rust/src as build.rs does
// (js/agentic/crate/self_source_links.mjs).
//
// Substitution: the `round_trip_proof` parse census (`total_link_count`,
// `named_node_count`, `faithful`) needs the meta-language/tree-sitter engine
// (`agentic_coding::self_ast::ast_census`), which has no JavaScript twin. The
// port renders what the Rust crate built without the `meta-language` feature
// renders: zero counts and `faithful false`. No committed artifact pins the
// engine values (the Rust tests assert this document live). See the header of
// crate/self_source_links.mjs.

import { cached } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { rustLines } from './content.mjs';
import { SourceLinks, ownedFileCount, ownedManifestContentId, ownedSourceFiles, ownedTotalBytes } from './crate/self_source_links.mjs';
import { trimEnd } from './crate/rust_str.mjs';

/** Mirrors `const SOURCE_LINKS_PATH`. */
export const SOURCE_LINKS_PATH = 'self-source-links.lino';

/** Mirrors `const SLICE_SIZE`. */
const SLICE_SIZE = 6;

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
 * Mirrors `fn representative_slice` in rust/src/agentic_coding/source_links.rs:
 * up to `SLICE_SIZE` files spread evenly across the path-sorted source tree.
 * @returns {Array<[string, string]>}
 */
export function representativeSlice() {
  const files = ownedSourceFiles();
  const total = files.length;
  const want = Math.min(SLICE_SIZE, total);
  if (want === 0) return [];
  const stride = Math.max(Math.floor(total / want), 1);
  return Array.from({ length: want }, (_, index) => files[Math.min(index * stride, total - 1)]);
}

/** Mirrors `fn cached_slice` (a `OnceLock`). */
const cachedSlice = () => cached('source-links-slice', () => SourceLinks.compile(representativeSlice()));

/**
 * Mirrors `fn render_document` in rust/src/agentic_coding/source_links.rs.
 * Deterministic and ends with exactly one trailing newline.
 */
export function renderDocument() {
  const slice = cachedSlice();
  const out = [
    'self_source_links',
    '  engine meta_language',
    '  language rust',
    '  task translate_entire_source_to_links_and_back',
    '  entire_source',
    `    file_count ${ownedFileCount()}`,
    `    total_bytes ${ownedTotalBytes()}`,
    `    manifest_content_id "${ownedManifestContentId()}"`,
    '  round_trip_proof',
    `    slice_size ${slice.moduleCount()}`,
    `    slice_faithful_count ${slice.faithfulCount()}`,
    `    slice_fully_faithful ${slice.isFullyFaithful()}`,
  ];
  for (const line of rustLines(slice.linksNotation())) out.push(line === '' ? '' : `    ${line}`);
  return `${trimEnd(`${out.join('\n')}\n`)}\n`;
}

/** Mirrors `fn slice` in rust/src/agentic_coding/source_links.rs. */
export function slice() {
  return cachedSlice();
}

/**
 * Mirrors `fn final_answer` in rust/src/agentic_coding/source_links.rs.
 * @param {string} document
 */
export function finalAnswer(document) {
  return agenticMessage('source_links_final_answer', {
    files: ownedFileCount(), slice: cachedSlice().moduleCount(), path: SOURCE_LINKS_PATH, document: trimEnd(document),
  });
}
