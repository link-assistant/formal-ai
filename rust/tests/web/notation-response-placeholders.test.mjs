// PR #1188 T935-T936: a notation rename carries response placeholders too.
import assert from 'node:assert/strict';
import test from 'node:test';
import { capturePlaceholderMapping } from '../../../experiments/formal_ai_subagent/apply-notation-rules.mjs';
import { rewriteNotation, rewriteSource } from '../../../scripts/lib/notation-substitution.mjs';
import { createWorkerContext, evaluate, plain } from './support/browser-runtime.mjs';

test('renaming a captured value carries its placeholder while preserving literal prose', () => {
  const source = 'value sample_value literal "sample_value"\ntext "{sample_value}; {unmapped}; sample_value"\n';
  assert.equal(rewriteNotation(source, new Map([['sample_value', 'sample-value']])),
    'value sample-value literal "sample_value"\ntext "{sample-value}; {unmapped}; sample_value"\n');
});

const worker = createWorkerContext();
await evaluate(worker, 'loadSeed()');

test('all five languages render both traffic documentation links for named and unnamed repositories', () => {
  const values = [
    { name: 'repository', value: 'example/project' },
    { name: 'traffic-ui-docs', value: 'https://docs.github.com/en/repositories/viewing-activity-and-data-for-your-repository/viewing-traffic-to-a-repository' },
    { name: 'traffic-api-docs', value: 'https://docs.github.com/en/rest/metrics/traffic' },
  ];
  for (const language of ['en', 'ru', 'hi', 'zh', 'es']) {
    for (const intent of ['github_repository_traffic', 'github_repository_traffic_unnamed']) {
      const template = evaluate(worker, 'handlerRulesResponseFor(' + JSON.stringify(intent) + ',' + JSON.stringify(language) + ')');
      assert.equal(typeof template, 'string', language + ': ' + intent);
      const rendered = evaluate(worker, 'handlerRulesSubstitute(' + JSON.stringify(template) + ',' + JSON.stringify(values) + ')');
      assert.ok(rendered.includes(values[1].value), rendered);
      assert.ok(rendered.includes(values[2].value), rendered);
      assert.doesNotMatch(rendered, /\{[A-Za-z][A-Za-z0-9_-]*\}/u);
    }
  }
});

test('the reported Russian prompt and an unnamed English query cite rendered documentation', async () => {
  for (const prompt of ['можно ли узнать заходил ли кто либо в твое репо на github?', 'Can I know who visited my GitHub repo?']) {
    const answer = plain(await worker.solve(prompt, [], {}, {}, [], {}));
    assert.equal(answer.intent, 'github_repository_traffic');
    assert.ok(answer.content.includes('https://docs.github.com/en/rest/metrics/traffic'), answer.content);
    assert.doesNotMatch(answer.content, /\{[A-Za-z][A-Za-z0-9_-]*\}/u);
  }
});

test('source templates migrate notation placeholders while Rust format variables retain their spelling', () => {
  const mapping = new Map([['sample_value', 'sample-value']]);
  const source = 'const answer = "{sample_value}";';
  assert.equal(rewriteSource(source, 'javascript', mapping, new Set()).text, 'const answer = "{sample-value}";');
  assert.equal(rewriteSource(source, 'rust', mapping, new Set(['sample_value'])).text, source);
});

test('Rust variable-backed template keys retain their spelling while policy names migrate', () => {
  const source = 'fn plan(sample_backup_value: &str) { policy("sample_backup_value"); fill(&[("sample_backup_value", sample_backup_value)]); }';
  const mapping = new Map([['sample_backup_value', 'sample-backup-value']]);
  const rewritten = rewriteSource(source, 'rust', mapping, new Set(['sample_backup_value']), new Set()).text;
  assert.equal(rewritten, 'fn plan(sample_backup_value: &str) { policy("sample-backup-value"); fill(&[("sample_backup_value", sample_backup_value)]); }');
  assert.equal(rewriteSource(source, 'rust', mapping, new Set(['sample_backup_value']), new Set(['sample_backup_value'])).text,
    'fn plan(sample_backup_value: &str) { policy("sample-backup-value"); fill(&[("sample-backup-value", sample_backup_value)]); }');
});

test('capture ownership carries related templates on the first pass and on replay without renaming unrelated fields', () => {
  const mapping = new Map([['sample_value', 'sample-value'], ['unrelated_field', 'unrelated-field']]);
  for (const spelling of ['sample_value', 'sample-value']) {
    const document = 'handler sample\n  value ' + spelling + ' literal "example"\n  unrelated-field "unrelated field"\n';
    assert.deepEqual([...capturePlaceholderMapping(mapping, document)], [['sample_value', 'sample-value']]);
  }
});
