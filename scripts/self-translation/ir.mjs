// portable-pure-v1: the meta-language form both frontends produce and both
// emitters consume. An item is a pure function or a literal constant; a
// function body is `let` bindings followed by one `return` or one
// two-way `if` whose branches are bodies of the same shape. The IR is
// written as Links Notation (`toLino` / `fromLino`), one item per link, the
// way relative-meta-logic's `portable-natural-v1` network spells its
// fragment (PR #184), so the meta leg is a document either runtime reads.
//
// Item:  { kind: 'function', name, exported, params: [{ name, type }], returns, body }
//        { kind: 'constant', name, exported, type, value }
// Stmt:  { kind: 'let', name, value } | { kind: 'return', value }
//        | { kind: 'if', cond, then: [Stmt], else: [Stmt] | null }
// Expr:  { kind: 'number', text } | { kind: 'string', value } | { kind: 'boolean', value }
//        | { kind: 'variable', name } | { kind: 'binary', op, left, right }
//        | { kind: 'unary', op, operand } | { kind: 'choose', cond, then, else }
//        | { kind: 'call', name, args } | { kind: 'builtin', name, args }
//        | { kind: 'method', name, receiver, args }
// Names are canonical snake_case (UPPER_SNAKE for constants); `type` is a
// type row of the construct map. `check` adds `type` to every expression.

import { constructMap, refuse } from './constructs.mjs';
import { encodeWord, field, parseLinks, textOf, writeLink } from './lino.mjs';

export const FRAGMENT = 'portable-pure-v1';

// The keyword vocabulary of both languages: a portable name is none of
// these, the rule relative-meta-logic applies across its four languages.
const KEYWORDS = new Set(`
  as async await break case catch class const continue crate debugger default delete do dyn else
  enum export extends extern false finally fn for function if impl import in instanceof let loop
  match mod move mut new null pub ref return self static struct super switch this throw trait true
  try type typeof undefined unsafe use var void where while with yield abstract become box final
  macro override priv typeof unsized virtual gen
`.trim().split(/\s+/u));

/**
 * The canonical snake_case of a JavaScript camelCase name, refusing a name
 * whose spelling does not come back.
 * @param {string} name
 * @returns {string}
 */
export function snakeFromCamel(name) {
  if (/^[A-Z][A-Z0-9_]*$/u.test(name) && name.length > 1) return name;
  if (!/^[a-z][A-Za-z0-9]*$/u.test(name)) refuse('name', name);
  const snake = name.replace(/[A-Z]/gu, (letter) => `_${letter.toLowerCase()}`);
  if (KEYWORDS.has(snake) || KEYWORDS.has(name) || camelFromSnake(snake) !== name) refuse('name', name);
  return snake;
}

/**
 * The JavaScript camelCase of a canonical snake_case name.
 * @param {string} name
 * @returns {string}
 */
export function camelFromSnake(name) {
  if (/^[A-Z][A-Z0-9_]*$/u.test(name) && name.length > 1) return name;
  if (!/^[a-z][a-z0-9]*(_[a-z][a-z0-9]*)*$/u.test(name) || KEYWORDS.has(name)) refuse('name', name);
  return name.replace(/_([a-z])/gu, (_, letter) => letter.toUpperCase());
}

/**
 * Check a Rust name is canonical (snake_case or UPPER_SNAKE) and portable.
 * @param {string} name
 * @returns {string}
 */
export function canonicalRustName(name) {
  camelFromSnake(name);
  return name;
}

const terminates = (body) => {
  const last = body[body.length - 1];
  if (!last) return false;
  if (last.kind === 'return') return true;
  return last.kind === 'if' && last.else !== null && terminates(last.then) && terminates(last.else);
};

/**
 * The canonical body: an `if` without `else` whose branch returns takes the
 * statements after it as its `else`, a returned conditional is a two-way
 * `if`, and a two-way branch never tests a negation (its branches are
 * swapped instead, the form clippy's `if_not_else` asks for), so both
 * frontends meet one shape and the Rust it is spelled as stays lint-clean.
 * @param {Array<object>} body
 * @returns {Array<object>}
 */
export function normalizeBody(body) {
  const out = [];
  for (let index = 0; index < body.length; index += 1) {
    const statement = canonicalStatement(body[index]);
    if (statement.kind !== 'if') {
      if (statement.kind === 'return' && index < body.length - 1) refuse('unsupported', 'a statement after return');
      out.push(statement);
      continue;
    }
    const then = normalizeBody(statement.then);
    if (statement.else === null && index < body.length - 1 && terminates(then)) {
      out.push(twoWay(statement.cond, then, normalizeBody(body.slice(index + 1))));
      return out;
    }
    if (index < body.length - 1 || statement.else === null) refuse('missing-return');
    out.push(twoWay(statement.cond, then, normalizeBody(statement.else)));
  }
  if (!terminates(out)) refuse('missing-return');
  return out;
}

function canonicalStatement(statement) {
  if (statement.kind === 'let') return { ...statement, value: canonicalExpr(statement.value) };
  if (statement.kind === 'if') return { ...statement, cond: canonicalExpr(statement.cond) };
  const value = canonicalExpr(statement.value);
  if (value.kind !== 'choose') return { ...statement, value };
  return {
    kind: 'if',
    cond: value.cond,
    then: [{ kind: 'return', value: value.then }],
    else: [{ kind: 'return', value: value.else }],
  };
}

function twoWay(cond, then, otherwise) {
  const positive = negation(cond);
  const elseIf = otherwise.length === 1 && otherwise[0].kind === 'if';
  return positive && !elseIf ? { kind: 'if', cond: positive, then: otherwise, else: then } : { kind: 'if', cond, then, else: otherwise };
}

function negation(cond) {
  if (cond.kind === 'unary' && cond.op === 'not') return cond.operand;
  if (cond.kind === 'binary' && cond.op === 'not-equal') return { ...cond, op: 'equal' };
  return null;
}

function canonicalExpr(expr) {
  switch (expr.kind) {
    case 'binary':
      return { ...expr, left: canonicalExpr(expr.left), right: canonicalExpr(expr.right) };
    case 'unary':
      return { ...expr, operand: canonicalExpr(expr.operand) };
    case 'choose': {
      const cond = canonicalExpr(expr.cond);
      const then = canonicalExpr(expr.then);
      const otherwise = canonicalExpr(expr.else);
      const positive = negation(cond);
      return positive ? { kind: 'choose', cond: positive, then: otherwise, else: then } : { kind: 'choose', cond, then, else: otherwise };
    }
    case 'call':
    case 'builtin':
      return { ...expr, args: expr.args.map(canonicalExpr) };
    case 'method':
      return { ...expr, receiver: canonicalExpr(expr.receiver), args: expr.args.map(canonicalExpr) };
    default:
      return expr;
  }
}

/**
 * The signatures a module's items declare, for calls between them.
 * @param {Array<object>} items
 * @returns {Map<string, object>}
 */
export function signatures(items) {
  const table = new Map();
  for (const item of items) table.set(item.name, item);
  return table;
}

/**
 * Type-check `item` against the module `table`; returns a copy whose
 * expressions carry `type`. Refuses what the construct map does not accept.
 * @param {object} item
 * @param {Map<string, object>} table
 * @returns {object}
 */
export function check(item, table) {
  const map = constructMap();
  for (const type of item.kind === 'function' ? [...item.params.map((param) => param.type), item.returns] : [item.type]) {
    if (!map.types.has(type)) refuse('untyped', type);
  }
  if (item.kind === 'constant') {
    const value = expression(item.value, new Map(), table);
    const literal = item.value.kind === 'unary' && item.value.op === 'negate' ? item.value.operand : item.value;
    if (!['number', 'string', 'boolean'].includes(literal.kind) || value.type !== item.type) refuse('type', item.name);
    return { ...item, value };
  }
  const scope = new Map(item.params.map((param) => [param.name, param.type]));
  const block = (body, names) => body.map((statement) => {
    if (statement.kind === 'let') {
      const value = expression(statement.value, names, table);
      names.set(statement.name, value.type);
      return { ...statement, value };
    }
    if (statement.kind === 'return') {
      const value = expression(statement.value, names, table);
      if (value.type !== item.returns) refuse('type', `returns ${value.type}, not ${item.returns}`);
      return { ...statement, value };
    }
    const cond = expression(statement.cond, names, table);
    if (cond.type !== 'boolean') refuse('type', 'a condition that is not boolean');
    return {
      ...statement,
      cond,
      then: block(statement.then, new Map(names)),
      else: statement.else === null ? null : block(statement.else, new Map(names)),
    };
  });
  return { ...item, body: block(item.body, scope) };
}

function expression(expr, names, table) {
  const map = constructMap();
  const typed = (fields, type) => ({ ...expr, ...fields, type });
  switch (expr.kind) {
    case 'number':
      return typed({}, 'number');
    case 'string':
      return typed({}, 'string');
    case 'boolean':
      return typed({}, 'boolean');
    case 'variable': {
      if (names.has(expr.name)) return typed({}, names.get(expr.name));
      const item = table.get(expr.name);
      if (item && item.kind === 'constant') return typed({}, item.type);
      return refuse('unsupported', `the free name ${expr.name}`);
    }
    case 'binary': {
      const row = map.operators.get(expr.op);
      const left = expression(expr.left, names, table);
      const right = expression(expr.right, names, table);
      if (row.operands === 'any' ? left.type !== right.type : left.type !== row.operands || right.type !== row.operands) {
        return refuse(left.type === 'string' && expr.op === 'add' ? 'concatenation' : 'type', `${expr.op} of ${left.type} and ${right.type}`);
      }
      return typed({ left, right }, row.yields);
    }
    case 'unary': {
      const row = map.unaries.get(expr.op);
      const operand = expression(expr.operand, names, table);
      if (operand.type !== row.operand) refuse('type', `${expr.op} of ${operand.type}`);
      return typed({ operand }, row.yields);
    }
    case 'choose': {
      const cond = expression(expr.cond, names, table);
      const then = expression(expr.then, names, table);
      const otherwise = expression(expr.else, names, table);
      if (cond.type !== 'boolean' || then.type !== otherwise.type) refuse('type', 'a conditional expression');
      return typed({ cond, then, else: otherwise }, then.type);
    }
    case 'call': {
      const callee = table.get(expr.name);
      if (!callee || callee.kind !== 'function') return refuse('unknown-call', expr.name);
      const args = expr.args.map((arg) => expression(arg, names, table));
      if (args.length !== callee.params.length || args.some((arg, index) => arg.type !== callee.params[index].type)) {
        refuse('type', `the arguments of ${expr.name}`);
      }
      return typed({ args }, callee.returns);
    }
    case 'builtin': {
      const row = map.builtins.get(expr.name);
      const args = expr.args.map((arg) => expression(arg, names, table));
      if (args.length !== row.arguments.length || args.some((arg, index) => arg.type !== row.arguments[index])) refuse('type', expr.name);
      return typed({ args }, row.yields);
    }
    case 'method': {
      const row = map.methods.get(expr.name);
      const receiver = expression(expr.receiver, names, table);
      const args = expr.args.map((arg) => expression(arg, names, table));
      if (receiver.type !== row.receiver || args.length !== row.arguments.length || args.some((arg, index) => arg.type !== row.arguments[index])) {
        refuse('type', expr.name);
      }
      return typed({ receiver, args }, row.yields);
    }
    default:
      return refuse('unsupported', expr.kind);
  }
}

/**
 * The item as one Links Notation link.
 * @param {object} item
 * @returns {string}
 */
export function toLino(item) {
  const flag = item.exported ? [['exported']] : [];
  if (item.kind === 'constant') return writeLink(['constant', item.name, ...flag, ['type', item.type], ['value', linkOf(item.value)]]);
  return writeLink([
    'function',
    item.name,
    ...flag,
    ['parameters', ...item.params.map((param) => [param.name, param.type])],
    ['returns', item.returns],
    ['body', ...item.body.map(statementLink)],
  ]);
}

function statementLink(statement) {
  if (statement.kind === 'let') return ['let', statement.name, linkOf(statement.value)];
  if (statement.kind === 'return') return ['return', linkOf(statement.value)];
  const link = ['if', linkOf(statement.cond), ['then', ...statement.then.map(statementLink)]];
  if (statement.else !== null) link.push(['else', ...statement.else.map(statementLink)]);
  return link;
}

function linkOf(expr) {
  switch (expr.kind) {
    case 'number':
      return ['number', expr.text];
    case 'string':
      return ['string', encodeWord(expr.value)];
    case 'boolean':
      return ['boolean', String(expr.value)];
    case 'variable':
      return ['variable', expr.name];
    case 'binary':
      return ['binary', expr.op, linkOf(expr.left), linkOf(expr.right)];
    case 'unary':
      return ['unary', expr.op, linkOf(expr.operand)];
    case 'choose':
      return ['choose', linkOf(expr.cond), linkOf(expr.then), linkOf(expr.else)];
    case 'method':
      return ['method', expr.name, linkOf(expr.receiver), ...expr.args.map(linkOf)];
    default:
      return [expr.kind, expr.name, ...expr.args.map(linkOf)];
  }
}

/**
 * The items of a meta document: every `function` and `constant` link.
 * @param {string} text
 * @returns {Array<object>}
 */
export function fromLino(text) {
  return parseLinks(text).filter((link) => link[0] === 'function' || link[0] === 'constant').map(itemOf);
}

function itemOf(link) {
  const exported = Boolean(field(link, 'exported'));
  if (link[0] === 'constant') {
    return { kind: 'constant', name: link[1], exported, type: field(link, 'type')[1], value: exprOf(field(link, 'value')[1]) };
  }
  return {
    kind: 'function',
    name: link[1],
    exported,
    params: field(link, 'parameters').slice(1).map(([name, type]) => ({ name, type })),
    returns: field(link, 'returns')[1],
    body: field(link, 'body').slice(1).map(statementOf),
  };
}

function statementOf(link) {
  if (link[0] === 'let') return { kind: 'let', name: link[1], value: exprOf(link[2]) };
  if (link[0] === 'return') return { kind: 'return', value: exprOf(link[1]) };
  const otherwise = field(link, 'else');
  return {
    kind: 'if',
    cond: exprOf(link[1]),
    then: field(link, 'then').slice(1).map(statementOf),
    else: otherwise ? otherwise.slice(1).map(statementOf) : null,
  };
}

function exprOf(link) {
  const [kind, ...rest] = link;
  switch (kind) {
    case 'number':
      return { kind, text: rest[0] };
    case 'string':
      return { kind, value: textOf(rest[0]) };
    case 'boolean':
      return { kind, value: rest[0] === 'true' };
    case 'variable':
      return { kind, name: rest[0] };
    case 'binary':
      return { kind, op: rest[0], left: exprOf(rest[1]), right: exprOf(rest[2]) };
    case 'unary':
      return { kind, op: rest[0], operand: exprOf(rest[1]) };
    case 'choose':
      return { kind, cond: exprOf(rest[0]), then: exprOf(rest[1]), else: exprOf(rest[2]) };
    case 'method':
      return { kind, name: rest[0], receiver: exprOf(rest[1]), args: rest.slice(2).map(exprOf) };
    case 'call':
    case 'builtin':
      return { kind, name: rest[0], args: rest.slice(1).map(exprOf) };
    default:
      throw new Error(`unknown meta expression ${kind}`);
  }
}
