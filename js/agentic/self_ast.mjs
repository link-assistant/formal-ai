// Fourth agentic recipe - store the CST/AST of our own Rust meta algorithm in
// our data (issue #538; rust/src/agentic_coding/self_ast.rs).
//
// The routing predicate is ported exactly. The census itself is a
// tree-sitter parse of rust/src/agentic_coding/planner.rs through the
// meta-language links network (`ast_census`), which has no JavaScript twin:
// native-only: rust/src/agentic_coding/self_ast.rs `ast_census` (the
// meta-language/tree-sitter engine). `renderDocument` therefore returns the
// committed artifact data/meta/self-ast.lino, which the Rust test
// `committed_self_ast_is_generated_and_written_by_the_driver` pins
// byte-for-byte to `render_document()`, so the planner writes the same bytes.

import { readText } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { trimEnd } from './crate/rust_str.mjs';

/** The meta-language grammar label for Rust (`RUST_GRAMMAR_LABEL`). */
const RUST_GRAMMAR_LABEL = 'rust';

/** Mirrors `TARGET_MODULE_PATH` in rust/src/agentic_coding/self_ast.rs. */
export const TARGET_MODULE_PATH = 'src/agentic_coding/planner.rs';

/** Mirrors `AST_PATH` in rust/src/agentic_coding/self_ast.rs. */
export const AST_PATH = 'self-ast.lino';

/** The committed twin of `render_document()` (data/meta/self-ast.lino). */
const COMMITTED_DOCUMENT = 'data/meta/self-ast.lino';

/** Mirrors `AST_TASK` in rust/src/agentic_coding/self_ast.rs. */
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
 * Mirrors the formatting half of `fn render_ast_document` in
 * rust/src/agentic_coding/self_ast.rs for an already computed census.
 * @param {string} targetPath
 * @param {ReturnType<typeof astCensus>} census
 */
export function renderCensusDocument(targetPath, census) {
  let out = 'self_ast\n';
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
 * CST/AST document of the pinned planner module (the committed artifact; see
 * the module comment).
 */
export function renderDocument() {
  return readText(COMMITTED_DOCUMENT);
}

/**
 * Mirrors `fn final_answer` in rust/src/agentic_coding/self_ast.rs.
 * @param {string} document
 */
export function finalAnswer(document) {
  return agenticMessage('self_ast_final_answer', { target: TARGET_MODULE_PATH, path: AST_PATH, document: trimEnd(document) });
}
