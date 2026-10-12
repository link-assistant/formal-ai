// LinksQL, the read-only query language over the links network
// (rust/src/links_query.rs `parse_links_query`, `run_links_query`,
// `LinksQueryResult::to_links_notation`).
//
// A parse failure is a `LinksQueryError` carrying a server-message key and its
// parameters, so the wording stays in data/meta/server-messages.lino.

import { serverMessage } from './messages.mjs';
import { escapeQuoted, graphBlocks, knowledgeGraph } from './network-graph.mjs';

/** A LinksQL parse failure (`LinksQueryError`). */
export class LinksQueryError extends Error {
  constructor(key, params = {}) {
    super(serverMessage(key, params));
    this.key = key;
    this.params = params;
  }
}

const fail = (key, params) => {
  throw new LinksQueryError(key, params);
};

/** Rust `str::trim_matches(ch)`. */
function trimMatches(text, ch) {
  let start = 0;
  let end = text.length;
  while (start < end && text[start] === ch) start += 1;
  while (end > start && text[end - 1] === ch) end -= 1;
  return text.slice(start, end);
}

/** `parse_links_query`. @param {string} query */
export function parseLinksQuery(query) {
  const trimmed = query.trim();
  const upper = trimmed.toUpperCase();
  const matchStart = upper.indexOf('MATCH');
  if (matchStart < 0) fail('links_query_match_missing');
  const returnStart = upper.indexOf('RETURN');
  if (returnStart < 0) fail('links_query_return_missing');
  if (returnStart < matchStart) fail('links_query_return_order');

  const afterMatch = trimmed.slice(matchStart + 'MATCH'.length, returnStart);
  const returnsText = trimmed.slice(returnStart + 'RETURN'.length);
  const relative = upper.slice(matchStart, returnStart).indexOf('WHERE');
  let patternText = afterMatch;
  let whereText = null;
  if (relative >= 0) {
    const absolute = matchStart + relative;
    patternText = trimmed.slice(matchStart + 'MATCH'.length, absolute);
    whereText = trimmed.slice(absolute + 'WHERE'.length, returnStart);
  }

  const { source, edge, target } = parsePattern(patternText.trim());
  const filters = whereText === null ? [] : parseFilters(whereText.trim());
  const returns = returnsText
    .split(',')
    .map((item) => item.trim().split('.')[0].trim())
    .filter((item) => item.length > 0);
  if (returns.length === 0) fail('links_query_return_empty');
  return { source, edge, target, filters, returns };
}

function parsePattern(text) {
  const sourceEnd = text.indexOf(')');
  if (sourceEnd < 0) fail('links_query_pattern_open');
  const source = parseNode(text.slice(0, sourceEnd + 1));
  const rest = text.slice(sourceEnd + 1).trim();
  if (!rest) return { source, edge: null, target: null };
  const arrowPos = rest.indexOf('->');
  if (arrowPos < 0) fail('links_query_edge_arrow');
  const edge = parseEdge(rest.slice(0, arrowPos).trim());
  const target = parseNode(rest.slice(arrowPos + 2).trim());
  return { source, edge, target };
}

function parseNode(text) {
  const trimmed = text.trim();
  if (!trimmed.startsWith('(') || !trimmed.slice(1).endsWith(')')) {
    fail('links_query_node_invalid', { text });
  }
  const inner = trimmed.slice(1, -1);
  const variable = inner.split(':')[0].trim();
  if (!variable) fail('links_query_node_variable');
  return { var: variable };
}

function parseEdge(text) {
  const trimmed = text.trim();
  const unprefixed = (trimmed.startsWith('-') ? trimmed.slice(1) : text).trim();
  if (!unprefixed.startsWith('[') || !unprefixed.slice(1).endsWith(']')) {
    fail('links_query_edge_invalid', { text });
  }
  const inner = unprefixed.slice(1, -1).trim();
  const colon = inner.indexOf(':');
  const varText = colon >= 0 ? inner.slice(0, colon).trim() : inner;
  const roleText = colon >= 0 ? inner.slice(colon + 1).trim() : null;
  return { var: varText ? varText : null, role: roleText ? roleText : null };
}

function parseFilters(text) {
  return splitFilters(text)
    .map((clause) => clause.trim())
    .filter((clause) => clause.length > 0)
    .map(parseFilter);
}

/** `split_filters`: split on AND outside quoted values. */
function splitFilters(text) {
  const parts = [];
  let current = '';
  let inQuotes = false;
  const chars = [...text];
  let index = 0;
  while (index < chars.length) {
    const ch = chars[index];
    if (ch === '"') {
      inQuotes = !inQuotes;
      current += ch;
      index += 1;
      continue;
    }
    if (!inQuotes) {
      const upper = chars.slice(index).join('').toUpperCase();
      const next = chars[index + 3];
      if (upper.startsWith('AND') && (next === undefined || /^\s$/u.test(next))) {
        parts.push(current);
        current = '';
        index += 3;
        continue;
      }
    }
    current += ch;
    index += 1;
  }
  parts.push(current);
  return parts;
}

function parseFilter(clause) {
  const found = findOperator(clause);
  if (!found) fail('links_query_filter_operator', { clause });
  const [op, splitAt, opLen] = found;
  const lhs = clause.slice(0, splitAt).trim();
  const rhs = clause.slice(splitAt + opLen).trim();
  const dot = lhs.indexOf('.');
  if (dot < 0) fail('links_query_filter_target', { clause });
  const variable = lhs.slice(0, dot);
  const fieldText = lhs.slice(dot + 1).trim().toLowerCase();
  if (!['id', 'label', 'role'].includes(fieldText)) fail('links_query_field_unknown', { field: fieldText });
  return { var: variable.trim(), field: fieldText, op, value: trimMatches(rhs.trim(), '"') };
}

/** `find_operator`: `=` or CONTAINS outside any quoted region. */
function findOperator(clause) {
  const upper = clause.toUpperCase();
  let inQuotes = false;
  for (let index = 0; index < clause.length; index += 1) {
    const ch = clause[index];
    if (ch === '"') {
      inQuotes = !inQuotes;
      continue;
    }
    if (inQuotes) continue;
    if (ch === '=') return ['eq', index, 1];
    if (upper.startsWith('CONTAINS', index)) return ['contains', index, 'CONTAINS'.length];
  }
  return null;
}

function applyNodeFilter(node, filter) {
  if (filter.field === 'role') return true;
  const haystack = filter.field === 'id' ? node.id : node.label;
  return filter.op === 'eq' ? haystack === filter.value : haystack.includes(filter.value);
}

function nodeFiltersMatch(node, variable, filters) {
  const relevant = filters.filter((filter) => filter.var === variable && filter.field !== 'role');
  if (relevant.length === 0) return true;
  if (!node) return false;
  return relevant.every((filter) => applyNodeFilter(node, filter));
}

function edgeFiltersMatch(edge, variable, filters) {
  if (variable === null) {
    return !filters.some((filter) => filter.field === 'role' && filter.var !== '__none__');
  }
  return filters
    .filter((filter) => filter.var === variable)
    .every((filter) => (filter.op === 'eq' ? edge.role === filter.value : edge.role.includes(filter.value)));
}

function pushNode(nodes, node) {
  if (!nodes.some((existing) => existing.id === node.id)) nodes.push(node);
}

/** `evaluate`. */
function evaluate(query, sourceText, graph) {
  const result = { query: sourceText.trim(), nodes: [], edges: [] };
  if (!query.edge) {
    for (const node of graph.nodes) {
      const matches = query.filters
        .filter((filter) => filter.var === query.source.var)
        .every((filter) => applyNodeFilter(node, filter));
      if (matches) pushNode(result.nodes, node);
    }
    return result;
  }
  const wantSource = query.returns.includes(query.source.var);
  const wantTarget = query.returns.includes(query.target.var);
  for (const edge of graph.edges) {
    if (query.edge.role !== null && edge.role !== query.edge.role) continue;
    const fromNode = graph.nodes.find((node) => node.id === edge.from);
    const toNode = graph.nodes.find((node) => node.id === edge.to);
    if (!nodeFiltersMatch(fromNode, query.source.var, query.filters)
      || !nodeFiltersMatch(toNode, query.target.var, query.filters)
      || !edgeFiltersMatch(edge, query.edge.var, query.filters)) {
      continue;
    }
    result.edges.push(edge);
    if (wantSource && fromNode) pushNode(result.nodes, fromNode);
    if (wantTarget && toNode) pushNode(result.nodes, toNode);
  }
  return result;
}

/** `run_links_query`: throws `LinksQueryError` on a parse failure. */
export function runLinksQuery(query, graph = knowledgeGraph()) {
  return evaluate(parseLinksQuery(query), query, graph);
}

/** `LinksQueryResult::to_links_notation`. */
export function linksQueryResultLinksNotation(result) {
  return `links_query_result\n  query "${escapeQuoted(result.query)}"\n${graphBlocks(result.nodes, result.edges)}`;
}
