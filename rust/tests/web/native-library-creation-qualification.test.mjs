// Pure source/metadata checks only: no Cargo, native executable or product run.
import assert from 'node:assert/strict';
import {test} from 'node:test';
import fs from 'node:fs';
import {selectLibraryTestArtifact, declaredCreationTestNames, verifyCreationTestInventory}
  from '../../../scripts/run-native-coding-fixtures.mjs';

const root = '/private/tmp/source-bound-cargo-fixture';
const executable = root + '/rust/target/release/deps/formal_ai-0123abcd';
function rows() {
  return [{reason: 'compiler-artifact', manifest_path: root + '/rust/Cargo.toml',
    target: {name: 'formal_ai', kind: ['lib'], src_path: root + '/rust/src/lib.rs'},
    profile: {test: true}, executable}, {reason: 'build-finished', success: true}];
}
const encoded = value => value.map(row => JSON.stringify(row)).join('\n') + '\n';

test('selects the compiled library test instead of a same-named binary target', () => {
  const messages = rows();
  messages.unshift({...messages[0], target: {...messages[0].target, kind: ['bin'], src_path: root + '/rust/src/main.rs'},
    executable: root + '/rust/target/release/deps/formal_ai-deadbeef'});
  assert.equal(selectLibraryTestArtifact(encoded(messages), root), executable);
  assert.equal(selectLibraryTestArtifact('macro note\n' + encoded(messages) + 'finished note\n', root), executable);
});

test('refuses failed, absent, ambiguous, foreign and incomplete Cargo identities', () => {
  const variants = [
    value => {value[1].success = false;}, value => value.pop(),
    value => value.unshift(structuredClone(value[0])),
    value => {value[0].target.src_path = '/foreign/rust/src/lib.rs';},
    value => {value[0].profile.test = false;}, value => {value[0].target.kind = ['bin'];},
    value => {value[0].manifest_path = '/foreign/rust/Cargo.toml';},
    value => {value[0].executable = '/foreign/formal_ai-0123abcd';},
    value => {value[0].executable = root + '/rust/target/release/deps/../formal_ai-0123abcd';},
    value => value.push({reason: 'compiler-artifact'})
  ];
  for (const mutate of variants) {
    const messages = rows(); mutate(messages);
    assert.throws(() => selectLibraryTestArtifact(encoded(messages), root));
  }
  assert.throws(() => selectLibraryTestArtifact('{incomplete', root));
});

test('declared source tests must equal the compiled inventory without missing or duplicate names', () => {
  const names = declaredCreationTestNames('#[test]\nfn first() {}\n#[test]\nfn held_out() {}\n');
  const listing = [...names].reverse().map(name => name + ': test').join('\n') + '\n';
  assert.deepEqual(verifyCreationTestInventory(listing, names), names);
  assert.throws(() => verifyCreationTestInventory(names[0] + ': test\n', names));
  assert.throws(() => verifyCreationTestInventory(listing + names[0] + ': test\n', names));
  assert.throws(() => declaredCreationTestNames('no registered tests'));
  const fixture = fs.readFileSync('rust/tests/fixtures/scoped-creation-evidence.rs', 'utf8');
  const actual = declaredCreationTestNames(fixture);
  assert(actual.some(name => name.endsWith('host_workspace_absolute_receipts_preserve_wire_arguments_and_prove_logical_creation')));
  assert(actual.some(name => name.endsWith('host_workspace_binding_never_substitutes_for_foreign_pending_or_missing_read_evidence')));
});

test('main producer reuses its sole compile and executes scoped library fixtures before uploads', () => {
  const workflow = fs.readFileSync('.github/workflows/release.yml', 'utf8');
  // Inspect the declared producer block through the following job boundary.
  const producer = workflow.slice(workflow.indexOf('\n  build-artifacts:'));
  assert.equal((producer.match(/cargo test --manifest-path rust\/Cargo.toml --release --no-run --bins --tests --all-features/gu) ?? []).length, 1);
  assert(producer.includes('--message-format=json'));
  assert(producer.includes('scripts/run-native-coding-fixtures.mjs'));
  assert(producer.indexOf('scripts/run-native-coding-fixtures.mjs') < producer.indexOf('name: Upload test executables'));
  assert(workflow.includes("github.ref == 'refs/heads/main'"));
});
