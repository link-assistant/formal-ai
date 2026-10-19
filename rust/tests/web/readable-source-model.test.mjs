import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {parseLinks, writeLink} from '../../../js/logging/lino-codec.mjs';
import {readSourceLinks, writeSourceLinks, sourceFromLinks} from '../../../js/meta/readable-source.js';
const root = new URL('../fixtures/source-model/', import.meta.url);
const corpus = parseLinks(fs.readFileSync(new URL('cases.lino', root), 'utf8'), {quotedText: 'json'});
for (const row of corpus) {
  const value = name => row.find(item => Array.isArray(item) && item[0] === name)[1].quoted;
  test('retained actual ' + value('language') + ' CST contains complete readable source', () => {
    const notation = fs.readFileSync(new URL(value('document'), root), 'utf8');
    const model = readSourceLinks(notation);
    assert.equal(sourceFromLinks(model), value('source'));
    assert.deepEqual(readSourceLinks(writeSourceLinks(model)), model);
    assert(!notation.includes('tokenIndex'));assert(!notation.includes('byteRange'));
    if (value('source').includes('你好')) assert(notation.includes('你好'));
  });
}
test('explicit readable text mode preserves Unicode, quotes, layout and percent signs', () => {
  const value = ['meaning', {quoted: '你好 café 🚀 "quoted"\r\n\t100%\\'}];
  assert.deepEqual(parseLinks(writeLink(value, {quotedText: 'json'}), {quotedText: 'json'}), [value]);
  assert.equal(writeLink(['legacy', {quoted: 'unchanged'}]), '(legacy "unchanged")');
});
test('malformed and hidden source payloads cannot become retained source', () => {
  const model = readSourceLinks(fs.readFileSync(new URL('JavaScript.lino', root), 'utf8'));
  assert.throws(() => sourceFromLinks([...model, 'hidden payload']));
  const duplicate = structuredClone(model);duplicate.push(['language', {quoted: 'JavaScript'}]);
  assert.throws(() => sourceFromLinks(duplicate));
  assert.throws(() => readSourceLinks('(source (language "JavaScript") (syntax (form "program") (text "unterminated)))'));
});
