// `serde_json::Deserializer::from_str(..).into_iter::<Value>()` as the
// orchestration controller uses it (rust/src/orchestration/runner.rs
// `parse_native_session`, rust/src/orchestration/analysis.rs
// `extract_agent_result`): a text read as a stream of whitespace-separated
// JSON values, where the first malformed value ends the stream with an error.

const WHITESPACE = new Set([' ', '\t', '\n', '\r']);
const NUMBER = /-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?/y;
const LITERALS = ['true', 'false', 'null'];

/** Mirrors `serde_json::Deserializer whitespace skipping` in rust/src/orchestration/runner.rs. The index of the first non-whitespace character at or after `position`. */
function skipWhitespace(text, position) {
  let index = position;
  while (index < text.length && WHITESPACE.has(text[index])) index += 1;
  return index;
}

/**
 * Mirrors `serde_json::Deserializer string scanning` in rust/src/orchestration/runner.rs. The end of the string literal starting at `start` (a `"`), or -1 when unterminated.
 * @param {string} text
 * @param {number} start
 * @returns {number}
 */
function stringEnd(text, start) {
  for (let index = start + 1; index < text.length; index += 1) {
    if (text[index] === '\\') index += 1;
    else if (text[index] === '"') return index + 1;
  }
  return -1;
}

/** Mirrors `serde_json::Deserializer container scanning` in rust/src/orchestration/runner.rs. The end of the object or array starting at `start`, or -1 when it never closes. */
function containerEnd(text, start) {
  let depth = 0;
  for (let index = start; index < text.length; index += 1) {
    const character = text[index];
    if (character === '"') {
      index = stringEnd(text, index) - 1;
      if (index < 0) return -1;
    } else if (character === '{' || character === '[') {
      depth += 1;
    } else if (character === '}' || character === ']') {
      depth -= 1;
      if (depth === 0) return index + 1;
    }
  }
  return -1;
}

/** Mirrors `serde_json::Deserializer value scanning` in rust/src/orchestration/runner.rs. The end of the JSON value starting at `start`, or -1 when none starts there. */
function valueEnd(text, start) {
  const character = text[start];
  if (character === '{' || character === '[') return containerEnd(text, start);
  if (character === '"') return stringEnd(text, start);
  if (character === '-' || (character >= '0' && character <= '9')) {
    NUMBER.lastIndex = start;
    const match = NUMBER.exec(text);
    return match ? start + match[0].length : -1;
  }
  const literal = LITERALS.find((word) => text.startsWith(word, start));
  return literal ? start + literal.length : -1;
}

/**
 * Mirrors `serde_json::Deserializer::from_str(..).into_iter::<Value>().next()` in rust/src/orchestration/runner.rs.
 * Parse the JSON value that starts at `start`.
 * @returns {{value: *, end: number}|null} null when the text there is not a complete JSON value
 */
function valueAt(text, start) {
  const end = valueEnd(text, start);
  if (end < 0) return null;
  try {
    return { value: JSON.parse(text.slice(start, end)), end };
  } catch {
    return null;
  }
}

/**
 * Mirrors `Deserializer::from_str(text).into_iter::<Value>()`: yields
 * `{value}` for each value and a final `{error: true}` at the first malformed one.
 * Rust dependency `serde_json::StreamDeserializer`.
 * @param {string} text
 * @returns {Generator<{value?: *, error?: boolean}>}
 */
export function* jsonValueStream(text) {
  let position = 0;
  for (;;) {
    position = skipWhitespace(text, position);
    if (position >= text.length) return;
    const parsed = valueAt(text, position);
    if (parsed === null) {
      yield { error: true };
      return;
    }
    yield { value: parsed.value };
    position = parsed.end;
  }
}

/**
 * Mirrors `Deserializer::from_str(text).into_iter::<Value>().next()` in rust/src/orchestration/analysis.rs taken
 * as `Some(Ok(value))`: the first JSON value of `text`, or `undefined`.
 * Rust dependency `serde_json::StreamDeserializer::next`.
 * @param {string} text
 */
export function firstJsonValue(text) {
  const start = skipWhitespace(text, 0);
  return start >= text.length ? undefined : valueAt(text, start)?.value;
}

/** Rust dependency `serde_json::Map::values`, as rust/src/orchestration/runner.rs reads it: a `BTreeMap` visits keys in byte order. */
export function sortedValues(object) {
  return Object.keys(object)
    .sort((left, right) => Buffer.compare(Buffer.from(left), Buffer.from(right)))
    .map((key) => object[key]);
}

/** Whether `value` is a JSON object (`Value::Object`). */
export const isJsonObject = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);
