// The js-first twin of the native js -> ts leg (plan 16 L2, 2026-10-06
// doctrine): scripts/translate-es.mjs must render what es_tokenizer.rs +
// es_meta.rs write_source render. The cases mirror the decisions the Rust
// tokenizer documents; the rust tier of layered-ci.yml diffs the two
// translators over the whole committed tree.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { layoutOf, renderSource, translateJsToTs, tokenize, tsTwinPath } from '../../../scripts/translate-es.mjs';

/** The canonical rendering: the tree alone, no layout carried. */
function canonical(source) {
  return renderSource(tokenize(source));
}

test('canonical spacing: one space between leaves, delimiters spaced, empty groups tight', () => {
  assert.equal(canonical('const a=[];f(x,y);'), 'const a = [] ; f ( x , y ) ;');
});

test('the canonical rendering drops comments and a shebang line', () => {
  assert.equal(canonical('#!/usr/bin/env node\n// one\nlet a /* two */ = 1;'), 'let a = 1 ;');
});

test('regex versus division follows the previous significant token', () => {
  assert.equal(canonical('x = a / b / c;'), 'x = a / b / c ;');
  assert.equal(canonical('return /a[/]b/gi.test(s);'), 'return /a[/]b/gi . test ( s ) ;');
  assert.equal(canonical('(a) / 2'), '( a ) / 2');
  assert.equal(canonical('{} /re/'), '{} /re/');
});

test('templates keep chunks verbatim and render interpolations canonically', () => {
  assert.equal(canonical('`a ${ b+1 } c`'), '`a ${b + 1} c`');
  assert.equal(canonical('`x${`y${z}`}`'), '`x${`y${z}`}`');
});

test('numbers, optional chaining before a digit, and private names', () => {
  assert.equal(canonical('a?.5:1e-3+0x1F'), 'a ? .5 : 1e-3 + 0x1F');
  assert.equal(canonical('this.#x?.y'), 'this . #x ?. y');
});

// Issue #1188 R1188-U23: the pivot carries the layout (the text before every
// token and after the last one), so a ts twin keeps its source's lines,
// indentation and comments instead of collapsing to one line.
test('the translation keeps the source layout: lines, indentation, comments, shebang', () => {
  const source = [
    '#!/usr/bin/env node',
    '// A comment line.',
    'export function sum(a, b) {',
    '  /* block */ return a + b; // trailing',
    '}',
    'const tpl = `x ${ a /* inside */ } y ${`nested ${ b }`}`;',
    'const re = /a\\/b/g;',
    '',
  ].join('\n');
  assert.equal(translateJsToTs(source), source);
  assert.ok(translateJsToTs(source).split('\n').length > 5, 'the twin is multi-line');
});

test('the layout has one slot per token, delimiter and interpolation close, plus the tail', () => {
  const source = 'f( a , `t${ b }` ) // end\n';
  const trees = tokenize(source);
  const layout = layoutOf(source, trees);
  // f, (, a, ",", template, b, interpolation }, ), tail
  assert.deepEqual(layout, ['', '', ' ', ' ', ' ', ' ', ' ', ' ', ' // end\n']);
  assert.equal(renderSource(trees, layout), source);
});

test('every committed js source renders back to itself through the pivot', () => {
  const root = new URL('../../../', import.meta.url);
  for (const rel of ['agentic/repair_loop.mjs', 'agentic/crate/es_tokenizer.mjs', 'worker/formal_ai_worker_solve.js']) {
    const source = readFileSync(new URL(`js/${rel}`, root), 'utf8');
    assert.equal(translateJsToTs(source), source, rel);
  }
});

test('non-ascii identifiers stay whole tokens', () => {
  assert.deepEqual(
    tokenize('é1 = "ж"').map((tree) => tree.text),
    ['é1', '=', '"ж"'],
  );
});

test('unbalanced input is refused, never guessed', () => {
  assert.throws(() => tokenize('f(a'), /unclosed group/);
  assert.throws(() => tokenize('a)'), /unexpected closing delimiter/);
  assert.throws(() => tokenize('`a${b'), /unterminated template literal/);
  assert.throws(() => tokenize('"abc'), /unterminated string literal/);
});

// Issues #1185 R8 and #1180 R11: the agentic modules gain a TS root. An
// `.mjs` module maps to `.mts` (its relative imports keep resolving under
// NodeNext), and the committed twin is exactly the translator's rendering.
test('the agentic ES modules mirror into ts/agentic as .mts twins', () => {
  assert.equal(tsTwinPath('agentic/repair_loop.mjs'), 'agentic/repair_loop.mts');
  assert.equal(tsTwinPath('worker/formal_ai_worker_solve.js'), 'worker/formal_ai_worker_solve.ts');
  const root = new URL('../../../', import.meta.url);
  for (const rel of ['agentic/repair_loop.mjs', 'agentic/command_reroute.mjs', 'agentic/crate/history_context.mjs']) {
    const source = readFileSync(new URL(`js/${rel}`, root), 'utf8');
    const twin = readFileSync(new URL(`ts/${tsTwinPath(rel)}`, root), 'utf8');
    assert.equal(twin, translateJsToTs(source));
  }
});
