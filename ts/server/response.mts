// The `ApiHttpResponse` value and its constructors (rust/src/server.rs).

import { serverMessage } from './messages.mjs';
import { toCompactJson, toPrettyJson } from './json.mjs';

export const JSON_TYPE = 'application/json';
export const TEXT_TYPE = 'text/plain';
export const SSE_TYPE = 'text/event-stream';

/**
 * @typedef {{statusCode: number, contentType: string, body: string, deprecated: boolean}} ApiHttpResponse
 */

/** @returns {ApiHttpResponse} */
export function rawResponse(statusCode, contentType, body) {
  return { statusCode, contentType, body, deprecated: false };
}

/** `json_response`: pretty JSON. @returns {ApiHttpResponse} */
export function jsonResponse(statusCode, value) {
  return rawResponse(statusCode, JSON_TYPE, toPrettyJson(value));
}

/** `error_response`: compact `{"error":{"message","type"}}`. @returns {ApiHttpResponse} */
export function errorResponse(statusCode, message) {
  return rawResponse(
    statusCode,
    JSON_TYPE,
    toCompactJson({ error: { message, type: 'formal_ai_error' } }),
  );
}

/** An error whose text is the server message `key`. @returns {ApiHttpResponse} */
export function messageError(statusCode, key, params = {}) {
  return errorResponse(statusCode, serverMessage(key, params));
}

/** `links_notation_response`. @returns {ApiHttpResponse} */
export function linksNotationResponse(statusCode, body) {
  return rawResponse(statusCode, TEXT_TYPE, body);
}

/** An SSE body. @returns {ApiHttpResponse} */
export function sseResponse(body) {
  return rawResponse(200, SSE_TYPE, body);
}

/** `ApiHttpResponse::into_deprecated_alias`. @returns {ApiHttpResponse} */
export function deprecatedAlias(response) {
  return { ...response, deprecated: true };
}
