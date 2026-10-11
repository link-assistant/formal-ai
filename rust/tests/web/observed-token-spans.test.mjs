import assert from 'node:assert/strict';
import test from 'node:test';
import { tokenize, tokenizeWithSpans } from '../../../js/agentic/crate/es_tokenizer.mjs';
const strip = (value) => Array.isArray(value) ? value.map(strip)
  : value && typeof value === 'object' ? Object.fromEntries(Object.entries(value)
    .filter(([key]) => key !== 'span').map(([key, child]) => [key, strip(child)])) : value;
const walk = function* (trees) {
  for (const tree of trees) {
    yield tree;
    if (tree.trees) yield* walk(tree.trees);
    if (tree.parts) yield* walk(tree.parts);
  }
};
for (const [name, source] of [
  ['Unicode comments and identifiers', '// 文\nexport function 描述(選択) { return 選択; }'],
  ['BOM preimage', '\uFEFFconst value = "文";'],
  ['nested template and comments', 'export function show(x) { return `文${x.kind /* 文 */}:${`内${x.id}`}`; }'],
  ['regex and escaped literal', 'const rx = /[文/]+/u; const value = "x\\n文";'],
]) {
  test(name + ': lossless byte spans preserve the existing public tree', () => {
    const trees = tokenizeWithSpans(source), bytes = Buffer.from(source);
    if (source.startsWith('\uFEFF')) {
      assert.equal(tokenize(source)[0].text, 'const');
      assert.equal(trees[0].text, '\uFEFFconst');
    } else assert.deepEqual(strip(trees), tokenize(source));
    for (const tree of walk(trees)) {
      assert.ok(tree.span && tree.span.start >= 0 && tree.span.end <= bytes.length);
      const slice = bytes.subarray(tree.span.start, tree.span.end).toString();
      if (tree.text !== undefined) assert.equal(slice, tree.text);
      if (tree.$ === 'group') assert.equal(slice[0] + slice.at(-1), { paren: '()', bracket: '[]', brace: '{}' }[tree.delim]);
      if (tree.$ === 'template') assert.equal(slice[0] + slice.at(-1), '``');
      if (tree.$ === 'interp') assert.ok(slice.startsWith('$' + '{') && slice.endsWith('}'));
    }
  });
}
test('malformed observed source retains the same exact tokenizer failure', () => {
  for (const source of ['function f() {', 'const x = "unterminated', '/* open']) {
    let ordinary, observed;
    try { tokenize(source); } catch (error) { ordinary = [error.kind, error.start]; }
    try { tokenizeWithSpans(source); } catch (error) { observed = [error.kind, error.start]; }
    assert.ok(ordinary);
    assert.deepEqual(observed, ordinary);
  }
});
