// Self-inspection uses actual recursive Rust CST content supplied by the host.
// No committed histogram substitutes for current source. The default frontend
// derives actual content; an explicitly selected SDK must validate its pin.
// Legacy astCensus remains an explicitly engine-disabled diagnostic adapter.

import { host, hasHost } from './host.mjs';
import { readSourceLinks, sourceFromLinks, sourceOwner } from '../meta/readable-source.js';
import { agenticMessage } from './messages.mjs';
import { trimEnd } from './crate/rust_str.mjs';

/** The meta-language grammar label for Rust (`RUST_GRAMMAR_LABEL`). */
const RUST_GRAMMAR_LABEL = 'rust';

/** Mirrors `TARGET_MODULE_PATH` in rust/src/agentic_coding/self_ast.rs. */
export const TARGET_MODULE_PATH = 'src/agentic_coding/planner.rs';

/** Mirrors `AST_PATH` in rust/src/agentic_coding/self_ast.rs. */
// Historical export name; the primary artifact is a complete source CST.
export const AST_PATH = 'self-cst.lino';

// No committed histogram substitutes for current recursive source content.

/** Mirrors `fn ast_task` in rust/src/agentic_coding/self_ast.rs. */
export function astTask() {
  return agenticMessage('self_ast_task');
}

/** Mirrors `AST_KEYWORDS` (data/meta/agentic-messages.lino `self_ast_keywords`) in rust/src/agentic_coding/self_ast.rs. */
const astKeywords = () => agenticMessage('self_ast_keywords').split('|');

/**
 * Mirrors `fn is_self_ast_task` in rust/src/agentic_coding/self_ast.rs: an
 * AST/CST intent word plus a self-reference.
 * @param {string} prompt
 */
export function isSelfAstTask(prompt) {
  const lower = prompt.toLowerCase();
  const namesAst = astKeywords().some((keyword) => lower.includes(keyword))
    || lower.split(/[^\p{L}\p{N}]+/u).includes('cst')
    || (lower.includes('ast') && lower.includes('meta algorithm'));
  const selfReference = lower.includes('our') || lower.includes('itself')
    || lower.includes('meta algorithm') || lower.includes('planner');
  return namesAst && selfReference;
}

/**
 * Mirrors `fn ast_census` with the `meta-language` feature disabled (the
 * unavailable census). native-only: the real census is a tree-sitter parse.
 * @returns {{total_link_count: number, named_node_count: number, text_preserved: boolean, clean: boolean, node_kinds: Array<{kind: string, count: number}>}}
 */
export function astCensus() {
  return { total_link_count: 0, named_node_count: 0, text_preserved: false, clean: false, node_kinds: [] };
}

/**
 * Mirrors `fn render_census_document`, the formatting half of `fn render_ast_document` in
 * rust/src/agentic_coding/self_ast.rs for an already computed census.
 * @param {string} targetPath
 * @param {ReturnType<typeof astCensus>} census
 */
export function renderCensusDocument(targetPath, census) {
  let out = 'source_census_diagnostic\n';
  out += `  target ${targetPath}\n`;
  out += '  language rust\n';
  out += '  engine meta_language\n';
  out += `  component ${'meta-language'}\n`;
  out += `  grammar_label ${RUST_GRAMMAR_LABEL}\n`;
  out += '  projection abstract_syntax\n';
  out += `  text_preserved ${census.text_preserved}\n`;
  out += `  clean ${census.clean}\n`;
  out += `  total_link_count ${census.total_link_count}\n`;
  out += `  named_node_count ${census.named_node_count}\n`;
  out += `  distinct_node_kinds ${census.node_kinds.length}\n`;
  out += '  node_kinds\n';
  for (const { kind, count } of census.node_kinds) out += `    ${kind} ${count}\n`;
  return `${trimEnd(out)}\n`;
}

/**
 * Mirrors `fn render_document` in rust/src/agentic_coding/self_ast.rs: the
 * current recursive CST of the pinned planner module through the actual
 * sourceDocument capability; a missing engine is explicitly refused.
 */
export async function renderDocument() {
  if (!hasHost() || typeof host().sourceDocument !== 'function') throw Error('SourceModelEngineUnavailable');
  const document = await host().sourceDocument(TARGET_MODULE_PATH);
  const model = readSourceLinks(document);
  if (sourceOwner(model) !== TARGET_MODULE_PATH) throw Error('SourceOwnerMismatch');
  sourceFromLinks(model);
  return document;
}

/**
 * Mirrors `fn final_answer` in rust/src/agentic_coding/self_ast.rs.
 * @param {string} document
 */
export function finalAnswer(document) {
  return agenticMessage('self_ast_final_answer', { target: TARGET_MODULE_PATH, path: AST_PATH, document: trimEnd(document) });
}
