// Deterministic offline fixtures own a cold source cache; offline alone replays
// the caller's existing captures and therefore does not mean shipped baseline.
import {mkdtempSync, realpathSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';

export function offlineSourceCache(env = process.env) {
  const directory = mkdtempSync(join(realpathSync(tmpdir()), 'formal-ai-offline-fixture-'));
  const values = {FORMAL_AI_OFFLINE: '1', FORMAL_AI_SOURCE_CACHE_DIR: directory, FORMAL_AI_CACHE_DIR: directory};
  const previous = new Map(Object.keys(values).map(key => [key, env[key]]));
  Object.assign(env, values);
  let restored = false;
  return () => {
    if (restored) return;
    restored = true;
    for (const [key, value] of previous) {
      if (value === undefined) delete env[key];
      else env[key] = value;
    }
    rmSync(directory, {recursive: true, force: true});
  };
}
