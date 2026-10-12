import { rustIdentifierIsValid } from '../code_task/identifier_domain.mjs';

function nodesOf(root) {
  const nodes = [], pending = [root];
  while (pending.length) {
    const node = pending.pop();
    nodes.push(node);
    for (let index = node.namedChildCount - 1; index >= 0; index -= 1) pending.push(node.namedChild(index));
  }
  return nodes;
}

/** A source transformation candidate; this function never grants effect authority. */
export function rustConstantRenameCandidate(tree, source, previous, replacement) {
  const root = tree?.rootNode;
  if (typeof source !== 'string' || !root || root.hasError || root.text !== source
    || !rustIdentifierIsValid(previous) || !rustIdentifierIsValid(replacement)
    || previous === replacement) return null;
  const nodes = nodesOf(root);
  // A separately declared module can contain unresolved descendants of this scope.
  if (nodes.some(node => node.type === 'mod_item' || node.type === 'macro_definition')) return null;
  const declarations = nodes.filter(node => node.type === 'const_item'
    && node.childForFieldName('name')?.text === previous);
  if (declarations.length !== 1 || declarations[0].parent?.type !== 'source_file'
    || declarations[0].namedChildren.some(node => node.type === 'visibility_modifier')) return null;
  if (nodes.some(node => node.type === 'identifier' && node.text === replacement)) return null;
  const name = declarations[0].childForFieldName('name');
  const occurrences = nodes.filter(node => node.type === 'identifier' && node.text === previous);
  if (occurrences.length === 0 || !occurrences.some(node => node.id === name.id)) return null;
  const allowed = new Set([
    'binary_expression', 'arguments', 'return_expression', 'reference_expression',
    'array_expression', 'tuple_expression', 'expression_statement', 'parenthesized_expression'
  ]);
  for (const node of occurrences) {
    if (source.slice(node.startIndex, node.endIndex) !== previous) return null;
    const parent = node.parent;
    const tail = parent?.type === 'block' && parent.lastNamedChild?.id === node.id;
    if (node.id !== name.id && !allowed.has(parent?.type) && !tail) return null;
  }
  const word = new RegExp('(?:^|[^A-Za-z0-9_])' + previous + '(?:$|[^A-Za-z0-9_])', 'u');
  if (nodes.some(node => node.type === 'macro_invocation' && word.test(node.text))) return null;
  const spans = occurrences.map(node => [node.startIndex, node.endIndex]).sort((a, b) => a[0] - b[0]);
  if (spans.some((span, index) => index > 0 && spans[index - 1][1] > span[0])) return null;
  let content = source;
  for (const [start, end] of [...spans].reverse()) content = content.slice(0, start) + replacement + content.slice(end);
  return Object.freeze({
    kind: 'private-constant-alpha-renaming', content, previous, replacement,
    spans: Object.freeze(spans.map(span => Object.freeze(span))), scope: 'one-source-file-private-constant',
    sourceSyntax: 'tree-sitter-clean', compilation: 'pending', writeAuthority: false,
    runtimeEffects: 'not-executed', unresolvedNeeds: Object.freeze(['trusted-source-read', 'owned-write', 'native-verification'])
  });
}
