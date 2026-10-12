// The tree-sitter backed `MetaLanguageFormalization` of
// `parse_with_meta_language` in rust/src/summarization/file.rs, for the
// grammars a host loads (the vendored tree-sitter-rust today,
// js/vendor/tree-sitter/). Like rust_ast_census.mjs this file never imports a
// host module: the caller hands in the web-tree-sitter runtime namespace and
// the grammars (a path, URL or bytes) and gets back a synchronous parser to
// give `installMetaLanguageParser` (summarization_file.mjs).
//
// What is reproduced, read off meta-language 0.58.2 (`LinkNetwork::parse` ->
// `tree_sitter_adapter::parse`), the crate the Rust build compiles in:
//
// * `syntax_link_count`: the adapter inserts one `Syntax` link per tree-sitter
//   node (every node reached through `child(i)`, extras and missing nodes
//   included) and nothing else in a Rust parse is `Syntax`-typed, so the count
//   is the node count of the tree;
// * `has_error`: `verify_full_match` reports an issue for every error, missing
//   or containing-error link, so the parse is clean exactly when the root node
//   has no error;
// * `text_preserved`: `reconstruct_text` concatenates the non-missing `Token`
//   links (leaf tokens and the gap tokens `convert_node` inserts between
//   children) ordered by start, skipping overlaps; replayed here on the same
//   leaf and gap rules;
// * `total_link_count`: the 13 self-description roots and their 13 relation
//   links, the language link, the document link, the syntax nodes, the tokens,
//   two trivia links (the default `Both` attachment policy) per extra token,
//   and one link per named field plus one point per field label not already a
//   term.
//
// Not reproduced: grammars other than the ones the host loads (a label with no
// loaded grammar yields `null`, where the Rust build always yields evidence for
// the 17 labels `meta_language_label_for_format` names), the data-format parsers
// meta-language tries before tree-sitter for json/yaml/toml/ini/xml/html/css,
// and embedded-region detection (none exists for Rust). The four numbers were
// derived from the crate source, not compared with a native run.

/** The 13 `SELF_DESCRIPTION_ROOTS` terms of meta-language's `self_description.rs`. */
const SELF_DESCRIPTION_TERMS = [
  'link', 'reference', 'relation link', 'language', 'grammar', 'type', 'Type', 'concept', 'point',
  'field', 'trivia', 'region', 'object',
];

/** Links per extra token under the default `TriviaAttachmentPolicy::Both`. */
const TRIVIA_LINKS_PER_EXTRA_TOKEN = 2;

/**
 * Mirrors `fn insert_gap_token` of meta-language's tree_sitter_adapter.rs: a
 * non-named extra token over `[start, end)` plus its trivia links.
 */
function insertGapToken(state, start, end) {
  const clampedStart = Math.min(start, state.length);
  const clampedEnd = Math.min(end, state.length);
  if (clampedStart === clampedEnd) return;
  state.tokens.push({ start: clampedStart, end: clampedEnd, term: state.text.slice(clampedStart, clampedEnd) });
  state.extraTokens += 1;
}

/**
 * Mirrors `fn insert_leaf_token` of meta-language's tree_sitter_adapter.rs.
 */
function insertLeafToken(state, node) {
  const start = node.startIndex;
  const end = Math.min(node.endIndex, state.length);
  if (node.isMissing || start >= end) return;
  state.tokens.push({ start, end, term: state.text.slice(start, end) });
  if (node.isExtra) state.extraTokens += 1;
}

/**
 * Mirrors `fn convert_node` of meta-language's tree_sitter_adapter.rs: counts
 * the links the node inserts and records its tokens.
 */
function convertNode(state, node) {
  state.syntax += 1;
  if (node.childCount === 0) {
    insertLeafToken(state, node);
    return;
  }
  let coveredUntil = node.startIndex;
  for (let index = 0; index < node.childCount; index += 1) {
    const child = node.child(index);
    insertGapToken(state, coveredUntil, child.startIndex);
    convertNode(state, child);
    const label = node.fieldNameForChild(index);
    if (label !== null && label !== undefined) {
      state.fieldLinks += 1;
      if (!state.terms.has(label)) {
        state.terms.add(label);
        state.fieldLabels += 1;
      }
    }
    coveredUntil = Math.min(child.endIndex, state.length);
  }
  insertGapToken(state, coveredUntil, node.endIndex);
}

/**
 * Mirrors `LinkNetwork::reconstruct_text`: the tokens ordered by start (ties by
 * creation order), overlaps skipped, concatenated.
 */
function reconstructText(tokens) {
  const ordered = tokens.map((token, id) => ({ ...token, id }));
  ordered.sort((left, right) => (left.start - right.start) || (left.id - right.id));
  let reconstructed = '';
  let coveredUntil = 0;
  for (const token of ordered) {
    if (token.start < coveredUntil) continue;
    reconstructed += token.term;
    coveredUntil = token.end;
  }
  return reconstructed;
}

/**
 * Mirrors `fn parse_with_meta_language` in rust/src/summarization/file.rs for
 * one tree-sitter `parser`.
 * @param {object} parser a web-tree-sitter `Parser` with a language set
 * @param {string} label
 * @param {string} source
 */
export function parseWithTreeSitter(parser, label, source) {
  const tree = parser.parse(source);
  try {
    const terms = new Set(SELF_DESCRIPTION_TERMS);
    terms.add(label);
    const state = {
      text: source, length: source.length, tokens: [], extraTokens: 0, syntax: 0, fieldLinks: 0, fieldLabels: 0, terms,
    };
    convertNode(state, tree.rootNode);
    const rootLinks = SELF_DESCRIPTION_TERMS.length * 2;
    const languageLink = SELF_DESCRIPTION_TERMS.includes(label) ? 0 : 1;
    const documentLink = 1;
    return {
      label,
      syntax_link_count: state.syntax,
      total_link_count: rootLinks + languageLink + documentLink + state.syntax + state.tokens.length
        + TRIVIA_LINKS_PER_EXTRA_TOKEN * state.extraTokens + state.fieldLinks + state.fieldLabels,
      has_error: tree.rootNode.hasError,
      text_preserved: reconstructText(state.tokens) === source,
    };
  } finally {
    tree.delete();
  }
}

/**
 * Load the grammars and return the synchronous parser
 * `installMetaLanguageParser` takes: `(label, source) => evidence`, `null` for a
 * label whose grammar was not loaded.
 * Rust dependency `meta_language::LinkNetwork::parse`, whose grammars are compiled in.
 * @param {{Parser: Function, Language: object}} runtime the web-tree-sitter namespace
 * @param {Object<string, string|URL|Uint8Array>} grammars label -> grammar wasm
 * @returns {Promise<(label: string, source: string) => (object|null)>}
 */
export async function loadMetaLanguageParser(runtime, grammars) {
  await runtime.Parser.init();
  const parsers = new Map();
  for (const label of Object.keys(grammars)) {
    const language = await runtime.Language.load(grammars[label]);
    const parser = new runtime.Parser();
    parser.setLanguage(language);
    parsers.set(label, parser);
  }
  return (label, source) => {
    const parser = parsers.get(label);
    return parser === undefined ? null : parseWithTreeSitter(parser, label, source);
  };
}
