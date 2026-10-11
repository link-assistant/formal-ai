// Actual Git source-input lifecycle plus build-script authority contracts; no Rust compilation.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync, symlinkSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {nativeInputIdentity} from '../../../scripts/native-release-source.mjs';

const source = readFileSync(new URL('../../../rust/build.rs', import.meta.url), 'utf8');

test('optional build watch never emits the missing legacy path and stays inside source authority', () => {
  const main = source.slice(source.indexOf('fn main()'), source.indexOf('fn optional_input_watch_path'));
  assert.match(main, /optional_input_watch_path\(&seed_dir, repository_root\)/u);
  assert.match(main, /rerun-if-changed=\{\}.*seed_watch\.display\(\)/u);
  assert.doesNotMatch(main, /rerun-if-changed=\{\}.*seed_dir\.display\(\)/u);
  const helper = source.slice(source.indexOf('fn optional_input_watch_path'), source.indexOf('fn emit_crate_edition'));
  assert.match(helper, /candidate\.starts_with\(source_root\)/u);
  assert.match(helper, /fs::symlink_metadata\(candidate\)/u);
  assert.match(helper, /ErrorKind::NotFound => continue/u);
  assert.match(helper, /canonical\.starts_with\(&canonical_root\)/u);
  assert.match(helper, /read_dir\(candidate\)\.expect/u);
  assert.match(helper, /Err\(error\) => panic!/u);
  assert.match(helper, /return candidate\.to_path_buf\(\)/u);
  assert.doesNotMatch(helper, /create_dir|unwrap_or_default|filter_map/u);
  assert.match(main, /enumerate_lino_files\(&seed_dir\)/u);
});

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

test('actual optional legacy creation, edits and removal change source identity without inventing a bundle', context => {
  const {root, git} = fixture(context);
  const absent = nativeInputIdentity(root);
  const directory = join(root, 'data/seed/api-cache');
  mkdirSync(directory, {recursive: true});
  assert.deepEqual(nativeInputIdentity(root), absent, 'empty optional directory contains no legacy bundle');
  const file = join(directory, 'legacy.lino');
  writeFileSync(file, 'legacy original\n');
  assert.throws(() => nativeInputIdentity(root), /untracked native compilation input/u);
  git('add', 'data');
  git('commit', '-m', 'legacy creation');
  const present = nativeInputIdentity(root);
  assert.notEqual(present.sha256, absent.sha256);
  writeFileSync(file, 'legacy edited\n');
  assert.throws(() => nativeInputIdentity(root), /uncommitted native compilation input/u);
  git('add', 'data');
  git('commit', '-m', 'legacy edit');
  assert.notEqual(nativeInputIdentity(root).sha256, present.sha256);
  rmSync(directory, {recursive: true});
  git('add', '-u');
  git('commit', '-m', 'legacy deletion');
  assert.deepEqual(nativeInputIdentity(root), absent);
});

test('an actual external legacy payload cannot silently join the selected native source identity', context => {
  const {root} = fixture(context);
  const outside = mkdtempSync(join(tmpdir(), 'external-build-input-'));
  context.after(() => rmSync(outside, {recursive: true, force: true}));
  writeFileSync(join(outside, 'foreign.lino'), 'unowned legacy payload\n');
  mkdirSync(join(root, 'data/seed'), {recursive: true});
  symlinkSync(outside, join(root, 'data/seed/api-cache'), 'dir');
  assert.throws(() => nativeInputIdentity(root), /untracked native compilation input/u);
});
