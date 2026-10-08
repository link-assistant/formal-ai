// The lowering workarounds of the js -> rust translation (R1188-U30).
//
// meta-language's portable core refuses methods of values, functions as
// values and string lengths. For an item carried for those constructs alone,
// this pass rewrites the JavaScript into a form the pinned core translates,
// then hands the result back to the same upstream frontend, checker and
// emitter that selfTranslate uses:
//
//   string-methods   `text.slice(a, b)` becomes `waStrSlice(text, a, b)`, a
//                    call of a typed stub declared beside the item. The stub's
//                    emitted Rust is replaced by the Rust that
//                    data/meta/translation-workarounds.lino records for it
//                    (`stub` rows), which follows JavaScript's UTF-16 semantics.
//                    A receiver is first read as a string, then as an array of
//                    strings or of numbers, the first that type-checks.
//   string-length    `text.length`, refused on a string with a span, becomes
//                    `waStrLength(text)`.
//   array-push       `xs.push(v);` on an array the item declares becomes
//                    `xs = [...xs, v];` (and its `const` a `let`).
//   callback-loops   `const r = xs.map((x) => e);` (and `.filter`, `.some`,
//                    `.every`, or `return xs.map(…)`) becomes a for…of loop.
//
// The item is translated with the translated items of its module that it
// names (their source joins the program, their Rust is not repeated) and with
// the signatures of what it imports. The projection keeps the item's source
// (`// |`) and shows the lowered source (`// ~`).

import { createHash } from 'node:crypto';

import { tokenize } from './translation-blockers.mjs';

const sha256 = (text) => createHash('sha256').update(text, 'utf8').digest('hex');
export const LOWERED_LINE = '// ~';
const KEYWORDS = new Set(['return', 'typeof', 'case', 'new', 'in', 'of', 'if', 'while', 'for', 'else', 'do', 'throw', 'await', 'yield', 'void', 'delete']);
const CALLBACKS = new Set(['map', 'filter', 'some', 'every']);

// ---------------------------------------------------------------- the stub table

/**
 * The stubs of data/meta/translation-workarounds.lino: under `stubs <rule>`,
 * which states the defaults its stubs share (`receiver`, `arity`) once, each
 * `stub <name>` gives a typed JavaScript declaration the lowered item calls
 * and the Rust (`rust` lines) that replaces its emitted body.
 * @param {string} text
 * @returns {{ name: string, rule: string, method: string, receiver: string, arity: number, javascript: string, rust: string[], needs: string[] }[]}
 */
export function readStubs(text) {
  const stubs = [];
  let group = null;
  let current = null;
  const value = (field) => (field[2] !== undefined ? field[2].replace(/\\(["\\])/gu, '$1') : field[3] ?? field[4]);
  for (const line of text.split('\n')) {
    const head = /^stubs (\S+)$/u.exec(line);
    if (head) {
      group = { rule: head[1], receiver: '', arity: -1 };
      current = null;
      continue;
    }
    if (/^\S/u.test(line)) {
      group = null;
      continue;
    }
    if (!group) continue;
    const stub = /^ {2}stub (\S+)$/u.exec(line);
    if (stub) {
      current = { name: stub[1], rule: group.rule, method: '', receiver: group.receiver, arity: group.arity, javascript: '', rust: [], needs: [] };
      stubs.push(current);
      continue;
    }
    const field = /^( {2}| {4})(\w+) (?:"((?:[^"\\]|\\.)*)"|'([^']*)'|(\S+))$/u.exec(line);
    if (!field) continue;
    const [, indent, key] = field;
    const data = value(field.slice(1));
    if (indent.length === 2) {
      // A default of the group.
      if (key === 'arity') group.arity = Number(data);
      else if (key === 'receiver') group.receiver = data;
    } else if (current) {
      if (key === 'rust') current.rust.push(data);
      else if (key === 'needs') current.needs.push(data);
      else if (key === 'arity') current.arity = Number(data);
      else current[key] = data;
    }
  }
  return stubs;
}

/** The Rust name meta-language gives a JavaScript function name. */
export function rustName(name) {
  return name.replace(/([a-z0-9])([A-Z])/gu, '$1_$2').replace(/([A-Z])([A-Z][a-z])/gu, '$1_$2').toLowerCase();
}

// ---------------------------------------------------------------- token helpers

const significant = (source) => tokenize(source).filter((token) => token.kind !== 'comment');

/** The index of the token that closes the bracket opened at `open`. */
function closing(tokens, open) {
  let depth = 0;
  for (let at = open; at < tokens.length; at += 1) {
    if (tokens[at].kind !== 'punct') continue;
    if (['(', '[', '{', '${'].includes(tokens[at].value)) depth += 1;
    if ([')', ']', '}'].includes(tokens[at].value)) {
      depth -= 1;
      if (depth === 0) return at;
    }
  }
  return -1;
}

/** The index of the token that opens the bracket closed at `close`. */
function opening(tokens, close) {
  let depth = 0;
  for (let at = close; at >= 0; at -= 1) {
    if (tokens[at].kind !== 'punct') continue;
    if ([')', ']', '}'].includes(tokens[at].value)) depth += 1;
    if (['(', '[', '{', '${'].includes(tokens[at].value)) {
      depth -= 1;
      if (depth === 0) return at;
    }
  }
  return -1;
}

/** The first token of the receiver ending at token `last`: a chain of names, calls, indexes and fields. */
function receiverStart(tokens, last) {
  let at = last;
  for (;;) {
    const token = tokens[at];
    if (!token || token.value === '}') return -1;
    if (token.kind === 'punct' && (token.value === ')' || token.value === ']')) {
      at = opening(tokens, at);
      if (at < 0) return -1;
      const callee = tokens[at - 1];
      if (token.value === ')' && callee?.kind === 'word' && !KEYWORDS.has(callee.value)) at -= 1;
      else if (token.value === ']' && callee && (callee.kind === 'word' || [')', ']'].includes(callee.value))) {
        at -= 1;
        continue;
      }
    } else if (token.kind === 'word') {
      if (KEYWORDS.has(token.value)) return -1;
    } else if (!['string', 'template'].includes(token.kind)) {
      return -1;
    }
    if (tokens[at - 1]?.value === '.' && tokens[at - 2]) {
      at -= 2;
      continue;
    }
    return at;
  }
}

/** The argument token ranges of the call whose `(` is at `open`. */
function callArguments(tokens, open) {
  const close = closing(tokens, open);
  const args = [];
  let start = open + 1;
  let depth = 0;
  for (let at = open + 1; at < close; at += 1) {
    const value = tokens[at].kind === 'punct' ? tokens[at].value : '';
    if (['(', '[', '{', '${'].includes(value)) depth += 1;
    if ([')', ']', '}'].includes(value)) depth -= 1;
    if (value === ',' && depth === 0) {
      args.push([start, at - 1]);
      start = at + 1;
    }
  }
  if (start < close) args.push([start, close - 1]);
  return { close, args };
}

// A template's substitution tokens follow the template token, so a range ends at its furthest token.
const textOf = (source, tokens, [first, last]) => source.slice(tokens[first].start,
  Math.max(...tokens.slice(first, last + 1).map((token) => token.end)));


// ---------------------------------------------------------------- the rules

/**
 * `xs.push(a, b);` on an array the item declares with `const xs = [` or
 * `let xs = [` becomes `xs = [...xs, a, b];`, and the declaration a `let`.
 * @param {string} source
 * @returns {string|null} the rewritten source, or null when nothing applies
 */
export function lowerPush(source) {
  const tokens = significant(source);
  for (let at = 0; at + 4 < tokens.length; at += 1) {
    if (!(tokens[at].kind === 'word' && tokens[at + 1].value === '.' && tokens[at + 2].value === 'push' && tokens[at + 3].value === '(')) continue;
    const name = tokens[at].value;
    const before = tokens[at - 1];
    const statementStart = !before || [';', '{', '}'].includes(before.value) || before.value === 'else'
      || (before.value === ')' && ['for', 'if', 'while'].includes(tokens[opening(tokens, at - 1) - 1]?.value));
    const { close, args } = callArguments(tokens, at + 3);
    if (!statementStart || tokens[close + 1]?.value !== ';' || args.length === 0) continue;
    const declared = tokens.findIndex((token, index) => ['const', 'let'].includes(token.value) && tokens[index + 1]?.value === name
      && tokens[index + 2]?.value === '=' && tokens[index + 3]?.value === '[');
    if (declared < 0) continue;
    const spread = `${name} = [...${name}, ${args.map((range) => textOf(source, tokens, range)).join(', ')}];`;
    let out = `${source.slice(0, tokens[at].start)}${spread}${source.slice(tokens[close + 1].end)}`;
    if (tokens[declared].value === 'const') out = `${out.slice(0, tokens[declared].start)}let${out.slice(tokens[declared].end)}`;
    return out;
  }
  return null;
}

/**
 * `const r = xs.map((x) => e);`, `return xs.some((x) => e);` and the like
 * become a for…of loop over `xs` that builds `r`.
 * @param {string} source
 * @param {number} fresh a number for the result name of a `return`
 * @returns {string|null}
 */
export function lowerCallback(source, fresh = 1) {
  const tokens = significant(source);
  for (let at = 0; at < tokens.length; at += 1) {
    const head = tokens[at];
    const declares = ['const', 'let'].includes(head.value) && tokens[at + 1]?.kind === 'word' && tokens[at + 2]?.value === '=';
    if (!declares && head.value !== 'return') continue;
    const first = declares ? at + 3 : at + 1;
    // The receiver chain ends at the `.` before the method.
    let dot = -1;
    for (let index = first; index < tokens.length; index += 1) {
      if (tokens[index].value === '.' && CALLBACKS.has(tokens[index + 1]?.value) && tokens[index + 2]?.value === '(') {
        dot = index;
        break;
      }
      if ([';', '?', ':', '+', '-', '&&', '||', '===', '!==', ','].includes(tokens[index].value) && tokens[index].kind === 'punct') break;
    }
    if (dot < 0 || receiverStart(tokens, dot - 1) !== first) continue;
    const method = tokens[dot + 1].value;
    const open = dot + 2;
    const { close, args } = callArguments(tokens, open);
    if (args.length !== 1 || tokens[close + 1]?.value !== ';') continue;
    const [argFirst, argLast] = args[0];
    let param;
    let bodyFirst;
    if (tokens[argFirst].kind === 'word' && tokens[argFirst + 1]?.value === '=>') {
      param = tokens[argFirst].value;
      bodyFirst = argFirst + 2;
    } else if (tokens[argFirst].value === '(' && tokens[argFirst + 1]?.kind === 'word' && tokens[argFirst + 2]?.value === ')' && tokens[argFirst + 3]?.value === '=>') {
      param = tokens[argFirst + 1].value;
      bodyFirst = argFirst + 4;
    } else {
      continue;
    }
    if (bodyFirst > argLast || tokens[bodyFirst].value === '{') continue;
    const receiver = textOf(source, tokens, [first, dot - 1]);
    const body = textOf(source, tokens, [bodyFirst, argLast]);
    const name = declares ? tokens[at + 1].value : `waResult${fresh}`;
    const loop = {
      map: [`let ${name} = [];`, `for (const ${param} of ${receiver}) {`, `  ${name} = [...${name}, ${body}];`, '}'],
      filter: [`let ${name} = [];`, `for (const ${param} of ${receiver}) {`, `  if (${body}) {`, `    ${name} = [...${name}, ${param}];`, '  }', '}'],
      some: [`let ${name} = false;`, `for (const ${param} of ${receiver}) {`, `  if (${body}) {`, `    ${name} = true;`, '    break;', '  }', '}'],
      every: [`let ${name} = true;`, `for (const ${param} of ${receiver}) {`, `  if (!(${body})) {`, `    ${name} = false;`, '    break;', '  }', '}'],
    }[method];
    if (!declares) loop.push(`return ${name};`);
    const indent = /[ \t]*$/u.exec(source.slice(0, head.start))[0];
    return `${source.slice(0, head.start)}${loop.join(`\n${indent}`)}${source.slice(tokens[close + 1].end)}`;
  }
  return null;
}

/**
 * The first `receiver.method(args)` call a stub covers, rewritten as a call
 * of the stub `receivers` picks for its method.
 * @param {string} source
 * @param {ReturnType<typeof readStubs>} stubs
 * @param {Map<string, string>} receivers method -> receiver kind (default 'string')
 * @returns {{ text: string, stub: string }|null}
 */
export function lowerMethod(source, stubs, receivers = new Map()) {
  const tokens = significant(source);
  for (let at = 1; at + 2 < tokens.length; at += 1) {
    if (!(tokens[at].value === '.' && tokens[at + 1].kind === 'word' && tokens[at + 2].value === '(')) continue;
    const method = tokens[at + 1].value;
    const receiver = receivers.get(method) ?? stubs.find((entry) => entry.method === method)?.receiver;
    const { close, args } = callArguments(tokens, at + 2);
    const stub = stubs.find((entry) => entry.method === method && entry.receiver === receiver && entry.arity === args.length);
    if (!stub) continue;
    const start = receiverStart(tokens, at - 1);
    if (start < 0 || /^[A-Z]/u.test(tokens[start].value)) continue;
    if (['replace', 'replaceAll'].includes(method)) {
      // A string pattern and a replacement without `$` patterns read the same in Rust.
      const [pattern, replacement] = args;
      if (tokens[pattern[0]].kind === 'regex' || pattern[0] !== pattern[1]) continue;
      if (tokens[replacement[0]].kind !== 'string' || replacement[0] !== replacement[1] || tokens[replacement[0]].value.includes('$')) continue;
    }
    const call = `${stub.name}(${[textOf(source, tokens, [start, at - 1]), ...args.map((range) => textOf(source, tokens, range))].join(', ')})`;
    return { text: `${source.slice(0, tokens[start].start)}${call}${source.slice(tokens[close].end)}`, stub: stub.name };
  }
  return null;
}

/**
 * `receiver.length` whose `.length` lies inside `span` becomes `waStrLength(receiver)`.
 * @param {string} source
 * @param {{ start: number, end: number }} span offsets in `source`
 * @returns {string|null}
 */
export function lowerLength(source, span) {
  const tokens = significant(source);
  for (let at = 1; at < tokens.length; at += 1) {
    if (tokens[at].value !== 'length' || tokens[at - 1].value !== '.' || tokens[at + 1]?.value === '(') continue;
    if (tokens[at].end < span.start || tokens[at - 1].start > span.end) continue;
    const start = receiverStart(tokens, at - 2);
    if (start < 0) continue;
    return `${source.slice(0, tokens[start].start)}waStrLength(${textOf(source, tokens, [start, at - 2])})${source.slice(tokens[at].end)}`;
  }
  return null;
}

/**
 * `for (const c of text)` whose loop lies inside `span` iterates
 * `waStrChars(text)`: a string iterates by code point, which upstream reads
 * as an array (`one value is used as a string and as an array`).
 * @param {string} source
 * @param {{ start: number, end: number }} span
 * @returns {string|null}
 */
export function lowerStringLoop(source, span) {
  const tokens = significant(source);
  for (let at = 0; at + 4 < tokens.length; at += 1) {
    if (tokens[at].value !== 'for' || tokens[at + 1].value !== '(' || !['const', 'let'].includes(tokens[at + 2].value) || tokens[at + 4]?.value !== 'of') continue;
    if (tokens[at].start < span.start || tokens[at].start > span.end) continue;
    const close = closing(tokens, at + 1);
    if (close < at + 6 || tokens[at + 5].value === 'waStrChars') continue;
    const iterable = textOf(source, tokens, [at + 5, close - 1]);
    return `${source.slice(0, tokens[at + 5].start)}waStrChars(${iterable})${source.slice(tokens[close - 1].end)}`;
  }
  return null;
}

/**
 * Applies the source rules to an item until none applies.
 * @param {string} source
 * @param {ReturnType<typeof readStubs>} stubs
 * @param {Map<string, string>} receivers
 * @returns {{ text: string, rules: Set<string>, stubs: Set<string> }}
 */
export function lowerItem(source, stubs, receivers = new Map()) {
  let text = source;
  const rules = new Set();
  const used = new Set();
  for (let round = 0; round < 200; round += 1) {
    const push = lowerPush(text);
    if (push !== null) {
      text = push;
      rules.add('array-push');
      continue;
    }
    const loop = lowerCallback(text, round + 1);
    if (loop !== null) {
      text = loop;
      rules.add('callback-loops');
      continue;
    }
    const method = lowerMethod(text, stubs, receivers);
    if (method !== null) {
      text = method.text;
      used.add(method.stub);
      rules.add(stubs.find((entry) => entry.name === method.stub).rule);
      continue;
    }
    break;
  }
  return { text, rules, stubs: used };
}

// ---------------------------------------------------------------- translating a lowered item

/** The names an item mentions. */
const mentions = (source) => new Set(significant(source).filter((token) => token.kind === 'word').map((token) => token.value));

/** The name a top-level item declares, if any. */
export function declaredName(source) {
  const tokens = significant(source);
  let at = tokens[0]?.value === 'export' ? 1 : 0;
  if (tokens[at]?.value === 'async') at += 1;
  if (['function', 'const', 'let'].includes(tokens[at]?.value) && tokens[at + 1]?.kind === 'word') return tokens[at + 1].value;
  return null;
}

/**
 * Translates one carried item through the lowering rules: tries the receiver
 * readings of its methods, lowers string lengths by the refusal's span, and
 * returns the item's Rust and the preludes it needs, or null.
 * @param {object} upstream `{ parseJavaScript, checkProgram, emitRust, TranslationError }` of the pinned checkout
 * @param {{ source: string, siblings: Map<string, string>, externals: object[], stubs: ReturnType<typeof readStubs> }} item
 *   siblings: name -> (lowered) source of the module's translated items
 * @returns {{ code: string, lowered: string, rules: string[], preludes: string[], stubs: string[] }|null}
 */
export function translateLowered(upstream, { source, siblings, externals, stubs, hint = '' }) {
  // A method several receivers have is read as each of them in turn, the first one recorded first.
  const kinds = new Map();
  for (const stub of stubs) if (stub.method) kinds.set(stub.method, [...new Set([...(kinds.get(stub.method) ?? []), stub.receiver])]);
  const readings = [new Map(), ...[...kinds].flatMap(([method, list]) => list.slice(1).map((kind) => new Map([[method, kind]])))];
  for (const receivers of readings) {
    let lowered = lowerItem(source, stubs, receivers);
    // An item with nothing to lower is tried only when upstream refused a string length.
    if (!lowered.rules.size && !hint.startsWith('length of a string')) return null;
    // Another reading of a method the item never calls gives the same program again.
    if (receivers.size && ![...receivers.keys()].some((method) => source.includes(`.${method}(`))) continue;
    for (let attempt = 0; attempt < 20; attempt += 1) {
      const result = emitWith(upstream, lowered, siblings, externals, stubs);
      if (result.code !== undefined) return result;
      const span = result.error?.span;
      const message = result.error?.message ?? '';
      if (!span) break;
      const at = { start: span.start - result.offset, end: span.end - result.offset };
      // Two refusals with a span name a string construct the rules lower.
      const [next, stub] = message.startsWith('length of a string') ? [lowerLength(lowered.text, at), 'waStrLength']
        : message.startsWith('one value is used as a string and as an array') ? [lowerStringLoop(lowered.text, at), 'waStrChars'] : [null, null];
      if (next === null) break;
      lowered = { ...lowered, text: next, rules: new Set([...lowered.rules, 'string-methods']), stubs: new Set([...lowered.stubs, stub]) };
    }
  }
  return null;
}

function emitWith(upstream, lowered, siblings, externals, stubs) {
  const used = stubs.filter((stub) => lowered.stubs.has(stub.name));
  // The translated items the lowered item names, and the ones they name.
  const context = [];
  const seen = new Set();
  const visit = (text) => {
    for (const name of mentions(text)) {
      if (seen.has(name) || !siblings.has(name)) continue;
      seen.add(name);
      visit(siblings.get(name));
      context.push(siblings.get(name));
    }
  };
  visit(lowered.text);
  const prefix = [...used.map((stub) => stub.javascript), ...context].join('\n\n');
  const emit = (text) => {
    try {
      const program = upstream.checkProgram(upstream.parseJavaScript(text, { externals }));
      if (program.main.effects.length > 0) return { error: { message: 'top-level statement' } };
      return { emitted: upstream.emitRust(program) };
    } catch (error) {
      if (!(error instanceof upstream.TranslationError)) throw error;
      return { error };
    }
  };
  const offset = prefix ? prefix.length + 2 : 0;
  const whole = emit(prefix ? `${prefix}\n\n${lowered.text}` : lowered.text);
  if (whole.error) return { error: whole.error, offset };
  const base = prefix ? emit(prefix) : { emitted: { definitions: [], preludes: [] } };
  if (base.error) return { error: base.error, offset };
  const known = new Set(base.emitted.definitions);
  const own = whole.emitted.definitions.filter((definition) => !known.has(definition));
  if (!own.length) return { error: { message: 'no definition' }, offset };
  for (const stub of used) {
    const emitted = base.emitted.definitions.find((definition) => definition.startsWith(`pub fn ${rustName(stub.name)}(`));
    if (!emitted || emitted.split('\n')[0] !== stub.rust[0]) {
      throw new Error(`stub ${stub.name}: meta-language declares ${emitted?.split('\n')[0] ?? 'nothing'}, the record ${stub.rust[0]}`);
    }
  }
  const code = own.join('\n\n');
  return {
    code,
    count: own.length,
    lowered: lowered.text,
    rules: [...lowered.rules].sort(),
    preludes: whole.emitted.preludes,
    stubs: used.map((stub) => stub.name),
  };
}

/**
 * The projection block of a lowered item: its marker (the rules applied), its
 * source as `// |` lines, the lowered source as `// ~` lines and its Rust.
 * @param {string} term
 * @param {string} source
 * @param {{ code: string, lowered: string, rules: string[] }} result
 */
export function loweredBlock(term, source, result) {
  const lines = (text, mark) => text.split(/\r?\n/u).map((line) => (line === '' ? mark : `${mark} ${line}`));
  return [
    `// formal-ai:workaround ${result.rules.join('+')} JavaScript ${term} items=${result.count} sha256=${sha256(result.code)}`,
    ...lines(source, '// |'),
    ...lines(result.lowered, LOWERED_LINE),
    result.code,
  ].join('\n');
}

/**
 * The Rust a module's lowered items need besides their own: each stub's
 * recorded Rust (and the helpers it needs), once.
 * @param {string[]} names the stubs used
 * @param {ReturnType<typeof readStubs>} stubs
 * @returns {string[]}
 */
export function stubPrelude(names, stubs) {
  const out = [];
  const add = (name) => {
    const stub = stubs.find((entry) => entry.name === name);
    if (!stub || out.includes(stub)) return;
    for (const need of stub.needs) add(need);
    out.push(stub);
  };
  for (const name of [...names].sort()) add(name);
  return out.map((stub) => stub.rust.join('\n'));
}

/**
 * The carried items of one module that the lowering rules translate, by
 * block index: each item is tried once its module's translated items are
 * known, and an item it translates joins them for the items after it, until
 * a pass translates nothing more.
 * @param {object} upstream see {@link translateLowered}
 * @param {{ blocks: object[], details: (string|null)[], externals: object[], stubs: ReturnType<typeof readStubs>, accept?: (source: string) => boolean }} module
 *   blocks: upstreamBlocks(code).blocks of the module; accept: whether an item may be translated (every parameter typed)
 * @returns {Map<number, { text: string, rules: string[], preludes: string[], stubs: string[] }>}
 */
export function lowerModule(upstream, { blocks, details, externals, stubs, accept = () => true }) {
  const siblings = new Map();
  for (const block of blocks) {
    const name = block.kind === 'translated' ? declaredName(block.source) : null;
    if (name) siblings.set(name, block.source);
  }
  const lowered = new Map();
  for (let pass = 0; pass < 4; pass += 1) {
    let progress = false;
    blocks.forEach((block, index) => {
      if (block.kind !== 'carried' || lowered.has(index) || block.term === 'import_statement' || !accept(block.source)) return;
      const result = translateLowered(upstream, { source: block.source, siblings, externals, stubs, hint: details[index] ?? '' });
      if (!result) return;
      lowered.set(index, { text: loweredBlock(block.term, block.source, result), rules: result.rules, preludes: result.preludes, stubs: result.stubs });
      const name = declaredName(block.source);
      if (name) siblings.set(name, result.lowered);
      progress = true;
    });
    if (!progress) break;
  }
  return lowered;
}
