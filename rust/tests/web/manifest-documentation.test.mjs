// Install at rust/tests/web/manifest-documentation.test.mjs; not run locally yet.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { deriveManifestDocumentation, deriveSourceDocumentation, sourceDocumentationPolicy } from '../../../js/agentic/source_documentation.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
const blocked = () => {
  throw Error('LocalWasmExecutionForbidden');
};
await installNodeHost(new WorkerHost({
  WebAssembly: {
    ...WebAssembly,
    instantiate: blocked,
    compile: blocked,
    Module: blocked,
    Instance: blocked
  }
}));
const fixtures = JSON.parse(readFileSync(new URL('../fixtures/manifest-documentation-cases.json', import.meta.url), 'utf8'));
for (const entry of fixtures.cases) test(entry.name, () => {
  const actual = deriveManifestDocumentation(entry.task, entry.owner, entry.source, entry.guide, fixtures.forms);
  if (entry.gap) {
    assert.equal(actual.gap, entry.gap);
    return;
  }
  assert.equal(actual.gap, undefined);
  assert.equal(actual.sourceIdentity, createHash('sha256').update(entry.source).digest('hex'));
  assert.equal(actual.value.length, entry.rows);
  assert.ok(actual.content.startsWith(entry.guide));
  for (const text of entry.contains) assert.ok(actual.paragraph.includes(text), text);
  for (const text of entry.excludes ?? []) assert.ok(!actual.paragraph.includes(text), text);
  const repeated = deriveManifestDocumentation(entry.task, entry.owner, entry.source, actual.content, fixtures.forms);
  assert.equal(repeated.content, actual.content);
});
test('refresh replaces only this manifest block, preserving other source declarations', () => {
  const first = deriveManifestDocumentation('Document dependencies.', 'package.json', '{"dependencies":{"shared-kit":"1"}}', '# Keep 雪\n', fixtures.forms);
  const other = deriveManifestDocumentation('Document dependencies.', 'native/package.json', '{"dependencies":{"shared-kit":"7"}}', first.content, fixtures.forms);
  assert.equal(other.gap, undefined);
  const updated = deriveManifestDocumentation('Document dependencies.', 'package.json', '{"dependencies":{"shared-kit":"2"}}', other.content, fixtures.forms);
  assert.equal(updated.gap, undefined);
  assert.ok(updated.content.includes('`shared-kit` | `2`'));
  assert.ok(!updated.content.includes('`shared-kit` | `1`'));
  assert.ok(updated.content.includes(other.paragraph));
  assert.ok(updated.content.startsWith('# Keep 雪\n'));
});
test('source and guide share one UTF8 byte cap; malformed owned markers refuse', () => {
  assert.equal(deriveManifestDocumentation('Document dependencies.', 'package.json', '{"dependencies":{"kit":"1"}}', '雪'.repeat(87381), fixtures.forms).gap, 'ManifestDocumentationByteLimit');
  const first = deriveManifestDocumentation('Document dependencies.', 'package.json', '{"dependencies":{"kit":"1"}}', '', fixtures.forms);
  assert.equal(deriveManifestDocumentation('Document dependencies.', 'package.json', '{"dependencies":{"kit":"1"}}', first.paragraph.split('\n')[0], fixtures.forms).gap, 'AmbiguousManifestDocumentationBlock');
});
test('actual source policy delegates only explicit declaration requests and retains primitive binding', () => {
  const policy = sourceDocumentationPolicy();
  assert.ok(policy.manifestRow);
  const task = 'Update guide.md to document dependencies, including their versions and dependency sections. Preserve all existing text.';
  const source = '{"dependencies":{"river-kit":"1"}}';
  const derived = deriveSourceDocumentation(task, 'package.json', source, '# Keep\n', policy.template, policy);
  assert.equal(derived.key, 'manifest-declarations');
  assert.equal(derived.facts.length, 1);
  const ambiguous = deriveSourceDocumentation(task + ' Document the package manager.', 'package.json', '{"packageManager":"tool@1","dependencies":{"river-kit":"1"}}', '', policy.template, policy);
  assert.equal(ambiguous.gap, 'AmbiguousDocumentationBinding');
  const quoted = deriveSourceDocumentation('Update guide.md. Preserve text. "dependencies versions sections"', 'package.json', source, '', policy.template, policy);
  assert.equal(quoted.gap, 'MissingPropertyBinding');
  const primitive = deriveSourceDocumentation('Document the package manager.', 'project.json', '{"packageManager":"tool@1"}', '', policy.template, policy);
  assert.equal(primitive.key, 'packageManager');
});
test('readable markers must pair by owner, never nest or repeat', () => {
  const run = guide => deriveManifestDocumentation('Document dependencies.', 'package.json', '{"dependencies":{"river-kit":"1"}}', guide, fixtures.forms);
  for (const guide of ['<!-- formal-ai-manifest-start: a.json -->\n<!-- formal-ai-manifest-end: b.json -->', '<!-- formal-ai-manifest-start: a.json -->\n<!-- formal-ai-manifest-start: b.json -->\n<!-- formal-ai-manifest-end: b.json -->\n<!-- formal-ai-manifest-end: a.json -->', '<!-- formal-ai-manifest-start: a.json -->\n<!-- formal-ai-manifest-end: a.json -->\n<!-- formal-ai-manifest-start: a.json -->\n<!-- formal-ai-manifest-end: a.json -->']) assert.equal(run(guide).gap, 'AmbiguousManifestDocumentationBlock');
  assert.ok(run('# Keep\n').paragraph.startsWith('<!-- formal-ai-manifest-start: package.json -->'));
});
