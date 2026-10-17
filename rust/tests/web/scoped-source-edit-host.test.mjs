import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {createScopedSourceEditHost} from '../../../experiments/js_dogfood/scoped-source-edit-host.mjs';
const digest = value => createHash('sha256').update(value).digest('hex');
const call = (tool, args) => ({tool, arguments: JSON.stringify(args)});
function fixture(context, bytes = 'const value = 1; \r\n') {
  const workspace = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'formal-scoped-edit-')));
  context.after(() => fs.rmSync(workspace, {recursive: true, force: true}));
  const ownedPath = 'src/value.mjs';
  fs.mkdirSync(path.join(workspace, 'src'));
  fs.writeFileSync(path.join(workspace, ownedPath), bytes);
  const host = options => createScopedSourceEditHost({workspace, ownedPath, ...options});
  return {workspace, ownedPath, target: path.join(workspace, ownedPath), host};
}
function refused(receipt, code) {
  assert.equal(receipt.is_error, true, JSON.stringify(receipt));
  if (code) assert.equal(JSON.parse(receipt.content).error, code);
}
test('complete read, exact preimage edit and actual reread preserve all other bytes', context => {
  const original = '\uFEFFconst π = "🙂"; \r\n// kept\n';
  const f = fixture(context, original), host = f.host();
  const read = host.execute(call('read', {filePath: f.ownedPath}));
  assert.equal(read.content, original);
  assert.deepEqual(read.source_read, {path: f.ownedPath, success: true, complete: true, format: 'raw'});
  const expected = original.replace('; \r\n', ';\r\n');
  const edit = host.execute(call('edit', {filePath: f.ownedPath, oldString: original, newString: expected}));
  assert.equal(edit.content, '');
  assert.deepEqual(edit.source_edit, {path: f.ownedPath, success: true,
    before_sha256: digest(original), after_sha256: digest(expected), verified: true});
  assert.equal(host.execute(call('read', {filePath: f.ownedPath})).content, expected);
  assert.deepEqual(fs.readFileSync(f.target), Buffer.from(expected));
});
test('standard path and string aliases retain the maintained tool contract', context => {
  const f = fixture(context, 'before\n'), host = f.host();
  assert.equal(host.execute(call('read', {file_path: f.ownedPath})).content, 'before\n');
  assert.equal(host.execute(call('edit', {path: f.ownedPath, old_string: 'before\n', new_string: 'after\n'})).content, '');
  assert.equal(host.execute(call('read', {path: f.ownedPath})).content, 'after\n');
});
test('edit requires prior actual read and cannot create an empty-preimage file', context => {
  const f = fixture(context);
  refused(f.host().execute(call('edit', {filePath: f.ownedPath, oldString: '', newString: 'overwrite'})), 'SourceReadRequired');
  assert.equal(fs.readFileSync(f.target, 'utf8'), 'const value = 1; \r\n');
  const empty = fixture(context, ''), host = empty.host();
  host.execute(call('read', {filePath: empty.ownedPath}));
  refused(host.execute(call('edit', {filePath: empty.ownedPath, oldString: '', newString: 'new'})), 'WholeObservedPreimageRequired');
});
test('only whole observed preimage owns an edit, not a matching substring', context => {
  const f = fixture(context, 'one one\n'), host = f.host();
  host.execute(call('read', {filePath: f.ownedPath}));
  refused(host.execute(call('edit', {filePath: f.ownedPath, oldString: 'one', newString: 'two'})), 'WholeObservedPreimageRequired');
  assert.equal(fs.readFileSync(f.target, 'utf8'), 'one one\n');
});
for (const tool of ['bash', 'write', 'list', 'grep', 'unknown']) {
  test('undeclared tool cannot gain filesystem authority: ' + tool, context => {
    const f = fixture(context);
    refused(f.host().execute(call(tool, {filePath: f.ownedPath, content: 'changed'})), 'UndeclaredSourceTool');
    assert.equal(fs.readFileSync(f.target, 'utf8'), 'const value = 1; \r\n');
  });
}
test('unknown, duplicate alias and replacement flags are refused without mutation', context => {
  const f = fixture(context, 'before');
  for (const args of [{filePath: f.ownedPath, other: true}, {filePath: f.ownedPath, path: f.ownedPath}]) {
    refused(f.host().execute(call('read', args)));
  }
  const host = f.host(); host.execute(call('read', {filePath: f.ownedPath}));
  refused(host.execute(call('edit', {filePath: f.ownedPath, oldString: 'before', newString: 'after', replaceAll: true})), 'UnknownOrAccessorField');
  assert.equal(fs.readFileSync(f.target, 'utf8'), 'before');
});
test('malformed/non-record tool arguments and accessor/proxy calls are refused', context => {
  const f = fixture(context);
  for (const argument of ['{', 'null', '[]', '1', '"x"']) {
    refused(f.host().execute({tool: 'read', arguments: argument}));
  }
  let touched = false;
  const accessor = {get tool() { touched = true; return 'read'; }, arguments: '{}'};
  refused(f.host().execute(accessor)); assert.equal(touched, false);
  refused(f.host().execute(new Proxy({}, {get() { throw new Error('must not inspect proxy'); }})));
});
test('single explicit owned path refuses alternate, absolute and traversal paths', context => {
  const f = fixture(context);
  for (const target of ['other.mjs', '../escape', f.target, 'src/../value.mjs', 'src\\value.mjs']) {
    refused(f.host().execute(call('read', {filePath: target})), 'UnownedSourcePath');
  }
  for (const ownedPath of ['../escape', '/absolute', 'src//value', 'src/./value', 'src\\value', 'x\0y']) {
    assert.throws(() => createScopedSourceEditHost({workspace: f.workspace, ownedPath}), /InvalidOwnedPath/);
  }
});
test('missing file and directory never become source-absence creation authority', context => {
  const f = fixture(context); fs.unlinkSync(f.target);
  refused(f.host().execute(call('read', {filePath: f.ownedPath})), 'ENOENT');
  fs.mkdirSync(f.target);
  refused(f.host().execute(call('read', {filePath: f.ownedPath})), 'SourceNotPrivateRegularFile');
});
test('symbolic target and symbolic ancestor refuse before reading their destinations', context => {
  const f = fixture(context), outside = path.join(f.workspace, 'outside');
  fs.writeFileSync(outside, 'private'); fs.unlinkSync(f.target); fs.symlinkSync(outside, f.target);
  refused(f.host().execute(call('read', {filePath: f.ownedPath})), 'SymbolicSourcePath');
  fs.unlinkSync(f.target); fs.rmdirSync(path.join(f.workspace, 'src'));
  fs.mkdirSync(path.join(f.workspace, 'other')); fs.writeFileSync(path.join(f.workspace, 'other/value.mjs'), 'private');
  fs.symlinkSync(path.join(f.workspace, 'other'), path.join(f.workspace, 'src'));
  refused(f.host().execute(call('read', {filePath: f.ownedPath})), 'SymbolicSourcePath');
});
test('hardlink aliases are refused so one owned path cannot modify another name', context => {
  const f = fixture(context); fs.linkSync(f.target, path.join(f.workspace, 'alias'));
  refused(f.host().execute(call('read', {filePath: f.ownedPath})), 'SourceNotPrivateRegularFile');
});
test('oversized inputs are refused before source read allocation; output budget also holds', context => {
  const f = fixture(context, '12345');
  refused(f.host({maximumBytes: 4}).execute(call('read', {filePath: f.ownedPath})), 'SourceByteBudget');
  const host = f.host({maximumBytes: 5}); host.execute(call('read', {filePath: f.ownedPath}));
  refused(host.execute(call('edit', {filePath: f.ownedPath, oldString: '12345', newString: '123456'})), 'ReplacementByteBudget');
  for (const maximumBytes of [0, -1, 1.5, Infinity, NaN, 1048577]) assert.throws(() => f.host({maximumBytes}), /InvalidByteBudget/);
});
test('invalid UTF8 cannot produce complete source custody', context => {
  const f = fixture(context, Buffer.from([65, 255, 66]));
  const receipt = f.host().execute(call('read', {filePath: f.ownedPath}));
  refused(receipt); assert.equal(receipt.source_read.complete, false);
});
test('same-size byte drift, inode replacement and symlink replacement stop before edit', context => {
  for (const mutation of ['bytes', 'inode', 'symlink']) {
    const f = fixture(context, 'before'), host = f.host();
    host.execute(call('read', {filePath: f.ownedPath}));
    if (mutation === 'bytes') fs.writeFileSync(f.target, 'BEFORE');
    if (mutation === 'inode') { fs.unlinkSync(f.target); fs.writeFileSync(f.target, 'before'); }
    if (mutation === 'symlink') { fs.unlinkSync(f.target); fs.symlinkSync('elsewhere', f.target); }
    refused(host.execute(call('edit', {filePath: f.ownedPath, oldString: 'before', newString: 'after'})));
    if (mutation !== 'symlink') assert.notEqual(fs.readFileSync(f.target, 'utf8'), 'after');
  }
});
test('post-edit foreign drift cannot be blessed by final reread', context => {
  const f = fixture(context, 'before'), host = f.host();
  host.execute(call('read', {filePath: f.ownedPath}));
  assert.equal(host.execute(call('edit', {filePath: f.ownedPath, oldString: 'before', newString: 'after'})).content, '');
  fs.writeFileSync(f.target, 'other');
  refused(host.execute(call('read', {filePath: f.ownedPath})), 'SourcePreimageDrift');
  refused(host.execute(call('read', {filePath: f.ownedPath})), 'SourceSessionInvalidated');
});
test('no-op, repeated edit and unknown option cannot inflate effect claims', context => {
  const f = fixture(context, 'before'), host = f.host();
  host.execute(call('read', {filePath: f.ownedPath}));
  refused(host.execute(call('edit', {filePath: f.ownedPath, oldString: 'before', newString: 'before'})), 'SourceEditHasNoEffect');
  assert.throws(() => f.host({extra: true}), /UnknownOrAccessorField/);
  const second = f.host(); second.execute(call('read', {filePath: f.ownedPath}));
  second.execute(call('edit', {filePath: f.ownedPath, oldString: 'before', newString: 'after'}));
  refused(second.execute(call('edit', {filePath: f.ownedPath, oldString: 'after', newString: 'next'})), 'SourceReadRequired');
});
