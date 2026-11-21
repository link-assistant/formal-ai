import assert from 'node:assert/strict';
import {mkdtempSync, rmSync, existsSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {offlineSourceCache} from '../../../experiments/issue-1162/offline-source-cache.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {cachedSourceFetch} from '../../../js/agentic/crate/source_cache.mjs';
import {boundedSourceCacheIo} from '../../../js/provenance/source-cache-budget.mjs';
import {baselineVersionSet, forGeneration} from '../../../js/agentic/crate/version_resolution.mjs';

test('offline baseline fixtures ignore a populated caller cache and restore it afterward', async () => {
  const caller = mkdtempSync(join(tmpdir(), 'formal-ai-populated-caller-'));
  const saved = new Map(['FORMAL_AI_OFFLINE', 'FORMAL_AI_SOURCE_CACHE_DIR', 'FORMAL_AI_CACHE_DIR'].map(key => [key, process.env[key]]));
  let restore;
  try {
    const bodies = new Map([
      ['https://api.github.com/repos/actions/checkout/releases/latest', '{"tag_name":"v999.0.0"}'],
      ['https://api.github.com/repos/actions/checkout/commits/v999.0.0', '{"sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}'],
    ]);
    const capture = cachedSourceFetch({cacheDir: caller, online: true,
      io: boundedSourceCacheIo(url => new TextEncoder().encode(bodies.get(url)))});
    for (const url of bodies.keys()) assert.ok(capture(url));
    process.env.FORMAL_AI_OFFLINE = '1';
    process.env.FORMAL_AI_SOURCE_CACHE_DIR = caller;
    process.env.FORMAL_AI_CACHE_DIR = caller;
    await installNodeHost({boot: async () => ({})});
    assert.equal(forGeneration().checkout.tag, 'v999.0.0', 'offline alone replays existing captures');
    restore = offlineSourceCache();
    const isolated = process.env.FORMAL_AI_SOURCE_CACHE_DIR;
    assert.notEqual(isolated, caller);
    assert.deepEqual(forGeneration(), baselineVersionSet());
    assert(!existsSync(join(isolated, 'source-cache')), 'no fixture network request wrote a capture');
    restore();
    assert(!existsSync(isolated));
    assert.equal(process.env.FORMAL_AI_SOURCE_CACHE_DIR, caller);
    assert.equal(forGeneration().checkout.tag, 'v999.0.0');
    restore();
  } finally {
    restore?.();
    for (const [key, value] of saved) {
      if (value === undefined) delete process.env[key]; else process.env[key] = value;
    }
    rmSync(caller, {recursive: true, force: true});
  }
});
