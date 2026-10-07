// The JavaScript and Rust frontends of portable-pure-v1: one top-level item
// (its tokens, with the comments directly before it) to one IR item, or a
// refusal naming the construct map's `carried` row. Both frontends share one
// expression grammar and differ only in the spellings the construct map
// gives each language, which is meta-language's per-language frontend over
// one IR (PR #196, js/src/translation/javascript.js and rust.js).

import { constructMap, idBySpelling, refuse } from './constructs.mjs';
import { camelFromSnake, canonicalRustName, snakeFromCamel } from './ir.mjs';

const UNSUPPORTED_BINARY = new Set(['==', '!=', '**', '??', '&', '|', '^', '<<', '>>', '>>>', 'instanceof', 'in', 'as']);
const ASSIGNMENTS = new Set(['=', '+=', '-=', '*=', '/=', '%=', '++', '--', '&&=', '||=', '??=']);
const TRANSPARENT_RUST = new Set(['to_string', 'to_owned', 'as_str', 'clone', 'into']);
const ESCAPES = { n: '\n', t: '\t', r: '\r', '\\': '\\', "'": "'", '"': '"', 0: '\0' };

/**
 * A cursor over the non-comment tokens of an item.
 * @param {Array<{type: string, text: string}>} tokens
 */
function cursor(tokens) {
  const list = tokens.filter((token) => token.type !== 'comment');
  let index = 0;
  const self = {
    peek: (ahead = 0) => list[index + ahead] ?? { type: 'end', text: '' },
    next: () => list[index++] ?? { type: 'end', text: '' },
    is: (text, ahead = 0) => self.peek(ahead).text === text && self.peek(ahead).type !== 'string',
    eat: (text) => {
      if (!self.is(text)) return false;
      index += 1;
      return true;
    },
    expect: (text) => {
      if (!self.eat(text)) refuse('unsupported', `${JSON.stringify(self.peek().text)} where ${JSON.stringify(text)} belongs`);
    },
    done: () => index >= list.length,
  };
  return self;
}

/**
 * The term of a top-level item: what it declares, read from its first
 * keywords (`export`, `pub`, attributes and `async` skipped).
 * @param {Array<{type: string, text: string}>} tokens
 * @param {string} language
 * @returns {string}
 */
export function itemTerm(tokens, language) {
  const words = tokens.filter((token) => token.type !== 'comment');
  if (language === 'Rust') {
    if (words[0]?.text === '#') return words[1]?.text === '!' ? 'attribute' : itemTerm(words.slice(attributeEnd(words)), language);
    const rest = words.filter((token, index) => !(index < 4 && ['pub', 'const', 'unsafe', 'async', 'extern'].includes(token.text) && words[index + 1]));
    const head = rest.find((token) => ['fn', 'use', 'struct', 'enum', 'impl', 'trait', 'mod', 'type', 'static', 'macro_rules!'].includes(token.text));
    if (head) return head.text === 'fn' ? 'function' : head.text.replace('!', '');
    return words.some((token) => token.text === 'const') ? 'constant' : 'statement';
  }
  const rest = words[0]?.text === 'export' ? words.slice(1) : words;
  const head = rest[0]?.text === 'async' ? rest[1]?.text : rest[0]?.text;
  if (head === 'function') return 'function';
  if (head === 'import' || words[0]?.text === 'import') return 'import';
  if (head === 'class') return 'class';
  if (head === 'const' && /^[A-Z][A-Z0-9_]+$/u.test(rest[1]?.text ?? '')) return 'constant';
  if (head === 'const' || head === 'let' || head === 'var') return 'binding';
  return words[0]?.text === 'export' ? 'export' : 'statement';
}

function attributeEnd(words) {
  let depth = 0;
  for (let index = 1; index < words.length; index += 1) {
    if (words[index].text === '[') depth += 1;
    if (words[index].text === ']') {
      depth -= 1;
      if (depth === 0) return index + 1;
    }
  }
  return words.length;
}

/**
 * The JSDoc types of `@param {T} name` and `@returns {T}` in `doc`.
 * @param {string} doc
 * @returns {{params: Map<string, string>, returns: string | null}}
 */
function jsdocTypes(doc) {
  const params = new Map();
  for (const match of doc.matchAll(/@param\s+\{([^}]*)\}\s+([A-Za-z_$][\w$]*)/gu)) params.set(match[2], match[1].trim());
  const returns = /@returns?\s+\{([^}]*)\}/u.exec(doc);
  return { params, returns: returns ? returns[1].trim() : null };
}

/**
 * Parse one JavaScript item into portable-pure-v1.
 * @param {Array<object>} tokens the item's tokens
 * @param {string} doc the comments directly before the item
 * @returns {object}
 */
export function parseJavaScriptItem(tokens, doc) {
  const at = cursor(tokens);
  const term = itemTerm(tokens, 'JavaScript');
  if (term === 'import') refuse('no-definition');
  if (term === 'class') refuse('class');
  if (term === 'statement') refuse('statement');
  const exported = at.eat('export');
  if (at.is('{') || at.is('*') || at.is('default')) refuse('no-definition');
  if (at.is('async')) refuse('async');
  if (at.eat('const')) {
    const name = at.next();
    if (name.type !== 'ident') refuse('unsupported', 'a destructuring binding');
    at.expect('=');
    // The value is read first, so a computed binding names what it computes.
    const value = expression(at, 'JavaScript');
    at.eat(';');
    if (!at.done()) refuse('unsupported', at.peek().text);
    if (!/^[A-Z][A-Z0-9_]+$/u.test(name.text)) refuse('statement');
    return { kind: 'constant', name: name.text, exported, type: literalType(value), value };
  }
  if (at.is('let') || at.is('var')) refuse('mutable');
  at.expect('function');
  if (at.is('*')) refuse('unsupported', 'a generator');
  const jsName = at.next().text;
  const name = snakeFromCamel(jsName);
  const types = jsdocTypes(doc);
  at.expect('(');
  const params = [];
  while (!at.eat(')')) {
    const param = at.next();
    if (param.type !== 'ident') refuse('unsupported', `the parameter ${param.text}`);
    if (at.is('=')) refuse('unsupported', 'a default parameter');
    const type = jsType(types.params.get(param.text));
    params.push({ name: snakeFromCamel(param.text), type });
    if (!at.is(')')) at.expect(',');
  }
  const returns = jsType(types.returns);
  const body = jsBlock(at);
  if (!at.done()) refuse('unsupported', at.peek().text);
  return { kind: 'function', name, exported, params, returns, body };
}

function literalType(value) {
  const literal = value.kind === 'unary' && value.op === 'negate' ? value.operand : value;
  if (!['number', 'string', 'boolean'].includes(literal.kind)) refuse('statement');
  return literal.kind;
}

function jsType(written) {
  const id = written === undefined || written === null ? undefined : idBySpelling(constructMap().types, 'javascript', written);
  if (!id) refuse('untyped', written ?? 'none');
  return id;
}

function jsBlock(at) {
  at.expect('{');
  const body = [];
  while (!at.eat('}')) {
    if (at.peek().type === 'end') refuse('unsupported', 'an unclosed block');
    body.push(jsStatement(at));
  }
  return body;
}

function jsStatement(at) {
  const token = at.peek();
  if (at.eat('const')) {
    const name = snakeFromCamel(at.next().text);
    at.expect('=');
    const value = expression(at, 'JavaScript');
    at.expect(';');
    return { kind: 'let', name, value };
  }
  if (at.eat('return')) {
    if (at.is(';')) refuse('unsupported', 'a return without a value');
    const value = expression(at, 'JavaScript');
    at.expect(';');
    return { kind: 'return', value };
  }
  if (at.eat('if')) {
    at.expect('(');
    const cond = expression(at, 'JavaScript');
    at.expect(')');
    const then = jsBlock(at);
    let otherwise = null;
    if (at.eat('else')) otherwise = at.is('if') ? [jsStatement(at)] : jsBlock(at);
    return { kind: 'if', cond, then, else: otherwise };
  }
  if (['for', 'while', 'do'].includes(token.text)) refuse('loop');
  if (['let', 'var'].includes(token.text) || ASSIGNMENTS.has(at.peek(1).text)) refuse('mutable');
  if (token.text === 'await') refuse('async');
  return refuse('unsupported', `the statement ${token.text}`);
}

/**
 * Parse one Rust item into portable-pure-v1.
 * @param {Array<object>} tokens
 * @returns {object}
 */
export function parseRustItem(tokens) {
  const at = cursor(tokens);
  const term = itemTerm(tokens, 'Rust');
  if (term === 'use') refuse('no-definition');
  if (term !== 'function' && term !== 'constant') refuse('rust-item');
  while (at.is('#')) {
    at.next();
    at.expect('[');
    const attribute = at.next().text;
    if (attribute !== 'must_use' && attribute !== 'inline') refuse('rust-item', `#[${attribute}]`);
    at.expect(']');
  }
  const exported = at.eat('pub');
  if (at.eat('const') && !at.is('fn')) {
    const name = at.next().text;
    if (!/^[A-Z][A-Z0-9_]+$/u.test(name)) refuse('name', name);
    at.expect(':');
    const type = rustType(at);
    at.expect('=');
    const value = expression(at, 'Rust');
    at.expect(';');
    return { kind: 'constant', name, exported, type, value };
  }
  if (at.is('async')) refuse('async');
  at.expect('fn');
  const name = canonicalRustName(at.next().text);
  if (at.is('<')) refuse('unsupported', 'a generic function');
  at.expect('(');
  const params = [];
  while (!at.eat(')')) {
    if (at.is('mut')) refuse('mutable');
    const param = canonicalRustName(at.next().text);
    at.expect(':');
    params.push({ name: param, type: rustType(at) });
    if (!at.is(')')) at.expect(',');
  }
  if (!at.eat('->')) refuse('untyped', 'no result type');
  const returns = rustType(at);
  const body = rustBlock(at);
  if (!at.done()) refuse('unsupported', at.peek().text);
  return { kind: 'function', name, exported, params, returns, body };
}

function rustType(at) {
  let written = '';
  if (at.eat('&')) {
    written = '&';
    if (at.peek().type === 'char') at.next();
  }
  written += at.next().text;
  const map = constructMap().types;
  const id = idBySpelling(map, 'rust', written) ?? idBySpelling(map, 'rustOwned', written) ?? (written === '&String' ? 'string' : undefined);
  if (!id) refuse('untyped', written);
  return id;
}

function rustBlock(at) {
  at.expect('{');
  const body = [];
  while (!at.eat('}')) {
    if (at.peek().type === 'end') refuse('unsupported', 'an unclosed block');
    if (at.eat('let')) {
      if (at.is('mut')) refuse('mutable');
      const name = canonicalRustName(at.next().text);
      if (at.eat(':')) rustType(at);
      at.expect('=');
      const value = expression(at, 'Rust');
      at.expect(';');
      body.push({ kind: 'let', name, value });
    } else if (at.eat('return')) {
      const value = expression(at, 'Rust');
      at.expect(';');
      body.push({ kind: 'return', value });
    } else if (at.is('if')) {
      body.push(rustIf(at));
    } else if (['for', 'while', 'loop'].includes(at.peek().text)) {
      refuse('loop');
    } else {
      const value = expression(at, 'Rust');
      if (!at.is('}')) refuse(ASSIGNMENTS.has(at.peek().text) ? 'mutable' : 'unsupported', 'an expression statement');
      body.push({ kind: 'return', value });
    }
  }
  return body;
}

function rustIf(at) {
  at.expect('if');
  const cond = expression(at, 'Rust');
  const then = rustBlock(at);
  let otherwise = null;
  if (at.eat('else')) otherwise = at.is('if') ? [rustIf(at)] : rustBlock(at);
  return { kind: 'if', cond, then, else: otherwise };
}

/**
 * One expression of `language`, by precedence climbing over the construct
 * map's operator spellings.
 * @param {ReturnType<typeof cursor>} at
 * @param {string} language
 * @returns {object}
 */
function expression(at, language) {
  if (language === 'Rust' && at.is('if')) {
    at.next();
    const cond = binary(at, language, 0);
    const then = rustValueBlock(at);
    at.expect('else');
    const otherwise = at.is('if') ? expression(at, language) : rustValueBlock(at);
    return { kind: 'choose', cond, then, else: otherwise };
  }
  const cond = binary(at, language, 0);
  if (language === 'JavaScript' && at.eat('?')) {
    const then = expression(at, language);
    at.expect(':');
    return { kind: 'choose', cond, then, else: expression(at, language) };
  }
  return cond;
}

function rustValueBlock(at) {
  at.expect('{');
  const value = expression(at, 'Rust');
  at.expect('}');
  return value;
}

function binary(at, language, minimum) {
  const operators = constructMap().operators;
  const key = language === 'Rust' ? 'rust' : 'javascript';
  let left = unary(at, language);
  for (;;) {
    const token = at.peek();
    if (token.type !== 'punct' && token.type !== 'ident') return left;
    const op = token.type === 'punct' ? idBySpelling(operators, key, token.text) : undefined;
    if (!op) {
      if (UNSUPPORTED_BINARY.has(token.text)) refuse('unsupported', `the operator ${token.text}`);
      return left;
    }
    const { precedence } = operators.get(op);
    if (precedence < minimum) return left;
    at.next();
    const right = binary(at, language, precedence + 1);
    left = { kind: 'binary', op, left, right };
  }
}

function unary(at, language) {
  const key = language === 'Rust' ? 'rust' : 'javascript';
  const token = at.peek();
  if (language === 'Rust' && at.eat('&')) return unary(at, language);
  if (token.type === 'punct') {
    const op = idBySpelling(constructMap().unaries, key, token.text);
    if (op) {
      at.next();
      return { kind: 'unary', op, operand: unary(at, language) };
    }
  }
  if (['typeof', 'void', 'delete', 'new'].includes(token.text)) refuse('unsupported', token.text);
  if (token.text === 'await') refuse('async');
  return postfix(at, language);
}

function postfix(at, language) {
  let value = primary(at, language);
  for (;;) {
    if (at.is('[')) refuse('collection');
    if (at.is('?.')) refuse('unsupported', 'optional chaining');
    if (!at.is('.')) return value;
    at.next();
    const name = at.next().text;
    if (language === 'Rust' && TRANSPARENT_RUST.has(name)) {
      at.expect('(');
      at.expect(')');
      continue;
    }
    if (name === 'length' || name === 'len') refuse('length');
    if (!at.is('(')) refuse('collection', `the property ${name}`);
    const method = idBySpelling(constructMap().methods, language === 'Rust' ? 'rust' : 'javascript', name);
    if (!method) refuse('unsupported', `the method ${name}`);
    value = { kind: 'method', name: method, receiver: value, args: argumentsOf(at, language) };
  }
}

function argumentsOf(at, language) {
  at.expect('(');
  const args = [];
  while (!at.eat(')')) {
    args.push(expression(at, language));
    if (!at.is(')')) at.expect(',');
  }
  return args;
}

function primary(at, language) {
  const token = at.next();
  if (token.type === 'number') return { kind: 'number', text: numberText(token.text, language) };
  if (token.type === 'string') return { kind: 'string', value: unquote(token.text) };
  if (token.type === 'template') refuse('template');
  if (token.type === 'regex' || token.type === 'char') refuse('unsupported', token.text);
  if (token.text === '(') {
    const value = expression(at, language);
    at.expect(')');
    return value;
  }
  if (token.text === '[' || token.text === '{') refuse('collection');
  if (token.type !== 'ident') refuse('unsupported', token.text || 'the end of the item');
  if (token.text === 'true' || token.text === 'false') return { kind: 'boolean', value: token.text === 'true' };
  if (['null', 'undefined', 'this', 'self', 'Self', 'function', 'None', 'Some'].includes(token.text)) refuse('unsupported', token.text);
  if (at.is('=>')) refuse('unsupported', 'an arrow function');
  const builtins = constructMap().builtins;
  if (language === 'JavaScript' && token.text === 'Math' && at.eat('.')) {
    const callee = `Math.${at.next().text}`;
    if (callee === 'Math.round') refuse('rounding');
    const name = idBySpelling(builtins, 'javascript', callee);
    if (!name) refuse('unsupported', callee);
    return { kind: 'builtin', name, args: argumentsOf(at, language) };
  }
  if (language === 'Rust' && at.is('::')) {
    at.next();
    const callee = `${token.text}::${at.next().text}`;
    if (callee === 'String::from') {
      const [value] = argumentsOf(at, language);
      return value;
    }
    if (callee === 'f64::round') refuse('rounding');
    const name = idBySpelling(builtins, 'rust', callee);
    if (!name) refuse('unsupported', callee);
    return { kind: 'builtin', name, args: argumentsOf(at, language) };
  }
  if (language === 'Rust' && token.text.endsWith('!')) refuse('unsupported', `the macro ${token.text}`);
  const name = language === 'Rust' ? canonicalRustName(token.text) : snakeFromCamel(token.text);
  if (at.is('(')) return { kind: 'call', name, args: argumentsOf(at, language) };
  return { kind: 'variable', name };
}

/**
 * The canonical text of a decimal literal: the JavaScript `String` of its
 * value, so `2`, `2.0` and `2_f64` all read `2`.
 * @param {string} text
 * @param {string} language
 * @returns {string}
 */
function numberText(text, language) {
  const plain = language === 'Rust' ? text.replace(/_?f64$/u, '') : text;
  if (!/^\d[\d_]*(\.\d[\d_]*)?$/u.test(plain)) refuse('unsupported', `the literal ${text}`);
  const canonical = String(Number(plain.replace(/_/gu, '')));
  if (/e/u.test(canonical)) refuse('unsupported', `the literal ${text}`);
  return canonical;
}

function unquote(text) {
  const body = text.slice(1, -1);
  return body.replace(/\\(.)/gu, (_, char) => {
    if (!(char in ESCAPES)) refuse('unsupported', `the escape \\${char}`);
    return ESCAPES[char];
  });
}

/**
 * The JavaScript name of a canonical name.
 * @param {string} name
 * @returns {string}
 */
export function javascriptName(name) {
  return camelFromSnake(name);
}
