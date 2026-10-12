import assert from 'node:assert/strict';
import {test} from 'node:test';
import fs from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {censusDocuments, censusSourceClosureComplete} from '../../../js/agentic/node-host.mjs';
import {stableId} from '../../../js/agentic/crate/engine_stable_identifier.mjs';

test('unavailable or malformed census closure never proves uniqueness', () => {
  const directory = fs.mkdtempSync(join(tmpdir(), 'census-closure-'));
  try {
    assert.equal(censusSourceClosureComplete(directory), false);
    fs.mkdirSync(join(directory, 'data/meta/self-ast/src'), {recursive: true});
    fs.mkdirSync(join(directory, 'rust/src'), {recursive: true});
    const source = 'pub const ARBITRARY: &str = "azure";\n';
    const target = 'src/arbitrary.rs';
    const document = 'self_ast_census\n  target ' + target + '\n  content_id '
      + stableId('source_module', source) + '\n  byte_len ' + Buffer.byteLength(source)
      + '\n  symbols\n    const ARBITRARY 1 1\n';
    fs.writeFileSync(join(directory, 'rust', target), source);
    fs.writeFileSync(join(directory, 'data/meta/self-ast/src/arbitrary.lino'), document);
    assert.equal(censusSourceClosureComplete(directory), true);
    const expected = censusDocuments(directory);
    assert.equal(expected.length, 1);
    fs.writeFileSync(join(directory, 'data/meta/self-ast/README.txt'), 'not a census');
    assert.deepEqual(censusDocuments(directory), expected);
    assert.equal(censusSourceClosureComplete(directory), true);
    fs.writeFileSync(join(directory, 'data/meta/self-ast/src/missing.lino'),
      document.replaceAll('src/arbitrary.rs', 'src/missing.rs'));
    assert.deepEqual(censusDocuments(directory), expected);
    assert.equal(censusSourceClosureComplete(directory), false);
    fs.rmSync(join(directory, 'data/meta/self-ast/src/missing.lino'));
    fs.writeFileSync(join(directory, 'rust', target), source.replace('azure', 'cobalt'));
    assert.deepEqual(censusDocuments(directory), []);
    assert.equal(censusSourceClosureComplete(directory), false);
    fs.writeFileSync(join(directory, 'rust', target), source);
    fs.writeFileSync(join(directory, 'data/meta/self-ast/src/arbitrary.lino'),
      document.replace('src/arbitrary.rs', '../outside.rs'));
    assert.equal(censusSourceClosureComplete(directory), false);
  } finally { fs.rmSync(directory, {recursive: true, force: true}); }
});

test('duplicate identity fields and nonregular source or census paths refuse completeness', () => {
  const directory = fs.mkdtempSync(join(tmpdir(), 'census-typed-closure-'));
  try {
    fs.mkdirSync(join(directory, 'data/meta/self-ast/src'), {recursive: true});
    fs.mkdirSync(join(directory, 'rust/src'), {recursive: true});
    const source = 'pub const ARBITRARY: &str = "azure";\n';
    const sourcePath = join(directory, 'rust/src/arbitrary.rs');
    const censusPath = join(directory, 'data/meta/self-ast/src/arbitrary.lino');
    const document = 'self_ast_census\n  target src/arbitrary.rs\n  content_id '
      + stableId('source_module', source) + '\n  byte_len ' + Buffer.byteLength(source)
      + '\n  symbols\n    const ARBITRARY 1 1\n';
    fs.writeFileSync(sourcePath, source);
    fs.writeFileSync(censusPath, document);
    assert.equal(censusSourceClosureComplete(directory), true);
    for (const field of ['target', 'content_id', 'byte_len']) {
      const original = document.split('\n').find(line => line.startsWith('  ' + field + ' '));
      for (const duplicate of [original, '  ' + field + ' invented']) {
        fs.writeFileSync(censusPath, document.replace(original, original + '\n' + duplicate));
        assert.equal(censusSourceClosureComplete(directory), false, field + ' duplicate');
        assert.deepEqual(censusDocuments(directory), []);
      }
    }
    fs.writeFileSync(censusPath, document);
    const outside = join(directory, 'outside.rs');
    fs.writeFileSync(outside, source);
    fs.rmSync(sourcePath);
    fs.symlinkSync(outside, sourcePath);
    assert.equal(censusSourceClosureComplete(directory), false, 'source leaf symlink');
    assert.deepEqual(censusDocuments(directory), []);
    fs.rmSync(sourcePath);
    fs.writeFileSync(sourcePath, source);
    fs.rmSync(censusPath);
    const outsideCensus = join(directory, 'outside.lino');
    fs.writeFileSync(outsideCensus, document);
    fs.symlinkSync(outsideCensus, censusPath);
    assert.equal(censusSourceClosureComplete(directory), false, 'census leaf symlink');
    fs.rmSync(censusPath);
    fs.writeFileSync(censusPath, document);
    fs.renameSync(join(directory, 'rust/src'), join(directory, 'actual-source'));
    fs.symlinkSync(join(directory, 'actual-source'), join(directory, 'rust/src'), 'dir');
    assert.equal(censusSourceClosureComplete(directory), false, 'source ancestor symlink');
    fs.rmSync(join(directory, 'rust/src'));
    fs.renameSync(join(directory, 'actual-source'), join(directory, 'rust/src'));
    fs.renameSync(join(directory, 'data/meta/self-ast'), join(directory, 'actual-census'));
    fs.symlinkSync(join(directory, 'actual-census'), join(directory, 'data/meta/self-ast'), 'dir');
    assert.equal(censusSourceClosureComplete(directory), false, 'census ancestor symlink');
  } finally { fs.rmSync(directory, {recursive: true, force: true}); }
});
