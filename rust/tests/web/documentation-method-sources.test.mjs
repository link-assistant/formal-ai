import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

const pandas = 'https://pandas.pydata.org/docs/';
const join = pandas + 'reference/api/pandas.DataFrame.join.html';
const diff = pandas + 'reference/api/pandas.Series.diff.html';
const index = pandas + 'searchindex.js';
const numpy = 'https://numpy.org/doc/stable/reference/generated/numpy.ndarray.reshape.html';
const empty = 'https://numpy.org/doc/stable/reference/generated/numpy.empty.html';
const fixture = name => readFileSync(new URL('../fixtures/documentation-method/' + name, import.meta.url), 'utf8');

test('documentation requests resolve their publisher and symbol and use only verified captures', async () => {
  const host = new WorkerHost();
  await host.boot();
  const calls = [];
  const pages = new Map([[join, fixture('join.html')], [diff, fixture('diff.html')],
    [numpy, fixture('reshape.html')], [empty, fixture('empty.html')],
    [index, fixture('pandas-searchindex.js')]]);
  host.context.fetch = async (url) => {
    calls.push(url);
    const text = pages.get(url);
    return text ? new Response(text) : new Response(null, { status: 404 });
  };
  const solve = (prompt, offline = false) => host.run('solve(__prompt, [], {offline: __offline})',
    { __prompt: prompt, __offline: offline });

  const missing = await solve('How does pandas Series.diff work?', true);
  assert.equal(missing.intent, 'docs_method_explanation');
  assert.equal(missing.confidence, 0);
  assert.doesNotMatch(missing.content, /First discrete difference/u);
  assert.deepEqual(calls, []);
  assert.ok(!missing.evidence.some(entry => entry.startsWith('source:')));

  const examples = [
    ['How does pandas Series.diff work?', diff, 'First discrete difference of Series elements'],
    ['Explain numpy.ndarray.reshape', numpy, 'Returns an array containing the same data with a new shape'],
    ['Explain numpy.empty', empty, 'Return a new array of given shape and type'],
    ['как работает pandas DataFrame.join?', join, 'Join columns of another DataFrame', 'Официальная документация для pandas.DataFrame.join'],
    ['pandas DataFrame.join कैसे काम करता है?', join, 'Join columns of another DataFrame', 'pandas.DataFrame.join के आधिकारिक दस्तावेज़'],
    ['pandas DataFrame.join 如何工作？', join, 'Join columns of another DataFrame', 'pandas.DataFrame.join 的官方文档'],
  ];
  for (const [prompt, source, excerpt, citation] of examples) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, 'docs_method_explanation', JSON.stringify(answer.evidence));
    assert.ok(answer.content.includes(excerpt), answer.content);
    assert.ok(answer.content.includes(source), answer.content);
    if (citation) assert.ok(answer.content.includes('[' + citation + ']'), answer.content);
    assert.ok(answer.evidence.includes('source:' + source), JSON.stringify(answer.evidence));
    assert.doesNotMatch(answer.content, /Fixture boundaries/u);
  }
  assert.deepEqual(calls, [diff, numpy, empty, join]);
  for (const [forced, citation] of [['en', 'Official documentation for pandas.DataFrame.join'],
    ['ru', 'Официальная документация для pandas.DataFrame.join']]) {
    const forcedAnswer = await host.run('solve(__prompt, [], {offline: true}, {}, [], {forcedResponseLanguage: __forced})',
      {__prompt: 'pandas DataFrame.join 如何工作？', __forced: forced});
    assert.ok(forcedAnswer.content.includes('[' + citation + ']'), forcedAnswer.content);
  }
  const replay = await solve(examples[0][0], true);
  assert.ok(replay.content.includes(examples[0][2]), replay.content);
  assert.deepEqual(calls, [diff, numpy, empty, join]);

  const ambiguity = await solve('how the join method works in pandas');
  assert.equal(ambiguity.confidence, 0);
  assert.match(ambiguity.content, /^Which `join` method do you mean\?/u);
  assert.match(ambiguity.content, /pandas\.DataFrame\.join.*pandas\.Index\.join.*pandas\.Series\.str\.join/u);
  assert.ok(ambiguity.evidence.includes('source:' + index));
  assert.deepEqual(calls, [diff, numpy, empty, join, index]);

  // Failed canonical spelling can be resolved against the same source index.
  const resolved = await solve('Explain pandas series.diff');
  assert.ok(resolved.content.includes(examples[0][2]), resolved.content);
  assert.ok(resolved.evidence.includes('docs_method:method:pandas.Series.diff'));

  const wrong = pandas + 'reference/api/pandas.Series.unrelated.html';
  pages.set(wrong, fixture('join.html'));
  const rejected = await solve('Explain pandas.Series.unrelated');
  assert.equal(rejected.confidence, 0);
  assert.doesNotMatch(rejected.content, /Join columns/u);
  assert.ok(!rejected.evidence.includes('source:' + wrong));
  const explicit = await host.run('handlerRuleSetClaims("docs_method_explanation", __prompt, __prompt)',
    { __prompt: 'search the web for pandas Series.diff' });
  assert.equal(explicit, false);
});
