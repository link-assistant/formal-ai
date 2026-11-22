// Seed an isolated replay cache through the same bounded writer used by the CLI.
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {cachedSourceFetchResult} from '../../js/agentic/crate/source_cache.mjs';
import {boundedSourceCacheIo, SOURCE_CACHE_POLICY} from '../../js/provenance/source-cache-budget.mjs';
import {readBoundedFilePath} from '../../js/provenance/bounded-file.mjs';

export function stageSourceCapture(cacheDir, fixture, sourceUrl) {
  const bytes = readBoundedFilePath(fixture, SOURCE_CACHE_POLICY.maxBodyBytes);
  const sha256 = createHash('sha256').update(bytes).digest('hex');
  const io = boundedSourceCacheIo(url => url === sourceUrl ? bytes : null);
  const result = cachedSourceFetchResult({cacheDir, online: true, io})(sourceUrl);
  if (result.kind !== 'capture') throw Error('source_capture_staging_failed:' + JSON.stringify(result));
  if (result.capture.sha256 !== sha256) throw Error('source_capture_fixture_mismatch');
  return {source_url: sourceUrl, fetched_at: result.capture.fetched_at, sha256};
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  if (process.argv.length !== 5) throw Error('usage: stage-source-capture.mjs <cache-directory> <fixture> <source-url>');
  console.log(JSON.stringify(stageSourceCapture(...process.argv.slice(2))));
}
