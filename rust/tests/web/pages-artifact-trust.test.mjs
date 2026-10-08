// PR #1188 T937-T938: Pages inputs verify the artifact, never select executable code.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import test from 'node:test';
const workflow = readFileSync(new URL('../../../.github/workflows/pages-artifact.yml', import.meta.url), 'utf8');
function shellFor(name) {
  const step = workflow.split('      - name: ' + name + '\n')[1].split('\n      - ')[0];
  return step.split('        run: |\n')[1].split('\n').map((line) => line.slice(10)).join('\n');
}
test('checkout uses the trusted triggering ref before verification or execution', () => {
  assert.match(workflow, /ref: \$\{\{ github\.ref \}\}/u);
  assert.doesNotMatch(workflow, /ref: \$\{\{ (?:inputs\.|steps\.pages_ref)/u);
  assert.ok(workflow.indexOf('Verify trusted release branch') < workflow.indexOf('Install web bundle dependencies'));
});
test('ref validation rejects missing values and shell payloads; accepts a full SHA', () => {
  for (const [sha, valid] of [['', false], ['main', false], ['a'.repeat(40), true], ['$(echo injected)', false], ['a'.repeat(40) + '\nother', false]]) {
    const result = spawnSync('bash', ['-c', shellFor('Select Pages deployment ref')], {
      encoding: 'utf8', env: { ...process.env, PAGES_DEPLOY_SHA: sha, GITHUB_OUTPUT: '/dev/null' },
    });
    assert.equal(result.status === 0, valid, sha);
  }
});
test('a changed branch or arbitrary input fails before build; matching release commit passes', () => {
  const script = 'git() { printf "%s\\n" "$CHECKOUT_SHA"; }\n' + shellFor('Verify trusted release branch matches the requested artifact commit');
  for (const [requested, expected] of [['a'.repeat(40), 0], ['b'.repeat(40), 1]]) {
    const result = spawnSync('bash', ['-c', script], {
      encoding: 'utf8', env: { ...process.env, CHECKOUT_SHA: 'a'.repeat(40), PAGES_DEPLOY_SHA: requested },
    });
    assert.equal(result.status, expected, result.stdout + result.stderr);
  }
});
