// The CI speed rule, read from `data/meta/ci-speed.lino` (PR #1188).
//
// The rule: no CI job or step runs longer than `limit-minutes` (30; 15 is the
// target), the long work starts first, and both hold automatically. Jobs that
// cannot meet the limit yet are listed one by one with the cap they may keep
// and the reason; the list only shrinks, because an entry whose job now meets
// the limit fails the gate until it is removed.

/** `key value` of one line, the value unquoted. */
function pair(line) {
  const match = line.trim().match(/^([A-Za-z0-9_-]+)(?:\s+(.*))?$/);
  if (!match) return null;
  const value = match[2] ?? '';
  const quoted = value.match(/^"((?:[^"\\]|\\.)*)"$/);
  return { key: match[1], value: quoted ? quoted[1].replace(/\\"/g, '"') : value };
}

/**
 * The policy as `{ limitMinutes, targetMinutes, longJobMinutes, shardedSuites,
 * exceptions: [{ workflow, job, timeoutMinutes, reason }] }`.
 */
export function parseSpeedPolicy(text) {
  const policy = { shardedSuites: [], exceptions: [] };
  let workflow = null;
  let exception = null;
  for (const line of text.split('\n')) {
    if (!line.trim() || line.trimStart().startsWith('#')) continue;
    const entry = pair(line);
    if (!entry) continue;
    const depth = line.length - line.trimStart().length;
    if (depth === 2 && entry.key === 'workflow') {
      workflow = entry.value;
      exception = null;
    } else if (depth === 2 && entry.key === 'sharded-suite') {
      policy.shardedSuites.push(entry.value);
    } else if (depth === 2) {
      policy[entry.key.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase())] = /^\d+$/.test(entry.value)
        ? Number(entry.value)
        : entry.value;
    } else if (depth === 4 && entry.key === 'over-limit-job' && workflow) {
      exception = { workflow, job: entry.value, timeoutMinutes: null, reason: '' };
      policy.exceptions.push(exception);
    } else if (depth === 6 && exception && entry.key === 'timeout-minutes') {
      exception.timeoutMinutes = Number(entry.value);
    } else if (depth === 6 && exception && entry.key === 'reason') {
      exception.reason = entry.value;
    }
  }
  return policy;
}
