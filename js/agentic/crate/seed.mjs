// The seed readers the planner calls (rust/src/seed.rs: `response_for`,
// `response_variant_for`, `render_response`, `localized_response`,
// `response_values_for`; rust/src/seed/planner_precedence.rs).

import { cached, childValue, childrenNamed, parseLino, readText } from '../host.mjs';
import { languageFromSlug } from './language.mjs';

/** The one inventory of the seed files and the lexicons that read them. */
const SEED_REGISTRY = 'data/meta/seed-registry.lino';

/** `RESPONSE_FILES`: every seed the registry (data/meta/seed-registry.lino) gives the
 * `response` lexicon, in its order, as rust/src/seed/embedded_registry.rs lists them. */
export function responseFiles() {
  return cached('response-files', () => childrenNamed(parseLino(readText(SEED_REGISTRY)), 'seed')
    .filter((seed) => childrenNamed(seed, 'lexicon').some((lexicon) => lexicon.id === 'response'))
    .map((seed) => `data/seed/${seed.id}.lino`));
}

/** Mirrors `fn multilingual_responses`: `{id, intent, language, text, variants}` records. */
export function multilingualResponses() {
  return cached('multilingual-responses', () => {
    const out = [];
    for (const file of responseFiles()) {
      const root = parseLino(readText(file));
      for (const entry of childrenNamed(root, 'response')) {
        const intent = childValue(entry, 'intent');
        const language = childValue(entry, 'language');
        if (!intent || !language) continue;
        out.push({
          id: entry.id,
          intent,
          language,
          text: childValue(entry, 'text'),
          variants: childrenNamed(entry, 'variant').map((child) => child.id),
        });
      }
    }
    return out;
  });
}

function record(intent, language) {
  return multilingualResponses().find((entry) => entry.intent === intent && entry.language === language) ?? null;
}

/** Mirrors `fn response_for`. */
export function responseFor(intent, language) {
  return record(intent, language)?.text ?? null;
}

/** Mirrors `fn response_variant_for` (FNV-1a over the trimmed prompt bytes). */
export function responseVariantFor(intent, language, prompt) {
  const found = record(intent, language);
  if (!found) return null;
  if (!found.variants.length) return found.text;
  let hash = 0xcbf29ce484222325n;
  for (const byte of new TextEncoder().encode(prompt.trim())) {
    hash = ((hash ^ BigInt(byte)) * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  return found.variants[Number(hash % BigInt(found.variants.length))];
}

/** Mirrors `fn render_response`. @param {Array<[string, string]>} values */
export function renderResponse(intent, language, values) {
  const text = responseFor(intent, language);
  if (text === null) return null;
  return values.reduce((rendered, [name, value]) => rendered.split(`{${name}}`).join(value), text);
}

/** Mirrors `fn localized_response`. */
export function localizedResponse(intent, language) {
  const text = responseFor(intent, language);
  if (text !== null) return text;
  if (languageFromSlug(language) !== null) {
    const gap = responseFor(intent, 'unknown');
    if (gap !== null) return gap;
  }
  return responseFor(intent, 'en');
}

/** Mirrors `fn response_values_for` (`split_pipe_list`). */
export function responseValuesFor(intent, language) {
  const text = responseFor(intent, language);
  return text === null ? null : splitPipeList(text);
}

/** Mirrors `fn split_pipe_list` in rust/src/seed/parser.rs. */
export function splitPipeList(raw) {
  const trimmed = raw.trim();
  if (!trimmed) return [];
  if (trimmed.startsWith('(') && trimmed.endsWith(')') && trimmed.length >= 2) {
    return splitReferenceTokens(trimmed.slice(1, -1));
  }
  return trimmed.split('|').map((part) => part.trim()).filter(Boolean);
}

/** Mirrors `fn split_reference_tokens` in rust/src/seed/parser.rs. */
export function splitReferenceTokens(body) {
  const chars = Array.from(body);
  const tokens = [];
  let index = 0;
  while (index < chars.length) {
    const character = chars[index];
    if (/\s/u.test(character)) {
      index += 1;
    } else if (character === '"' || character === "'" || character === '`') {
      index += 1;
      let value = '';
      let escaped = false;
      while (index < chars.length) {
        const current = chars[index];
        index += 1;
        if (escaped) {
          value += current;
          escaped = false;
        } else if ((character === '"' || character === '`') && current === '\\') {
          escaped = true;
        } else if (current === character) {
          break;
        } else {
          value += current;
        }
      }
      tokens.push(value);
    } else {
      let value = '';
      while (index < chars.length && !/\s/u.test(chars[index])) {
        value += chars[index];
        index += 1;
      }
      if (value) tokens.push(value);
    }
  }
  return tokens;
}

/** Mirrors `fn planner_precedence` (rust/src/seed/planner_precedence.rs). */
export function plannerPrecedence() {
  return cached('planner-precedence', () =>
    parseLino(readText('data/seed/planner-precedence.lino')).children.map((child) => child.name));
}
