// OpenAI embeddings backed by the existing multilingual meaning lexicon.
// The native HTTP server uses the same coordinate and matching rules.

import { lexicon } from '../agentic/crate/seed_meanings.mjs';
import { estimateTokens } from './ids.mjs';
import { sortedKeys } from './json.mjs';
import { ensureDataHost } from './learning-cycle.mjs';
import { jsonResponse, messageError } from './response.mjs';
import { tryResolveModelId } from './seed.mjs';

export const EMBEDDING_MODEL = 'formal-ai-embeddings';
export const EMBEDDING_DIMENSIONS = 4096;
const MAX_INPUTS = 128;
const MAX_INPUT_TOKENS = 8192;

function words(text) {
  const normalized = Array.from(text, character => character.toLowerCase()).join('').normalize('NFC');
  return normalized.match(/[\p{L}\p{M}\p{N}]+/gu) ?? [];
}

let index;

function conceptIndex() {
  if (index) return index;
  ensureDataHost();
  const bySurface = new Map();
  const concepts = new Set();
  let maxWords = 0;
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
        maxWords = Math.max(maxWords, tokens.length);
      }
    }
  }
  if (concepts.size > EMBEDDING_DIMENSIONS) return null;
  // Byte order mirrors Rust's string ordering and is independent of locale.
  const sorted = [...concepts].map(concept => Buffer.from(concept)).sort(Buffer.compare);
  const coordinates = new Map(sorted.map((concept, coordinate) => [concept.toString(), coordinate]));
  const surfaces = new Map([...bySurface].map(([surface, links]) => [
    surface, [...links].map(link => coordinates.get(link)),
  ]));
  index = { surfaces, maxWords };
  return index;
}

function conceptVector(text, { surfaces, maxWords }) {
  const tokens = words(text);
  const vector = new Array(EMBEDDING_DIMENSIONS).fill(0);
  for (let start = 0; start < tokens.length;) {
    let surface = '';
    let match;
    let next = start + 1;
    for (let end = start; end < Math.min(tokens.length, start + maxWords); end += 1) {
      surface = end === start ? tokens[end] : `${surface} ${tokens[end]}`;
      const coordinates = surfaces.get(surface);
      if (coordinates) {
        match = coordinates;
        next = end + 1;
      }
    }
    for (const coordinate of match ?? []) vector[coordinate] = 1;
    start = next;
  }
  return vector;
}

function encodeBase64(vector) {
  const bytes = Buffer.alloc(vector.length * 4);
  vector.forEach((weight, coordinate) => bytes.writeFloatLE(weight, coordinate * 4));
  return bytes.toString('base64');
}

function invalid(field) {
  return messageError(400, 'invalid_embeddings_request', { field });
}

/** `POST /v1/embeddings`. */
export function handleEmbeddings(_ctx, { body }) {
  let request;
  try {
    request = JSON.parse(body);
  } catch {
    return messageError(400, 'invalid_embeddings_json');
  }
  if (!request || typeof request !== 'object' || Array.isArray(request)) {
    return messageError(400, 'invalid_embeddings_json');
  }
  const { model, input, dimensions, encoding_format: format = 'float' } = request;
  if (typeof model !== 'string' || !model.trim()) return invalid('model');
  if (model.trim().toLowerCase() !== EMBEDDING_MODEL && !tryResolveModelId(model)) {
    return messageError(400, 'unsupported_embeddings_model', { model: model.trim(), canonical: EMBEDDING_MODEL });
  }
  if (dimensions !== undefined && dimensions !== EMBEDDING_DIMENSIONS) return invalid('dimensions');
  if (format !== 'float' && format !== 'base64') return invalid('encoding_format');
  const texts = typeof input === 'string' ? [input] : input;
  if (!Array.isArray(texts) || texts.length === 0 || texts.length > MAX_INPUTS) return invalid('input');

  let usage = 0;
  for (const text of texts) {
    if (typeof text !== 'string' || !text.trim()) return invalid('input');
    const tokens = estimateTokens(text);
    if (tokens > MAX_INPUT_TOKENS) return invalid('input');
    usage += tokens;
  }
  const catalog = conceptIndex();
  if (!catalog) return messageError(500, 'embeddings_catalog_capacity');
  const data = texts.map((text, position) => {
    const vector = conceptVector(text, catalog);
    return { embedding: format === 'base64' ? encodeBase64(vector) : vector, index: position, object: 'embedding' };
  });
  return jsonResponse(200, sortedKeys({
    data, model: EMBEDDING_MODEL, object: 'list', usage: { prompt_tokens: usage, total_tokens: usage },
  }));
}
