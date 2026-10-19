import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import {readSourceLinks, writeSourceLinks, sourceFromLinks, ownedSourceLinks, sourceOwner} from '../../../js/meta/readable-source.js';
const original = readSourceLinks(fs.readFileSync(new URL('../fixtures/source-model/Rust.lino', import.meta.url), 'utf8'));
test('readable owner survives source-free reload without replacing recursive source content', () => {
  const owned = ownedSourceLinks('src/retained.rs', original);
  const reloaded = readSourceLinks(writeSourceLinks(owned));
  assert.equal(sourceOwner(reloaded), 'src/retained.rs');
  assert.equal(sourceFromLinks(reloaded), sourceFromLinks(original));
  assert.deepEqual(reloaded[2], original);
  assert.equal(sourceOwner(original), null);
});
test('source ownership rejects traversal, platform aliases, duplicate paths and ambiguous payloads', () => {
  for (const path of ['', '/outside.rs', '../outside.rs', 'src/../outside.rs', 'src//retained.rs', 'C:/outside.rs', 'src\\retained.rs']) {
    assert.throws(() => ownedSourceLinks(path, original));
  }
  const owned = ownedSourceLinks('src/retained.rs', original);
  for (const extra of [original, ['path', {quoted: 'src/other.rs'}], 'hidden payload']) {
    assert.throws(() => sourceFromLinks([...owned, extra]));
  }
  assert.throws(() => sourceFromLinks(['owned-source', ['path', {quoted: 'src/retained.rs'}]]));
});
