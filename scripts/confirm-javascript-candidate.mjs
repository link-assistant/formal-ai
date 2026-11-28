// Called only after the same job's JavaScript checks succeed.
// Mirrors the Rust template's exact-candidate JavaScript prerequisite.
import assert from 'node:assert/strict';
import {appendFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';

const expected = process.env.GITHUB_SHA;
assert.match(expected ?? '', /^[a-f0-9]{40}$/u, 'exact event commit required');
assert(process.env.GITHUB_OUTPUT, 'job output path required');
const git = args => execFileSync('git', args, {encoding:'utf8', maxBuffer:4*1024*1024});
assert.equal(git(['rev-parse', 'HEAD']).trim(), expected, 'JavaScript checked another candidate');
assert.equal(git(['status', '--porcelain=v1', '--untracked-files=all']), '',
  'JavaScript checks must leave the candidate tree clean');
appendFileSync(process.env.GITHUB_OUTPUT, 'tested-sha=' + expected + '\n');
console.log('JAVASCRIPT_CANDIDATE_CONFIRMED ' + expected);
