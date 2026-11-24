import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {mkdirSync, writeFileSync} from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import {lockFixture} from './dependency-lock-fixture.mjs';

const literalOwners = ['', 'nested', 'with space', 'line\nbreak', 'tab\tλ', 'back\\slash', 'quote"directory', 'valid-\uFFFD', '\uFEFFvalid-bom'];
const toolArgs = {
  bun: ['install', '--lockfile-only'],
  npm: ['install', '--package-lock-only', '--ignore-scripts', '--no-audit', '--no-fund'],
};

test('actual JavaScript resolver preserves every literal tracked owning directory', {timeout: 15000}, () => {
  const fixture = lockFixture(literalOwners);
  try {
    const result = fixture.run('javascript');
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.calls.length, literalOwners.length * 2, JSON.stringify(result.calls));
    const expected = literalOwners.flatMap(owner => ['bun', 'npm'].map(tool =>
      JSON.stringify({tool, cwd: path.join(fixture.directory, owner), args: toolArgs[tool]}))).sort();
    assert.deepEqual(result.calls.map(call => JSON.stringify(call)).sort(), expected);
  } finally {fixture.close();}
});

for (const gitMode of ['failed', 'truncated', 'duplicate', 'unsafe']) {
  test('actual JavaScript resolver refuses ' + gitMode + ' Git inventory before effects', {timeout: 15000}, () => {
    const fixture = lockFixture();
    try {
      const result = fixture.run('javascript', {gitMode});
      assert.notEqual(result.status, 0, result.stderr);
      if (gitMode === 'failed') assert.equal(result.status, 23, result.stderr);
      assert.deepEqual(result.calls, []);
    } finally {fixture.close();}
  });
}

test('actual JavaScript resolver accepts an empty complete inventory without effects', {timeout: 15000}, () => {
  const fixture = lockFixture();
  try {
    const result = fixture.run('javascript', {gitMode: 'empty'});
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(result.calls, []);
  } finally {fixture.close();}
});

test('actual resolver preserves the first tool failure and stops before the next lock', {timeout: 15000}, () => {
  const fixture = lockFixture();
  try {
    const result = fixture.run('javascript', {failCall: 1});
    assert.equal(result.status, 17, result.stderr);
    assert.equal(result.calls.length, 1);
  } finally {fixture.close();}
});

test('actual Cargo resolver passes literal manifest paths as one argument each', {timeout: 15000}, () => {
  const fixture = lockFixture([], ['rust', 'space owner', 'line\nowner', 'tab\tλ']);
  try {
    const result = fixture.run('cargo');
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(result.calls, [...fixture.activeCargo].sort().map(manifest =>
      ({tool: 'cargo', cwd: fixture.directory, args: ['update', '--workspace', '--manifest-path', manifest]})));
  } finally {fixture.close();}
});

for (const kind of ['javascript', 'cargo']) {
  test('actual ' + kind + ' resolver refuses non-UTF-8 Git paths before any effects', {timeout: 15000}, () => {
    const fixture = lockFixture();
    try {
      const directory = Buffer.concat([Buffer.from(fixture.directory + '/'), Buffer.from([0xff])]);
      mkdirSync(directory);
      writeFileSync(Buffer.concat([directory, Buffer.from('/package-lock.json')]), '{}\n');
      execFileSync('git', ['add', '--all'], {cwd: fixture.directory, timeout: 3000});
      const result = fixture.run(kind);
      assert.notEqual(result.status, 0, result.stderr);
      assert.deepEqual(result.calls, [], JSON.stringify(result.calls));
      assert.match(result.stderr, /UTF-8/);
    } finally {fixture.close();}
  });
}
