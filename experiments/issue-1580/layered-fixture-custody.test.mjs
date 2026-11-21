// Check acquisition using Git's real sparse rules before reading local fixtures.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join, relative, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import YAML from 'yaml';
import {loadRenameMap} from '../formal_ai_subagent/rename-by-rule.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
function admitted(paths) {
  const workflow = YAML.parse(readFileSync(join(root, '.github/workflows/layered-ci.yml'), 'utf8'));
  const rules = workflow.jobs.js.steps.find(step => step.with?.['sparse-checkout']).with['sparse-checkout'];
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-layered-fixtures-'));
  try {
    const file = join(directory, 'rules');
    writeFileSync(file, rules);
    const result = spawnSync('git', ['sparse-checkout', 'check-rules', '--no-cone', '--rules-file', file, '-z'], {
      cwd: root, input: paths.join('\0') + '\0', encoding: 'utf8', timeout: 5000, maxBuffer: 1024 * 1024,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.split('\0').filter(Boolean);
  } finally { rmSync(directory, {recursive: true, force: true}); }
}

test('Layered checkout acquires every moved unit fixture and the consumed authorship transcript', {timeout: 10000}, () => {
  const fixtures = new Set(['docs/case-studies/issue-703/self-hosting-authorship/agent-cli.log']);
  const moves = loadRenameMap(root).renames.filter(move => move.tree === 'unit-tests-by-area');
  assert(moves.length > 0);
  for (const move of moves) {
    const source = readFileSync(join(root, move.to), 'utf8');
    for (const match of source.matchAll(/(?:include_str!|include_bytes!)\s*\(\s*"([^"\n]+)"/gu)) {
      const fixture = relative(root, resolve(root, dirname(move.to), match[1]));
      assert(!fixture.startsWith('..'), move.to + ': ' + fixture);
      assert(existsSync(join(root, fixture)), fixture);
      fixtures.add(fixture);
    }
  }
  const paths = [...fixtures].sort();
  assert(paths.includes('docs/case-studies/issue-859/raw-data/agent-cli-changelog-session.log'));
  assert.deepEqual(admitted(paths), paths, 'hosted acquisition must preserve every consumed fixture');
});

test('Layered checkout still excludes unrelated historical archives', {timeout: 10000}, () => {
  assert.deepEqual(admitted(['docs/case-studies/unrelated/large.log', 'docs/case-studies/unrelated/large.gz',
    'experiments/unrelated/evidence/large.gz']), []);
});
