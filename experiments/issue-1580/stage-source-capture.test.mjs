import assert from 'node:assert/strict';
import {mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {test} from 'node:test';
import {stageSourceCapture} from './stage-source-capture.mjs';
import {cachedSourceFetchResult} from '../../js/agentic/crate/source_cache.mjs';
import {boundedSourceCacheIo} from '../../js/provenance/source-cache-budget.mjs';

const fixture = new URL('../../docs/case-studies/issue-498/raw-data/google-trends-us-rss.xml', import.meta.url);
const sourceUrl = 'https://trends.google.com/trending/rss?geo=US';
function cache(t) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-source-replay-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  return directory;
}
const offline = cacheDir => cachedSourceFetchResult({cacheDir, online: false,
  io: boundedSourceCacheIo(() => {throw Error('offline replay attempted a fetch');})});

test('the recorded RSS replays offline with exact source bytes and URL ownership', {timeout: 5000}, t => {
  const directory = cache(t);
  assert.deepEqual(offline(directory)(sourceUrl), {kind: 'miss'});
  const identity = stageSourceCapture(directory, fixture, sourceUrl);
  const replay = offline(directory)(sourceUrl);
  assert.equal(replay.kind, 'capture');
  assert.equal(replay.capture.cached, true);
  assert.equal(replay.capture.sha256, identity.sha256);
  assert.deepEqual(Buffer.from(replay.capture.bytes), readFileSync(fixture));
  assert.deepEqual(offline(directory)(sourceUrl.replace('US', 'GB')), {kind: 'miss'});
});

test('a physically changed source object cannot be accepted or restaged', {timeout: 5000}, t => {
  const directory = cache(t);
  stageSourceCapture(directory, fixture, sourceUrl);
  const objects = join(directory, 'source-cache', 'objects');
  const [name] = readdirSync(objects);
  writeFileSync(join(objects, name), 'tampered RSS');
  const replay = offline(directory)(sourceUrl);
  assert.equal(replay.kind, 'error');
  assert.match(replay.error, /source_cache_content_hash_mismatch/u);
  assert.throws(() => stageSourceCapture(directory, fixture, sourceUrl), /source_cache_content_hash_mismatch/u);
});

test('a preexisting capture for the same URL cannot substitute another fixture', {timeout: 5000}, t => {
  const directory = cache(t);
  stageSourceCapture(directory, fixture, sourceUrl);
  const replacement = join(directory, 'different.xml');
  writeFileSync(replacement, '<rss>different capture</rss>');
  assert.throws(() => stageSourceCapture(directory, replacement, sourceUrl), /source_capture_fixture_mismatch/u);
});
