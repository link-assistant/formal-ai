// Tuple syntax and immutable iteration patterns for the conservative native reader.
// Unknown syntax rejects the complete case; types and arity remain observable.
import { close } from './rust-specification-values.mjs';

export function tupleParts(tokens) {
  const parts = [[]];
  let comma = false;
  for (let cursor = 0; cursor < tokens.length; cursor += 1) {
    const token = tokens[cursor];
    if (token.kind === 'punct' && ['(', '[', '{'].includes(token.text)) {
      const end = close(tokens, cursor);
      parts.at(-1).push(...tokens.slice(cursor, end + 1));
      cursor = end;
    } else if (token.kind === 'punct' && token.text === ',') {
      comma = true;
      if (!parts.at(-1).length) throw new Error('empty native tuple element');
      parts.push([]);
    } else parts.at(-1).push(token);
  }
  if (!parts.at(-1).length) parts.pop();
  return { parts, comma };
}

export function tupleTypes(type) {
  if (!type.startsWith('tuple:')) throw new Error('native tuple pattern needs tuple element type');
  const types = JSON.parse(type.slice(6));
  if (!Array.isArray(types) || !types.every(value => typeof value === 'string')) throw new Error('invalid native tuple type');
  return types;
}

export function iterationBindings(tokens, type, borrowed) {
  const bindings = [];
  const names = new Set();
  function visit(pattern, actual, reference) {
    if (pattern[0]?.text === '&') {
      if (!reference) throw new Error('native reference pattern needs borrowed iteration');
      visit(pattern.slice(1), actual, false);
      return;
    }
    if (pattern[0]?.text === '(' && close(pattern, 0) === pattern.length - 1) {
      const { parts, comma } = tupleParts(pattern.slice(1, -1));
      if (!comma && parts.length === 1) { visit(parts[0], actual, reference); return; }
      const types = tupleTypes(actual);
      if (parts.length !== types.length) throw new Error('native tuple pattern arity mismatch');
      for (let index = 0; index < parts.length; index += 1) visit(parts[index], types[index], false);
      return;
    }
    if (pattern.length !== 1 || pattern[0].kind !== 'word'
      || ['mut', 'ref', 'self', 'Self', 'true', 'false', 'in', 'match'].includes(pattern[0].text)) {
      throw new Error('unsupported native iteration binding');
    }
    const name = pattern[0].text;
    if (name === '_') return;
    if (names.has(name)) throw new Error('duplicate native iteration binding');
    names.add(name);
    bindings.push({ name, type: actual });
  }
  visit(tokens, type, borrowed);
  return bindings;
}

export function bindIteration(tokens, value, borrowed) {
  const bindings = [];
  function visit(pattern, observed, reference) {
    if (pattern[0]?.text === '&') {
      if (!reference) throw new Error('native reference pattern needs borrowed iteration');
      visit(pattern.slice(1), observed, false);
    } else if (pattern[0]?.text === '(') {
      const { parts, comma } = tupleParts(pattern.slice(1, -1));
      if (!comma && parts.length === 1) { visit(parts[0], observed, reference); return; }
      if (!Array.isArray(observed) || observed.length !== parts.length) throw new Error('observed native tuple arity mismatch');
      for (let index = 0; index < parts.length; index += 1) visit(parts[index], observed[index], false);
    } else if (pattern[0].text !== '_') bindings.push([pattern[0].text, observed]);
  }
  visit(tokens, value, borrowed);
  return bindings;
}
