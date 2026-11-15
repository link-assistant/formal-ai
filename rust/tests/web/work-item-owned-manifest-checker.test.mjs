// Independent literal-field qualification only; no production renderer is used.
import test from 'node:test';
import assert from 'node:assert/strict';
import { observedManifestEvidence } from '../../../scripts/work-item-qualification/manifest-claims.mjs';
import { checkOwnedManifestTables } from '../../../scripts/work-item-qualification/owned-manifest-claims.mjs';
import { unexpectedCandidateChanges } from '../../../scripts/work-item-qualification/independent-oracle.mjs';
const notice = 'These are literal manifest declarations; they do not establish executable runtime use.';
const header = '| Dependency | Declared version | Dependency section | Target qualifier | Literal options |';
const separator = '| --- | --- | --- | --- | --- |';
const block = (owner, rows) => ['<!-- formal-ai-manifest-start: ' + owner + ' -->', 'Dependency declarations from `' + owner + '`:', notice, header, separator, ...rows, '<!-- formal-ai-manifest-end: ' + owner + ' -->'].join('\n');
const row = (name, version, section, target = 'unqualified', options = '{}') => '| `' + name + '` | `' + version + '` | `' + section + '` | `' + target + '` | `' + options + '` |';
function evidence() {
  const files = [{
    path: 'config/packages.json',
    content: '{"peerDependencies":{"stream-kit":"^7"}}'
  }, {
    path: 'native/Project.toml',
    content: '[dependencies]\nlocal={path="../local"}\n[target.\'cfg(unix)\'.dependencies]\ncodec={version="3",default-features=false,features=["read"]}\n'
  }];
  const value = observedManifestEvidence(files);
  assert.ok(value.observations.every(source => source.status === 'observed'), 'independent Python parsers must be available for this CI test');
  return value;
}
const check = (text, evidence, options) => checkOwnedManifestTables(text, evidence.facts, evidence.observations, options);
test('canonical display rows bind all literal fields to independently parsed frozen hashes', () => {
  const observed = evidence();
  const text = block('config/packages.json', [row('stream-kit', '^7', 'peerDependencies')]) + '\n\n' + block('native/Project.toml', [row('local', 'not declared', 'dependencies', 'unqualified', '{"path":"../local"}'), row('codec', '3', 'dependencies', 'cfg(unix)', '{"default-features":false,"features":["read"]}')]);
  const actual = check(text, observed, {
    expectedOwners: ['config/packages.json', 'native/Project.toml']
  });
  assert.equal(actual.length, 3);
  assert.ok(actual.every(claim => claim.status === 'verified-literal-declaration'));
  for (const claim of actual) assert.equal(claim.sourceIdentity, observed.observations.find(source => source.path === claim.owner).sourceIdentity);
  assert.ok(actual.every(claim => claim.semantics.includes('whole-task acceptance remain unverified')));
});
test('wrong versions targets options and nonexistent names contradict frozen declarations', () => {
  const observed = evidence();
  for (const bad of [row('codec', '9', 'dependencies', 'cfg(unix)', '{"default-features":false,"features":["read"]}'), row('codec', '3', 'dependencies', 'cfg(windows)', '{"default-features":false,"features":["read"]}'), row('codec', '3', 'dependencies', 'cfg(unix)', '{"default-features":true,"features":["read"]}'), row('invented', '3', 'dependencies')]) {
    assert.ok(check(block('native/Project.toml', [bad]), observed).some(claim => claim.status === 'contradicted'));
  }
});
test('missing owned evidence and unknown observations remain separate from contradictions', () => {
  const observed = evidence();
  assert.equal(check('# No generated declaration table', observed, {
    expectedOwners: ['native/Project.toml']
  })[0].status, 'missing');
  assert.equal(check(block('native/Project.toml', []), observed)[0].status, 'missing');
  const unknown = {
    ...observed,
    observations: observed.observations.map(source => ({
      ...source,
      status: 'unknown'
    }))
  };
  assert.equal(check(block('native/Project.toml', [row('local', 'not declared', 'dependencies', 'unqualified', '{"path":"../local"}')]), unknown)[0].status, 'unknown');
});
test('whole-line paired unique markers refuse nested mismatched inline and fenced claims', () => {
  const observed = evidence(),
    valid = block('config/packages.json', [row('stream-kit', '^7', 'peerDependencies')]);
  const malformed = [valid.replace('<!-- formal-ai-manifest-end: config/packages.json -->', '<!-- formal-ai-manifest-end: another.json -->'), valid.replace(notice, '<!-- formal-ai-manifest-start: nested.json -->\n' + notice), valid.replace('<!-- formal-ai-manifest-start: config/packages.json -->', 'prefix <!-- formal-ai-manifest-start: config/packages.json -->'), valid + '\n' + valid, '```markdown\n' + valid + '\n```'];
  for (const text of malformed) {
    const actual = check(text, observed);
    assert.ok(actual.some(claim => claim.status === 'unknown'));
    assert.ok(actual.every(claim => claim.status !== 'verified-literal-declaration'));
  }
  assert.equal(check(valid.replaceAll('\n', '\r\n'), observed)[0].status, 'verified-literal-declaration');
});
test('duplicate JSON options and noncanonical options never acquire literal credit', () => {
  const observed = evidence();
  for (const options of ['{"path":"../wrong","path":"../local"}', '{ "path": "../local" }']) {
    const actual = check(block('native/Project.toml', [row('local', 'not declared', 'dependencies', 'unqualified', options)]), observed);
    assert.equal(actual[0].status, 'unknown');
  }
});
test('display sentinel collisions refuse ambiguous manifest bindings', () => {
  const source = {
    path: 'config/packages.json',
    sourceIdentity: 'a'.repeat(64),
    status: 'observed'
  };
  const base = {
    kind: 'manifest-declaration',
    manifest: source.path,
    sourceIdentity: source.sourceIdentity,
    section: 'dependencies',
    name: 'ambiguous',
    version: '1'
  };
  const actual = checkOwnedManifestTables(block(source.path, [row('ambiguous', '1', 'dependencies')]), [base, {
    ...base,
    target: 'unqualified'
  }], [source]);
  assert.equal(actual[0].status, 'unknown');
  assert.equal(actual[0].reason, 'ambiguous-display-sentinel-binding');
});
test('candidate custody flags changed tests and rename sources outside the guide', () => {
  const file = (filename, status = 'modified', extra = {}) => ({
    filename,
    status,
    changes: 1,
    ...extra
  });
  assert.deepEqual(unexpectedCandidateChanges([file('guide.md'), file('dev/log/run.log'), file('.formal-ai/general-change-plan.lino')], 'guide.md'), []);
  assert.deepEqual(unexpectedCandidateChanges([file('guide.md'), file('tests/candidate-proof.mjs')], 'guide.md'), ['tests/candidate-proof.mjs']);
  assert.deepEqual(unexpectedCandidateChanges([file('guide.md', 'renamed', {
    previous_filename: 'src/owner.mjs'
  })], 'guide.md'), ['src/owner.mjs']);
});
