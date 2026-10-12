// The tokens and top-level items of a JavaScript or Rust source, as
// self-translation reads them.
//
// One lexer serves both languages because the portable fragment shares its
// lexical shape: identifiers, decimal numbers, quoted strings, punctuators
// and the two comment forms. What only one language has (a JavaScript
// template or regular expression literal, a Rust lifetime) is still lexed
// far enough to find the item boundaries, so a carried item is cut where the
// language cuts it, then refused by the frontend. Portable-subset style.

const PUNCTUATORS = [
  '>>>=', '...', '===', '!==', '**=', '<<=', '>>=', '>>>', '&&=', '||=', '??=', '=>', '->', '::',
  '==', '!=', '<=', '>=', '&&', '||', '??', '?.', '++', '--', '+=', '-=', '*=', '/=', '%=', '&=',
  '|=', '^=', '<<', '>>', '**', '{', '}', '(', ')', '[', ']', ';', ',', '<', '>', '+', '-', '*',
  '/', '%', '&', '|', '^', '!', '~', '?', ':', '=', '.', '@', '#',
];

const REGEX_AFTER = new Set(['return', 'typeof', 'case', 'do', 'else', 'in', 'of', 'new', 'delete', 'void', 'throw', 'yield', 'await']);

/**
 * Lex `source` as `language` ('JavaScript' or 'Rust').
 * A token is `{ type, text, start, end }` with UTF-16 offsets; `type` is
 * one of ident, number, string, template, regex, char, punct, comment.
 * @param {string} source
 * @param {string} language
 * @returns {Array<{type: string, text: string, start: number, end: number}>}
 */
export function lex(source, language) {
  const tokens = [];
  let index = 0;
  const push = (type, start) => tokens.push({ type, text: source.slice(start, index), start, end: index });
  const significant = () => {
    for (let at = tokens.length - 1; at >= 0; at -= 1) {
      if (tokens[at].type !== 'comment') return tokens[at];
    }
    return null;
  };
  while (index < source.length) {
    const char = source[index];
    const start = index;
    if (/\s/u.test(char)) {
      index += 1;
    } else if (source.startsWith('//', index)) {
      while (index < source.length && source[index] !== '\n') index += 1;
      if (source[index - 1] === '\r') index -= 1;
      push('comment', start);
    } else if (source.startsWith('/*', index)) {
      const end = source.indexOf('*/', index + 2);
      if (end < 0) throw new Error(`unterminated block comment at ${start}`);
      index = end + 2;
      push('comment', start);
    } else if (/[A-Za-z_$]/u.test(char) || char.charCodeAt(0) >= 0x80) {
      while (index < source.length && (/[A-Za-z0-9_$]/u.test(source[index]) || source.charCodeAt(index) >= 0x80)) index += 1;
      if (language === 'Rust' && source[index] === '!' && source[index + 1] !== '=') index += 1;
      push('ident', start);
    } else if (/[0-9]/u.test(char)) {
      while (index < source.length && /[0-9_]/u.test(source[index])) index += 1;
      if (source[index] === '.' && /[0-9]/u.test(source[index + 1] ?? '')) {
        index += 1;
        while (index < source.length && /[0-9_]/u.test(source[index])) index += 1;
      }
      if (/[eE]/u.test(source[index] ?? '') && /[0-9+-]/u.test(source[index + 1] ?? '')) {
        index += 2;
        while (index < source.length && /[0-9_]/u.test(source[index])) index += 1;
      }
      while (index < source.length && /[A-Za-z0-9_]/u.test(source[index])) index += 1;
      push('number', start);
    } else if (char === '"' || (char === "'" && language === 'JavaScript')) {
      index = quotedEnd(source, index, char);
      push('string', start);
    } else if (char === "'") {
      // A Rust char literal or a lifetime: both refused by the frontend.
      const literal = /^'(\\.|[^\\'])'/u.exec(source.slice(index));
      index += literal ? literal[0].length : 1;
      while (!literal && index < source.length && /[A-Za-z0-9_]/u.test(source[index])) index += 1;
      push('char', start);
    } else if (char === '`' && language === 'JavaScript') {
      index = templateEnd(source, index);
      push('template', start);
    } else if (char === '/' && language === 'JavaScript' && regexAllowed(significant())) {
      index = regexEnd(source, index);
      push('regex', start);
    } else {
      const punct = PUNCTUATORS.find((candidate) => source.startsWith(candidate, index));
      if (!punct) throw new Error(`unexpected character ${JSON.stringify(char)} at ${start}`);
      index += punct.length;
      push('punct', start);
    }
  }
  return tokens;
}

function quotedEnd(source, start, quote) {
  let index = start + 1;
  while (index < source.length && source[index] !== quote) {
    if (source[index] === '\\') index += 1;
    index += 1;
  }
  if (index >= source.length) throw new Error(`unterminated string at ${start}`);
  return index + 1;
}

function templateEnd(source, start) {
  let index = start + 1;
  let depth = 0;
  while (index < source.length) {
    const char = source[index];
    if (char === '\\') {
      index += 2;
    } else if (depth === 0 && char === '`') {
      return index + 1;
    } else if (char === '$' && source[index + 1] === '{') {
      depth += 1;
      index += 2;
    } else if (depth > 0 && char === '}') {
      depth -= 1;
      index += 1;
    } else if (depth > 0 && (char === '"' || char === "'" || char === '`')) {
      index = char === '`' ? templateEnd(source, index) : quotedEnd(source, index, char);
    } else {
      index += 1;
    }
  }
  throw new Error(`unterminated template at ${start}`);
}

function regexAllowed(previous) {
  if (!previous) return true;
  if (previous.type === 'ident') return REGEX_AFTER.has(previous.text);
  if (previous.type === 'number' || previous.type === 'string' || previous.type === 'template' || previous.type === 'regex') return false;
  return ![')', ']', '}'].includes(previous.text);
}

function regexEnd(source, start) {
  let index = start + 1;
  let inClass = false;
  while (index < source.length && source[index] !== '\n') {
    const char = source[index];
    if (char === '\\') {
      index += 2;
      continue;
    }
    if (char === '[') inClass = true;
    if (char === ']') inClass = false;
    index += 1;
    if (char === '/' && !inClass) {
      while (index < source.length && /[a-z]/u.test(source[index])) index += 1;
      return index;
    }
  }
  throw new Error(`unterminated regular expression at ${start}`);
}

/**
 * The top-level items of a lexed source: each comment token is an item, and
 * every other item runs from its first token to the `;` or the closing `}`
 * at depth zero that ends it. A Rust outer attribute (`#[...]`) belongs to
 * the item after it; an inner attribute (`#![...]`) is an item of its own.
 * Returns `{ start, end, comment, tokens }` with UTF-16 offsets.
 * @param {Array<{type: string, text: string, start: number, end: number}>} tokens
 * @returns {Array<{start: number, end: number, comment: boolean, tokens: Array<object>}>}
 */
export function topLevelItems(tokens) {
  const items = [];
  let index = 0;
  while (index < tokens.length) {
    const first = tokens[index];
    if (first.type === 'comment') {
      items.push({ start: first.start, end: first.end, comment: true, tokens: [first] });
      index += 1;
      continue;
    }
    const begin = index;
    const statementOnly = endsAtSemicolon(tokens, begin);
    let depth = 0;
    let sawBlock = false;
    let end = -1;
    for (; index < tokens.length; index += 1) {
      const token = tokens[index];
      if (token.type !== 'punct') continue;
      if (token.text === '{' || token.text === '(' || token.text === '[') {
        depth += 1;
        if (token.text === '{') sawBlock = true;
      } else if (token.text === '}' || token.text === ')' || token.text === ']') {
        depth -= 1;
        const inner = tokens[begin].text === '#' && tokens[index].text === ']';
        if (depth === 0 && inner && tokens[begin + 1]?.text === '!') {
          end = index;
          break;
        }
        if (depth === 0 && token.text === '}' && sawBlock && !statementOnly && !continues(tokens, index)) {
          end = index;
          break;
        }
      } else if (token.text === ';' && depth === 0) {
        end = index;
        break;
      }
    }
    if (end < 0) end = tokens.length - 1;
    items.push({ start: tokens[begin].start, end: tokens[end].end, comment: false, tokens: tokens.slice(begin, end + 1) });
    index = end + 1;
  }
  return items;
}

// A declaration that only a `;` ends, whatever braces it holds: an import,
// a binding, a Rust `use`, `const` or `static`, and a re-export.
function endsAtSemicolon(tokens, begin) {
  let at = begin;
  while (['export', 'pub'].includes(tokens[at]?.text)) at += 1;
  const head = tokens[at];
  if (!head) return false;
  if (head.text === 'const') return tokens[at + 1]?.text !== 'fn';
  return ['import', 'let', 'var', 'use', 'static', 'type', '{', '*'].includes(head.text);
}

// A closing brace that the same statement continues past: `} else`, an
// object literal followed by an operator, or `};` closing a declaration.
function continues(tokens, index) {
  const next = tokens.slice(index + 1).find((token) => token.type !== 'comment');
  if (!next) return false;
  if (next.type === 'punct') return next.text !== '}' && next.text !== '#' && next.text !== '@';
  return next.text === 'else' || next.text === 'as';
}
