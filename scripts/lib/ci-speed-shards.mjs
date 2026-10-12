// Split tests across parallel shards longest-first (PR #1188, issue #1047).
//
// Splitting by listed index puts tests on machines regardless of how long
// they take, so one shard collects the slow ones and the run waits on it while
// the others idle (run 37753751698: specification shards of 236s, 641s and
// 378s). Longest-processing-time-first fixes that: sort the tests longest
// first and give each to the shard with the least work so far. Graham's bound
// puts the slowest shard within 4/3 of the optimum.
//
// The plan is a pure function of the test list and the recorded durations, so
// every shard computes the same plan independently and runs only its own part:
// each test lands in exactly one shard, none is dropped and none runs twice.

/** The seconds a test is assumed to take when no duration is recorded. */
export const DEFAULT_SECONDS = 0.1;

/** The test name of a planned item: the text after an optional `<group>\t`. */
export const testName = (item) => item.slice(item.indexOf('\t') + 1);

/**
 * Assign every item to one of `shardCount` shards, longest first.
 *
 * `secondsOf(item)` is the item's expected duration; `reserved[i]` is work
 * shard i already carries outside the plan (a gate step only it runs). Ties
 * are broken by name and by the lower shard index, so the plan is identical on
 * every machine. Each returned shard lists its items longest first.
 */
export function planShards(items, shardCount, secondsOf, reserved = []) {
  if (!Number.isInteger(shardCount) || shardCount < 1) {
    throw new Error(`a plan needs at least one shard, got ${shardCount}`);
  }
  const ordered = [...new Set(items)]
    .map((item) => ({ item, seconds: secondsOf(item) }))
    .sort((left, right) => right.seconds - left.seconds || (left.item < right.item ? -1 : left.item > right.item ? 1 : 0));
  const load = Array.from({ length: shardCount }, (_, index) => reserved[index] ?? 0);
  const shards = Array.from({ length: shardCount }, () => []);
  for (const { item, seconds } of ordered) {
    let lightest = 0;
    for (let index = 1; index < shardCount; index += 1) {
      if (load[index] < load[lightest]) lightest = index;
    }
    load[lightest] += seconds;
    shards[lightest].push(item);
  }
  return { shards, load };
}

/** A lookup of recorded seconds with a default for unrecorded tests. */
export function durationLookup(recorded, fallback = DEFAULT_SECONDS) {
  return (item) => recorded.get(testName(item)) ?? fallback;
}

/**
 * What a partition loses or repeats: every item must be in exactly one shard.
 * Returns `{ missing, repeated, extra }`, all empty for a sound partition.
 */
export function partitionProblems(items, shards) {
  const counts = new Map();
  for (const shard of shards) {
    for (const item of shard) counts.set(item, (counts.get(item) ?? 0) + 1);
  }
  const wanted = new Set(items);
  return {
    missing: [...wanted].filter((item) => !counts.has(item)),
    repeated: [...counts].filter(([, count]) => count > 1).map(([item]) => item),
    extra: [...counts.keys()].filter((item) => !wanted.has(item)),
  };
}

/** Whether every shard runs its items longest first. */
export function isLongestFirst(shards, secondsOf) {
  return shards.every((shard) => shard.every((item, index) => index === 0 || secondsOf(shard[index - 1]) >= secondsOf(item)));
}

/** Parse `1=330,2=80` into per-shard reserved seconds (index 0 is shard 1). */
export function parseReserved(text, shardCount) {
  const reserved = Array.from({ length: shardCount }, () => 0);
  for (const part of (text ?? '').split(/[,\s]+/).filter(Boolean)) {
    const match = part.match(/^(\d+)=(\d+(?:\.\d+)?)$/);
    if (!match || Number(match[1]) < 1 || Number(match[1]) > shardCount) {
      throw new Error(`reserved work must read <shard>=<seconds> within 1..${shardCount}, got ${part}`);
    }
    reserved[Number(match[1]) - 1] += Number(match[2]);
  }
  return reserved;
}
