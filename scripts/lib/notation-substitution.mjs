// The substitution rule of the links notation passes (PR #1188, R1188-U6).
//
// A name of the notation we own is renamed in three kinds of place: the
// `.lino` files, where it is an unquoted token; the code that reads them,
// where it is a token inside a string or regular-expression literal; and the
// documentation, where it is a backticked token. Code identifiers are never
// touched: a Rust or JavaScript variable keeps its language's convention, so
// only literal, comment and documentation spans are rewritten.
// experiments/formal_ai_subagent/apply-notation-rules.mjs drives this module.

import { namesOfLine } from './links-notation-names.mjs';

/** A character that continues a name: a token boundary is anything else. */
const NAME_CHARACTER = /[A-Za-z0-9_-]/u;

/**
 * Rewrite the names of a `.lino` document by `mapping`, only where a name is
 * an unquoted token or a response-template placeholder. Literal prose stays unchanged.
 * @param {string} text
 * @param {Map<string, string>} mapping
 * @returns {string}
 */
export function rewriteNotation(text, mapping) {
  return text
    .split('\n')
    .map((line) => {
      let output = line;
      for (const { name, start } of namesOfLine(line).reverse()) {
        const renamed = mapping.get(name);
        if (renamed !== undefined) {
          output = output.slice(0, start) + renamed + output.slice(start + name.length);
        }
      }
      return output.replace(/\{([A-Za-z][A-Za-z0-9_-]*)\}/gu, (placeholder, name) => {
        const renamed = mapping.get(name);
        return renamed === undefined ? placeholder : `{${renamed}}`;
      });
    })
    .join('\n');
}

/**
 * The spans of a JavaScript or Rust source: `code`, `comment`, `string` and
 * `regex` (JavaScript regular-expression literals), in order and covering the
 * whole text.
 * @param {string} source
 * @param {'javascript' | 'rust'} language
 * @returns {Array<{kind: string, start: number, end: number}>}
 */
export function sourceSpans(source, language) {
  const spans = [];
  const push = (kind, start, end) => {
    if (end <= start) {
      return;
    }
    const last = spans[spans.length - 1];
    if (last && last.kind === kind && last.end === start) {
      last.end = end;
    } else {
      spans.push({ kind, start, end });
    }
  };
  const scan = language === 'rust' ? scanRust : scanJavaScript;
  scan(source, push);
  return spans;
}

function skipQuoted(source, index, quote) {
  let cursor = index + 1;
  while (cursor < source.length && source[cursor] !== quote) {
    cursor += source[cursor] === '\\' ? 2 : 1;
  }
  return Math.min(cursor + 1, source.length);
}

function scanRust(source, push) {
  let index = 0;
  let codeStart = 0;
  while (index < source.length) {
    const character = source[index];
    const next = source[index + 1];
    let kind = null;
    let end = index;
    if (character === '/' && next === '/') {
      kind = 'comment';
      end = source.indexOf('\n', index);
      end = end === -1 ? source.length : end;
    } else if (character === '/' && next === '*') {
      kind = 'comment';
      let depth = 0;
      end = index;
      while (end < source.length) {
        if (source.startsWith('/*', end)) {
          depth += 1;
          end += 2;
        } else if (source.startsWith('*/', end)) {
          depth -= 1;
          end += 2;
          if (depth === 0) {
            break;
          }
        } else {
          end += 1;
        }
      }
    } else if (/[rb]/u.test(character) && !NAME_CHARACTER.test(source[index - 1] ?? ' ')) {
      const raw = /^(?:br|r)(#*)"/u.exec(source.slice(index, index + 260));
      const bytes = /^b"/u.exec(source.slice(index, index + 2));
      if (raw) {
        kind = 'string';
        const closing = `"${raw[1]}`;
        const found = source.indexOf(closing, index + raw[0].length);
        end = found === -1 ? source.length : found + closing.length;
      } else if (bytes) {
        kind = 'string';
        end = skipQuoted(source, index + 1, '"');
      }
    } else if (character === '"') {
      kind = 'string';
      end = skipQuoted(source, index, '"');
    } else if (character === "'") {
      const charLiteral = /^'(?:\\(?:u\{[0-9a-fA-F]+\}|x[0-9a-fA-F]{2}|.)|[^\\'])'/u.exec(source.slice(index, index + 12));
      if (charLiteral) {
        kind = 'string';
        end = index + charLiteral[0].length;
      }
    }
    if (kind === null) {
      index += 1;
      continue;
    }
    push('code', codeStart, index);
    push(kind, index, end);
    index = end;
    codeStart = end;
  }
  push('code', codeStart, source.length);
}

/** The words after which a `/` opens a regular-expression literal. */
const REGEX_AFTER_WORD = /(?:^|[^\w$])(?:return|typeof|case|do|else|in|of|void|yield|await|delete|throw|new)$/u;

function regexMayStart(source, index) {
  let cursor = index - 1;
  while (cursor >= 0 && /\s/u.test(source[cursor])) {
    cursor -= 1;
  }
  if (cursor < 0) {
    return true;
  }
  if (/[(,=:[!&|?{};+\-*%<>~^]/u.test(source[cursor])) {
    return true;
  }
  return REGEX_AFTER_WORD.test(source.slice(Math.max(0, cursor - 8), cursor + 1));
}

function skipRegex(source, index) {
  let cursor = index + 1;
  let inClass = false;
  while (cursor < source.length && source[cursor] !== '\n') {
    const character = source[cursor];
    if (character === '\\') {
      cursor += 2;
      continue;
    }
    if (character === '[') {
      inClass = true;
    } else if (character === ']') {
      inClass = false;
    } else if (character === '/' && !inClass) {
      cursor += 1;
      while (cursor < source.length && /[a-z]/u.test(source[cursor])) {
        cursor += 1;
      }
      return cursor;
    }
    cursor += 1;
  }
  return -1;
}

function scanJavaScript(source, push) {
  let index = 0;
  let codeStart = 0;
  // Each open template literal records the brace depth of its `${`.
  const templates = [];
  let braces = 0;
  const openSpan = (kind, start, end) => {
    push('code', codeStart, start);
    push(kind, start, end);
    codeStart = end;
    return end;
  };
  const scanTemplate = (start) => {
    let cursor = start;
    while (cursor < source.length) {
      if (source[cursor] === '\\') {
        cursor += 2;
      } else if (source[cursor] === '`') {
        return { end: cursor + 1, opensExpression: false };
      } else if (source.startsWith('${', cursor)) {
        return { end: cursor + 2, opensExpression: true };
      } else {
        cursor += 1;
      }
    }
    return { end: source.length, opensExpression: false };
  };
  while (index < source.length) {
    const character = source[index];
    const next = source[index + 1];
    if (character === '/' && next === '/') {
      const end = source.indexOf('\n', index);
      index = openSpan('comment', index, end === -1 ? source.length : end);
    } else if (character === '/' && next === '*') {
      const end = source.indexOf('*/', index + 2);
      index = openSpan('comment', index, end === -1 ? source.length : end + 2);
    } else if (character === '"' || character === "'") {
      index = openSpan('string', index, skipQuoted(source, index, character));
    } else if (character === '`') {
      const template = scanTemplate(index + 1);
      index = openSpan('string', index, template.end);
      if (template.opensExpression) {
        templates.push(braces);
      }
    } else if (character === '}' && templates.length > 0 && templates[templates.length - 1] === braces) {
      templates.pop();
      const template = scanTemplate(index + 1);
      index = openSpan('string', index, template.end);
      if (template.opensExpression) {
        templates.push(braces);
      }
    } else if (character === '/' && regexMayStart(source, index)) {
      const end = skipRegex(source, index);
      index = end === -1 ? index + 1 : openSpan('regex', index, end);
    } else {
      if (character === '{') {
        braces += 1;
      } else if (character === '}') {
        braces -= 1;
      }
      index += 1;
    }
  }
  push('code', codeStart, source.length);
}

/**
 * Replace every whole-token occurrence of a mapped name inside `text`.
 * `spelling` turns the old and new name into the replacement text.
 * @param {string} text
 * @param {Map<string, string>} mapping
 * @param {(old: string, renamed: string) => string} spelling
 * @returns {{text: string, count: number}}
 */
export function replaceTokens(text, mapping, spelling = (_old, renamed) => renamed) {
  let count = 0;
  // An escape such as `\b` or `\n` right before a token is a boundary.
  const replaced = text.replace(/(\\[A-Za-z])?([a-z][a-z0-9]*(?:[-_][a-z0-9]+)*)/gu, (whole, escape, token, offset) => {
    const before = escape ? ' ' : text[offset - 1] ?? ' ';
    const after = text[offset + whole.length] ?? ' ';
    if (NAME_CHARACTER.test(before) || NAME_CHARACTER.test(after) || !mapping.has(token)) {
      return whole;
    }
    // `{name}` and `{name:?}` in a format string name a code variable.
    if (before === '{' && (after === '}' || after === ':')) {
      return whole;
    }
    count += 1;
    return (escape ?? '') + spelling(token, mapping.get(token));
  });
  return { text: replaced, count };
}

/** In a regular expression the name accepts both spellings during the transition. */
export function bothSpellingsPattern(old) {
  return old.replace(/_/gu, '[-_]');
}

/**
 * The names of `mapping` used as code identifiers (outside literals and
 * comments) in a source: those keep their comments untouched.
 * @param {string} source
 * @param {'javascript' | 'rust'} language
 * @param {Map<string, string>} mapping
 * @returns {Set<string>}
 */
export function identifiersIn(source, language, mapping) {
  const found = new Set();
  for (const span of sourceSpans(source, language)) {
    if (span.kind === 'code') {
      const code = source.slice(span.start, span.end);
      replaceTokens(code, mapping, (old) => {
        found.add(old);
        return old;
      });
    }
  }
  return found;
}

/**
 * Rewrite the mapped names of a source inside its literals (a regular
 * expression accepts both spellings) and, for names that are no code
 * identifier anywhere (`identifiers`), inside its comments.
 * @param {string} source
 * @param {'javascript' | 'rust'} language
 * @param {Map<string, string>} mapping
 * @param {Set<string>} identifiers
 * @returns {{text: string, count: number}}
 */
export function rewriteSource(source, language, mapping, identifiers) {
  const commentMapping = new Map([...mapping].filter(([old]) => !identifiers.has(old)));
  let count = 0;
  const text = sourceSpans(source, language)
    .map((span) => {
      const piece = source.slice(span.start, span.end);
      let result = { text: piece, count: 0 };
      if (span.kind === 'string') {
        result = replaceTokens(piece, mapping);
        // A template placeholder is notation when no code identifier owns it.
        // Rust formatting variables retain their language spelling.
        result.text = result.text.replace(/\{([A-Za-z][A-Za-z0-9_-]*)\}/gu, (placeholder, name) => {
          const renamed = identifiers.has(name) ? undefined : mapping.get(name);
          if (renamed === undefined) return placeholder;
          count += 1;
          return `{${renamed}}`;
        });
      } else if (span.kind === 'regex') {
        result = replaceTokens(piece, mapping, (old) => bothSpellingsPattern(old));
      } else if (span.kind === 'comment') {
        result = replaceTokens(piece, commentMapping);
      }
      count += result.count;
      return result.text;
    })
    .join('');
  return { text, count };
}

/**
 * Rewrite the backticked mapped names of a Markdown document.
 * @param {string} text
 * @param {Map<string, string>} mapping
 * @returns {{text: string, count: number}}
 */
export function rewriteDocument(text, mapping) {
  let count = 0;
  const rewritten = text.replace(/`([^`\n]+)`/gu, (whole, inner) => {
    const result = replaceTokens(inner, mapping);
    count += result.count;
    return `\`${result.text}\``;
  });
  return { text: rewritten, count };
}
