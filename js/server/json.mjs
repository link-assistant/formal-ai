// serde_json-compatible JSON rendering for the JavaScript server.
//
// The Rust server answers through `serde_json::to_string_pretty` (successes)
// and `Value::to_string` (errors and SSE frames). Two details of that output
// are not what `JSON.stringify` does on its own:
//
// * values built with the `json!` macro are `serde_json::Map`s, which are
//   B-tree maps, so their keys come out sorted; values serialized from a
//   `#[derive(Serialize)]` struct keep declaration order. `sortedKeys` marks
//   the former, plain objects keep insertion order for the latter.
// * a Rust float prints with a fractional part (`0.0`, `1.0`) where a
//   JavaScript number prints `0` and `1`. `f64` / `f32` wrap such a value so
//   the renderer can print it the Rust way.

const FLOAT = Symbol('rustFloat');

/**
 * Wrap a number that Rust serializes as a float.
 * @param {number} value
 * @returns {object}
 */
export function f64(value) {
  return { [FLOAT]: true, value: Number(value) };
}

export const f32 = f64;

/** @param {unknown} value @returns {boolean} */
function isFloat(value) {
  return Boolean(value && typeof value === 'object' && value[FLOAT] === true);
}

/** @param {number} value @returns {string} */
function renderFloat(value) {
  if (!Number.isFinite(value)) return 'null';
  if (Number.isInteger(value) && Math.abs(value) < 1e16) return `${value}.0`;
  return String(value);
}

/**
 * Deep copy with every object's keys sorted, mirroring a `json!` value.
 * Float markers and arrays are preserved.
 * @param {unknown} value
 * @returns {unknown}
 */
export function sortedKeys(value) {
  if (Array.isArray(value)) return value.map(sortedKeys);
  if (value && typeof value === 'object' && !isFloat(value)) {
    const out = {};
    for (const key of Object.keys(value).sort()) {
      if (value[key] !== undefined) out[key] = sortedKeys(value[key]);
    }
    return out;
  }
  return value;
}

/** @param {string} text @returns {string} */
function renderString(text) {
  return JSON.stringify(String(text));
}

/**
 * Render a value the way serde_json does.
 * @param {unknown} value
 * @param {boolean} pretty
 * @param {string} indent
 * @returns {string}
 */
function render(value, pretty, indent) {
  if (value === null || value === undefined) return 'null';
  if (isFloat(value)) return renderFloat(value.value);
  if (typeof value === 'number') return Number.isFinite(value) ? String(value) : 'null';
  if (typeof value === 'boolean') return value ? 'true' : 'false';
  if (typeof value === 'bigint') return value.toString();
  if (typeof value === 'string') return renderString(value);
  const inner = `${indent}  `;
  if (Array.isArray(value)) {
    if (value.length === 0) return '[]';
    const items = value.map((item) => render(item, pretty, inner));
    return pretty ? `[\n${inner}${items.join(`,\n${inner}`)}\n${indent}]` : `[${items.join(',')}]`;
  }
  const keys = Object.keys(value).filter((key) => value[key] !== undefined);
  if (keys.length === 0) return '{}';
  const entries = keys.map((key) =>
    `${renderString(key)}${pretty ? ': ' : ':'}${render(value[key], pretty, inner)}`,
  );
  return pretty ? `{\n${inner}${entries.join(`,\n${inner}`)}\n${indent}}` : `{${entries.join(',')}}`;
}

/** `serde_json::to_string_pretty`. @param {unknown} value @returns {string} */
export function toPrettyJson(value) {
  return render(value, true, '');
}

/** `serde_json::to_string` / `Value::to_string`. @param {unknown} value @returns {string} */
export function toCompactJson(value) {
  return render(value, false, '');
}

/**
 * Strip float markers, yielding plain data (for tests and internal reads).
 * @param {unknown} value
 * @returns {unknown}
 */
export function plainData(value) {
  if (isFloat(value)) return value.value;
  if (Array.isArray(value)) return value.map(plainData);
  if (value && typeof value === 'object') {
    const out = {};
    for (const [key, item] of Object.entries(value)) {
      if (item !== undefined) out[key] = plainData(item);
    }
    return out;
  }
  return value;
}
