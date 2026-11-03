// Main-only execution of an already compiled, source-identified library test target.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync, spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const PREFIX = 'agentic_coding::progress::creation::tests::';
const FIXTURE = 'rust/tests/fixtures/scoped-creation-evidence.rs';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');

export function selectLibraryTestArtifact(messages, repository) {
  assert.equal(typeof messages, 'string');
  assert.ok(Buffer.byteLength(messages) <= 16 * 1024 * 1024, 'oversized Cargo build metadata');
  const rows = messages.split(/\r?\n/u).filter(line => line.trimStart().startsWith('{'))
    .map(line => JSON.parse(line));
  const finished = rows.filter(row => row.reason === 'build-finished');
  assert.equal(finished.length, 1, 'one actual Cargo completion required');
  assert.equal(finished[0].success, true, 'Cargo build did not succeed');
  assert.equal(rows.at(-1), finished[0], 'Cargo metadata after completion');
  const root = path.resolve(repository);
  const candidates = rows.filter(row => row.reason === 'compiler-artifact'
    && row.target?.name === 'formal_ai' && row.target?.src_path === path.join(root, 'rust/src/lib.rs')
    && Array.isArray(row.target.kind) && row.target.kind.length === 1 && row.target.kind[0] === 'lib'
    && row.profile?.test === true && typeof row.executable === 'string');
  assert.equal(candidates.length, 1, 'one source-identified library test executable required');
  const selected = candidates[0];
  assert.equal(selected.manifest_path, path.join(root, 'rust/Cargo.toml'));
  const executable = selected.executable;
  assert.ok(path.isAbsolute(executable) && path.resolve(executable) === executable, 'noncanonical executable');
  assert.equal(path.dirname(executable), path.join(root, 'rust/target/release/deps'), 'foreign executable root');
  assert.match(path.basename(executable), /^formal_ai-[a-f0-9]+$/u);
  return executable;
}

export function declaredCreationTestNames(source) {
  const names = [...source.matchAll(/^#\[test\]\s*\nfn ([A-Za-z_][A-Za-z0-9_]*)\(/gmu)]
    .map(match => PREFIX + match[1]);
  assert.ok(names.length > 0, 'declared creation fixture tests missing');
  assert.equal(new Set(names).size, names.length, 'duplicate declared creation test');
  return names.sort();
}

export function verifyCreationTestInventory(listing, expected) {
  const names = listing.split(/\r?\n/u).filter(line => line.endsWith(': test'))
    .map(line => line.slice(0, -6)).sort();
  assert.equal(new Set(names).size, names.length, 'duplicate actual test listing');
  assert.deepEqual(names, expected, 'compiled creation fixture inventory differs from source');
  return names;
}

export function runNativeCreationFixtures(messagesFile, reportFile, repository = process.cwd()) {
  const root = path.resolve(repository);
  const messages = fs.readFileSync(messagesFile, 'utf8');
  const executable = selectLibraryTestArtifact(messages, root);
  const status = fs.lstatSync(executable);
  assert.ok(status.isFile() && !status.isSymbolicLink() && status.size > 0, 'invalid library executable');
  const executableHash = digest(fs.readFileSync(executable));
  const fixture = path.join(root, FIXTURE);
  const source = fs.readFileSync(fixture);
  const expected = declaredCreationTestNames(source.toString('utf8'));
  const listing = execFileSync(executable, [PREFIX, '--list', '--format', 'terse'], {
    cwd: root, encoding: 'utf8', timeout: 10000, maxBuffer: 1024 * 1024
  });
  const names = verifyCreationTestInventory(listing, expected);
  console.log(JSON.stringify({schema: 'native-creation-fixture-selection/v1',
    executableSHA256: executableHash, fixtureSHA256: digest(source), testNames: names}));
  const started = Date.now();
  const execution = spawnSync(executable, [PREFIX, '--include-ignored', '--nocapture'], {
    cwd: root, timeout: 60000, stdio: 'inherit'
  });
  assert.equal(execution.error, undefined);
  assert.equal(execution.signal, null);
  assert.equal(execution.status, 0, 'native creation fixtures failed');
  assert.equal(digest(fs.readFileSync(executable)), executableHash, 'library executable changed');
  assert.equal(digest(fs.readFileSync(fixture)), digest(source), 'fixture source changed');
  const commit = execFileSync('git', ['rev-parse', 'HEAD'], {cwd: root, encoding: 'utf8', timeout: 5000, maxBuffer: 64 * 1024}).trim();
  const report = {
    schema: 'native-creation-fixture-qualification/v1', sourceCommit: commit,
    executableSHA256: executableHash, fixtureSHA256: digest(source), testNames: names,
    listingSHA256: digest(listing), status: execution.status, elapsedMs: Date.now() - started,
    scope: PREFIX, compiledAgain: false,
    remainingLibraryFixtures: 'not selected by this scoped qualification'
  };
  fs.writeFileSync(reportFile, JSON.stringify(report, null, 2) + '\n', {flag: 'wx'});
  console.log(JSON.stringify(report));
  return report;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [messagesFile, reportFile] = process.argv.slice(2);
  assert.ok(messagesFile && reportFile, 'Cargo metadata and receipt paths required');
  runNativeCreationFixtures(messagesFile, reportFile);
}
