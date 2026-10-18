import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, mkdirSync, readFileSync, readdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import YAML from 'yaml';
import { installHost } from '../../../js/agentic/host.mjs';
import { parseLino } from '../../../js/server/lino.mjs';
import { runtimeSteps } from '../../../js/agentic/crate/coding_program_contract.mjs';

const repository = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
before(() => installHost({ readText: (path) => readFileSync(join(repository, path), 'utf8'), parseLino, realm: {} }));

function setupWithFakeTools(configuration = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-fake-rust-setup-'));
  try {
    const binaryDirectory = join(directory, 'fake-bin');
    const cargoDirectory = join(directory, 'cargo with spaces');
    const temporaryDirectory = join(directory, 'temporary');
    for (const path of [binaryDirectory, join(cargoDirectory, 'bin'), temporaryDirectory]) mkdirSync(path, { recursive: true });
    const log = join(directory, 'tools.log');
    const githubPath = join(directory, 'github-path');
    writeFileSync(log, '');
    writeFileSync(githubPath, '');
    const rustupFixture = join(directory, 'fake-rustup.sh');
    writeFileSync(rustupFixture, [
      '#!/bin/sh',
      'printf \'rustup %s\n\' "$*" >> "$FORMAL_AI_TEST_TOOL_LOG"',
      'if [ "$1" = toolchain ]; then exit "$FORMAL_AI_TEST_TOOLCHAIN_EXIT"; fi',
      'exit 0',
      '',
    ].join('\n'), { mode: 0o700 });
    const installerFixture = join(directory, 'fake-installer.sh');
    writeFileSync(installerFixture, [
      '#!/bin/sh',
      'set -eu',
      'printf \'installer %s\n\' "$*" >> "$FORMAL_AI_TEST_TOOL_LOG"',
      'if [ "$FORMAL_AI_TEST_INSTALLER_EXIT" -ne 0 ]; then exit "$FORMAL_AI_TEST_INSTALLER_EXIT"; fi',
      'cp "$FORMAL_AI_TEST_RUSTUP_FIXTURE" "$CARGO_HOME/bin/rustup"',
      'chmod 700 "$CARGO_HOME/bin/rustup"',
      '',
    ].join('\n'), { mode: 0o700 });
    writeFileSync(join(binaryDirectory, 'curl'), [
      '#!/bin/sh',
      'set -eu',
      'printf \'curl %s\n\' "$*" >> "$FORMAL_AI_TEST_TOOL_LOG"',
      'if [ "$FORMAL_AI_TEST_DOWNLOAD_EXIT" -ne 0 ]; then exit "$FORMAL_AI_TEST_DOWNLOAD_EXIT"; fi',
      'destination=',
      'while [ "$#" -gt 0 ]; do',
      '  if [ "$1" = --output ]; then shift; destination=$1; fi',
      '  shift',
      'done',
      'test -n "$destination"',
      'cp "$FORMAL_AI_TEST_INSTALLER_FIXTURE" "$destination"',
      '',
    ].join('\n'), { mode: 0o700 });
    if (configuration.available) writeFileSync(join(cargoDirectory, 'bin/rustup'), readFileSync(rustupFixture), { mode: 0o700 });
    const environment = {
      PATH: (configuration.available ? join(cargoDirectory, 'bin') + ':' : '') + binaryDirectory + ':/usr/bin:/bin',
      CARGO_HOME: cargoDirectory, GITHUB_PATH: githubPath, TMPDIR: temporaryDirectory,
      FORMAL_AI_TEST_TOOL_LOG: log, FORMAL_AI_TEST_INSTALLER_FIXTURE: installerFixture,
      FORMAL_AI_TEST_RUSTUP_FIXTURE: rustupFixture,
      FORMAL_AI_TEST_DOWNLOAD_EXIT: String(configuration.downloadExit ?? 0),
      FORMAL_AI_TEST_INSTALLER_EXIT: String(configuration.installerExit ?? 0),
      FORMAL_AI_TEST_TOOLCHAIN_EXIT: String(configuration.toolchainExit ?? 0),
    };
    const probe = spawnSync('/bin/sh', ['-c', 'command -v rustup'], { env: environment, encoding: 'utf8' });
    if (configuration.available) assert.equal(probe.status, 0);
    else assert.notEqual(probe.status, 0);
    const fragment = runtimeSteps('rust');
    const document = YAML.parse('steps:\n' + fragment);
    assert.equal(document.steps.length, 1);
    assert.equal(document.steps[0].name, 'Set up Rust');
    const script = configuration.script ?? document.steps[0].run;
    const executed = spawnSync('/bin/sh', ['-eu', '-c', script], { env: environment, cwd: directory,
      encoding: 'utf8', timeout: 10000, maxBuffer: 65536 });
    assert.equal(executed.error, undefined);
    return { status: executed.status, stdout: executed.stdout, stderr: executed.stderr,
      commands: readFileSync(log, 'utf8').trim().split('\n').filter(Boolean),
      cargoPath: readFileSync(githubPath, 'utf8'), expectedCargoPath: join(cargoDirectory, 'bin') + '\n',
      temporaryFiles: readdirSync(temporaryDirectory) };
  } finally { rmSync(directory, { recursive: true, force: true }); }
}

test('cold CI bootstrap executes only fake setup tools and exports their actual Cargo path', () => {
  const result = setupWithFakeTools();
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(result.commands.map((line) => line.split(' ')[0]), ['curl', 'installer', 'rustup', 'rustup']);
  assert.match(result.commands[0], /--proto =https --tlsv1\.2 --fail --silent --show-error/u);
  assert.ok(result.commands[0].endsWith('https://sh.rustup.rs'));
  assert.equal(result.commands[1], 'installer --no-modify-path --profile minimal --default-toolchain none -y');
  assert.equal(result.commands[2], 'rustup toolchain install stable --profile minimal');
  assert.equal(result.commands[3], 'rustup default stable');
  assert.equal(result.cargoPath, result.expectedCargoPath);
  assert.deepEqual(result.temporaryFiles, []);
});

test('available rustup is reused without invoking the fake network or installer', () => {
  const result = setupWithFakeTools({ available: true });
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(result.commands, ['rustup toolchain install stable --profile minimal', 'rustup default stable']);
  assert.equal(result.cargoPath, result.expectedCargoPath);
  assert.deepEqual(result.temporaryFiles, []);
});

test('failed fake download propagates its status without executing an installer', () => {
  const result = setupWithFakeTools({ downloadExit: 23 });
  assert.equal(result.status, 23);
  assert.equal(result.commands.length, 1);
  assert.equal(result.cargoPath, '');
  assert.deepEqual(result.temporaryFiles, []);
});

test('failed fake installer never advertises an available toolchain', () => {
  const result = setupWithFakeTools({ installerExit: 31 });
  assert.equal(result.status, 31);
  assert.deepEqual(result.commands.map((line) => line.split(' ')[0]), ['curl', 'installer']);
  assert.equal(result.cargoPath, '');
  assert.deepEqual(result.temporaryFiles, []);
});

test('failed fake toolchain install cannot select a default or export a success path', () => {
  const result = setupWithFakeTools({ available: true, toolchainExit: 19 });
  assert.equal(result.status, 19);
  assert.deepEqual(result.commands, ['rustup toolchain install stable --profile minimal']);
  assert.equal(result.cargoPath, '');
});
