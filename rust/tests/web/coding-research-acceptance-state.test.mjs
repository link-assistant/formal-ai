import { deriveCodingProcedure } from '../../../js/agentic/coding_research.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { observeCodingResearchAcceptanceTokens } from '../../../js/agentic/coding_research_observation.mjs';
import { independentAcceptanceKernel } from '../../../js/agentic/portable/coding_research_acceptance.mjs';
import * as state from '../../../js/agentic/portable/coding_research_state.mjs';
const vectors = JSON.parse(fs.readFileSync(new URL('../fixtures/coding-research/held-out.json', import.meta.url)));
function tokens(source) {
  return observeCodingResearchAcceptanceTokens(source);
}

function accepts(source, name, arity) {
  const observed = tokens(source);
  return independentAcceptanceKernel(observed.texts, observed.kinds, name, './source.mjs', '', '',
    'acceptance.test.mjs', 'node --test acceptance.test.mjs', arity, true);
}
test('actual tokenizer and canonical acceptance preserve zero and nonzero literal-only assertions', () => {
  for (const arity of [0, 1, 2]) {
    const args = Array.from({ length: arity }, (_, index) => String(index + 1)).join(',');
    const source = "import { signal } from './source.mjs'; import assert from 'node:assert/strict';"
      + 'assert.equal(signal(' + args + '), 3);';
    assert.equal(accepts(source, 'signal', arity), true);
    assert.equal(accepts(source, 'signal', arity + 1), false);
    assert.equal(accepts(source.replace(', 3)', ', signal())'), 'signal', arity), false);
    assert.equal(accepts(source + 'process.exit(0);', 'signal', arity), false);
  }
});
test('all carried held-out fixture assertions reach actual canonical grammar', () => {
  for (const row of vectors) {
    const observed = tokens(row.acceptance);
    const match = /function ([\w$]+)\(([^)]*)\)/u.exec(row.prompt);
    assert(match, row.name);
    const arity = match[2].trim() ? match[2].split(',').length : 0;
    assert.equal(independentAcceptanceKernel(observed.texts, observed.kinds, match[1], './' + row.module,
      '', '', 'acceptance.test.mjs', 'node --test acceptance.test.mjs', arity, true), true, row.name);
  }
});
test('canonical state refuses unsupported, unobserved and failed evidence without authoring success', () => {
  const args = ['javascript', true, true, 'signal', [], true, true, true, true, false,
    true, true, false, true, true, true];
  assert.equal(state.codingResearchPreflightKernel(...args), 'discover');
  const duplicate = [...args]; duplicate[4] = ['same', 'same'];
  assert.equal(state.codingResearchPreflightKernel(...duplicate), 'gap:UnsupportedBinding');
  const unseen = [...args]; unseen[7] = false;
  assert.equal(state.codingResearchPreflightKernel(...unseen), 'read:module');
  assert.equal(state.codingResearchSearchDecision(false, true), 'search');
  assert.equal(state.codingResearchDiscoveryKernel(2, true, ''), 'gap:AmbiguousSourceProcedure');
  const effect = ['', 'export function signal() { return 1; }', false, true, true, true, 'final'];
  assert.equal(state.codingResearchEffectKernel(...effect, 'unreported', '', true), 'gap:VerificationReceiptIncomplete');
  assert.equal(state.codingResearchEffectKernel(...effect, 'failed', '', true), 'created:failed');
  assert.equal(state.codingResearchEffectKernel(...effect, 'succeeded', '', true), 'created:verified');
  assert.equal(state.codingResearchEffectKernel(...effect, 'succeeded', 'NoTestsExecuted', true), 'gap:NoTestsExecuted');
});
test('canonical observation selectors retain exact search query, execution boundary and bounded HTTPS inputs', () => {
  assert.equal(state.codingResearchSearchIndex(['search', 'search'], [true, true], [true, true], ['other', 'need'], [1, 1], 'need'), 1);
  assert.equal(state.codingResearchSearchIndex(['search'], [true], [true], ['other'], [1], 'need'), -1);
  assert.equal(state.codingResearchLatestReadIndex(['read', 'run', 'read']), 2);
  assert.equal(state.codingResearchRunIndex(['run', 'read', 'run'], [true, true, true], ['test', '', 'test'], 1, 'test'), 2);
  assert.equal(state.codingResearchRunIndex(['run', 'read'], [true, true], ['test', ''], 1, 'test'), -1);
  const urls = state.codingResearchSourceURLs(['unsafe', 'one', 'one', 'two', 'three', 'four'], [false, true, true, true, true, true]);
  assert.deepEqual(urls, ['one', 'two', 'three']);
  assert.equal(state.codingResearchNextURL(urls, ['one']), 'two');
  assert.equal(state.codingResearchNextURL(urls, urls), '');
});

test('canonical binding, capture, language and pure repair decisions remain fail closed', () => {
  assert.equal(state.codingResearchBindingsKernel(['fresh', 'input'], [Array.from('fresh'), Array.from('input')], [false, false]), true);
  assert.equal(state.codingResearchBindingsKernel(['eval'], [Array.from('eval')], [false]), false);
  assert.equal(state.codingResearchCaptureDecision(131073, true), 'gap:SearchResultTooLarge');
  assert.equal(state.codingResearchLanguageEligible('javascript', 'rust'), false);
  assert.equal(state.codingResearchLanguageEligible('javascript', 'javascript'), true);
  assert.equal(state.codingResearchPureRepairKernel(true, true, true, false, true, 1, 0, 'fresh', [], ['fresh'], ['fresh'], 'fresh', []), false);
});

test('wrong-language malformed blocks are excluded before catalog or host syntax observation', () => {
  const request = { language: 'javascript', name: 'heldOut', parameters: [], module: 'held-out.mjs' };
  assert.equal(deriveCodingProcedure(request, { codeBlocks: [{ language: 'rust', text: '{{{ malformed' }] }), null);
});
