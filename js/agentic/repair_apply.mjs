// The apply half of the issue #1185 R3 repair loop
// (rust/src/agentic_coding/repair_apply.rs): a `repair_edit` record rendered
// back into the artifact's source.
//
// A `RepairEdit` is `{language, file, line, error_code, retained}`. A gap is
// `{gap: 'not_a_record'|'no_retained_fix'|'no_location'|'line_out_of_range'|
// 'unvalidated'|'syntax_invalid', ...}`. The Rust root validates the rendering
// through the meta-language CST engine; this root has no CST engine, so
// `renderRepairEdit` takes the validator as a parameter and, without one,
// refuses with `unvalidated` — the gap the Rust build gives with its
// `meta-language` feature compiled out.

/** Mirrors `fn read_quoted`. */
function readQuoted(text, start) {
  let value = '';
  let escaped = false;
  for (let index = start; index < text.length; index += 1) {
    const character = text[index];
    if (escaped) {
      value += character;
      escaped = false;
    } else if (character === '\\') {
      escaped = true;
    } else if (character === '"') {
      return [value, index + 1];
    } else {
      value += character;
    }
  }
  return [value, text.length];
}

/** Mirrors `fn record_fields`. */
function recordFields(document) {
  const fields = [];
  let cursor = 0;
  while (cursor < document.length) {
    const rest = document.slice(cursor);
    const trimmed = rest.trimStart();
    if (trimmed === '') break;
    cursor += rest.length - trimmed.length;
    const space = trimmed.search(/\s/);
    const nameEnd = space < 0 ? trimmed.length : space;
    const name = trimmed.slice(0, nameEnd);
    cursor += nameEnd;
    const afterName = document.slice(cursor);
    const newline = afterName.indexOf('\n');
    const lineEnd = newline < 0 ? afterName.length : newline;
    const inline = afterName.slice(0, lineEnd).trimStart();
    if (inline.startsWith('"')) {
      const quoteAt = cursor + (lineEnd - inline.length);
      const [value, next] = readQuoted(document, quoteAt + 1);
      fields.push([name, value]);
      cursor = next;
    } else {
      fields.push([name, inline.trimEnd()]);
      cursor += lineEnd;
    }
  }
  return fields;
}

/** Mirrors `fn parse_repair_edit`. */
export function parseRepairEdit(document) {
  const fields = recordFields(document);
  if (fields.length === 0 || fields[0][0] !== 'repair_edit') return null;
  const edit = { language: '', file: null, line: null, error_code: null, retained: null };
  for (const [name, value] of fields.slice(1)) {
    if (name === 'language') edit.language = value;
    else if (name === 'file') edit.file = value;
    else if (name === 'line') edit.line = /^\d+$/.test(value) ? Number(value) : null;
    else if (name === 'error_code') edit.error_code = value;
    else if (name === 'retained' && value !== 'none') edit.retained = value;
  }
  return edit;
}

const indentation = (line) => line.slice(0, line.length - line.trimStart().length);

/** Mirrors `fn reindented`. */
function reindented(fragment, indent) {
  const lines = fragment.replace(/\n$/, '').split('\n');
  const widths = lines.filter((line) => line.trim() !== '').map((line) => indentation(line).length);
  const common = widths.length ? Math.min(...widths) : 0;
  return lines.map((line) => (line.trim() === '' ? '' : `${indent}${line.slice(common)}`));
}

/** Mirrors `fn splice_repair_edit`: `{source}` or `{gap}`. */
export function spliceRepairEdit(edit, source) {
  if (edit.retained === null) return { gap: 'no_retained_fix' };
  if (edit.line === null || edit.line < 1) return { gap: 'no_location' };
  const lines = source.split('\n');
  if (source.endsWith('\n')) lines.pop();
  if (edit.line > lines.length) return { gap: 'line_out_of_range', line: edit.line, lines: lines.length };
  const indent = indentation(lines[edit.line - 1]);
  const out = [...lines.slice(0, edit.line - 1), ...reindented(edit.retained, indent), ...lines.slice(edit.line)];
  return { source: out.join('\n') + (source.endsWith('\n') ? '\n' : '') };
}

/**
 * Mirrors `fn render_repair_edit`. `validate(language, source)` returns
 * `true`/`false` for a parse verdict, or `null` when it has no grammar.
 */
export function renderRepairEdit(document, source, validate = null) {
  const edit = parseRepairEdit(document);
  if (edit === null) return { gap: 'not_a_record' };
  const spliced = spliceRepairEdit(edit, source);
  if (spliced.gap) return spliced;
  const verdict = validate === null ? null : validate(edit.language, spliced.source);
  if (verdict === null) return { gap: 'unvalidated', language: edit.language };
  if (!verdict) return { gap: 'syntax_invalid', language: edit.language };
  return spliced;
}
