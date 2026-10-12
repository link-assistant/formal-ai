// HTTP handlers for the links-network view of the knowledge store
// (rust/src/network_endpoint.rs `handle_network_request`,
// `handle_links_query_request`, `parse_links_query_body`, and the
// `GET /v1/links` arm of rust/src/server.rs).

import { LinksQueryError, linksQueryResultLinksNotation, runLinksQuery } from './links-query.mjs';
import { serverMessage } from './messages.mjs';
import { isKnownTraceId, knowledgeGraph, knowledgeGraphDot, knowledgeGraphLinksNotation } from './network-graph.mjs';
import { jsonResponse, linksNotationResponse, messageError, rawResponse, TEXT_TYPE } from './response.mjs';

/** `GET /v1/network` (and the deprecated `/v1/graph` alias). */
export function handleNetwork(_ctx, request) {
  let trace = null;
  let format = null;
  for (const pair of String(request.query || '').split('&').filter((part) => part.length > 0)) {
    const at = pair.indexOf('=');
    if (at < 0) continue;
    const key = pair.slice(0, at);
    const value = pair.slice(at + 1);
    if (key === 'trace') trace = value;
    else if (key === 'format') format = value;
  }
  if (trace !== null && !isKnownTraceId(trace)) return messageError(404, 'network_unknown_trace');
  if (format === 'dot') return rawResponse(200, TEXT_TYPE, knowledgeGraphDot());
  const graph = knowledgeGraph();
  return jsonResponse(200, { nodes: graph.nodes, edges: graph.edges });
}

/** `GET /v1/links`. */
export function handleLinks() {
  return linksNotationResponse(200, knowledgeGraphLinksNotation());
}

/** Rust `str::lines`: split on `\n`, dropping one trailing `\r` per line and a final empty line. */
function rustLines(text) {
  const lines = text.split('\n');
  if (lines.length > 0 && lines[lines.length - 1] === '') lines.pop();
  return lines.map((line) => (line.endsWith('\r') ? line.slice(0, -1) : line));
}

function trimQuotes(text) {
  let start = 0;
  let end = text.length;
  while (start < end && text[start] === '"') start += 1;
  while (end > start && text[end - 1] === '"') end -= 1;
  return text.slice(start, end);
}

/** `parse_links_query_body`: a JSON `{"query"}` object or a Links-Notation envelope. */
export function parseLinksQueryBody(body) {
  try {
    const value = JSON.parse(body);
    if (value && typeof value === 'object' && !Array.isArray(value) && typeof value.query === 'string') {
      return value.query;
    }
  } catch {
    // not JSON: fall through to the Links-Notation envelope
  }
  for (const line of rustLines(String(body))) {
    const trimmed = line.trim();
    if (!trimmed.startsWith('query ')) continue;
    const unquoted = trimQuotes(trimmed.slice('query '.length).trim());
    if (unquoted) return unquoted.split('\\"').join('"');
  }
  return null;
}

/** `POST /v1/links/query`. */
export function handleLinksQuery(_ctx, request) {
  const query = parseLinksQueryBody(request.body ?? '');
  if (query === null) return messageError(400, 'links_query_missing');
  try {
    return linksNotationResponse(200, linksQueryResultLinksNotation(runLinksQuery(query)));
  } catch (error) {
    if (!(error instanceof LinksQueryError)) throw error;
    return messageError(400, 'links_query_invalid', { error: serverMessage(error.key, error.params) });
  }
}
