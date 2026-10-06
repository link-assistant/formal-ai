// The js-first twin of the native js -> ts leg (plan 16 L2, 2026-10-06
// doctrine): scripts/translate-es.mjs must render what es_tokenizer.rs +
// es_meta.rs write_source render. The cases mirror the decisions the Rust
// tokenizer documents; the rust tier of layered-ci.yml diffs the two
// translators over the whole committed tree.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { translateJsToTs, tokenize } from '../../../scripts/translate-es.mjs';

test('canonical spacing: one space between leaves, delimiters spaced, empty groups tight', () => {
  assert.equal(translateJsToTs('const a=[];f(x,y);'), 'const a = [] ; f ( x , y ) ;');
});

test('comments and a shebang line are dropped', () => {
  assert.equal(translateJsToTs('#!/usr/bin/env node\n// one\nlet a /* two */ = 1;'), 'let a = 1 ;');
});

test('regex versus division follows the previous significant token', () => {
  assert.equal(translateJsToTs('x = a / b / c;'), 'x = a / b / c ;');
  assert.equal(translateJsToTs('return /a[/]b/gi.test(s);'), 'return /a[/]b/gi . test ( s ) ;');
  assert.equal(translateJsToTs('(a) / 2'), '( a ) / 2');
  assert.equal(translateJsToTs('{} /re/'), '{} /re/');
});

test('templates keep chunks verbatim and render interpolations canonically', () => {
  assert.equal(translateJsToTs('`a ${ b+1 } c`'), '`a ${b + 1} c`');
  assert.equal(translateJsToTs('`x${`y${z}`}`'), '`x${`y${z}`}`');
});

test('numbers, optional chaining before a digit, and private names', () => {
  assert.equal(translateJsToTs('a?.5:1e-3+0x1F'), 'a ? .5 : 1e-3 + 0x1F');
  assert.equal(translateJsToTs('this.#x?.y'), 'this . #x ?. y');
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
