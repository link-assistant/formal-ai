import { strict as assert } from 'node:assert';
import { spawnSync } from 'node:child_process';
import { chmodSync, copyFileSync, mkdirSync, mkdtempSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const script = join(root, 'scripts/check-changelog-fragment.mjs');
const { isSourceFile, parseGitPaths } = await import(pathToFileURL(script).href);
const fixture = resolve(dirname(fileURLToPath(import.meta.url)), '../fixtures/changelog-fragment/git-output.mjs');
const quote = (value) => `'${value.replaceAll("'", "'\\''")}'`;

function observe(mode, installGit = true) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-changelog-output-'));
  try {
    if (installGit) {
      const fixturePath = join(directory, 'fixture.mjs');
      copyFileSync(fixture, fixturePath);
      const git = join(directory, 'git');
      writeFileSync(git, `#!/bin/sh\nexec ${quote(process.execPath)} ${quote(fixturePath)} "$@"\n`);
      chmodSync(git, 0o755);
    }
    return spawnSync(process.execPath, [script], {
      cwd: directory,
      env: { ...process.env, PATH: directory, RUST_ROOT: 'rust', GITHUB_BASE_REF: 'main', CHANGELOG_GIT_FIXTURE_MODE: mode },
      encoding: 'utf8', maxBuffer: 32 * 1024 * 1024,
    });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test('a Git diff larger than one MiB retains every path and its source/fragment decision', () => {
  const result = observe('large');
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  assert.ok(Buffer.byteLength(result.stdout) > 1024 * 1024);
  for (let index = 0; index < 11000; index += 1) {
    assert.ok(result.stdout.includes(`  data/evidence/${String(index).padStart(5, '0')}/${'x'.repeat(100)}.lino\n`));
  }
  assert.match(result.stdout, /Source files changed: 1\n  rust\/src\/fixture\.rs\n/);
  assert.match(result.stdout, /Changelog fragments added: 1\n  changelog\.d\/fixture\.md\n/);
  assert.match(result.stdout, /Changelog check passed \(source files changed: 1, fragments added: 1\)/);
  assert.doesNotMatch(result.stdout, /No changed files found/);
});

test('a failed Git command fails closed rather than claiming an empty diff', () => {
  const result = observe('failed');
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Error executing git/);
  assert.match(result.stderr, /fixture git failure/);
  assert.doesNotMatch(result.stdout, /No changed files found|Changelog check passed/);
});

test('Git output beyond the explicit bound fails closed', () => {
  const result = observe('overflow');
  assert.equal(result.status, 1);
  assert.match(result.stderr, /output exceeded 16777216 bytes/);
  assert.doesNotMatch(result.stdout, /No changed files found|Changelog check passed/);
});

test('an unavailable Git executable fails closed', () => {
  const result = observe('empty', false);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Failed to execute git/);
  assert.doesNotMatch(result.stdout, /No changed files found|Changelog check passed/);
});

test('a genuinely empty successful Git diff remains successful', () => {
  const result = observe('empty');
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /No changed files found/);
});


// Real Git, finite local objects and file transport only: no network credentials.
function promisorCheckout(context, fragment) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-changelog-promisor-'));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  const full = join(directory, 'full'), partial = join(directory, 'partial');
  const configuration = join(directory, 'gitconfig');
  writeFileSync(configuration, ''); mkdirSync(full);
  const env = { ...process.env, GIT_CONFIG_NOSYSTEM:'1', GIT_CONFIG_GLOBAL:configuration,
    GIT_CONFIG_COUNT:'0', GIT_ALLOW_PROTOCOL:'file', GIT_TERMINAL_PROMPT:'0',
    RUST_ROOT:'.', GITHUB_BASE_REF:'main' };
  const run = (cwd, args) => spawnSync('git', args, {
    cwd, env, encoding:'utf8', timeout:10000, maxBuffer:1024 * 1024,
  });
  const git = (cwd, args) => {
    const result = run(cwd, args);
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.trim();
  };
  git(full, ['init', '--initial-branch=main']);
  git(full, ['config', 'user.name', 'Changelog fixture']);
  git(full, ['config', 'user.email', 'changelog@example.invalid']);
  git(full, ['config', 'uploadpack.allowFilter', 'true']);
  mkdirSync(join(full, 'src'));
  const body = Array.from({length:100}, (_, i) => `let value_${i} = ${i};`).join('\n') + '\n';
  writeFileSync(join(full, 'src/before.rs'), body);
  git(full, ['add', '.']); git(full, ['commit', '-m', 'base source']);
  git(full, ['switch', '-c', 'feature']);
  git(full, ['mv', 'src/before.rs', 'src/after.rs']);
  writeFileSync(join(full, 'src/after.rs'), body + '// Changed during the rename.\n');
  if (fragment) {
    mkdirSync(join(full, 'changelog.d'));
    writeFileSync(join(full, 'changelog.d/fixture.md'), '### Changed\n\n- Rename fixture.\n');
  }
  git(full, ['add', '.']); git(full, ['commit', '-m', 'rename with a content change']);
  git(directory, ['clone', '--filter=blob:none', '--no-checkout', pathToFileURL(full).href, partial]);
  assert.equal(git(partial, ['config', '--get', 'remote.origin.promisor']), 'true');
  assert.equal(git(partial, ['rev-parse', '--is-shallow-repository']), 'false');
  git(partial, ['config', 'diff.renames', 'true']);
  git(partial, ['remote', 'set-url', 'origin', join(directory, 'unavailable-origin')]);
  assert.match(git(partial, ['rev-list', '--objects', '--all', '--missing=print']), /^\?[0-9a-f]+$/mu);
  // Negative control: similarity detection needs the intentionally absent bodies.
  const legacy = run(partial, ['diff', '--name-only', 'origin/main...HEAD']);
  assert.notEqual(legacy.status, 0);
  assert.match(legacy.stderr, /could not fetch|promisor remote/u);
  const exact = run(partial, ['diff', '--find-renames=100%', '--diff-filter=A', '--name-only', '-z', 'origin/main...HEAD']);
  assert.equal(exact.status, 0, exact.stderr);
  return spawnSync(process.execPath, [script], {
    cwd:partial, env, encoding:'utf8', timeout:10000, maxBuffer:1024 * 1024,
  });
}

test('a promisor checkout lists both rename paths without fetching unavailable blobs', {timeout:30000}, context => {
  const result = promisorCheckout(context, true);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Source files changed: 2\n  src\/after\.rs\n  src\/before\.rs\n/u);
  assert.match(result.stdout, /Changelog fragments added: 1\n  changelog\.d\/fixture\.md\n/u);
  assert.match(result.stdout, /Changelog check passed/u);
  assert.doesNotMatch(result.stderr, /could not fetch|promisor remote/u);
});

test('source renames in a promisor checkout still fail without a changelog fragment', {timeout:30000}, context => {
  const result = promisorCheckout(context, false);
  assert.equal(result.status, 1);
  assert.match(result.stdout, /Source files changed: 2\n/u);
  assert.match(result.stderr, /No changelog fragment found in this PR/u);
  assert.doesNotMatch(result.stderr, /could not fetch|promisor remote/u);
});


test('source classification supports repository-root and nested Rust layouts', () => {
  for (const [rustRoot, paths] of [
    ['.', ['src/lib.rs', 'tests/fixture.rs', 'scripts/check.mjs', 'Cargo.toml']],
    ['rust', ['rust/src/lib.rs', 'rust/tests/fixture.rs', 'rust/scripts/check.rs', 'scripts/check.mjs', 'rust/Cargo.toml']],
  ]) {
    for (const path of paths) assert.equal(isSourceFile(path, rustRoot), true, path);
    for (const path of ['changelog.d/fixture.md', 'docs/guide.md', 'rustscripts/check.rs', 'other/src/lib.rs']) {
      assert.equal(isSourceFile(path, rustRoot), false, path);
    }
  }
});


// Real commits distinguish additions from edits, deletions and byte-identical moves.
for (const rustRoot of ['.', 'rust']) {
  for (const mode of ['edit', 'delete', 'move', 'replace', 'add', 'nested', 'newline-source', 'newline-fragment']) {
    test(`only new fragments satisfy source changes: ${rustRoot} / ${mode}`, {timeout:30000}, context => {
      const directory = mkdtempSync(join(tmpdir(), 'formal-ai-fragment-addition-'));
      context.after(() => rmSync(directory, {recursive:true, force:true}));
      const env = {...process.env, GIT_CONFIG_NOSYSTEM:'1', GIT_CONFIG_GLOBAL:'/dev/null',
        GIT_CONFIG_COUNT:'0', GIT_TERMINAL_PROMPT:'0', RUST_ROOT:rustRoot, GITHUB_BASE_REF:'main'};
      const git = args => {
        const result = spawnSync('git', args, {cwd:directory, env, encoding:'utf8', timeout:10000, maxBuffer:1024*1024});
        assert.equal(result.status, 0, result.stderr);
      };
      const sourceDirectory = rustRoot === '.' ? 'src' : 'rust/src';
      mkdirSync(join(directory, sourceDirectory), {recursive:true});
      mkdirSync(join(directory, 'changelog.d'));
      const old = join(directory, 'changelog.d/old.md');
      const body = '### Changed\n\n' + '- An existing change.\n'.repeat(40);
      writeFileSync(old, body);
      writeFileSync(join(directory, sourceDirectory, 'fixture.rs'), '// before\n');
      git(['init','--initial-branch=main']);
      git(['config','user.name','Fragment fixture']); git(['config','user.email','fragment@example.invalid']);
      git(['add','.']); git(['commit','-m','existing pending fragment']);
      git(['update-ref','refs/remotes/origin/main','HEAD']);
      git(['switch','-c','feature']);
      writeFileSync(join(directory, sourceDirectory, 'fixture.rs'), '// after\n');
      if (mode === 'edit') writeFileSync(old, body + '- An edit.\n');
      if (mode === 'delete') rmSync(old);
      if (mode === 'move' || mode === 'replace') renameSync(old, join(directory,'changelog.d/new.md'));
      if (mode === 'replace') writeFileSync(join(directory,'changelog.d/new.md'), body + '- A new change.\n');
      if (mode === 'add') writeFileSync(join(directory,'changelog.d/new.md'), '### Fixed\n\n- New entry.\n');
      if (mode === 'nested') {
        mkdirSync(join(directory,'changelog.d/archive'));
        writeFileSync(join(directory,'changelog.d/archive/new.md'), '### Fixed\n\n- Nested entry.\n');
      }
      if (mode === 'newline-source') writeFileSync(join(directory, sourceDirectory, 'line\nbreak.rs'), '// new\n');
      if (mode === 'newline-fragment') writeFileSync(join(directory,'changelog.d/line\nbreak.md'), '### Fixed\n\n- New entry.\n');
      git(['add','.']); git(['commit','-m',mode]);
      const result = spawnSync(process.execPath, [script], {cwd:directory, env, encoding:'utf8', timeout:10000, maxBuffer:1024*1024});
      const accepted = ['replace','add','newline-fragment'].includes(mode);
      assert.equal(result.status, accepted ? 0 : 1, result.stdout + result.stderr);
      assert.match(result.stdout, new RegExp(`Changelog fragments added: ${accepted ? 1 : 0}\\n`, 'u'));
      assert.match(result.stdout, new RegExp(`Source files changed: ${mode === 'newline-source' ? 2 : 1}\\n`, 'u'));
    });
  }
}


test('NUL inventories retain whitespace, newlines and a leading BOM in paths',()=>{
  const paths=[' leading path ', 'src/line\nbreak.rs', '\ufeffleading-bom'];
  assert.deepEqual(parseGitPaths(Buffer.from(paths.join('\0')+'\0')),paths);
  assert.deepEqual(parseGitPaths(Buffer.alloc(0)),[]);
});
for(const mode of ['invalid-utf8','unterminated','empty-record']) {
  test(`malformed NUL inventory fails closed: ${mode}`,()=>{
    const result=observe(mode);
    assert.equal(result.status,1);
    assert.match(result.stderr,/Invalid Git path inventory/u);
    assert.doesNotMatch(result.stdout,/No changed files found|Changelog check passed/u);
  });
}
