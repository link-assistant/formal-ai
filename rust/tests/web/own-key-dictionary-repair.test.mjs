// Install rust/tests/web/own-key-dictionary-repair.test.mjs. Not run locally yet.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import * as parser from '@babel/parser';
import { babelSourceFrontend } from '../../../scripts/lib/source-cst-babel.mjs';
import { readSyntaxRules } from '../../../js/meta/readable-source.js';
import { deriveOwnKeyDictionaryRepair } from '../../../js/agentic/source_dictionary_binding.mjs';
const fixtures = JSON.parse(fs.readFileSync(new URL('../fixtures/own-key-dictionary-repair-cases.json', import.meta.url), 'utf8'));
const knowledge = JSON.parse(fs.readFileSync(new URL('../../../data/meta/ecmascript-own-key-dictionary.json', import.meta.url), 'utf8'));
const rules = readSyntaxRules(fs.readFileSync(new URL('../../../data/meta/source-model/syntax-rules.lino', import.meta.url), 'utf8'));
for (const entry of fixtures.cases) test(entry.name, () => {
  const derive = () => deriveOwnKeyDictionaryRepair({
    ...entry,
    knowledge,
    frontend: {
      parse: babelSourceFrontend(parser, entry.owner),
      rules
    }
  });
  if (entry.gap) {
    assert.throws(derive, error => error.message === entry.gap);
    return;
  }
  const result = derive(),
    proof = result.verification,
    candidate = result.candidate.source;
  assert.equal(proof.writeCount, entry.writes);
  assert.equal(proof.readCount, entry.reads);
  assert.equal(proof.spreadCount, entry.spreads);
  assert.equal(proof.completeConcreteResidual, true);
  assert.equal(proof.initializerTriviaPreserved, true);
  assert.equal(proof.independentGrammarReparse, true);
  assert.equal(proof.exactInverse, true);
  const [start, end] = result.binding.initializerSpan,
    before = Buffer.from(entry.source),
    after = Buffer.from(candidate);
  assert.deepEqual(after.subarray(0, start), before.subarray(0, start));
  assert.ok(candidate.endsWith(before.subarray(end).toString()));
  if (entry.preserveTrivia) assert.deepEqual(candidate.match(/\s/gu), entry.source.match(/\s/gu));
  assert.equal(result.authored, false);
  assert.equal(result.verified, false);
  assert.equal(result.independentAcceptance, false);
});
test('candidate own-key storage actually preserves ordinary and special legal key values', async () => {
  for (const entry of fixtures.cases.slice(0, 2)) {
    const result = deriveOwnKeyDictionaryRepair({
      ...entry,
      knowledge,
      frontend: {
        parse: babelSourceFrontend(parser, entry.owner),
        rules
      }
    });
    const module = await import('data:text/javascript,' + encodeURIComponent(result.candidate.source));
    const method = module.collect ?? module.resolve;
    for (const key of ['lower', '__proto__', 'constructor', 'toString']) {
      const result = method(key, 17);
      if (typeof result === 'number') assert.equal(result, 17);else {
        assert.equal(Object.hasOwn(result, key), true);
        assert.equal(result[key], 17);
      }
    }
  }
});
