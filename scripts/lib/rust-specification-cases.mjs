// Rust specification tests → JavaScript cases (R1188-U29, R1188-U30): the
// rust → js direction of the automated translation, as a recorded workaround
// until the meta language reads Rust tests.
//
// The specification suite under rust/tests/unit/specification/ pins most
// requirements through the native engine only. Many of its tests have one
// shape: ask the engine a prompt, then assert that the answer contains (or
// does not contain) a text, or that the intent is a name. Such a test means
// the same thing for the browser worker, so this module reads it into a case
// that a JavaScript runner can ask the worker. A test with any other
// statement is left to Rust and counted.
//
// The reader is a small tokenizer (strings, raw strings, identifiers,
// punctuation; comments dropped) and a matcher of whole statements.

import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

/**
 * The tokens of a Rust source, comments dropped.
 * @param {string} source
 * @returns {Array<{kind: string, text: string}>}
 */
export function tokenize(source) {
  const tokens = [];
  let index = 0;
  while (index < source.length) {
    const rest = source.slice(index);
    const space = /^\s+/u.exec(rest);
    if (space) {
      index += space[0].length;
      continue;
    }
    if (rest.startsWith('//')) {
      const end = source.indexOf('\n', index);
      index = end < 0 ? source.length : end;
      continue;
    }
    if (rest.startsWith('/*')) {
      const end = source.indexOf('*/', index + 2);
      index = end < 0 ? source.length : end + 2;
      continue;
    }
    const raw = /^r(#*)"/u.exec(rest);
    if (raw) {
      const close = `"${raw[1]}`;
      const end = source.indexOf(close, index + raw[0].length);
      tokens.push({ kind: 'string', text: source.slice(index + raw[0].length, end) });
      index = end + close.length;
      continue;
    }
    if (rest.startsWith('"')) {
      const { value, length } = readString(rest);
      tokens.push({ kind: 'string', text: value });
      index += length;
      continue;
    }
    const char = /^'(?:\\.|[^\\'])'/u.exec(rest);
    if (char) {
      tokens.push({ kind: 'char', text: char[0] });
      index += char[0].length;
      continue;
    }
    const word = /^[A-Za-z_][A-Za-z0-9_]*/u.exec(rest);
    if (word) {
      tokens.push({ kind: 'word', text: word[0] });
      index += word[0].length;
      continue;
    }
    const number = /^[0-9][0-9_.a-z]*/u.exec(rest);
    if (number) {
      tokens.push({ kind: 'number', text: number[0] });
      index += number[0].length;
      continue;
    }
    tokens.push({ kind: 'punct', text: rest[0] });
    index += 1;
  }
  return tokens;
}

/**
 * Read one double-quoted Rust string literal at the start of `text`.
 * @param {string} text
 * @returns {{value: string, length: number}}
 */
function readString(text) {
  let value = '';
  let index = 1;
  while (index < text.length && text[index] !== '"') {
    if (text[index] !== '\\') {
      value += text[index];
      index += 1;
      continue;
    }
    const next = text[index + 1];
    if (next === 'u') {
      const close = text.indexOf('}', index);
      value += String.fromCodePoint(Number.parseInt(text.slice(index + 3, close), 16));
      index = close + 1;
    } else if (next === '\n') {
      index += 2;
      while (/\s/u.test(text[index])) {
        index += 1;
      }
    } else {
      value += { n: '\n', t: '\t', r: '\r', 0: '\0' }[next] ?? next;
      index += 2;
    }
  }
  return { value, length: index + 1 };
}

/**
 * The `#[test]` functions of a token list, with their body tokens.
 * Ignored tests (`#[ignore]`) are left out.
 * @param {Array<{kind: string, text: string}>} tokens
 * @returns {Array<{name: string, body: Array<{kind: string, text: string}>}>}
 */
export function testFunctions(tokens) {
  const tests = [];
  for (let index = 0; index < tokens.length; index += 1) {
    if (!isAttribute(tokens, index, 'test')) {
      continue;
    }
    let cursor = index + 4;
    let ignored = false;
    while (tokens[cursor]?.text === '#') {
      ignored ||= tokens[cursor + 2]?.text === 'ignore';
      cursor = closing(tokens, cursor + 1) + 1;
    }
    if (tokens[cursor]?.text !== 'fn') {
      continue;
    }
    const name = tokens[cursor + 1].text;
    let open = cursor;
    while (tokens[open].text !== '{') {
      open += 1;
    }
    const end = closing(tokens, open);
    if (!ignored) {
      tests.push({ name, body: tokens.slice(open + 1, end) });
    }
    index = end;
  }
  return tests;
}

function isAttribute(tokens, index, name) {
  return tokens[index]?.text === '#' && tokens[index + 1]?.text === '[' && tokens[index + 2]?.text === name
    && tokens[index + 3]?.text === ']';
}

/**
 * The index of the bracket that closes the one at `open`.
 * @param {Array<{kind: string, text: string}>} tokens
 * @param {number} open
 * @returns {number}
 */
function closing(tokens, open) {
  const pairs = { '(': ')', '[': ']', '{': '}' };
  const stack = [];
  for (let index = open; index < tokens.length; index += 1) {
    const { kind, text } = tokens[index];
    if (kind !== 'punct') {
      continue;
    }
    if (pairs[text]) {
      stack.push(pairs[text]);
    } else if (text === stack.at(-1)) {
      stack.pop();
      if (stack.length === 0) {
        return index;
      }
    }
  }
  return tokens.length;
}

/**
 * Split a body into statements at top-level `;`.
 * @param {Array<{kind: string, text: string}>} body
 * @returns {Array<Array<{kind: string, text: string}>>}
 */
function statements(body) {
  const result = [[]];
  let depth = 0;
  // A trailing comma before a closing bracket (rustfmt's layout) means nothing.
  const tokens = body.filter((token, index) => !(token.text === ',' && token.kind === 'punct'
    && [')', ']', '}'].includes(body[index + 1]?.text)));
  for (const token of tokens) {
    if (token.kind === 'punct' && '([{'.includes(token.text)) {
      depth += 1;
    } else if (token.kind === 'punct' && ')]}'.includes(token.text)) {
      depth -= 1;
    }
    if (depth === 0 && token.text === ';' && token.kind === 'punct') {
      result.push([]);
    } else {
      result.at(-1).push(token);
    }
  }
  return result.filter((statement) => statement.length > 0);
}

/**
 * Whether `tokens` spell `pattern` exactly. A pattern element is a literal
 * text, or `STR` (any string) or `ID` (any word), which are captured.
 * @param {Array<{kind: string, text: string}>} tokens
 * @param {Array<string>} pattern
 * @returns {Array<string> | null}
 */
function match(tokens, pattern) {
  if (tokens.length !== pattern.length) {
    return null;
  }
  const captured = [];
  for (let index = 0; index < pattern.length; index += 1) {
    const element = pattern[index];
    const token = tokens[index];
    if (element === 'STR' && token.kind === 'string') {
      captured.push(token.text);
    } else if (element === 'ID' && token.kind === 'word') {
      captured.push(token.text);
    } else if (element !== token.text || token.kind === 'string') {
      return null;
    }
  }
  return captured;
}

const FIELDS = { answer: 'content', intent: 'intent' };

/**
 * The assertion of one statement, without its message arguments.
 * @param {Array<{kind: string, text: string}>} statement
 * @param {Map<string, string>} bindings name → `response` or `content`
 * @returns {{binding: string, field: string, op: string, value: string, lower: boolean} | null}
 */
function assertion(statement, bindings) {
  const macro = match(statement.slice(0, 3), ['ID', '!', '(']);
  if (!macro || statement.at(-1).text !== ')') {
    return null;
  }
  const args = splitArguments(statement.slice(3, -1));
  if (macro[0] === 'assert') {
    return condition(args[0], bindings);
  }
  if ((macro[0] === 'assert_eq' || macro[0] === 'assert_ne') && args.length >= 2) {
    const [left, right] = args;
    const value = match(right, ['STR']);
    const field = fieldOf(left, bindings);
    if (!value || !field) {
      return null;
    }
    return { ...field, value: value[0], lower: false, op: macro[0] === 'assert_eq' ? 'equals' : 'differs' };
  }
  return null;
}

/**
 * A boolean condition: one test, or tests joined by `&&` (every one holds)
 * or by `||` (one of them holds).
 * @returns {object | null}
 */
function condition(tokens, bindings) {
  for (const [operator, op] of [['|', 'any'], ['&', 'all']]) {
    const parts = splitOperator(tokens, operator);
    if (parts.length > 1) {
      const checks = parts.map((part) => condition(part, bindings));
      return checks.every(Boolean) ? { op, checks } : null;
    }
  }
  const negated = tokens[0]?.text === '!';
  const expression = negated ? tokens.slice(1) : tokens;
  const evidence = evidenceOf(expression, bindings);
  if (evidence) {
    return { ...evidence, op: negated ? `lacks-${evidence.op}` : `has-${evidence.op}` };
  }
  const subject = subjectOf(expression, bindings);
  return subject ? { ...subject, op: negated ? 'excludes' : 'includes' } : null;
}

/**
 * Split at a top-level doubled operator (`||` or `&&`).
 */
function splitOperator(tokens, operator) {
  const parts = [[]];
  let depth = 0;
  for (let index = 0; index < tokens.length; index += 1) {
    const { kind, text } = tokens[index];
    if (kind === 'punct' && '([{'.includes(text)) {
      depth += 1;
    } else if (kind === 'punct' && ')]}'.includes(text)) {
      depth -= 1;
    }
    if (depth === 0 && kind === 'punct' && text === operator && tokens[index + 1]?.text === operator) {
      parts.push([]);
      index += 1;
    } else {
      parts.at(-1).push(tokens[index]);
    }
  }
  return parts;
}

/**
 * The response field an expression reads: `name.answer`, `name.intent`, or a
 * bound content string.
 */
function fieldOf(tokens, bindings) {
  const bound = match(tokens, ['ID']);
  if (bound && bindings.get(bound[0]) === 'content') {
    return { binding: bound[0], field: 'content' };
  }
  const read = match(tokens, ['ID', '.', 'ID']);
  if (read && bindings.get(read[0]) === 'response' && FIELDS[read[1]]) {
    return { binding: read[0], field: FIELDS[read[1]] };
  }
  return null;
}

const ANY_LINK = ['ID', '.', 'evidence_links', '.', 'iter', '(', ')', '.', 'any', '(', '|', 'ID', '|'];

/**
 * `<response>.evidence_links.iter().any(|link| link == "…")`, or with
 * `link.starts_with("…")`: an evidence link equal to, or starting with, a text.
 */
function evidenceOf(tokens, bindings) {
  for (const [suffix, op] of [
    [['ID', '=', '=', 'STR', ')'], 'evidence'],
    [['ID', '.', 'starts_with', '(', 'STR', ')', ')'], 'evidence-prefix'],
  ]) {
    const captured = match(tokens, [...ANY_LINK, ...suffix]);
    if (captured && bindings.get(captured[0]) === 'response' && captured[1] === captured[2]) {
      return { binding: captured[0], field: 'evidence', value: captured[3], lower: false, op };
    }
  }
  return null;
}

/**
 * `<field>.contains("…")` or `<field>.to_lowercase().contains("…")`.
 */
function subjectOf(tokens, bindings) {
  for (const [suffix, lower] of [
    [['.', 'contains', '(', 'STR', ')'], false],
    [['.', 'to_lowercase', '(', ')', '.', 'contains', '(', 'STR', ')'], true],
  ]) {
    const head = tokens.slice(0, tokens.length - suffix.length);
    const tail = match(tokens.slice(tokens.length - suffix.length), suffix);
    const field = tail && fieldOf(head, bindings);
    if (field) {
      return { ...field, value: tail[0], lower };
    }
  }
  return null;
}

function splitArguments(tokens) {
  const args = [[]];
  let depth = 0;
  for (const token of tokens) {
    if (token.kind === 'punct' && '([{'.includes(token.text)) {
      depth += 1;
    } else if (token.kind === 'punct' && ')]}'.includes(token.text)) {
      depth -= 1;
    }
    if (depth === 0 && token.kind === 'punct' && token.text === ',') {
      args.push([]);
    } else {
      args.at(-1).push(token);
    }
  }
  return args.filter((arg) => arg.length > 0);
}

/** An offline solver with the default configuration, as the worker runs. */
const SOLVERS = [
  ['let', 'ID', '=', 'UniversalSolver', ':', ':', 'default', '(', ')'],
  ['let', 'ID', '=', 'UniversalSolver', ':', ':', 'new', '(', 'SolverConfig', '{', 'offline', ':', 'true', ',', '.',
    '.', 'SolverConfig', ':', ':', 'default', '(', ')', '}', ')'],
];

/**
 * A prompt asked of the engine: `let r = <helper>("…")`,
 * `let r = FormalAiEngine.answer("…")` or `let r = <solver>.solve("…")`,
 * each optionally followed by `.answer` (the bound value is then the text).
 */
function askOf(statement, bindings, helper) {
  const calls = [['FormalAiEngine', '.', 'answer'], ['ID', '.', 'solve']];
  if (helper) {
    calls.push([helper]);
  }
  for (const call of calls) {
    for (const [tail, kind] of [[[], 'response'], [['.', 'answer'], 'content']]) {
      const captured = match(statement, ['let', 'ID', '=', ...call, '(', 'STR', ')', ...tail]);
      if (!captured) {
        continue;
      }
      if (call[0] === 'ID' && bindings.get(captured[1]) !== 'solver') {
        continue;
      }
      return { binding: captured[0], prompt: captured.at(-1), kind };
    }
  }
  return null;
}

/**
 * The JavaScript case of one test, or why it stays in Rust.
 * @param {Array<{kind: string, text: string}>} body
 * @param {string} helper the file's `fn <helper>(prompt: &str) -> SymbolicAnswer`
 * @returns {{case?: {asks: Array<{binding: string, prompt: string}>, checks: Array<object>}, reason?: string}}
 */
export function caseOf(body, helper) {
  const bindings = new Map();
  const asks = [];
  const checks = [];
  for (const statement of statements(body)) {
    const solver = SOLVERS.map((pattern) => match(statement, pattern)).find(Boolean);
    if (solver) {
      bindings.set(solver[0], 'solver');
      continue;
    }
    const ask = askOf(statement, bindings, helper);
    if (ask) {
      bindings.set(ask.binding, ask.kind);
      asks.push({ binding: ask.binding, prompt: ask.prompt });
      continue;
    }
    const check = assertion(statement, bindings);
    if (!check) {
      return { reason: statement.slice(0, 4).map((token) => token.text).join(' ') };
    }
    checks.push(check);
  }
  if (asks.length === 0 || checks.length === 0) {
    return { reason: 'no prompt or no check' };
  }
  return { case: { asks, checks } };
}

/**
 * The name of a file's prompt helper: a `fn <name>(<arg>: &str) ->
 * SymbolicAnswer`, or null.
 * @param {string} source
 * @returns {string | null}
 */
export function promptHelper(source) {
  return /\bfn ([a-z_]+)\(\s*[a-z_]+: &str\s*\)\s*->\s*SymbolicAnswer\b/u.exec(source)?.[1] ?? null;
}

export const SPECIFICATION = 'rust/tests/unit/specification';

/**
 * The prompt helper a specification file calls: its own, or its parent
 * module's when it imports it with `use super::`.
 * @param {string} root
 * @param {string} name the file's path under the specification directory
 * @param {string} source
 * @returns {string | null}
 */
export function helperOf(root, name, source) {
  const own = promptHelper(source);
  if (own || !name.includes('/')) {
    return own;
  }
  const parent = join(root, SPECIFICATION, `${name.split('/').slice(0, -1).join('/')}.rs`);
  let inherited = null;
  try {
    inherited = promptHelper(readFileSync(parent, 'utf8'));
  } catch {
    return null;
  }
  const imported = new RegExp(`use super::(?:\\*|\\{[^}]*\\b${inherited}\\b|${inherited}\\b)`, 'u');
  return inherited && imported.test(source) ? inherited : null;
}

/**
 * Every specification test the extractor carries, and how many there are.
 * @param {string} root
 * @returns {{total: number, cases: Array<{id: string, asks: Array<object>, checks: Array<object>}>}}
 */
export function specificationCases(root = '.') {
  const files = readdirSync(join(root, SPECIFICATION), { recursive: true })
    .map(String)
    .filter((name) => name.endsWith('.rs'))
    .sort();
  const cases = [];
  let total = 0;
  for (const name of files) {
    const source = readFileSync(join(root, SPECIFICATION, name), 'utf8');
    const helper = helperOf(root, name, source);
    for (const test of testFunctions(tokenize(source))) {
      total += 1;
      const result = caseOf(test.body, helper);
      if (result.case) {
        cases.push({ id: `${SPECIFICATION}/${name}::${test.name}`, ...result.case });
      }
    }
  }
  return { total, cases };
}

const HOLDS = {
  includes: (subject, value) => subject.includes(value),
  excludes: (subject, value) => !subject.includes(value),
  equals: (subject, value) => subject === value,
  differs: (subject, value) => subject !== value,
  'has-evidence': (links, value) => links.includes(value),
  'lacks-evidence': (links, value) => !links.includes(value),
  'has-evidence-prefix': (links, value) => links.some((link) => link.startsWith(value)),
  'lacks-evidence-prefix': (links, value) => !links.some((link) => link.startsWith(value)),
};

/**
 * Null when one check holds for the worker responses (`{content, intent,
 * evidence}` by binding); otherwise what was expected and what came.
 * @param {{binding?: string, field?: string, op: string, value?: string, lower?: boolean, checks?: Array<object>}} check
 * @param {Map<string, {content?: string, intent?: string, evidence?: Array<string>}>} responses
 * @returns {string | null}
 */
export function checkFailure(check, responses) {
  if (check.op === 'all') {
    return check.checks.map((part) => checkFailure(part, responses)).find(Boolean) ?? null;
  }
  if (check.op === 'any') {
    const failures = check.checks.map((part) => checkFailure(part, responses));
    return failures.some((failure) => failure === null) ? null : failures.join(' or ');
  }
  const response = responses.get(check.binding);
  const values = {
    content: String(response.content ?? ''),
    intent: String(response.intent ?? ''),
    evidence: response.evidence ?? [],
  };
  const subject = check.lower ? values[check.field].toLowerCase() : values[check.field];
  if (HOLDS[check.op](subject, check.value)) {
    return null;
  }
  const got = check.field === 'content' ? values.content.slice(0, 160) : values[check.field];
  return `${check.field} ${check.op} ${JSON.stringify(check.value)}; got ${JSON.stringify(got)}`;
}

/**
 * Ask the worker every case and return the failing ones with their first
 * failed check.
 * @param {{solve: (prompt: string) => Promise<object>}} host
 * @param {Array<{id: string, asks: Array<{binding: string, prompt: string}>, checks: Array<object>}>} cases
 * @returns {Promise<Map<string, string>>}
 */
export async function failingCases(host, cases) {
  const failing = new Map();
  for (const item of cases) {
    const responses = new Map();
    for (const ask of item.asks) {
      responses.set(ask.binding, await host.solve(ask.prompt));
    }
    const failure = item.checks.map((check) => checkFailure(check, responses)).find(Boolean);
    if (failure) {
      failing.set(item.id, failure);
    }
  }
  return failing;
}
