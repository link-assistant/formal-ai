#!/usr/bin/env node
// Print one shard of a test list, planned longest-first (PR #1188).
//
// CI shards the Rust suites over parallel jobs. Each job lists the whole
// suite, pipes the names through this script and runs only what it prints, so
// the shards are planned from recorded durations instead of by listed index:
//
//   dist/tests/unit --list --format terse | sed -n 's/: test$//p' \
//     | node scripts/plan-test-shards.mjs --shard 2 --of 4
//
// Input: one test per line, optionally `<group>\t<test>` (a test executable
// and a test in it); the duration is looked up by the test name. Output: the
// shard's lines, longest first. Every line of the input lands in exactly one
// shard, and unrecorded tests weigh `default-seconds` from the durations file.
//
// Options:
//   --shard <i> --of <n>   print shard i of n (1-based)
//   --reserve "1=330,2=80" seconds of other work a shard already carries
//   --durations <path>     default data/meta/test-durations.lino
//   --report               print each shard's predicted seconds and test count
//   --check                exit 1 unless the shards partition the input exactly

import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { readTestDurations } from './lib/ci-speed-durations.mjs';
import {
  DEFAULT_SECONDS, durationLookup, isLongestFirst, parseReserved, partitionProblems, planShards,
} from './lib/ci-speed-shards.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const argv = process.argv.slice(2);
const option = (flag, fallback) => {
  const index = argv.indexOf(flag);
  return index >= 0 ? argv[index + 1] : fallback;
};

function defaultSeconds(path) {
  try {
    const value = readFileSync(path, 'utf8').match(/^\s+default-seconds (\d+(?:\.\d+)?)$/m)?.[1];
    return value ? Number(value) : DEFAULT_SECONDS;
  } catch {
    return DEFAULT_SECONDS;
  }
}

function main() {
  const shardCount = Number(option('--of'));
  const durationsPath = option('--durations', join(ROOT, 'data/meta/test-durations.lino'));
  const items = readFileSync(0, 'utf8').split('\n').map((line) => line.trimEnd()).filter(Boolean);
  const selectedSeconds = durationLookup(readTestDurations(durationsPath), defaultSeconds(durationsPath));
  const nativeSeconds = durationLookup(readTestDurations(join(ROOT, 'data/meta/test-durations.lino')), defaultSeconds(durationsPath));
  const secondsOf = argv.includes('--native-floor')
    ? (item) => Math.max(selectedSeconds(item), nativeSeconds(item)) : selectedSeconds;
  const reserved = parseReserved(option('--reserve', ''), shardCount);
  const { shards, load } = planShards(items, shardCount, secondsOf, reserved);

  if (argv.includes('--check')) {
    const problems = partitionProblems(items, shards);
    const failures = Object.entries(problems).filter(([, list]) => list.length);
    if (!isLongestFirst(shards, secondsOf)) failures.push(['not longest first', ['a shard']]);
    for (const [kind, list] of failures) {
      console.error(`::error::shard plan ${kind}: ${list.slice(0, 5).join(', ')}${list.length > 5 ? ' ...' : ''}`);
    }
    if (failures.length) process.exit(1);
    console.log(`${items.length} test(s) in ${shardCount} shard(s), each exactly once, longest first`);
    return;
  }
  if (argv.includes('--report')) {
    shards.forEach((shard, index) => {
      console.log(`shard ${index + 1}/${shardCount}: ${Math.round(load[index])}s predicted, ${shard.length} test(s)`);
    });
    return;
  }
  const shard = Number(option('--shard'));
  if (!Number.isInteger(shard) || shard < 1 || shard > shardCount) {
    console.error(`--shard must be within 1..${shardCount}, got ${option('--shard')}`);
    process.exit(2);
  }
  process.stdout.write(shards[shard - 1].map((item) => `${item}\n`).join(''));
}

main();
