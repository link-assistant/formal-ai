// The Rust and JavaScript emitters of portable-pure-v1. Each spells a
// checked IR item (expressions carry `type`) with the construct map's
// spellings and parenthesizes by its precedences.
//
// The Rust emitter writes the layout `rustfmt --edition 2024 --style-edition
// 2024` keeps for items whose lines fit in 100 columns (a two-way `if` stays
// on one line up to 50 columns, rustfmt's `single_line_if_else_max_width`),
// so a translation committed beside hand-written Rust passes `cargo fmt`
// unchanged. Strings are borrowed (`&str`) as parameters and owned
// (`String`) as results, the split meta-language's emitter also makes; the
// emitter borrows an owned value where a parameter or pattern wants `&str`
// and calls `.to_string()` where a borrowed value is returned.

import { constructMap } from './constructs.mjs';
import { camelFromSnake } from './ir.mjs';

const PRIMARY = 20;
const UNARY = 14;
const CHOOSE = 0;
const SINGLE_LINE_IF = 50;

const isComparison = (op) => op !== undefined && [8, 9].includes(constructMap().operators.get(op)?.precedence);

/**
 * The digits of a canonical number as a Rust `f64` literal, grouped by
 * three past four integer digits so clippy's `unreadable_literal` holds.
 * @param {string} text
 * @returns {string}
 */
function rustNumber(text) {
  const [whole, fraction] = text.split('.');
  const grouped = whole.length > 4 ? whole.replace(/\B(?=(\d{3})+$)/gu, '_') : whole;
  return `${grouped}.${fraction ?? '0'}`;
}

const rustString = (value) => `"${value.replace(/[\\"\n\t\r\0]/gu, (char) => ({ '\\': '\\\\', '"': '\\"', '\n': '\\n', '\t': '\\t', '\r': '\\r', '\0': '\\0' })[char])}"`;
const jsString = (value) => `'${value.replace(/[\\'\n\t\r\0]/gu, (char) => ({ '\\': '\\\\', "'": "\\'", '\n': '\\n', '\t': '\\t', '\r': '\\r', '\0': '\\0' })[char])}'`;

/**
 * Spell a checked item in Rust.
 * @param {object} item
 * @returns {{code: string, lints: Set<string>}}
 */
export function emitRust(item) {
  const map = constructMap();
  const lints = new Set();
  const rustExpr = (expr, env) => {
    switch (expr.kind) {
      case 'number':
        return { text: rustNumber(expr.text), prec: PRIMARY, owned: false };
      case 'string':
        return { text: rustString(expr.value), prec: PRIMARY, owned: false };
      case 'boolean':
        return { text: String(expr.value), prec: PRIMARY, owned: false };
      case 'variable':
        return { text: expr.name, prec: PRIMARY, owned: env.get(expr.name) ?? false };
      case 'binary': {
        const row = map.operators.get(expr.op);
        if ((expr.op === 'equal' || expr.op === 'not-equal') && expr.left.type === 'number') lints.add('clippy::float_cmp');
        if ((expr.op === 'add' || expr.op === 'subtract') && [expr.left, expr.right].some((side) => side.kind === 'binary' && side.op === 'multiply')) {
          lints.add('clippy::suboptimal_flops');
        }
        const left = rustExpr(expr.left, env);
        const right = rustExpr(expr.right, env);
        const comparison = isComparison(expr.op);
        const l = left.prec < row.precedence || (comparison && isComparison(expr.left.op) && expr.left.kind === 'binary') ? `(${left.text})` : left.text;
        const r = right.prec <= row.precedence || (comparison && isComparison(expr.right.op) && expr.right.kind === 'binary') ? `(${right.text})` : right.text;
        return { text: `${l} ${row.rust} ${r}`, prec: row.precedence, owned: false };
      }
      case 'unary': {
        const operand = rustExpr(expr.operand, env);
        const wrap = operand.prec < UNARY || (expr.operand.kind === 'unary' && expr.operand.op === expr.op);
        return { text: `${map.unaries.get(expr.op).rust}${wrap ? `(${operand.text})` : operand.text}`, prec: UNARY, owned: false };
      }
      case 'choose': {
        const cond = rustExpr(expr.cond, env);
        let then = rustExpr(expr.then, env);
        let otherwise = rustExpr(expr.else, env);
        if (expr.type === 'string' && then.owned !== otherwise.owned) {
          then = owned(then);
          otherwise = owned(otherwise);
        }
        return { text: `if ${cond.text} { ${then.text} } else { ${otherwise.text} }`, prec: CHOOSE, owned: then.owned };
      }
      case 'call':
        return { text: `${expr.name}(${expr.args.map((arg) => borrowed(rustExpr(arg, env))).join(', ')})`, prec: PRIMARY, owned: expr.type === 'string' };
      case 'builtin':
        if (expr.name === 'sqrt') lints.add('clippy::imprecise_flops');
        return { text: `${map.builtins.get(expr.name).rust}(${expr.args.map((arg) => rustExpr(arg, env).text).join(', ')})`, prec: PRIMARY, owned: false };
      case 'method': {
        const receiver = rustExpr(expr.receiver, env);
        const text = receiver.prec < PRIMARY ? `(${receiver.text})` : receiver.text;
        return { text: `${text}.${map.methods.get(expr.name).rust}(${expr.args.map((arg) => borrowed(rustExpr(arg, env))).join(', ')})`, prec: PRIMARY, owned: false };
      }
      default:
        throw new Error(`no Rust spelling for ${expr.kind}`);
    }
  };
  const result = (expr, env) => {
    const value = rustExpr(expr, env);
    return item.returns === 'string' ? owned(value).text : value.text;
  };
  const body = (statements, env, indent) => {
    const pad = '    '.repeat(indent);
    const lines = [];
    for (const statement of statements) {
      if (statement.kind === 'let') {
        const value = rustExpr(statement.value, env);
        env.set(statement.name, value.owned);
        if (statement.value.kind === 'choose' && value.text.length > SINGLE_LINE_IF) {
          const scope = new Map(env);
          lines.push(`${pad}let ${statement.name} = if ${rustExpr(statement.value.cond, scope).text} {`);
          lines.push(`${pad}    ${rustExpr(statement.value.then, scope).text}`, `${pad}} else {`, `${pad}    ${rustExpr(statement.value.else, scope).text}`, `${pad}};`);
        } else {
          lines.push(`${pad}let ${statement.name} = ${value.text};`);
        }
      } else if (statement.kind === 'return') {
        lines.push(`${pad}${result(statement.value, env)}`);
      } else {
        lines.push(...branch(statement, env, indent));
      }
    }
    return lines;
  };
  const branch = (statement, env, indent) => {
    const pad = '    '.repeat(indent);
    const single = (side) => side.length === 1 && side[0].kind === 'return';
    const cond = rustExpr(statement.cond, env).text;
    if (single(statement.then) && single(statement.else)) {
      const line = `if ${cond} { ${result(statement.then[0].value, env)} } else { ${result(statement.else[0].value, env)} }`;
      if (line.length <= SINGLE_LINE_IF) return [`${pad}${line}`];
    }
    const lines = [`${pad}if ${cond} {`, ...body(statement.then, new Map(env), indent + 1)];
    let rest = statement.else;
    while (rest.length === 1 && rest[0].kind === 'if') {
      lines.push(`${pad}} else if ${rustExpr(rest[0].cond, env).text} {`, ...body(rest[0].then, new Map(env), indent + 1));
      rest = rest[0].else;
    }
    lines.push(`${pad}} else {`, ...body(rest, new Map(env), indent + 1), `${pad}}`);
    return lines;
  };
  const visibility = item.exported ? 'pub ' : '';
  if (item.kind === 'constant') {
    const type = map.types.get(item.type);
    return { code: `${visibility}const ${item.name}: ${type.rust} = ${rustExpr(item.value, new Map()).text};`, lints };
  }
  lints.add('clippy::missing_const_for_fn');
  const params = item.params.map((param) => `${param.name}: ${map.types.get(param.type).rust}`).join(', ');
  const env = new Map(item.params.map((param) => [param.name, false]));
  const lines = [
    '#[must_use]',
    `${visibility}fn ${item.name}(${params}) -> ${map.types.get(item.returns).rustOwned} {`,
    ...body(item.body, env, 1),
    '}',
  ];
  return { code: lines.join('\n'), lints };
}

function owned(value) {
  if (value.owned) return value;
  return { text: value.prec < PRIMARY ? `(${value.text}).to_string()` : `${value.text}.to_string()`, prec: PRIMARY, owned: true };
}

function borrowed(value) {
  if (!value.owned) return value.text;
  return value.prec < UNARY ? `&(${value.text})` : `&${value.text}`;
}

/**
 * The `#![allow(...)]` prelude line for `lints`, laid out as rustfmt lays
 * out a function-like attribute (one line while its arguments fit in 70
 * columns, `attr_fn_like_width`).
 * @param {Set<string>} lints
 * @returns {string | null}
 */
export function rustAllow(lints) {
  if (lints.size === 0) return null;
  const sorted = [...lints].sort();
  const inline = sorted.join(', ');
  return inline.length <= 70 ? `#![allow(${inline})]` : `#![allow(\n${sorted.map((lint) => `    ${lint}`).join(',\n')}\n)]`;
}

/**
 * Spell an IR item in JavaScript, with the JSDoc types the JavaScript
 * frontend reads back under the item's description lines.
 * @param {object} item
 * @param {Array<string>} [description] the source's doc comment lines
 * @returns {string}
 */
export function emitJavaScript(item, description = []) {
  const map = constructMap();
  const js = (expr) => {
    switch (expr.kind) {
      case 'number':
        return { text: expr.text, prec: PRIMARY };
      case 'string':
        return { text: jsString(expr.value), prec: PRIMARY };
      case 'boolean':
        return { text: String(expr.value), prec: PRIMARY };
      case 'variable':
        return { text: camelFromSnake(expr.name), prec: PRIMARY };
      case 'binary': {
        const row = map.operators.get(expr.op);
        const left = js(expr.left);
        const right = js(expr.right);
        const l = left.prec < row.precedence ? `(${left.text})` : left.text;
        const r = right.prec <= row.precedence ? `(${right.text})` : right.text;
        return { text: `${l} ${row.javascript} ${r}`, prec: row.precedence };
      }
      case 'unary': {
        const operand = js(expr.operand);
        const wrap = operand.prec < UNARY || (expr.operand.kind === 'unary' && expr.operand.op === expr.op);
        return { text: `${map.unaries.get(expr.op).javascript}${wrap ? `(${operand.text})` : operand.text}`, prec: UNARY };
      }
      case 'choose': {
        const cond = js(expr.cond);
        return { text: `${cond.prec <= 2 ? `(${cond.text})` : cond.text} ? ${js(expr.then).text} : ${js(expr.else).text}`, prec: 2 };
      }
      case 'call':
        return { text: `${camelFromSnake(expr.name)}(${expr.args.map((arg) => js(arg).text).join(', ')})`, prec: PRIMARY };
      case 'builtin':
        return { text: `${map.builtins.get(expr.name).javascript}(${expr.args.map((arg) => js(arg).text).join(', ')})`, prec: PRIMARY };
      case 'method': {
        const receiver = js(expr.receiver);
        const text = receiver.prec < PRIMARY ? `(${receiver.text})` : receiver.text;
        return { text: `${text}.${map.methods.get(expr.name).javascript}(${expr.args.map((arg) => js(arg).text).join(', ')})`, prec: PRIMARY };
      }
      default:
        throw new Error(`no JavaScript spelling for ${expr.kind}`);
    }
  };
  const body = (statements, indent) => {
    const pad = '  '.repeat(indent);
    const lines = [];
    for (const statement of statements) {
      if (statement.kind === 'let') {
        lines.push(`${pad}const ${camelFromSnake(statement.name)} = ${js(statement.value).text};`);
      } else if (statement.kind === 'return') {
        lines.push(`${pad}return ${js(statement.value).text};`);
      } else {
        lines.push(`${pad}if (${js(statement.cond).text}) {`, ...body(statement.then, indent + 1));
        let rest = statement.else;
        while (rest !== null && rest.length === 1 && rest[0].kind === 'if') {
          lines.push(`${pad}} else if (${js(rest[0].cond).text}) {`, ...body(rest[0].then, indent + 1));
          rest = rest[0].else;
        }
        if (rest !== null) lines.push(`${pad}} else {`, ...body(rest, indent + 1));
        lines.push(`${pad}}`);
      }
    }
    return lines;
  };
  const exported = item.exported ? 'export ' : '';
  if (item.kind === 'constant') return `${exported}const ${item.name} = ${js(item.value).text};`;
  const type = (id) => map.types.get(id).javascript;
  return [
    '/**',
    ...description.map((line) => (line === '' ? ' *' : ` * ${line}`)),
    ...item.params.map((param) => ` * @param {${type(param.type)}} ${camelFromSnake(param.name)}`),
    ` * @returns {${type(item.returns)}}`,
    ' */',
    `${exported}function ${camelFromSnake(item.name)}(${item.params.map((param) => camelFromSnake(param.name)).join(', ')}) {`,
    ...body(item.body, 1),
    '}',
  ].join('\n');
}
