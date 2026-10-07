// `crate::source_fetch::CachedSourceClient::fetch` (rust/src/source_fetch.rs)
// over an injected io, so the version pins of a generated workflow (issue
// #1168) walk the same live -> cache -> baseline ladder in the JavaScript
// root that the native generator walks.
//
// The cache layout is the native one, byte for byte, so either runtime
// replays the other's captures: `<cacheDir>/source-cache/<key>.meta` holds
// `source-capture-v2`, `url=`, `fetched_at=` and `sha256=` lines, and the
// body is the content-addressed `<cacheDir>/source-cache/objects/<sha256>.body`
// (a legacy `source-capture-v1` record keeps its body beside the metadata).
// `<key>` is `translation::cache::cache_key`, FNV-1a 64 of the URL bytes.
//
// The module stays host-agnostic (no `node:fs`, no network): the host hands
// in `io = {readText(path), readBytes(path), writeBytes(path, bytes),
// createDirAll(path), get(url)}`, each answering null on a miss or failure.
// js/agentic/node-host.mjs builds the Node io (curl, as `CurlSourceTransport`).

import { sha256Hex } from './source_fetch.mjs';

const CACHE_FORMAT_VERSION = 'source-capture-v2';
const LEGACY_CACHE_FORMAT_VERSION = 'source-capture-v1';
/** Mirrors `DEFAULT_TTL_SECONDS`: sixty days. */
export const DEFAULT_TTL_SECONDS = 60 * 60 * 24 * 60;

/** Mirrors `translation::cache::cache_key`: FNV-1a 64 of the URL, 16 hex digits. */
export function cacheKey(url) {
  let hash = 0xcbf29ce484222325n;
  for (const byte of new TextEncoder().encode(url)) {
    hash ^= BigInt(byte);
    hash = (hash * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  return hash.toString(16).padStart(16, '0');
}

/** Mirrors `fn validate_url`. */
function validUrl(url) {
  const rest = url.startsWith('https://') ? url.slice(8) : url.startsWith('http://') ? url.slice(7) : null;
  return rest !== null && rest !== '' && !rest.startsWith('/') && !/[\s\p{Cc}]/u.test(rest);
}

const join = (...parts) => parts.join('/');

/** Mirrors `fn read_capture`: the cached capture, null on a miss or a damaged record. */
function readCapture(io, url, root, metadataPath) {
  const metadata = io.readText(metadataPath);
  if (metadata === null) return null;
  const [version, ...rest] = metadata.split('\n');
  if (version !== CACHE_FORMAT_VERSION && version !== LEGACY_CACHE_FORMAT_VERSION) return null;
  const field = (line, name) => (line !== undefined && line.startsWith(`${name}=`) ? line.slice(name.length + 1) : null);
  const sourceUrl = field(rest[0], 'url');
  const fetchedAt = field(rest[1], 'fetched_at');
  const recorded = field(rest[2], 'sha256');
  if (sourceUrl !== url || fetchedAt === null || recorded === null || !/^[0-9a-f]{64}$/.test(recorded)) return null;
  const bodyPath = version === CACHE_FORMAT_VERSION
    ? join(root, 'objects', `${recorded}.body`)
    : metadataPath.replace(/\.meta$/, '.body');
  const bytes = io.readBytes(bodyPath);
  if (bytes === null || sha256Hex(bytes) !== recorded) return null;
  return capture(url, fetchedAt, recorded, true, bytes);
}

function capture(url, fetchedAt, sha256, cachedFlag, bytes) {
  return { source_url: url, fetched_at: fetchedAt, sha256, cached: cachedFlag, text: new TextDecoder().decode(bytes), bytes };
}

/** Mirrors `fn write_capture` (metadata written after the body it names). */
function writeCapture(io, root, metadataPath, record) {
  io.createDirAll(join(root, 'objects'));
  io.writeBytes(join(root, 'objects', `${record.sha256}.body`), record.bytes);
  const metadata = `${CACHE_FORMAT_VERSION}\nurl=${record.source_url}\nfetched_at=${record.fetched_at}\nsha256=${record.sha256}\n`;
  io.writeBytes(metadataPath, new TextEncoder().encode(metadata));
}

/**
 * Mirrors `CachedSourceClient::new(cache_dir, transport).with_online(online)`
 * and its `fetch`: a fetch function `(url) => capture|null` for
 * `resolveVersionSet`. A fresh cached capture (or any cached one offline) is
 * replayed with `cached: true`; offline, a miss is null; online, the
 * transport's bytes are captured, written back and returned `cached: false`.
 * @param {{cacheDir: string, online: boolean, io: object, now?: () => number, ttlSeconds?: number}} options
 */
export function cachedSourceFetch({ cacheDir, online, io, now = () => Math.floor(Date.now() / 1000), ttlSeconds = DEFAULT_TTL_SECONDS }) {
  const root = join(cacheDir, 'source-cache');
  return (url) => {
    if (!validUrl(url)) return null;
    const metadataPath = join(root, `${cacheKey(url)}.meta`);
    const cachedCapture = readCapture(io, url, root, metadataPath);
    if (cachedCapture !== null) {
      const age = now() - (Number.parseInt(cachedCapture.fetched_at, 10) || 0);
      if (!online || age <= ttlSeconds) return cachedCapture;
    }
    if (!online) return null;
    const bytes = io.get(url);
    if (bytes === null) return null;
    const live = capture(url, String(now()), sha256Hex(bytes), false, bytes);
    try {
      writeCapture(io, root, metadataPath, live);
    } catch {
      // Rust returns the cache write failure as the fetch error.
      return null;
    }
    return live;
  };
}
