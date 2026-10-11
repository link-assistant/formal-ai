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
//   JavaScript number prints `0` and `1`, and switches to exponent form at
//   other magnitudes (`1e+16`, `1e-6`). `f64` / `f32` wrap such a value so the
//   renderer prints it the way serde_json's float writer (the `zmij` crate)
//   does; `parseJson` keeps a parsed float a float, as `serde_json::Value`
//   does, so a body that is read and written back prints `1.0` again.

const FLOAT = Symbol('rustFloat');

/**
 * Wrap a number that Rust serializes as an `f64`.
 * @param {number} value
 * @returns {object}
 */
export function f64(value) {
  return { [FLOAT]: true, value: Number(value), bits: 64 };
}

/**
 * Wrap a number that Rust serializes as an `f32` (rounded to single precision).
 * @param {number} value
 * @returns {object}
 */
export function f32(value) {
  return { [FLOAT]: true, value: Math.fround(Number(value)), bits: 32 };
}

/** Whether `value` is a float marker. @param {unknown} value @returns {boolean} */
export function isFloat(value) {
  return Boolean(value && typeof value === 'object' && value[FLOAT] === true);
}

// `zmij::FloatTraits::FIXED_DEC_EXP`: the decimal exponents printed without
// exponent notation.
const FIXED_EXPONENTS = { 64: [-5, 15], 32: [-6, 12] };

/** The shortest round-tripping digits and decimal exponent of a finite, non-zero `value`. */
function shortestDigits(value, bits) {
  let text = value.toExponential();
  if (bits === 32) {
    for (let precision = 1; precision <= 9; precision += 1) {
      const candidate = value.toExponential(precision - 1);
      if (Math.fround(Number(candidate)) === value) {
        text = candidate;
        break;
      }
    }
  }
  const [mantissa, exponent] = text.split('e');
  const digits = mantissa.replace('-', '').replace('.', '').replace(/0+$/u, '') || '0';
  return { digits, exponent: Number(exponent) };
}

/**
 * Mirrors the `zmij` crate's `Buffer::format_finite` (serde_json's
 * `write_f64` / `write_f32` and `Number`'s `Display`).
 * @param {number} value
 * @param {number} [bits]
 * @returns {string}
 */
export function renderFloat(value, bits = 64) {
  if (!Number.isFinite(value)) return 'null';
  const sign = value < 0 || Object.is(value, -0) ? '-' : '';
  if (value === 0) return `${sign}0.0`;
  const { digits, exponent } = shortestDigits(Math.abs(value), bits);
  const [low, high] = FIXED_EXPONENTS[bits] || FIXED_EXPONENTS[64];
  if (exponent >= low && exponent <= high) {
    if (digits.length - 1 <= exponent) return `${sign}${digits}${'0'.repeat(exponent + 1 - digits.length)}.0`;
    if (exponent >= 0) return `${sign}${digits.slice(0, exponent + 1)}.${digits.slice(exponent + 1)}`;
    return `${sign}0.${'0'.repeat(-exponent - 1)}${digits}`;
  }
  const mantissa = digits.length > 1 ? `${digits[0]}.${digits.slice(1)}` : digits;
  return `${sign}${mantissa}e${exponent < 0 ? '-' : '+'}${Math.abs(exponent)}`;
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
  if (isFloat(value)) return renderFloat(value.value, value.bits);
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

const U64_MAX = (1n << 64n) - 1n;
const SCALAR = /true|false|null|-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?/uy;
const I64_MIN = -(1n << 63n);

/**
 * Mirrors serde_json's number parsing into a `Value`: an integer within
 * `u64` / `i64` stays an integer (a `bigint` past 2^53), anything with a
 * fraction or exponent, `-0`, or an out-of-range integer becomes an `f64`.
 * @param {string} token
 * @returns {number|bigint|object}
 */
function parsedNumber(token) {
  if (/^-?(0|[1-9][0-9]*)$/u.test(token)) {
    const integer = BigInt(token);
    if (token === '-0') return f64(-0);
    if (integer > U64_MAX || integer < I64_MIN) return f64(Number(token));
    return Number.isSafeInteger(Number(token)) ? Number(token) : integer;
  }
  return f64(Number(token));
}

/**
 * `serde_json::from_str::<Value>`: `JSON.parse` with every number kept as the
 * kind serde_json reads it as, so floats print back as floats.
 * Throws the `JSON.parse` error for invalid text.
 * @param {string} text
 * @returns {unknown}
 */
export function parseJson(text) {
  const plain = JSON.parse(text);
  if (!/[.eE]|-0|[0-9]{16,}/u.test(text)) return plain;
  let at = 0;
  const skip = () => {
    while (at < text.length && ' \t\n\r'.includes(text[at])) at += 1;
  };
  const value = () => {
    skip();
    const char = text[at];
    if (char === '{') {
      at += 1;
      const out = {};
      skip();
      if (text[at] === '}') {
        at += 1;
        return out;
      }
      for (;;) {
        skip();
        const key = string();
        skip();
        at += 1;
        const item = value();
        if (key === '__proto__') Object.defineProperty(out, key, { value: item, enumerable: true, writable: true, configurable: true });
        else out[key] = item;
        skip();
        if (text[at++] === '}') return out;
      }
    }
    if (char === '[') {
      at += 1;
      const out = [];
      skip();
      if (text[at] === ']') {
        at += 1;
        return out;
      }
      for (;;) {
        out.push(value());
        skip();
        if (text[at++] === ']') return out;
      }
    }
    if (char === '"') return string();
    SCALAR.lastIndex = at;
    const match = SCALAR.exec(text);
    at += match[0].length;
    if (match[0] === 'true') return true;
    if (match[0] === 'false') return false;
    if (match[0] === 'null') return null;
    return parsedNumber(match[0]);
  };
  const string = () => {
    const start = at;
    at += 1;
    while (text[at] !== '"') at += text[at] === '\\' ? 2 : 1;
    at += 1;
    return JSON.parse(text.slice(start, at));
  };
  return value();
}
