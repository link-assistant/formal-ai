// The full set of constructs that keep each carried item of the js -> rust
// translation out of meta-language's portable core (R1188-U30).
//
// Upstream `translateProgram` stops at the first refusal, so the census of
// data/meta/js-rust-translation.lino counts one construct per carried item.
// An item usually holds several. This module scans the item's JavaScript
// source for every construct the pinned portable core refuses (the list
// follows meta-language's js/src/translation/javascript.js) and finds, by a
// greedy set cover, which constructs lifted together would leave the most
// items with nothing refused. The scan is static and approximate: it reads
// tokens, not types, so an item it clears may still meet a type refusal.
// The table it renders, data/meta/translation-blockers.lino, is the
// measurement the workarounds of data/meta/translation-workarounds.lino are
// chosen by, biggest first.

// ---------------------------------------------------------------- lexer

const PUNCTUATORS = [
  '>>>=', '...', '===', '!==', '**=', '<<=', '>>=', '>>>', '&&=', '||=', '??=',
  '=>', '==', '!=', '<=', '>=', '&&', '||', '??', '?.', '++', '--', '+=', '-=', '*=', '/=', '%=', '&=', '|=', '^=', '**', '<<', '>>',
  '{', '}', '(', ')', '[', ']', ';', ',', '<', '>', '+', '-', '*', '/', '%', '&', '|', '^', '!', '~', '?', ':', '=', '.', '@', '#',
];
// After these, a `/` starts a regular expression rather than a division.
const REGEX_AFTER = new Set(['(', ',', '=', ':', '[', '!', '&', '|', '?', '{', '}', ';', '&&', '||', '??', '=>', '+', '-', '*', '%', '<', '>', '==', '===', '!=', '!==', '<=', '>=', '+=', '-=']);
const REGEX_AFTER_WORDS = new Set(['return', 'typeof', 'case', 'do', 'else', 'in', 'of', 'new', 'delete', 'void', 'throw', 'yield', 'await']);

/**
 * Tokens of a JavaScript source: `{ kind, value }` with kind `word`, `number`,
 * `string`, `template`, `regex`, `punct` or `comment`. A template's
 * substitutions are tokenized too and follow it, between `${` and `}` puncts.
 * @param {string} source
 * @returns {{ kind: string, value: string }[]}
 */
export function tokenize(source) {
  const tokens = [];
  let index = 0;
  const significant = () => {
    for (let at = tokens.length - 1; at >= 0; at -= 1) if (tokens[at].kind !== 'comment') return tokens[at];
    return null;
  };
  while (index < source.length) {
    const char = source[index];
    if (/\s/u.test(char)) {
      index += 1;
    } else if (source.startsWith('//', index)) {
      const end = source.indexOf('\n', index);
      const stop = end < 0 ? source.length : end;
      tokens.push({ kind: 'comment', value: source.slice(index, stop) });
      index = stop;
    } else if (source.startsWith('/*', index)) {
      const end = source.indexOf('*/', index + 2);
      const stop = end < 0 ? source.length : end + 2;
      tokens.push({ kind: 'comment', value: source.slice(index, stop) });
      index = stop;
    } else if (char === '"' || char === "'") {
      let at = index + 1;
      while (at < source.length && source[at] !== char && source[at] !== '\n') at += source[at] === '\\' ? 2 : 1;
      tokens.push({ kind: 'string', value: source.slice(index, at + 1) });
      index = at + 1;
    } else if (char === '`') {
      index = template(source, index, tokens);
    } else if (/[A-Za-z_$\\]/u.test(char) || char.codePointAt(0) > 0x7f) {
      const match = /^[\w$\\\u0080-￿]+/u.exec(source.slice(index));
      tokens.push({ kind: 'word', value: match[0] });
      index += match[0].length;
    } else if (/\d/u.test(char) || (char === '.' && /\d/u.test(source[index + 1] ?? ''))) {
      const match = /^(?:0[xXbBoO][\da-fA-F_]+|[\d_]*\.?[\d_]*(?:[eE][+-]?\d+)?)n?/u.exec(source.slice(index));
      tokens.push({ kind: 'number', value: match[0] });
      index += Math.max(1, match[0].length);
    } else if (char === '/' && regexStarts(significant())) {
      let at = index + 1;
      let inClass = false;
      while (at < source.length && source[at] !== '\n' && (inClass || source[at] !== '/')) {
        if (source[at] === '\\') at += 1;
        else if (source[at] === '[') inClass = true;
        else if (source[at] === ']') inClass = false;
        at += 1;
      }
      at += 1;
      while (/[a-z]/u.test(source[at] ?? '')) at += 1;
      tokens.push({ kind: 'regex', value: source.slice(index, at) });
      index = at;
    } else {
      const punct = PUNCTUATORS.find((candidate) => source.startsWith(candidate, index)) ?? char;
      tokens.push({ kind: 'punct', value: punct });
      index += punct.length;
    }
  }
  return tokens;
}

function regexStarts(previous) {
  if (!previous) return true;
  if (previous.kind === 'punct') return REGEX_AFTER.has(previous.value);
  return previous.kind === 'word' && REGEX_AFTER_WORDS.has(previous.value);
}

// A template literal: its text is one token; each substitution's tokens follow it.
function template(source, start, tokens) {
  let at = start + 1;
  const substitutions = [];
  while (at < source.length && source[at] !== '`') {
    if (source[at] === '\\') {
      at += 2;
    } else if (source.startsWith('${', at)) {
      let depth = 1;
      let end = at + 2;
      while (end < source.length && depth > 0) {
        if (source[end] === '{') depth += 1;
        else if (source[end] === '}') depth -= 1;
        else if (source[end] === '`') {
          end = skipTemplate(source, end);
          continue;
        } else if (source[end] === '"' || source[end] === "'") {
          const quote = source[end];
          end += 1;
          while (end < source.length && source[end] !== quote) end += source[end] === '\\' ? 2 : 1;
        }
        end += 1;
      }
      substitutions.push(source.slice(at + 2, end - 1));
      at = end;
    } else {
      at += 1;
    }
  }
  tokens.push({ kind: 'template', value: source.slice(start, at + 1) });
  for (const inner of substitutions) tokens.push({ kind: 'punct', value: '${' }, ...tokenize(inner), { kind: 'punct', value: '}' });
  return at + 1;
}

function skipTemplate(source, start) {
  let at = start + 1;
  while (at < source.length && source[at] !== '`') at += source[at] === '\\' ? 2 : 1;
  return at + 1;
}

// ---------------------------------------------------------------- module context

const KEYWORDS = new Set([
  'break', 'case', 'catch', 'class', 'const', 'continue', 'debugger', 'default', 'delete', 'do', 'else', 'export', 'extends',
  'false', 'finally', 'for', 'function', 'if', 'import', 'in', 'instanceof', 'new', 'null', 'return', 'super', 'switch', 'this',
  'throw', 'true', 'try', 'typeof', 'var', 'void', 'while', 'with', 'yield', 'let', 'static', 'async', 'await', 'of', 'undefined',
  'from', 'as', 'get', 'set',
]);
// The portable core's own names: Math and Number reads, the string tests and maps.
const PORTABLE_GLOBALS = new Set(['Math', 'Number', 'isFinite', 'isNaN', 'Infinity', 'NaN', 'Error', 'RangeError', 'TypeError', 'console', 'BigInt']);
const STRING_METHODS = new Set(['startsWith', 'endsWith', 'includes', 'toLowerCase', 'toUpperCase', 'trim', 'trimStart', 'trimEnd', 'toString']);
const PORTABLE_MATH = /^(?:Math\.(?:abs|floor|ceil|trunc|round|sign|sqrt|max|min|PI|E|LN2|LN10|LOG2E|LOG10E|SQRT2|SQRT1_2)|Number\.(?:isInteger|isSafeInteger|isFinite|isNaN|MAX_SAFE_INTEGER|MIN_SAFE_INTEGER|EPSILON|MAX_VALUE|MIN_VALUE|POSITIVE_INFINITY|NEGATIVE_INFINITY|NaN))$/u;
const PORTABLE_TYPE = /^(?:number|bigint|boolean|string)$/u;

/**
 * The names a module declares at its top level (`siblings`), the names it
 * imports (`imports`) and its portable data types (`typedefs`: @typedef
 * unions whose alternatives carry a `$` tag).
 * @param {string} source
 */
export function moduleContext(source) {
  const tokens = tokenize(source).filter((token) => token.kind !== 'comment');
  const siblings = new Set();
  const imports = new Set();
  let depth = 0;
  for (let at = 0; at < tokens.length; at += 1) {
    const token = tokens[at];
    if (token.kind === 'punct' && ['{', '(', '[', '${'].includes(token.value)) depth += 1;
    else if (token.kind === 'punct' && ['}', ')', ']'].includes(token.value)) depth -= 1;
    else if (token.kind === 'word' && token.value === 'import' && depth === 0) {
      let end = at + 1;
      while (end < tokens.length && !(tokens[end].kind === 'word' && tokens[end].value === 'from') && tokens[end].kind !== 'string') {
        const word = tokens[end];
        if (word.kind === 'word' && !KEYWORDS.has(word.value) && !(tokens[end + 1]?.kind === 'word' && tokens[end + 1].value === 'as')) imports.add(word.value);
        end += 1;
      }
      at = end;
    } else if (depth === 0 && token.kind === 'word' && ['function', 'const', 'let', 'class'].includes(token.value)) {
      let next = at + 1;
      if (tokens[next]?.value === '*') next += 1;
      if (tokens[next]?.kind === 'word') siblings.add(tokens[next].value);
    }
  }
  const typedefs = new Set([...source.matchAll(/@typedef\s+\{([^@]*?)\}\s+([A-Za-z_$][\w$]*)/gsu)]
    .filter((match) => /\$\s*:\s*'/u.test(match[1]))
    .map((match) => match[2]));
  return { siblings, imports, typedefs };
}

// ---------------------------------------------------------------- one item

/** Whether a JSDoc type is portable: a primitive, a portable data type, or an array of one. */
export function portableType(type, typedefs = new Set()) {
  const text = type.trim();
  if (text.endsWith('[]')) return portableType(text.slice(0, -2), typedefs);
  if (/^Array<(.+)>$/u.test(text)) return portableType(text.slice(6, -1), typedefs);
  if (/^\(.+\)$/u.test(text)) return portableType(text.slice(1, -1), typedefs);
  return PORTABLE_TYPE.test(text) || typedefs.has(text);
}

function jsdocBlockers(comment, typedefs, found) {
  for (const match of comment.matchAll(/@(param|returns?|type|typedef)\s*\{((?:[^{}]|\{[^{}]*\})*)\}/gu)) {
    if (match[1] === 'typedef') {
      if (!/\$\s*:\s*'/u.test(match[2])) found.add('@typedef without a $ tag');
    } else if (!portableType(match[2], typedefs)) {
      found.add('JSDoc type {…}');
    }
  }
}

/**
 * Every construct of one top-level item the portable core refuses.
 * @param {string} source the item, with its doc comments
 * @param {{ siblings: Set<string>, imports: Set<string>, typedefs: Set<string> }} context
 * @returns {string[]} sorted construct classes
 */
export function itemBlockers(source, context = { siblings: new Set(), imports: new Set(), typedefs: new Set() }) {
  const found = new Set();
  const all = tokenize(source);
  for (const token of all) if (token.kind === 'comment' && token.value.startsWith('/**')) jsdocBlockers(token.value, context.typedefs, found);
  const tokens = all.filter((token) => token.kind !== 'comment');
  const first = tokens[0]?.value;
  if (first === 'import') {
    // Upstream translates a named import of a module in its directory; it is
    // carried only while that module carries the names it imports.
    const specifier = tokens.find((token) => token.kind === 'string')?.value.slice(1, -1) ?? '';
    if (!tokens.some((token) => token.value === '{')) found.add('default or namespace import');
    else if (/^\.\/[^/]+$/u.test(specifier)) found.add('import of names its module carries');
    else found.add('import from outside the module directory');
    return [...found].sort();
  }
  if (first === 'export' && !['function', 'const', 'async'].includes(tokens[1]?.value)) found.add('export …');
  const local = declaredNames(tokens);
  // The stack of open brackets: for each, what it opens.
  const stack = [];
  const isWord = (at, value) => tokens[at]?.kind === 'word' && (value === undefined || tokens[at].value === value);
  const isPunct = (at, value) => tokens[at]?.kind === 'punct' && tokens[at].value === value;
  for (let at = 0; at < tokens.length; at += 1) {
    const token = tokens[at];
    const previous = tokens[at - 1];
    if (token.kind === 'regex') found.add('regular expression');
    if (token.kind === 'punct') {
      switch (token.value) {
        case '(': {
          const callee = previous?.kind === 'word' && tokens[at - 2]?.value === '.' ? `.${previous.value}()` : null;
          stack.push({ open: '(', callee });
          break;
        }
        case '{':
          stack.push({ open: '{', object: objectStarts(previous), tagged: false, line: at });
          if (objectStarts(previous) && tokens.slice(at + 1, at + 3).map((entry) => entry.value).join(' ') === '$ :') stack.at(-1).tagged = true;
          break;
        case '[':
        case '${':
          stack.push({ open: token.value });
          break;
        case ')':
        case ']':
        case '}': {
          const closed = stack.pop();
          if (closed?.object && !closed.tagged && token.value === '}') found.add('object without a $ tag');
          break;
        }
        case '=>': {
          const callback = [...stack].reverse().find((entry) => entry.open === '(' || entry.open === '{');
          found.add(callback?.callee ? `arrow callback of ${callback.callee}` : 'arrow function');
          break;
        }
        case '?.': found.add('optional chaining'); break;
        case '??': case '??=': found.add('nullish coalescing'); break;
        case '...':
          if (stack.at(-1)?.open === '{') found.add('object spread');
          else if (stack.at(-1)?.open === '(' && !stack.at(-1).callee && !isDeclarationParams(tokens, at)) found.add('spread argument');
          break;
        case '++': case '--': found.add(`${token.value} operator`); break;
        case '==': case '!=': found.add('loose equality'); break;
        case '**': case '**=': found.add('exponentiation'); break;
        case '&': case '|': case '^': case '~': case '<<': case '>>': case '>>>': case '&=': case '|=': case '^=':
          found.add('bitwise operator');
          break;
        case '=':
          if (previous?.value === ']' || (tokens[at - 2]?.value === '.' && previous?.kind === 'word')) found.add('assignment of a field or element');
          break;
        default:
          break;
      }
      continue;
    }
    if (token.kind !== 'word') continue;
    const word = token.value;
    const afterDot = previous?.value === '.' || previous?.value === '?.';
    if (afterDot) {
      if (isPunct(at + 1, '(')) {
        const owner = tokens[at - 2];
        const path = owner?.kind === 'word' && tokens[at - 3]?.value !== '.' ? `${owner.value}.${word}` : null;
        if (path && /^[A-Z]/u.test(owner.value) && !context.siblings.has(owner.value) && !local.has(owner.value)) {
          if (!PORTABLE_MATH.test(path)) found.add(path);
        } else if (!STRING_METHODS.has(word)) {
          found.add(`method call .${word}()`);
        }
      } else if (tokens[at - 2]?.kind === 'word' && /^(?:Object|Array|JSON|String|Symbol|Reflect|Date|Promise)$/u.test(tokens[at - 2].value)) {
        found.add(`${tokens[at - 2].value}.${word}`);
      } else if (word !== 'length' && context.typedefs.size === 0 && !/^[A-Z]/u.test(tokens[at - 2]?.value ?? '')) {
        // A field of a value that is not a portable data type: the module declares none.
        found.add('field access');
      }
      continue;
    }
    if (isPunct(at + 1, ':') && stack.at(-1)?.object) continue;
    switch (word) {
      case 'null': found.add('null'); break;
      case 'undefined': found.add('undefined'); break;
      case 'typeof': found.add('typeof operator'); break;
      case 'try': found.add('try statement'); break;
      case 'class': found.add('class'); break;
      case 'async': case 'await': found.add('async function'); break;
      case 'var': found.add('var declaration'); break;
      case 'in': if (!isWord(at - 2, 'const') && !isWord(at - 2, 'let')) found.add('in operator'); break;
      case 'instanceof': found.add('instanceof operator'); break;
      case 'new':
        if (!(isWord(at - 1, 'throw') && /^(?:Error|RangeError|TypeError)$/u.test(tokens[at + 1]?.value ?? '') && tokens[at + 3]?.kind === 'string')) {
          found.add(`new ${/^[A-Z]/u.test(tokens[at + 1]?.value ?? '') ? tokens[at + 1].value : 'expression'}`);
        }
        break;
      case 'throw':
        if (!isWord(at + 1, 'new')) found.add('throw of a non-error value');
        break;
      case 'function':
        if (at > 0 && !(at === 1 && first === 'export') && !(at === 1 && first === 'async') && !(at === 2 && tokens[1]?.value === 'async')) found.add('function value …');
        break;
      case 'const': case 'let':
        if (isPunct(at + 1, '{') || isPunct(at + 1, '[')) found.add('destructuring');
        else if (word === 'let' && isWord(at + 1) && (isPunct(at + 2, ';') || isPunct(at + 2, ','))) found.add('let without a value');
        break;
      default:
        if (KEYWORDS.has(word) || local.has(word) || PORTABLE_GLOBALS.has(word)) break;
        if (previous?.value === 'new') break;
        if (/^(?:String|Boolean|Array|Object|JSON|Symbol|Map|Set|WeakMap|WeakSet|Date|Promise|Reflect|parseInt|parseFloat|structuredClone|globalThis|process|encodeURIComponent|decodeURIComponent|TextEncoder|TextDecoder|Buffer|setTimeout|queueMicrotask|Intl|URL|crypto)$/u.test(word)) {
          if (isPunct(at + 1, '(')) found.add(`global call ${word}()`);
          else if (!isPunct(at + 1, '.')) found.add(`global value ${word}`);
        } else if (context.imports.has(word)) {
          found.add(isPunct(at + 1, '(') ? 'call of an imported function' : 'imported value');
        } else if (context.siblings.has(word)) {
          found.add(isPunct(at + 1, '(') ? 'call of a sibling function' : 'sibling value');
        }
        break;
    }
  }
  return [...found].sort();
}

// Whether a `{` after `previous` opens an object literal rather than a block.
function objectStarts(previous) {
  if (!previous) return false;
  if (previous.kind === 'word') return ['return', 'yield', 'case', 'typeof', 'in', 'of', 'throw'].includes(previous.value) && previous.value !== 'case';
  return previous.kind === 'punct' && ['(', ',', '=', ':', '[', '?', '=>', '||', '&&', '??', '${'].includes(previous.value);
}

// Whether the `...` at `at` is a rest parameter of a function's parameter list.
function isDeclarationParams(tokens, at) {
  let depth = 0;
  for (let index = at; index < tokens.length; index += 1) {
    if (tokens[index].value === '(') depth += 1;
    if (tokens[index].value === ')') {
      if (depth === 0) return tokens[index + 1]?.value === '=>' || tokens[index + 1]?.value === '{';
      depth -= 1;
    }
  }
  return false;
}

// Every name the item declares: functions, parameters, bindings, loop variables.
function declaredNames(tokens) {
  const names = new Set();
  for (let at = 0; at < tokens.length; at += 1) {
    const token = tokens[at];
    if (token.kind !== 'word') continue;
    if (['function', 'const', 'let', 'var', 'class'].includes(token.value)) {
      let next = at + 1;
      if (tokens[next]?.value === '*') next += 1;
      if (tokens[next]?.kind === 'word') names.add(tokens[next].value);
      if (['{', '['].includes(tokens[next]?.value)) {
        for (let inner = next + 1; inner < tokens.length && !['=', 'of', 'in'].includes(tokens[inner].value); inner += 1) {
          if (tokens[inner].kind === 'word') names.add(tokens[inner].value);
        }
      }
    }
    // Parameters: the words of a list followed by `=>` or by a function's `{`.
    if (tokens[at + 1]?.value === '=>') names.add(token.value);
  }
  for (let at = 0; at < tokens.length; at += 1) {
    if (tokens[at].value !== '(') continue;
    let depth = 0;
    let end = at;
    for (; end < tokens.length; end += 1) {
      if (tokens[end].value === '(') depth += 1;
      if (tokens[end].value === ')') {
        depth -= 1;
        if (depth === 0) break;
      }
    }
    const declares = tokens[end + 1]?.value === '=>' || (tokens[end + 1]?.value === '{' && (tokens[at - 1]?.kind === 'word' || tokens[at - 2]?.value === 'function'));
    if (!declares) continue;
    for (let inner = at + 1; inner < end; inner += 1) {
      const word = tokens[inner];
      if (word.kind === 'word' && ![',', '(', '{', '[', '...'].includes(tokens[inner - 1]?.value) && tokens[inner - 1]?.value !== '=') continue;
      if (word.kind === 'word') names.add(word.value);
    }
  }
  return names;
}

// ---------------------------------------------------------------- the table

/**
 * The greedy set cover: at each step the construct that, lifted with the
 * ones before it, leaves the most items with nothing refused.
 * @param {string[][]} items the blocker set of each carried item
 * @param {number} steps
 * @returns {{ construct: string, unlocked: number, cumulative: number }[]}
 */
export function unlockOrder(items, steps = 25) {
  const lifted = new Set();
  const order = [];
  const freed = () => items.filter((set) => set.length > 0 && set.every((construct) => lifted.has(construct))).length;
  let before = freed();
  const constructs = [...new Set(items.flat())].sort();
  for (let step = 0; step < steps; step += 1) {
    let best = null;
    for (const construct of constructs) {
      if (lifted.has(construct)) continue;
      lifted.add(construct);
      const count = freed();
      lifted.delete(construct);
      // Ties go to the construct that blocks more items, so the next steps gain most.
      const reach = items.filter((set) => set.includes(construct)).length;
      if (!best || count > best.count || (count === best.count && reach > best.reach)) best = { construct, count, reach };
    }
    if (!best) break;
    lifted.add(best.construct);
    order.push({ construct: best.construct, unlocked: best.count - before, cumulative: best.count });
    before = best.count;
  }
  return order;
}

const quote = (text) => `"${text.replace(/"/gu, "'")}"`;

/**
 * Renders data/meta/translation-blockers.lino from the blocker sets of the
 * carried items (the `// formal-ai:blockers` lines of the projections).
 * @param {{ blockers: string[] }[]} items
 * @returns {string}
 */
export function renderBlockers(items) {
  const sets = items.map((item) => item.blockers);
  const counts = new Map();
  for (const set of sets) for (const construct of set) counts.set(construct, (counts.get(construct) ?? 0) + 1);
  const single = new Map();
  for (const set of sets) if (set.length === 1) single.set(set[0], (single.get(set[0]) ?? 0) + 1);
  const sizes = new Map();
  for (const set of sets) sizes.set(set.length, (sizes.get(set.length) ?? 0) + 1);
  const lines = [
    '# The full set of constructs that keep each carried item of the js -> rust',
    '# translation out of meta-language\'s portable core (R1188-U30).',
    '#',
    '# Generated by `node scripts/translate-js-rust.mjs --write`; never edited by',
    '# hand. The census of data/meta/js-rust-translation.lino counts only the',
    '# first refusal of an item, where upstream stops; here every construct of',
    '# the item is counted (scripts/lib/translation-blockers.mjs reads its',
    '# tokens, so the scan is static and approximate). `items` is how many carried',
    '# items hold a construct, `only` how many hold it and nothing else (an item',
    '# with no line was cleared by the scan, so only a type refusal holds it). The',
    '# `unlock` rows are the greedy order: lifting the constructs down to a row',
    '# together leaves `cumulative` items with nothing refused by this scan.',
    'translation_blockers',
    `  carried_items ${items.length}`,
    `  scanned_clear ${sets.filter((set) => set.length === 0).length}`,
    ...[...sizes].sort((a, b) => a[0] - b[0]).map(([size, count]) => `  blockers_per_item ${size} ${count}`),
    ...[...counts].sort((a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : 1))
      .map(([construct, count]) => `construct ${quote(construct)} items ${count} only ${single.get(construct) ?? 0}`),
    ...unlockOrder(sets).map((row, index) => `unlock ${index + 1} ${quote(row.construct)} unlocked ${row.unlocked} cumulative ${row.cumulative}`),
  ];
  return `${lines.join('\n')}\n`;
}
