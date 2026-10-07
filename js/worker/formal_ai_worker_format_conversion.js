// Browser twin of the issue #1177 JSON↔YAML format converter. Mirrors
// `handle_format_conversion` in rust/src/solver_handlers/format_conversion.rs:
// the same YAML subset (block mappings and sequences, two-space indent,
// scalars), the same refusal reasons (seed templates), and the same
// verify-before-show rule.
//
// Values are tagged records rather than native JS objects so the conversion is
// lossless exactly where serde_json's is: integer vs float numbers (`1` vs
// `1.0`), u64/i64 ranges, key order, and duplicate-key replacement all behave as
// they do in Rust. Number text follows serde_json (ryu) for JSON output and Rust
// `Display` for YAML output.
//
// Beyond Rust (issue #1177 R10): every conversion is checked in BOTH directions
// before it is shown — JSON→YAML re-parses the YAML and re-emits JSON, YAML→JSON
// re-parses the JSON and re-renders YAML — and must reproduce the input value.

// ---------------------------------------------------------------------------
// Tagged values
// ---------------------------------------------------------------------------

/**
 * A tagged scalar or container value.
 * @param {string} kind null | bool | number | string | array | object
 * @param {*} payload kind-specific payload
 * @returns {object} the tagged value
 */
function formatValue(kind, payload) {
  switch (kind) {
    case "null":
      return { kind: "null" };
    case "bool":
      return { kind: "bool", value: payload };
    case "string":
      return { kind: "string", value: payload };
    case "array":
      return { kind: "array", items: payload };
    case "object":
      return { kind: "object", entries: payload };
    default:
      throw new Error("unknown format value kind");
  }
}

/**
 * A tagged number: "pos" (u64), "neg" (negative i64) or "float" (f64).
 * @param {string} repr number representation
 * @param {string} text canonical integer text (ints only)
 * @param {number} value numeric value
 * @returns {object} the tagged number
 */
function formatNumber(repr, text, value) {
  return { kind: "number", repr: repr, text: text, value: value };
}

/**
 * The tagged number for an i64 parse (serde `From<i64>`).
 * @param {bigint} big integer value
 * @returns {object} tagged number
 */
function formatNumberFromI64(big) {
  return formatNumber(big < 0n ? "neg" : "pos", big.toString(), Number(big));
}

/**
 * Insert or replace a key in an entry list (IndexMap semantics).
 * @param {Array<Array>} entries [key, value] pairs
 * @param {string} key the key
 * @param {object} value the value
 * @returns {void}
 */
function formatObjectInsert(entries, key, value) {
  for (const entry of entries) {
    if (entry[0] === key) {
      entry[1] = value;
      return;
    }
  }
  entries.push([key, value]);
}

/**
 * Structural equality (serde `Value == Value`; object order ignored).
 * @param {object} left tagged value
 * @param {object} right tagged value
 * @returns {boolean} whether equal
 */
function formatValuesEqual(left, right) {
  if (left.kind !== right.kind) return false;
  switch (left.kind) {
    case "null":
      return true;
    case "bool":
    case "string":
      return left.value === right.value;
    case "number":
      if (left.repr !== right.repr) return false;
      return left.repr === "float" ? left.value === right.value : left.text === right.text;
    case "array":
      if (left.items.length !== right.items.length) return false;
      for (let index = 0; index < left.items.length; index += 1) {
        if (!formatValuesEqual(left.items[index], right.items[index])) return false;
      }
      return true;
    case "object":
      if (left.entries.length !== right.entries.length) return false;
      for (const entry of left.entries) {
        let match = null;
        for (const other of right.entries) {
          if (other[0] === entry[0]) {
            match = other;
            break;
          }
        }
        if (match === null || !formatValuesEqual(entry[1], match[1])) return false;
      }
      return true;
    default:
      return false;
  }
}

// ---------------------------------------------------------------------------
// Number text
// ---------------------------------------------------------------------------

/**
 * The shortest round-trip decimal digits of a finite, non-negative number and
 * the decimal-point position (`0.d1d2… × 10^point`).
 * @param {number} magnitude non-negative finite number
 * @returns {object} {digits, point}
 */
function formatShortestDigits(magnitude) {
  const exponential = magnitude.toExponential();
  const marker = exponential.indexOf("e");
  const mantissa = exponential.slice(0, marker).replace(".", "");
  const exponent = Number(exponential.slice(marker + 1));
  return { digits: mantissa, point: exponent + 1 };
}

/**
 * Rust `f64::to_string` (Display): shortest digits, never an exponent.
 * @param {number} value finite number
 * @returns {string} display text
 */
function formatRustFloatDisplay(value) {
  const negative = value < 0 || Object.is(value, -0);
  const shortest = formatShortestDigits(Math.abs(value));
  const digits = shortest.digits;
  const point = shortest.point;
  let text = "";
  if (digits === "0") {
    text = "0";
  } else if (point <= 0) {
    text = "0." + "0".repeat(-point) + digits;
  } else if (point >= digits.length) {
    text = digits + "0".repeat(point - digits.length);
  } else {
    text = digits.slice(0, point) + "." + digits.slice(point);
  }
  return (negative ? "-" : "") + text;
}

/**
 * serde_json's float text (ryu `format_finite`).
 * @param {number} value finite number
 * @returns {string} JSON number text
 */
function formatRyuFloat(value) {
  const negative = value < 0 || Object.is(value, -0);
  const shortest = formatShortestDigits(Math.abs(value));
  const digits = shortest.digits;
  const length = digits.length;
  const kk = shortest.point;
  const k = kk - length;
  let text = "";
  if (digits === "0") {
    text = "0.0";
  } else if (k >= 0 && kk <= 16) {
    text = digits + "0".repeat(k) + ".0";
  } else if (kk > 0 && kk <= 16) {
    text = digits.slice(0, kk) + "." + digits.slice(kk);
  } else if (kk > -5 && kk <= 0) {
    text = "0." + "0".repeat(-kk) + digits;
  } else if (length === 1) {
    text = digits + "e" + (kk - 1);
  } else {
    text = digits.slice(0, 1) + "." + digits.slice(1) + "e" + (kk - 1);
  }
  return (negative ? "-" : "") + text;
}

/**
 * True when Rust `str::parse::<f64>` accepts the token.
 * @param {string} token candidate text
 * @returns {boolean} whether it parses
 */
function formatParsesAsFloat(token) {
  return /^[+-]?(((\d+(\.\d*)?)|(\.\d+))([eE][+-]?\d+)?|inf|infinity|nan)$/i.test(token);
}

// ---------------------------------------------------------------------------
// JSON (serde_json-compatible parse and pretty print)
// ---------------------------------------------------------------------------

/**
 * Parse a JSON document into a tagged value.
 * @param {string} text JSON text
 * @returns {object} {ok, value} or {ok: false, error}
 */
function formatJsonParse(text) {
  const state = { text: text, index: 0, error: "" };
  const value = formatJsonParseValue(state);
  if (value === null) return { ok: false, error: state.error };
  formatJsonSkipWhitespace(state);
  if (state.index < text.length) {
    return { ok: false, error: "trailing characters at " + formatJsonPosition(state) };
  }
  return { ok: true, value: value };
}

/**
 * The "line L column C" position of the parser cursor.
 * @param {object} state parser state
 * @returns {string} position text
 */
function formatJsonPosition(state) {
  const before = state.text.slice(0, Math.min(state.index + 1, state.text.length));
  const lines = before.split("\n");
  return "line " + lines.length + " column " + lines[lines.length - 1].length;
}

/**
 * Record a parse error and return null.
 * @param {object} state parser state
 * @param {string} message error message
 * @returns {null} always null
 */
function formatJsonFail(state, message) {
  if (state.error === "") state.error = message + " at " + formatJsonPosition(state);
  return null;
}

/**
 * Skip JSON whitespace.
 * @param {object} state parser state
 * @returns {void}
 */
function formatJsonSkipWhitespace(state) {
  while (state.index < state.text.length && " \t\n\r".includes(state.text[state.index])) state.index += 1;
}

/**
 * Parse one JSON value at the cursor.
 * @param {object} state parser state
 * @returns {object|null} tagged value, or null on error
 */
function formatJsonParseValue(state) {
  formatJsonSkipWhitespace(state);
  if (state.index >= state.text.length) return formatJsonFail(state, "EOF while parsing a value");
  const ch = state.text[state.index];
  switch (ch) {
    case "{":
      return formatJsonParseObject(state);
    case "[":
      return formatJsonParseArray(state);
    case "\"": {
      const text = formatJsonParseString(state);
      return text === null ? null : formatValue("string", text);
    }
    case "t":
      return formatJsonParseKeyword(state, "true", formatValue("bool", true));
    case "f":
      return formatJsonParseKeyword(state, "false", formatValue("bool", false));
    case "n":
      return formatJsonParseKeyword(state, "null", formatValue("null", null));
    default:
      if (ch === "-" || (ch >= "0" && ch <= "9")) return formatJsonParseNumber(state);
      return formatJsonFail(state, "expected value");
  }
}

/**
 * Parse a literal keyword.
 * @param {object} state parser state
 * @param {string} word keyword text
 * @param {object} value tagged value to return
 * @returns {object|null} the value, or null
 */
function formatJsonParseKeyword(state, word, value) {
  if (state.text.slice(state.index, state.index + word.length) !== word) {
    return formatJsonFail(state, "expected ident");
  }
  state.index += word.length;
  return value;
}

/**
 * Parse a JSON number with serde_json's integer/float split.
 * @param {object} state parser state
 * @returns {object|null} tagged number, or null
 */
function formatJsonParseNumber(state) {
  const match = /^-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?/.exec(state.text.slice(state.index));
  if (match === null) return formatJsonFail(state, "invalid number");
  const literal = match[0];
  state.index += literal.length;
  if (match[2] === undefined && match[3] === undefined) {
    const big = BigInt(literal);
    if (big >= 0n && big <= 18446744073709551615n && !literal.startsWith("-")) {
      return formatNumber("pos", big.toString(), Number(big));
    }
    if (big < 0n && big >= -9223372036854775808n) return formatNumber("neg", big.toString(), Number(big));
  }
  const value = Number(literal);
  if (!Number.isFinite(value)) return formatJsonFail(state, "number out of range");
  return formatNumber("float", "", value);
}

/**
 * Parse a JSON string literal at the cursor.
 * @param {object} state parser state
 * @returns {string|null} the decoded string, or null
 */
function formatJsonParseString(state) {
  state.index += 1;
  let out = "";
  while (state.index < state.text.length) {
    const ch = state.text[state.index];
    if (ch === "\"") {
      state.index += 1;
      return out;
    }
    if (ch === "\\") {
      const escape = state.text[state.index + 1];
      const simple = { "\"": "\"", "\\": "\\", "/": "/", b: "\b", f: "\f", n: "\n", r: "\r", t: "\t" };
      if (escape !== undefined && Object.prototype.hasOwnProperty.call(simple, escape)) {
        out += simple[escape];
        state.index += 2;
        continue;
      }
      if (escape === "u") {
        const hex = state.text.slice(state.index + 2, state.index + 6);
        if (!/^[0-9a-fA-F]{4}$/.test(hex)) return formatJsonFail(state, "invalid escape");
        out += String.fromCharCode(parseInt(hex, 16));
        state.index += 6;
        continue;
      }
      return formatJsonFail(state, "invalid escape");
    }
    if (ch.charCodeAt(0) < 0x20) return formatJsonFail(state, "control character (\\u0000-\\u001F) found while parsing a string");
    out += ch;
    state.index += 1;
  }
  return formatJsonFail(state, "EOF while parsing a string");
}

/**
 * Parse a JSON array at the cursor.
 * @param {object} state parser state
 * @returns {object|null} tagged array, or null
 */
function formatJsonParseArray(state) {
  state.index += 1;
  const items = [];
  formatJsonSkipWhitespace(state);
  if (state.text[state.index] === "]") {
    state.index += 1;
    return formatValue("array", items);
  }
  for (;;) {
    const item = formatJsonParseValue(state);
    if (item === null) return null;
    items.push(item);
    formatJsonSkipWhitespace(state);
    const ch = state.text[state.index];
    if (ch === ",") {
      state.index += 1;
      continue;
    }
    if (ch === "]") {
      state.index += 1;
      return formatValue("array", items);
    }
    return formatJsonFail(state, ch === undefined ? "EOF while parsing a list" : "expected `,` or `]`");
  }
}

/**
 * Parse a JSON object at the cursor.
 * @param {object} state parser state
 * @returns {object|null} tagged object, or null
 */
function formatJsonParseObject(state) {
  state.index += 1;
  const entries = [];
  formatJsonSkipWhitespace(state);
  if (state.text[state.index] === "}") {
    state.index += 1;
    return formatValue("object", entries);
  }
  for (;;) {
    formatJsonSkipWhitespace(state);
    if (state.text[state.index] !== "\"") {
      return formatJsonFail(state, state.index >= state.text.length ? "EOF while parsing an object" : "key must be a string");
    }
    const key = formatJsonParseString(state);
    if (key === null) return null;
    formatJsonSkipWhitespace(state);
    if (state.text[state.index] !== ":") return formatJsonFail(state, "expected `:`");
    state.index += 1;
    const value = formatJsonParseValue(state);
    if (value === null) return null;
    formatObjectInsert(entries, key, value);
    formatJsonSkipWhitespace(state);
    const ch = state.text[state.index];
    if (ch === ",") {
      state.index += 1;
      continue;
    }
    if (ch === "}") {
      state.index += 1;
      return formatValue("object", entries);
    }
    return formatJsonFail(state, ch === undefined ? "EOF while parsing an object" : "expected `,` or `}`");
  }
}

/**
 * The JSON text of a scalar (serde_json compact form).
 * @param {object} value tagged scalar
 * @returns {string} JSON text
 */
function formatJsonScalar(value) {
  switch (value.kind) {
    case "null":
      return "null";
    case "bool":
      return value.value ? "true" : "false";
    case "number":
      return value.repr === "float" ? formatRyuFloat(value.value) : value.text;
    case "string":
      return JSON.stringify(value.value);
    default:
      return "";
  }
}

/**
 * serde_json `to_string_pretty`: two-space indent.
 * @param {object} value tagged value
 * @param {string} indent current indentation
 * @returns {string} JSON text
 */
function formatJsonPretty(value, indent) {
  const inner = indent + "  ";
  switch (value.kind) {
    case "array": {
      if (value.items.length === 0) return "[]";
      const parts = [];
      for (const item of value.items) parts.push(inner + formatJsonPretty(item, inner));
      return "[\n" + parts.join(",\n") + "\n" + indent + "]";
    }
    case "object": {
      if (value.entries.length === 0) return "{}";
      const parts = [];
      for (const entry of value.entries) {
        parts.push(inner + JSON.stringify(entry[0]) + ": " + formatJsonPretty(entry[1], inner));
      }
      return "{\n" + parts.join(",\n") + "\n" + indent + "}";
    }
    default:
      return formatJsonScalar(value);
  }
}

// ---------------------------------------------------------------------------
// YAML subset rendering and parsing
// ---------------------------------------------------------------------------

/**
 * True when a string renders safely as a plain YAML scalar.
 * @param {string} text candidate scalar
 * @returns {boolean} whether it may stay unquoted
 */
function formatYamlPlainSafe(text) {
  if (text === "") return false;
  const first = Array.from(text)[0];
  if (!(codeTaskIsAlphanumeric(first) || first === "_")) return false;
  if (text.trim() !== text) return false;
  for (const marker of [":", "#", "\"", "'", "\n", "\t"]) {
    if (text.includes(marker)) return false;
  }
  if (["true", "false", "null", "yes", "no", "on", "off", "~"].includes(text)) return false;
  return !formatParsesAsFloat(text);
}

/**
 * Render one scalar as a YAML token.
 * @param {object} value tagged scalar
 * @returns {string} YAML token
 */
function formatYamlScalar(value) {
  switch (value.kind) {
    case "null":
      return "null";
    case "bool":
      return value.value ? "true" : "false";
    case "number":
      if (value.repr !== "float") return value.text;
      if (Number.isInteger(value.value)) return formatRustFloatDisplay(Math.trunc(value.value)) + ".0";
      return formatRustFloatDisplay(value.value);
    case "string":
      return formatYamlPlainSafe(value.value) ? value.value : JSON.stringify(value.value);
    default:
      return "";
  }
}

/**
 * Render a value as block YAML at the given indent width.
 * @param {object} value tagged value
 * @param {number} indent indent width
 * @returns {string} YAML text
 */
function formatYamlRender(value, indent) {
  const pad = " ".repeat(indent);
  const child = function (lead, item) {
    if (item.kind === "object" && item.entries.length > 0) return lead + ":\n" + formatYamlRender(item, indent + 2);
    if (item.kind === "array" && item.items.length > 0) return lead + ":\n" + formatYamlRender(item, indent + 2);
    if (item.kind === "object") return lead + ": {}\n";
    if (item.kind === "array") return lead + ": []\n";
    return lead + ": " + formatYamlScalar(item) + "\n";
  };
  switch (value.kind) {
    case "object": {
      let out = "";
      for (const entry of value.entries) {
        const key = formatYamlPlainSafe(entry[0]) ? entry[0] : JSON.stringify(entry[0]);
        out += child(pad + key, entry[1]);
      }
      return out;
    }
    case "array": {
      let out = "";
      for (const item of value.items) {
        if ((item.kind === "object" && item.entries.length > 0) || (item.kind === "array" && item.items.length > 0)) {
          out += pad + "-\n" + formatYamlRender(item, indent + 2);
        } else if (item.kind === "object") {
          out += pad + "- {}\n";
        } else if (item.kind === "array") {
          out += pad + "- []\n";
        } else {
          out += pad + "- " + formatYamlScalar(item) + "\n";
        }
      }
      return out;
    }
    default:
      return indent === 0 ? formatYamlScalar(value) + "\n" : formatYamlScalar(value);
  }
}

/**
 * Parse a YAML scalar token.
 * @param {string} token scalar text
 * @returns {object} tagged value
 */
function formatYamlParseScalar(token) {
  if (token.startsWith("\"")) {
    const parsed = formatJsonParse(token);
    if (parsed.ok && parsed.value.kind === "string") return parsed.value;
    return formatValue("string", codeTaskTrimChar(token, "\""));
  }
  switch (token) {
    case "null":
    case "~":
      return formatValue("null", null);
    case "true":
      return formatValue("bool", true);
    case "false":
      return formatValue("bool", false);
    default:
      break;
  }
  if (/^[+-]?[0-9]+$/.test(token)) {
    const big = BigInt(token.startsWith("+") ? token.slice(1) : token);
    if (big >= -9223372036854775808n && big <= 9223372036854775807n) return formatNumberFromI64(big);
    if (big >= 0n && big <= 18446744073709551615n) return formatNumber("pos", big.toString(), Number(big));
  }
  if (formatParsesAsFloat(token)) {
    const value = Number(token);
    if (Number.isFinite(value)) return formatNumber("float", "", value);
  }
  return formatValue("string", token);
}

/**
 * Split a mapping line into [key, value text]; null when it is not one.
 * @param {string} content trimmed line content
 * @returns {Array<string>|null} [key, value]
 */
function formatYamlSplitMappingLine(content) {
  if (content.startsWith("\"")) {
    let index = 1;
    while (index < content.length) {
      if (content[index] === "\\") {
        index += 2;
      } else if (content[index] === "\"") {
        if (content[index + 1] !== ":") return null;
        const key = formatJsonParse(content.slice(0, index + 1));
        if (!key.ok || key.value.kind !== "string") return null;
        return [key.value.value, content.slice(index + 2).trim()];
      } else {
        index += 1;
      }
    }
    return null;
  }
  const colon = content.indexOf(":");
  if (colon === -1) return null;
  const key = content.slice(0, colon).trim();
  if (key === "") return null;
  return [key, content.slice(colon + 1).trim()];
}

/**
 * True when a YAML line is a sequence entry.
 * @param {object} line {indent, content}
 * @returns {boolean} whether it starts with a dash
 */
function formatYamlIsEntry(line) {
  return line.content === "-" || line.content.startsWith("- ");
}

/**
 * Parse a block at `indent` (Rust `parse_block`).
 * @param {Array<object>} lines significant lines
 * @param {number} start first line index
 * @param {number} indent block indent
 * @returns {Array|null} [value, next index], or null
 */
function formatYamlParseBlock(lines, start, indent) {
  if (start >= lines.length || lines[start].indent !== indent) return null;
  if (formatYamlIsEntry(lines[start])) {
    const items = [];
    let index = start;
    while (index < lines.length && lines[index].indent === indent && formatYamlIsEntry(lines[index])) {
      const rest = lines[index].content.startsWith("- ") ? lines[index].content.slice(2) : "";
      if (rest === "") {
        const next = lines[index + 1];
        if (next !== undefined && next.indent > indent) {
          const parsed = formatYamlParseBlock(lines, index + 1, next.indent);
          if (parsed === null) return null;
          items.push(parsed[0]);
          index = parsed[1];
        } else {
          items.push(formatValue("null", null));
          index += 1;
        }
      } else if (rest.indexOf(": ") !== -1 || rest.endsWith(":")) {
        const itemIndent = indent + 2;
        const itemLines = [{ indent: itemIndent, content: rest }];
        let scan = index + 1;
        while (scan < lines.length && lines[scan].indent === itemIndent && !lines[scan].content.startsWith("- ")) {
          itemLines.push({ indent: itemIndent, content: lines[scan].content });
          scan += 1;
        }
        const parsed = formatYamlParseBlock(itemLines, 0, itemIndent);
        if (parsed === null) return null;
        items.push(parsed[0]);
        index = scan;
      } else if (rest === "{}" || rest === "[]") {
        // The renderer emits `- {}` / `- []` for empty containers inside a
        // sequence; Rust's parser reads them back as the strings "{}" / "[]",
        // so its round-trip check refuses such documents. Read them back as
        // the containers they render.
        items.push(rest === "{}" ? formatValue("object", []) : formatValue("array", []));
        index += 1;
      } else {
        items.push(formatYamlParseScalar(rest));
        index += 1;
      }
    }
    return [formatValue("array", items), index];
  }
  const entries = [];
  let index = start;
  while (index < lines.length && lines[index].indent === indent && !formatYamlIsEntry(lines[index])) {
    const pair = formatYamlSplitMappingLine(lines[index].content);
    if (pair === null) return null;
    const key = pair[0];
    const valueText = pair[1];
    if (valueText === "") {
      const next = lines[index + 1];
      if (next !== undefined && next.indent > indent) {
        const parsed = formatYamlParseBlock(lines, index + 1, next.indent);
        if (parsed === null) return null;
        formatObjectInsert(entries, key, parsed[0]);
        index = parsed[1];
      } else {
        formatObjectInsert(entries, key, formatValue("null", null));
        index += 1;
      }
    } else if (valueText === "{}") {
      formatObjectInsert(entries, key, formatValue("object", []));
      index += 1;
    } else if (valueText === "[]") {
      formatObjectInsert(entries, key, formatValue("array", []));
      index += 1;
    } else {
      formatObjectInsert(entries, key, formatYamlParseScalar(valueText));
      index += 1;
    }
  }
  return [formatValue("object", entries), index];
}

/**
 * Parse a YAML-subset document.
 * @param {string} text YAML text
 * @returns {object|null} tagged value, or null outside the subset
 */
function formatYamlParse(text) {
  const trimmed = text.trim();
  if (trimmed === "{}") return formatValue("object", []);
  if (trimmed === "[]") return formatValue("array", []);
  const lines = [];
  for (const raw of codeTaskLines(text)) {
    const content = raw.trimEnd();
    const significant = content.trimStart();
    if (significant === "" || significant.startsWith("#")) continue;
    lines.push({ indent: content.length - significant.length, content: significant });
  }
  if (lines.length === 0) return null;
  const parsed = formatYamlParseBlock(lines, 0, lines[0].indent);
  if (parsed === null || parsed[1] !== lines.length) return null;
  return parsed[0];
}

// ---------------------------------------------------------------------------
// Extraction of the document under discussion
// ---------------------------------------------------------------------------

/**
 * The body of the first fenced block tagged `tag`.
 * @param {string} prompt raw prompt
 * @param {string} tag fence tag
 * @returns {string|null} body
 */
function formatConversionFenced(prompt, tag) {
  const open = "```" + tag + "\n";
  const at = prompt.indexOf(open);
  if (at === -1) return null;
  const start = at + open.length;
  const end = prompt.indexOf("```", start);
  if (end === -1) return null;
  const body = prompt.slice(start, end);
  return body.trim() === "" ? null : body;
}

/**
 * The body of the first fenced block with any tag.
 * @param {string} prompt raw prompt
 * @returns {string|null} body
 */
function formatConversionFencedAny(prompt) {
  const at = prompt.indexOf("```");
  if (at === -1) return null;
  const rest = prompt.slice(at + 3);
  const newline = rest.indexOf("\n");
  if (newline === -1) return null;
  const afterTag = rest.slice(newline + 1);
  const end = afterTag.indexOf("```");
  if (end === -1) return null;
  const body = afterTag.slice(0, end);
  return body.trim() === "" ? null : body;
}

/**
 * The first non-blank backtick span.
 * @param {string} prompt raw prompt
 * @returns {string|null} span
 */
function formatConversionBacktickSpan(prompt) {
  const start = prompt.indexOf("`");
  if (start === -1) return null;
  const end = prompt.indexOf("`", start + 1);
  if (end === -1) return null;
  const span = prompt.slice(start + 1, end);
  return span.trim() === "" ? null : span;
}

/**
 * The first brace-balanced JSON document in the prompt (string-aware).
 * @param {string} prompt raw prompt
 * @returns {string|null} JSON text
 */
function formatConversionJsonSpan(prompt) {
  const brace = prompt.indexOf("{");
  const bracket = prompt.indexOf("[");
  let start = -1;
  if (brace === -1) {
    start = bracket;
  } else if (bracket === -1) {
    start = brace;
  } else {
    start = Math.min(brace, bracket);
  }
  if (start === -1) return null;
  let depth = 0;
  let inString = false;
  let escaped = false;
  for (let index = start; index < prompt.length; index += 1) {
    const ch = prompt[index];
    if (inString) {
      if (ch === "\\" && !escaped) {
        escaped = true;
      } else if (ch === "\"" && !escaped) {
        inString = false;
      } else {
        escaped = false;
      }
      continue;
    }
    if (ch === "\"") {
      inString = true;
    } else if (ch === "{" || ch === "[") {
      depth += 1;
    } else if (ch === "}" || ch === "]") {
      depth -= 1;
      if (depth === 0) return prompt.slice(start, index + 1);
    }
  }
  return null;
}

/**
 * The JSON document under discussion.
 * @param {string} prompt raw prompt
 * @returns {string|null} JSON text
 */
function formatConversionJsonText(prompt) {
  let text = formatConversionFenced(prompt, "json");
  if (text === null) text = formatConversionFencedAny(prompt);
  if (text === null) {
    const span = formatConversionBacktickSpan(prompt);
    if (span !== null && (span.includes("{") || span.includes("["))) text = span;
  }
  if (text === null) text = formatConversionJsonSpan(prompt);
  return text;
}

/**
 * The YAML under discussion.
 * @param {string} prompt raw prompt
 * @returns {string|null} YAML text
 */
function formatConversionYamlText(prompt) {
  let body = formatConversionFenced(prompt, "yaml");
  if (body === null) body = formatConversionFencedAny(prompt);
  if (body !== null) return body;
  const span = formatConversionBacktickSpan(prompt);
  if (span !== null) return span;
  let seen = 0;
  for (const line of codeTaskLines(prompt)) {
    const start = seen;
    seen += line.length + 1;
    const trimmed = line.trim();
    if (trimmed.indexOf(": ") !== -1 || trimmed.startsWith("- ") || trimmed.endsWith(":")) return prompt.slice(start);
  }
  return null;
}

/**
 * The CSV table under discussion: a fenced block, else the text after the
 * request's first colon when it spans at least two lines (Rust `csv_text`).
 * @param {string} prompt raw prompt
 * @returns {string|null} CSV text
 */
function formatConversionCsvText(prompt) {
  let body = formatConversionFenced(prompt, "csv");
  if (body === null) body = formatConversionFencedAny(prompt);
  if (body !== null) return body;
  const colon = prompt.search(/[:：]/u);
  if (colon === -1) return null;
  const rest = prompt.slice(colon + (prompt[colon] === ":" ? 1 : "：".length)).trim();
  return rest.includes(",") && rest.includes("\n") ? rest : null;
}

/**
 * Parse the CSV subset: comma-separated fields, double-quoted fields with
 * doubled quotes inside, one record per line, a header of unique non-empty
 * names and at least one row of the same width (Rust `parse_csv`).
 * @param {string} text CSV text
 * @returns {Array<Array<string>>|null} header then rows, or null outside the subset
 */
function formatConversionParseCsv(text) {
  const records = [];
  let record = [];
  let field = "";
  let quoted = false;
  let closed = false;
  const chars = Array.from(text.split("\r\n").join("\n"));
  for (let index = 0; index < chars.length; index += 1) {
    const ch = chars[index];
    if (quoted) {
      if (ch === "\"" && chars[index + 1] === "\"") {
        field += "\"";
        index += 1;
      } else if (ch === "\"") {
        quoted = false;
        closed = true;
      } else {
        field += ch;
      }
    } else if (ch === ",") {
      record.push(field);
      field = "";
      closed = false;
    } else if (ch === "\n") {
      record.push(field);
      records.push(record);
      record = [];
      field = "";
      closed = false;
    } else if (closed) {
      return null;
    } else if (ch === "\"" && field === "") {
      quoted = true;
    } else {
      field += ch;
    }
  }
  if (quoted) return null;
  record.push(field);
  records.push(record);
  const rows = records.filter((row) => !(row.length === 1 && row[0].trim() === ""));
  if (rows.length < 2) return null;
  const header = rows[0];
  if (header.some((name) => name === "") || new Set(header).size !== header.length) return null;
  if (rows.some((row) => row.length !== header.length)) return null;
  return rows;
}

/**
 * Render CSV rows as a JSON array of objects keyed by the header, two-space
 * indented, every value a string (Rust `render_csv_json`).
 * @param {Array<Array<string>>} rows header then rows
 * @returns {string} JSON text with a trailing newline
 */
function formatConversionCsvJson(rows) {
  const header = rows[0];
  const objects = rows.slice(1).map((row) => "  {\n" + header
    .map((name, column) => "    " + JSON.stringify(name) + ": " + JSON.stringify(row[column]))
    .join(",\n") + "\n  }");
  return "[\n" + objects.join(",\n") + "\n]\n";
}

/**
 * Convert a CSV table to JSON and check the result parses back to the same
 * cells (Rust `convert_csv`).
 * @param {string} prompt raw prompt
 * @param {Array<Array<string>>} log handler log
 * @returns {Array} [body, confidence]
 */
function formatConversionCsvToJson(prompt, log) {
  const text = formatConversionCsvText(prompt);
  const rows = text === null ? null : formatConversionParseCsv(text);
  if (rows === null) {
    codeTaskLogAppend(log, "format_conversion:refusal", text === null ? "csv=none" : "csv=bad");
    return [formatConversionRefusal("csv"), 0.4];
  }
  const json = formatConversionCsvJson(rows);
  let reparsed = null;
  try {
    reparsed = JSON.parse(json);
  } catch {
    reparsed = null;
  }
  const header = rows[0];
  const matches = Array.isArray(reparsed) && reparsed.length === rows.length - 1 && reparsed.every((object, index) =>
    object !== null && typeof object === "object" && Object.keys(object).length === header.length
      && header.every((name, column) => object[name] === rows[index + 1][column]));
  if (!matches) {
    codeTaskLogAppend(log, "format_conversion:refusal", "roundtrip=fail");
    return [formatConversionRefusal("roundtrip"), 0.4];
  }
  codeTaskLogAppend(log, "format_conversion:converted", "roundtrip=ok");
  return [codeTaskTemplate("format_conversion_csv_to_json", [
    ["rows", String(rows.length - 1)],
    ["columns", header.join(", ")],
    ["json", json],
  ]), 0.7];
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/**
 * A refusal body for a named reason code.
 * @param {string} reason reason code
 * @returns {string} refusal text
 */
function formatConversionRefusal(reason) {
  return codeTaskTemplate("format_conversion_refusal", [
    ["reason", codeTaskTemplate("format_conversion_reason_" + reason, [])],
  ]);
}

/**
 * Convert JSON↔YAML within the subset, verifying both directions first.
 * Mirrors `handle_format_conversion`.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleFormatConversion(prompt, normalized) {
  const toYaml = codeTaskCued("format_conversion", "to_yaml", prompt, normalized);
  const toJson = codeTaskCued("format_conversion", "to_json", prompt, normalized);
  const fromCsv = !toYaml && !toJson && codeTaskCued("format_conversion", "csv_to_json", prompt, normalized);
  if (!toYaml && !toJson && !fromCsv) return null;
  const log = codeTaskLog();
  if (fromCsv) {
    codeTaskLogAppend(log, "format_conversion:request", "dir=csv");
    const converted = formatConversionCsvToJson(prompt, log);
    return codeTaskAnswer(log, "format_conversion", "response:format_conversion", converted[0], converted[1]);
  }
  codeTaskLogAppend(log, "format_conversion:request", "dir=" + (toYaml ? "yaml" : "json"));
  let body = "";
  let confidence = 0.4;
  if (toYaml) {
    const text = formatConversionJsonText(prompt);
    if (text === null) {
      codeTaskLogAppend(log, "format_conversion:refusal", "json=none");
      body = formatConversionRefusal("subset");
    } else {
      const parsed = formatJsonParse(text);
      if (!parsed.ok) {
        codeTaskLogAppend(log, "format_conversion:refusal", "json=bad");
        body = codeTaskTemplate("format_conversion_refusal", [
          ["reason", codeTaskTemplate("format_conversion_reason_bad_json", [["detail", parsed.error]])],
        ]);
      } else {
        let yaml = formatYamlRender(parsed.value, 0);
        if (yaml === "") yaml = parsed.value.kind === "object" ? "{}" : "[]";
        const reparsed = formatYamlParse(yaml);
        const reverse = reparsed === null ? null : formatJsonParse(formatJsonPretty(reparsed, ""));
        if (reparsed !== null && formatValuesEqual(reparsed, parsed.value) &&
            reverse !== null && reverse.ok && formatValuesEqual(reverse.value, parsed.value)) {
          codeTaskLogAppend(log, "format_conversion:converted", "roundtrip=ok");
          codeTaskLogAppend(log, "format_conversion:reverse_roundtrip", "json=ok");
          body = codeTaskTemplate("format_conversion_to_yaml", [["yaml", yaml]]);
          confidence = 0.7;
        } else {
          codeTaskLogAppend(log, "format_conversion:refusal", "roundtrip=fail");
          body = formatConversionRefusal("roundtrip");
        }
      }
    }
  } else {
    const text = formatConversionYamlText(prompt);
    const value = text === null ? null : formatYamlParse(text);
    if (text === null) {
      codeTaskLogAppend(log, "format_conversion:refusal", "yaml=none");
      body = formatConversionRefusal("subset");
    } else if (value === null) {
      codeTaskLogAppend(log, "format_conversion:refusal", "yaml=bad");
      body = formatConversionRefusal("subset");
    } else {
      const json = formatJsonPretty(value, "");
      const reparsed = formatJsonParse(json);
      let yaml = formatYamlRender(value, 0);
      if (yaml === "") yaml = value.kind === "object" ? "{}" : "[]";
      const reverse = formatYamlParse(yaml);
      if (reparsed.ok && formatValuesEqual(reparsed.value, value) &&
          reverse !== null && formatValuesEqual(reverse, value)) {
        codeTaskLogAppend(log, "format_conversion:converted", "roundtrip=ok");
        codeTaskLogAppend(log, "format_conversion:reverse_roundtrip", "yaml=ok");
        body = codeTaskTemplate("format_conversion_to_json", [["json", json]]);
        confidence = 0.7;
      } else {
        codeTaskLogAppend(log, "format_conversion:refusal", "roundtrip=fail");
        body = formatConversionRefusal("roundtrip");
      }
    }
  }
  return codeTaskAnswer(log, "format_conversion", "response:format_conversion", body, confidence);
}

/**
 * Browser binding for the `format_conversion` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryFormatConversion(prompt) {
  return handleFormatConversion(prompt, prompt.toLowerCase());
}
