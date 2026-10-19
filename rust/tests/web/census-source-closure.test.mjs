import assert from 'node:assert/strict';
import {test} from 'node:test';
import fs from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {censusDocuments, censusSourceClosureComplete} from '../../../js/agentic/node-host.mjs';
import {moduleFromDocument} from '../../../js/agentic/crate/self_ast_census.mjs';
import {stableId} from '../../../js/agentic/crate/engine_stable_identifier.mjs';

test('actual source closure is independent of missing, stale or forged count documents', () => {
  const directory = fs.mkdtempSync(join(tmpdir(), 'source-content-closure-'));
  try {
    assert.equal(censusSourceClosureComplete(directory), false);
    fs.mkdirSync(join(directory, 'rust/src'), {recursive: true});
    const source = 'pub const ARBITRARY: &str = "azure";\n';
    const sourcePath = join(directory, 'rust/src/arbitrary.rs');
    fs.writeFileSync(sourcePath, source);
    assert.equal(censusSourceClosureComplete(directory), true);
    const before = censusDocuments(directory);
    assert.equal(before.length, 1);
    assert.equal(before[0].sourceIdentity.path, 'src/arbitrary.rs');
    assert.equal(before[0].sourceIdentity.content_id, stableId('source_module', source));
    assert.equal(before[0].module.fidelity, 'navigation');
    assert.equal(before[0].text, null);
    fs.mkdirSync(join(directory, 'data/meta/self-ast/src'), {recursive: true});
    const legacy = 'self_ast_census\n  target src/missing.rs\n  content_id forged\n  byte_len 1\n  symbols\n    const PHANTOM 1 1\n';
    fs.writeFileSync(join(directory, 'data/meta/self-ast/src/missing.lino'), legacy);
    assert.equal(moduleFromDocument(legacy), null, 'a counter is not recursive source content');
    assert.deepEqual(censusDocuments(directory), before);
    assert.equal(censusSourceClosureComplete(directory), true);
    fs.writeFileSync(sourcePath, source.replace('azure', 'cobalt'));
    const after = censusDocuments(directory);
    assert.notEqual(after[0].sourceIdentity.content_id, before[0].sourceIdentity.content_id);
    assert.equal(after[0].sourceIdentity.content_id, stableId('source_module', source.replace('azure', 'cobalt')));
    for (const field of ['target', 'content_id', 'byte_len']) {
      const row = legacy.split('\n').find(row => row.startsWith('  ' + field + ' '));
      for (const extra of [row, '  ' + field + ' ../outside']) {
        const forged = legacy.replace(row, row + '\n' + extra);
        fs.writeFileSync(join(directory, 'data/meta/self-ast/src/missing.lino'), forged);
        assert.equal(moduleFromDocument(forged), null);
        assert.deepEqual(censusDocuments(directory), after, 'unowned data never supplies source owners');
      }
    }
  } finally { fs.rmSync(directory, {recursive: true, force: true}); }
});

test('nonregular, unreadable encoding and oversized actual sources never prove complete ownership', () => {
  const directory = fs.mkdtempSync(join(tmpdir(), 'source-content-refusals-'));
  try {
    fs.mkdirSync(join(directory, 'rust/src'), {recursive: true});
    const source = 'pub const ARBITRARY: &str = "azure";\n';
    const sourcePath = join(directory, 'rust/src/arbitrary.rs');
    const outside = join(directory, 'outside.rs');
    fs.writeFileSync(outside, source);
    fs.symlinkSync(outside, sourcePath);
    assert.equal(censusSourceClosureComplete(directory), false, 'source leaf symlink');
    assert.deepEqual(censusDocuments(directory), []);
    fs.rmSync(sourcePath);
    fs.writeFileSync(sourcePath, Buffer.from([0xff]));
    assert.equal(censusSourceClosureComplete(directory), false, 'invalid UTF-8');
    assert.deepEqual(censusDocuments(directory), []);
    fs.truncateSync(sourcePath, 4 * 1024 * 1024 + 1);
    assert.equal(censusSourceClosureComplete(directory), false, 'bounded source read');
    assert.deepEqual(censusDocuments(directory), []);
    fs.rmSync(sourcePath);
    fs.rmdirSync(join(directory, 'rust/src'));
    const target = join(directory, 'outside-sources'); fs.mkdirSync(target);
    fs.writeFileSync(join(target, 'arbitrary.rs'), source);
    fs.symlinkSync(target, join(directory, 'rust/src'));
    assert.equal(censusSourceClosureComplete(directory), false, 'source ancestor symlink');
    assert.deepEqual(censusDocuments(directory), []);
  } finally { fs.rmSync(directory, {recursive: true, force: true}); }
});
