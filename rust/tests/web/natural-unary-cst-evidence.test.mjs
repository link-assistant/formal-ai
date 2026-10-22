import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import test, {before, after} from 'node:test';
import {parse} from '@babel/parser';
import {sourceCstFrontends} from '../../../scripts/lib/source-cst-frontends.mjs';
import {readSourceLinks, sourceFromLinks, sourceOwner, verifySourceLinks} from '../../../js/meta/readable-source.js';

const directory = new URL('../../../docs/case-studies/issue-1328/evidence/unary-property/', import.meta.url);
const evidence = JSON.parse(readFileSync(new URL('completion.json', directory), 'utf8'));
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
let runtime;
before(async () => { runtime = await sourceCstFrontends(undefined, ['JavaScript']); });
after(() => runtime?.close());

for (const phase of ['before', 'after']) {
  test('persisted ' + phase + ' self-edit CST restores complete source and reparses', () => {
    const receipt = evidence.sourceFreeReadableRecursiveCst[phase];
    const notation = readFileSync(new URL(receipt.model, directory), 'utf8');
    assert.equal(digest(notation), receipt.modelSha256);
    const document = readSourceLinks(notation);
    assert.equal(sourceOwner(document), evidence.sourceEffect.owner);
    const source = sourceFromLinks(document);
    assert.equal(digest(source), receipt.sourceSha256);
    assert.equal(verifySourceLinks(document, runtime.forSource(sourceOwner(document), 'JavaScript')), true);
    assert.equal(notation.includes('tokenIndex'), false);
    assert.match(notation, /\(comment \(form "comment"\) \(text /u);
    assert.match(notation, /\(layout \(form "whitespace"\) \(text /u);
  });
}

test('the historical source change retains all comment content and the captured callback', () => {
  const sources = ['before', 'after'].map(phase => sourceFromLinks(readSourceLinks(
    readFileSync(new URL(evidence.sourceFreeReadableRecursiveCst[phase].model, directory), 'utf8')
  )));
  const comments = source => parse(source, {sourceType: 'module'}).comments.map(comment => comment.value);
  assert.deepEqual(comments(sources[0]), comments(sources[1]));
  assert(sources[0].includes('token => token.kind'));
  assert(sources[1].includes('tokens.map(canonicalUnaryProperty("kind"))'));
  for (const source of sources) assert(source.includes('item => item.local === declaration?.name'));
});
