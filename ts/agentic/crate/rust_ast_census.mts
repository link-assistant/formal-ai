// The Rust node-kind histogram of `self_ast::ast_census`
// (rust/src/agentic_coding/self_ast.rs), JavaScript root (issue #1180 R11).
//
// The native census parses through the meta-language links network, whose
// tree-sitter adapter inserts one syntax link per tree-sitter node — every
// child, extras such as comments included — marked named exactly when the
// node is named, with the node kind as its term. The census keeps the named
// syntax links of the abstract-syntax projection and counts them per term.
// So the histogram is the named-node kind count of the tree-sitter-rust parse,
// which this module takes with the vendored web-tree-sitter runtime and the
// tree-sitter-rust grammar of the version meta-language compiles in
// (js/vendor/tree-sitter/, provenance.lino records both). The walk follows
// `child(i)` like the adapter, so extras are counted where they occur.
//
// Like history_store.mjs, this file never imports a host module: the caller
// hands in the web-tree-sitter runtime namespace and the grammar (a path, URL
// or bytes). node-host.mjs `rustAstCensus` passes the vendored pair. Loading
// is asynchronous; the census it returns is synchronous, which is the shape
// `io.astCensus` of history_store.mjs takes.

/**
 * Byte order of two kind names, like the native `BTreeMap<String, _>`.
 * @param {string} left
 * @param {string} right
 * @returns {number}
 */
function byteOrder(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

/**
 * Count the named nodes of a tree per kind (the census walk).
 * Mirrors `fn named_node_histogram` in rust/src/agentic_coding/self_ast.rs.
 * @param {object} root a web-tree-sitter root node
 * @returns {{named_node_count: number, node_kinds: Array<{kind: string, count: number}>}}
 */
export function namedNodeHistogram(root) {
  const histogram = new Map();
  const pending = [root];
  while (pending.length > 0) {
    const node = pending.pop();
    if (node.isNamed) histogram.set(node.type, (histogram.get(node.type) ?? 0) + 1);
    for (let index = node.childCount - 1; index >= 0; index -= 1) pending.push(node.child(index));
  }
  const node_kinds = [...histogram.keys()].sort(byteOrder).map((kind) => ({ kind, count: histogram.get(kind) }));
  const named_node_count = node_kinds.reduce((sum, row) => sum + row.count, 0);
  return { named_node_count, node_kinds };
}

/**
 * The census of one Rust source: named-node count, whether the parse is
 * free of error and missing nodes, and the node-kind histogram in byte order.
 * Mirrors `fn ast_census` in rust/src/agentic_coding/self_ast.rs.
 * @param {{Parser: Function, Language: object}} runtime the web-tree-sitter namespace
 * @param {string|URL|Uint8Array} grammar the tree-sitter-rust wasm
 * @returns {Promise<(source: string) => {named_node_count: number, clean: boolean, node_kinds: Array<{kind: string, count: number}>}>}
 */
export async function loadRustAstCensus(runtime, grammar) {
  await runtime.Parser.init();
  const language = await runtime.Language.load(grammar);
  const parser = new runtime.Parser();
  parser.setLanguage(language);
  return (source) => {
    const tree = parser.parse(source);
    try {
      const { named_node_count, node_kinds } = namedNodeHistogram(tree.rootNode);
      return { named_node_count, clean: !tree.rootNode.hasError, node_kinds };
    } finally {
      tree.delete();
    }
  };
}

/**
 * The `io.astCensus` history_store.mjs takes: a census's histogram rows.
 * Mirrors `fn census_kinds` in rust/src/history_context/commits.rs.
 * @param {(source: string) => {node_kinds: Array<{kind: string, count: number}>}} census
 * @returns {(source: string) => Array<{kind: string, count: number}>}
 */
export function historyAstCensus(census) {
  return (source) => census(source).node_kinds;
}
