import assert from 'node:assert/strict';
import fs from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { execFileSync } from 'node:child_process';
import { test } from 'node:test';
import { nativeInputIdentity } from '../../../scripts/native-release-source.mjs';
import { immutableSourceWitness, nativeDerivationRows, verifyPhysicalDerivationInputs } from '../../../scripts/lib/qualified-native-data-collector.mjs';
function fixture() {
  const directory = fs.mkdtempSync(join(tmpdir(), 'formal-ai-native-root-'));
  const env = {
    ...process.env,
    GIT_CONFIG_NOSYSTEM: '1',
    GIT_CONFIG_GLOBAL: '/dev/null',
    GIT_TERMINAL_PROMPT: '0'
  };
  const git = (...args) => execFileSync('git', args, {
    cwd: directory,
    env,
    encoding: 'utf8',
    timeout: 5000
  }).trim();
  const files = {
    'rust/Cargo.toml': '[package]\nname="heldout"\nversion="0.1.0"\n',
    'rust/Cargo.lock': '# held-out lock data\n',
    'rust/src/lib.rs': 'pub fn heldout() {}\n',
    'rust/embedded/heldout.lino': 'heldout compilation data\n',
    'rust/build.rs': 'fn main() {}\n',
    'data/seed/api-cache/heldout.lino': 'heldout cached data\n',
    '.cargo/config.toml': '[build]\njobs=1\n',
    'rust/.cargo/config.toml': '[net]\nretry=1\n',
    'data/meta/source-model/heldout.grammar': 'heldout CST grammar\n',
    'scripts/aggregate-source-content.mjs': 'heldout aggregation source\n',
    'scripts/mirror-package-data.mjs': 'heldout mirror source\n',
    'js/crypto/blake3.mjs': 'heldout digest helper\n',
    'js/agentic/crate/rust_str.mjs': 'heldout string helper\n',
    'other/unrelated.txt': 'whole-source witness only\n'
  };
  for (const [name, content] of Object.entries(files)) {
    fs.mkdirSync(join(directory, name, '..'), {
      recursive: true
    });
    fs.writeFileSync(join(directory, name), content);
  }
  git('init', '-q');
  git('config', 'user.name', 'Source fixture');
  git('config', 'user.email', 'fixture@example.invalid');
  git('config', 'commit.gpgSign', 'false');
  const hooks = join(directory, 'empty-hooks');
  fs.mkdirSync(hooks);
  git('config', 'core.hooksPath', hooks);
  git('add', '-A');
  git('commit', '-qm', 'held-out source');
  return {
    directory,
    head: git('rev-parse', 'HEAD'),
    close: () => fs.rmSync(directory, {
      recursive: true,
      force: true
    })
  };
}
test('independent immutable native witness agrees with actual compiler-input producer without CST-only helpers', async () => {
  const item = fixture();
  try {
    const witness = await immutableSourceWitness(item.directory, item.head);
    assert.deepEqual(witness.native, nativeInputIdentity(item.directory));
    assert(witness.whole.files > witness.native.files);
    fs.writeFileSync(join(item.directory, 'scripts/aggregate-source-content.mjs'), 'changed CST-only helper\n');
    assert.deepEqual(nativeInputIdentity(item.directory), witness.native, 'CST helper edits do not change compilation identity');
    assert.throws(() => verifyPhysicalDerivationInputs(item.directory, nativeDerivationRows(witness.rows, [])), /physical derivation source drift/u);
  } finally {
    item.close();
  }
});
test('expanded CST derivation custody still checks every helper and native bytes', async () => {
  const item = fixture();
  try {
    const witness = await immutableSourceWitness(item.directory, item.head),
      rows = nativeDerivationRows(witness.rows, []);
    for (const file of ['data/meta/source-model/heldout.grammar', 'scripts/aggregate-source-content.mjs', 'scripts/mirror-package-data.mjs', 'js/crypto/blake3.mjs', 'js/agentic/crate/rust_str.mjs', 'rust/src/lib.rs']) assert(rows.some(row => row.path === file));
    assert(!rows.some(row => row.path === 'other/unrelated.txt'));
    assert.equal(verifyPhysicalDerivationInputs(item.directory, rows).length, rows.length);
    fs.writeFileSync(join(item.directory, 'rust/src/lib.rs'), 'changed actual compilation input\n');
    assert.throws(() => nativeInputIdentity(item.directory), /uncommitted native compilation input/u);
    assert.throws(() => verifyPhysicalDerivationInputs(item.directory, rows), /physical derivation source drift/u);
    assert.throws(() => nativeDerivationRows(witness.rows, ['missing/producer.rs']), /declared derivation source missing/u);
  } finally {
    item.close();
  }
});
