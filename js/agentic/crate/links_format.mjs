// Compact Links Notation records (rust/src/links_format.rs), with the
// `lino_objects_codec::format` quoting they borrow
// (lino-objects-codec 0.7.0 `format_indented_ordered`, `escape_reference`,
// `format_indented_value`).

/** Rust dependency `lino_objects_codec::format::escape_reference`. @param {string} value */
export function escapeReference(value) {
  const needsEscaping = /[\s()'":]/u.test(value) || value.includes('\n');
  if (!needsEscaping) return value;
  const hasSingle = value.includes("'");
  const hasDouble = value.includes('"');
  if (hasSingle && !hasDouble) return `"${value}"`;
  if (hasDouble && !hasSingle) return `'${value}'`;
  if (hasSingle && hasDouble) {
    const singleCount = value.split("'").length - 1;
    const doubleCount = value.split('"').length - 1;
    if (doubleCount < singleCount) return `"${value.replaceAll('"', '""')}"`;
    return `'${value.replaceAll("'", "''")}'`;
  }
  return `'${value}'`;
}

/** Mirrors `fn format_indented_value` in lino-objects-codec `format`. @param {string} value */
function formatIndentedValue(value) {
  const hasSingle = value.includes("'");
  const hasDouble = value.includes('"');
  if (hasDouble && !hasSingle) return `'${value}'`;
  if (hasSingle && !hasDouble) return `"${value}"`;
  if (hasSingle && hasDouble) return `'${value.replaceAll("'", "''")}'`;
  return `"${value}"`;
}

/**
 * Rust dependency `lino_objects_codec::format::format_indented_ordered`.
 * @param {string} id
 * @param {Array<[string, string]>} pairs
 * @param {string} indent
 */
export function formatIndentedOrdered(id, pairs, indent) {
  if (!id) throw new Error('format_indented_ordered: missing id');
  const lines = [id];
  for (const [key, value] of pairs) lines.push(`${indent}${escapeReference(key)} ${formatIndentedValue(value)}`);
  return lines.join('\n');
}

/**
 * Mirrors `fn format_lino_record` in rust/src/links_format.rs.
 * @param {string} id
 * @param {Array<[string, string]>} pairs
 */
export function formatLinoRecord(id, pairs) {
  return formatIndentedOrdered(id, pairs.map(([key, value]) => [key, sanitizeLinoValue(value)]), '  ');
}

/** Mirrors `fn format_lino_value_verbatim`. @param {string} value */
export function formatLinoValueVerbatim(value) {
  return formatIndentedValue(value);
}

/** Mirrors `fn format_lino_value`. @param {string} value */
export function formatLinoValue(value) {
  return formatLinoValueVerbatim(sanitizeLinoValue(value));
}

/**
 * Mirrors `fn push_lino_node`: returns `out` with one `name "value"` line appended.
 * @param {string} out
 * @param {number} indent
 * @param {string} name
 * @param {string|null} value
 */
export function pushLinoNode(out, indent, name, value) {
  return `${out}${' '.repeat(indent)}${name}${value === null || value === undefined ? '' : ` ${formatLinoValue(value)}`}\n`;
}

/**
 * Mirrors `fn push_lino_field`: returns `out` with one `name value` line appended.
 * @param {string} out
 * @param {number} indent
 * @param {string} name
 * @param {string|null} value
 */
export function pushLinoField(out, indent, name, value) {
  return `${out}${' '.repeat(indent)}${name}${value === null || value === undefined ? '' : ` ${value}`}\n`;
}

/** Mirrors `fn sanitize_lino_value`. @param {string} value */
export function sanitizeLinoValue(value) {
  return value.replaceAll('\\', '\\\\').replaceAll('\r', '\\r').replaceAll('\n', '\\n').replaceAll('\t', '\\t');
}

/** Mirrors `fn flatten_lino_value`. @param {string} value */
export function flattenLinoValue(value) {
  return value.replaceAll('\r', '\\r').replaceAll('\n', '\\n').replaceAll('\t', '\\t');
}
