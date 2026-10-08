// The socket side (rust/src/server/transport.rs): `node:http` in,
// `ApiHttpResponse` out, written with the same status lines and headers.

import http from 'node:http';

import { dispatch } from './dispatch.mjs';
import { runInRequestScope } from './debug-session.mjs';
import { beginForegroundActivity } from './dreaming-runtime.mjs';
import { serverMessage } from './messages.mjs';

const STATUS_TEXT = new Map([
  [200, 'OK'],
  [204, 'No Content'],
  [400, 'Bad Request'],
  [401, 'Unauthorized'],
  [403, 'Forbidden'],
  [404, 'Not Found'],
  [405, serverMessage('status_text_405')],
  [409, 'Conflict'],
]);

const ALLOW_HEADERS = 'content-type,authorization,x-api-key,x-goog-api-key,anthropic-api-key';

/** The header pairs as the client sent them (`request_headers`). */
function requestHeaders(request) {
  const pairs = [];
  for (let index = 0; index < request.rawHeaders.length; index += 2) {
    pairs.push([request.rawHeaders[index], String(request.rawHeaders[index + 1]).trim()]);
  }
  return pairs;
}

function readBody(request) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    request.on('data', (chunk) => chunks.push(chunk));
    request.on('end', () => resolve(Buffer.concat(chunks)));
    request.on('error', reject);
  });
}

/**
 * Write `response` exactly as `write_response` does: a status outside the
 * known set goes out as 500, every response closes the connection.
 */
export function writeResponse(socketResponse, response) {
  const known = STATUS_TEXT.has(response.statusCode);
  const status = known ? response.statusCode : 500;
  const body = Buffer.from(response.body, 'utf8');
  const headers = [
    ['content-type', response.contentType],
    ['content-length', String(body.length)],
    ['access-control-allow-origin', '*'],
    ['access-control-allow-methods', 'GET,POST,OPTIONS'],
    ['access-control-allow-headers', ALLOW_HEADERS],
  ];
  if (response.deprecated) {
    headers.push(['deprecation', 'true'], ['link', '</v1/network>; rel="successor-version"']);
  }
  headers.push(['connection', 'close']);
  socketResponse.sendDate = false;
  socketResponse.writeHead(status, known ? STATUS_TEXT.get(status) : serverMessage('status_text_500'), headers.flat());
  socketResponse.end(body);
}

/**
 * Create (not start) the HTTP server for `ctx`.
 * @param {object} ctx
 * @returns {http.Server}
 */
export function createServer(ctx) {
  return http.createServer(async (request, socketResponse) => {
    // Every request holds the dreaming idle gate (rust/src/server.rs
    // `ForegroundActivity::begin`), the Telegram webhook included.
    const foreground = beginForegroundActivity();
    const disconnected = new AbortController();
    socketResponse.on('close', () => { if (!socketResponse.writableEnded) disconnected.abort(); });
    try {
      const headers = requestHeaders(request);
      const raw = await readBody(request);
      // Like the Rust reader, only a declared content-length carries a body.
      const declared = Number.parseInt(request.headers['content-length'] || '0', 10) || 0;
      const body = raw.subarray(0, Math.min(declared, raw.length)).toString('utf8');
      const response = await runInRequestScope(disconnected.signal, () => dispatch(ctx, { method: request.method, path: request.url, headers, body }));
      writeResponse(socketResponse, response);
    } catch (error) {
      process.stderr.write(`request failed: ${error?.stack || error}\n`);
      if (!socketResponse.headersSent) socketResponse.destroy();
    } finally {
      foreground.end();
    }
  });
}
