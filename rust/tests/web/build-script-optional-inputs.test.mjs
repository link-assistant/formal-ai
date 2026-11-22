// Verify that local replay files cannot affect the native package source.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {nativeInputIdentity} from '../../../scripts/native-release-source.mjs';

const source = readFileSync(new URL('../../../rust/build.rs', import.meta.url), 'utf8');

function fixture(context) {
  const root = mkdtempSync(join(tmpdir(), 'optional-build-input-'));
  context.after(() => rmSync(root, {recursive: true, force: true}));
  const git = (...args) => execFileSync('git', args, {cwd: root, encoding: 'utf8', stdio: 'pipe'}).trim();
  git('init', '--initial-branch=main');
  git('config', 'user.name', 'Fixture');
  git('config', 'user.email', 'fixture@example.invalid');
  git('config', 'core.hooksPath', '/dev/null');
  mkdirSync(join(root, 'rust'), {recursive: true});
  writeFileSync(join(root, 'rust/Cargo.toml'), '[package]\nname="fixture"\nversion="1.0.0"\n');
  writeFileSync(join(root, 'rust/Cargo.lock'), 'version = 4\n');
  writeFileSync(join(root, 'rust/build.rs'), source);
  git('add', '.');
  git('commit', '-m', 'source');
  return {root, git};
}

test('local API response fixtures cannot become inputs to a production binary', context => {
  const {root, git} = fixture(context);
  const absent = nativeInputIdentity(root);
  const directory = join(root, 'data/seed/api-cache');
  mkdirSync(directory, {recursive: true});
  const file = join(directory, 'legacy.lino');
  writeFileSync(file, 'fixture response\n');
  assert.deepEqual(nativeInputIdentity(root), absent, 'an untracked fixture is outside the build');
  git('add', 'data');
  git('commit', '-m', 'record replay fixture');
  assert.deepEqual(nativeInputIdentity(root), absent, 'a tracked fixture is outside the build');
  writeFileSync(file, 'modified fixture response\n');
  assert.deepEqual(nativeInputIdentity(root), absent, 'local changes cannot alter a binary');
  assert.doesNotMatch(source, /api-cache|SEED_BUNDLE_FILES/u);
});
