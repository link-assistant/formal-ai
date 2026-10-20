// Independent physical consumer oracle, fixed before the native smoke attempt.
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
import { parse } from '@babel/parser';
import { readBoundedFilePath } from '../../../../js/provenance/bounded-file.mjs';

export async function verifyNativeSourceOperation(kind, filename) {
  const source = readBoundedFilePath(filename, 1024 * 1024).toString('utf8');
  const tree = parse(source, { sourceType: 'module' });
  assert.ok(tree.comments.some(comment =>
    comment.value.trim() === 'Native optional operation owner comment.'));
  const loaded = await import(pathToFileURL(filename).href + '?physical-oracle');
  if (kind === 'unary') {
    const declarations = tree.program.body.filter(node => node.type === 'ImportDeclaration');
    const helper = declarations.find(node => node.source.value === './callback_values.mjs');
    assert.ok(helper, 'canonical helper import required');
    const binding = helper.specifiers.find(node => node.imported?.name === 'callbackText');
    assert.ok(binding, 'canonical text helper binding required');
    const declaration = tree.program.body.find(node =>
      node.type === 'ExportNamedDeclaration' && node.declaration?.id?.name === 'collect');
    const callback = declaration.declaration.body.body[0].argument.arguments[0];
    assert.equal(callback.type, 'Identifier', 'all fixture callbacks must be consolidated');
    assert.equal(callback.name, binding.local.name);
    const pending = [tree.program];
    for (let visited = 0; pending.length; visited++) {
      assert.ok(visited < 10000, 'bounded fixture syntax');
      const node = pending.pop();
      assert.notEqual(node.type, 'ArrowFunctionExpression', 'no fixture callback may remain');
      for (const value of Object.values(node)) {
        if (Array.isArray(value)) {
          pending.push(...value.filter(item => item && typeof item.type === 'string'));
        } else if (value && typeof value.type === 'string') pending.push(value);
      }
    }
    const reference = {};
    assert.deepEqual(loaded.collect([{ text: 'heldout' }, { text: '' }, { text: 0 }]),
      ['heldout', '', 0]);
    assert.equal(loaded.collect([{ text: reference }])[0], reference);
    let reads = 0;
    assert.deepEqual(loaded.collect([{ get text() { reads++; return 'once'; } }]), ['once']);
    assert.equal(reads, 1);
    assert.throws(() => loaded.collect([null]), TypeError);
  } else if (kind === 'parameter') {
    const declaration = tree.program.body.find(node =>
      node.type === 'ExportNamedDeclaration' && node.declaration?.id?.name === 'increment');
    assert.equal(declaration.declaration.params[0].name, 'input');
    assert.equal(declaration.declaration.body.body[0].argument.left.name, 'input');
    assert.equal(loaded.increment(4), 5);
    assert.equal(loaded.increment(-2), -1);
    assert.equal(loaded.increment('x'), 'x1');
    assert.ok(Number.isNaN(loaded.increment(NaN)));
  } else {
    throw Error('UnknownNativeSourceOracle');
  }
  return { kind, coverage: 'complete-tiny-fixture-only', behavior: 'passed' };
}
