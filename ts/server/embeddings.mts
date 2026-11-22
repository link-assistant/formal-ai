// OpenAI embeddings backed by the existing multilingual meaning lexicon.
// The native HTTP server uses the same coordinate and matching rules.

import { lexicon } from '../agentic/crate/seed_meanings.mjs';
import { estimateTokens } from './ids.mjs';
import { f64, sortedKeys } from './json.mjs';
import { ensureDataHost } from './learning-cycle.mjs';
import { serverMessage } from './messages.mjs';
import { jsonResponse, messageError } from './response.mjs';
import { tryResolveModelId } from './seed.mjs';

export const EMBEDDING_MODEL = 'formal-ai-embeddings';
export const MINIMUM_EMBEDDING_DIMENSIONS = 1;
export const EMBEDDING_DIMENSIONS = 4096;
const MAXIMUM_INPUTS = 128;
const MAXIMUM_INPUT_TOKENS = 8192;

function words(text) {
  const normalized = Array.from(text, character => character.toLowerCase()).join('').normalize('NFC');
  return normalized.match(/[\p{L}\p{M}\p{N}]+/gu) ?? [];
}

let index;

function conceptIndex() {
  if (index !== undefined) return index;
  ensureDataHost();
  const bySurface = new Map();
  const concepts = new Set();
  let maximumWords = 0;
  for (const meaning of lexicon()) {
    const concept = /^[QP][0-9]+$/u.test(meaning.wikidata)
      ? `wikidata:${meaning.wikidata}` : `meaning:${meaning.slug}`;
    for (const lexeme of meaning.lexemes) {
      for (const form of lexeme.words) {
        if (form.text.includes('…')) continue;
        const tokens = words(form.text);
        if (tokens.length === 0) continue;
        const surface = tokens.join(' ');
        if (!bySurface.has(surface)) bySurface.set(surface, new Set());
        bySurface.get(surface).add(concept);
        concepts.add(concept);
        maximumWords = Math.max(maximumWords, tokens.length);
      }
    }
  }
  if (concepts.size > EMBEDDING_DIMENSIONS) {
    index = null;
    return index;
  }
  // Byte order mirrors Rust's string ordering and is independent of locale.
  const sorted = [...concepts].map(concept => Buffer.from(concept)).sort(Buffer.compare);
  const coordinates = new Map(sorted.map((concept, coordinate) => [concept.toString(), coordinate]));
  const surfaces = new Map([...bySurface].map(([surface, links]) => [
    surface, [...links].map(link => coordinates.get(link)),
  ]));
  index = { surfaces, maximumWords };
  return index;
}

function matchedCoordinates(text, { surfaces, maximumWords }) {
  const tokens = words(text);
  const active = new Set();
  for (let start = 0; start < tokens.length;) {
    let surface = '';
    let match;
    let next = start + 1;
    for (let end = start; end < Math.min(tokens.length, start + maximumWords); end += 1) {
      surface = end === start ? tokens[end] : `${surface} ${tokens[end]}`;
      const coordinates = surfaces.get(surface);
      if (coordinates) {
        match = coordinates;
        next = end + 1;
      }
    }
    for (const coordinate of match ?? []) active.add(coordinate);
    start = next;
  }
  return active;
}

function projectionSign(source, target) {
  // The seed and 32-bit operations match the native endpoint.
  let word = (source * EMBEDDING_DIMENSIONS + target + 0x9e3779b9) >>> 0;
  word = Math.imul(word ^ (word >>> 16), 0x7feb352d);
  word = Math.imul(word ^ (word >>> 15), 0x846ca68b);
  word ^= word >>> 16;
  return (word >>> 31) === 0 ? 1 : -1;
}

function project(coordinates, dimensions) {
  let weights = Array.from({ length: dimensions }, (_, target) => {
    let weight = 0;
    for (const source of coordinates) weight += projectionSign(source, target);
    return weight;
  });
  // Very small projections can cancel. Use the first active concept's row in that case.
  if (weights.every(weight => weight === 0)) {
    const source = Math.min(...coordinates);
    weights = weights.map((_, target) => projectionSign(source, target));
  }
  const norm = Math.sqrt(weights.reduce((sum, weight) => sum + weight * weight, 0));
  return weights.map(weight => Math.fround(weight / norm));
}

function conceptVector(coordinates, dimensions) {
  if (dimensions < EMBEDDING_DIMENSIONS) return project(coordinates, dimensions);
  const vector = new Array(EMBEDDING_DIMENSIONS).fill(0);
  for (const coordinate of coordinates) vector[coordinate] = 1;
  return vector;
}

function encodeBase64(vector) {
  const bytes = Buffer.alloc(vector.length * 4);
  vector.forEach((weight, coordinate) => bytes.writeFloatLE(weight, coordinate * 4));
  return bytes.toString('base64');
}

function invalid(field) {
  return messageError(400, 'invalid-embeddings-request', { field });
}

/** `POST /v1/embeddings`. */
export function handleEmbeddings(_ctx, { body }) {
  let request;
  try {
    request = JSON.parse(body);
  } catch {
    return messageError(400, 'invalid-embeddings-json');
  }
  if (!request || typeof request !== 'object' || Array.isArray(request)) {
    return messageError(400, 'invalid-embeddings-json');
  }
  const { model, input, dimensions = EMBEDDING_DIMENSIONS, encoding_format: format = 'float' } = request;
  if (typeof model !== 'string' || !model.trim()) return invalid('model');
  if (model.trim().toLowerCase() !== EMBEDDING_MODEL && !tryResolveModelId(model)) {
    return messageError(400, 'unsupported-embeddings-model', { model: model.trim(), canonical: EMBEDDING_MODEL });
  }
  if (!Number.isInteger(dimensions) || dimensions < MINIMUM_EMBEDDING_DIMENSIONS || dimensions > EMBEDDING_DIMENSIONS) {
    return invalid('dimensions');
  }
  if (format !== 'float' && format !== 'base64') return invalid('encoding_format');
  const texts = typeof input === 'string' ? [input] : input;
  if (!Array.isArray(texts) || texts.length === 0 || texts.length > MAXIMUM_INPUTS) return invalid('input');

  let usage = 0;
  for (const text of texts) {
    if (typeof text !== 'string' || !text.trim()) return invalid('input');
    const tokens = estimateTokens(text);
    if (tokens > MAXIMUM_INPUT_TOKENS) return invalid('input');
    usage += tokens;
  }
  const catalog = conceptIndex();
  if (!catalog) return messageError(500, 'embeddings-catalog-capacity');
  const data = [];
  for (const [position, text] of texts.entries()) {
    const coordinates = matchedCoordinates(text, catalog);
    if (coordinates.size === 0) {
      return jsonResponse(400, sortedKeys({
        error: {
          code: 'no_concepts',
          input_index: position,
          message: serverMessage('embeddings-no-concepts'),
          type: 'formal_ai_error',
        },
      }));
    }
    const vector = conceptVector(coordinates, dimensions);
    let embedding = vector;
    if (format === 'base64') embedding = encodeBase64(vector);
    else if (dimensions !== EMBEDDING_DIMENSIONS) embedding = vector.map(f64);
    data.push({ embedding, index: position, object: 'embedding' });
  }
  return jsonResponse(200, sortedKeys({
    data, model: EMBEDDING_MODEL, object: 'list', usage: { prompt_tokens: usage, total_tokens: usage },
  }));
}
