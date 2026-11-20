import { serverMessage } from '../messages.mjs';
import { createReadOnlyFetch } from '../read-access.mjs';
import { boundedManifest, sha256 } from './packages.mjs';
import { readRecipe, renderRecipe } from './recipe.mjs';
import { parseLinkSubstitution, renderLinkSubstitution } from './substitutions.mjs';

const MAX_DOWNLOAD = 128 * 1024;

function originOf(value) {
  const url = new URL(value);
  if (url.protocol !== 'https:' || url.username || url.password || url.pathname !== '/' || url.search || url.hash) {
    throw new Error(serverMessage('dynamic_meta_origin_https'));
  }
  return url.origin;
}

async function boundedBody(response) {
  const reader = response.body?.getReader();
  if (!reader) throw new Error(serverMessage('dynamic_meta_source_empty'));
  const parts = [];
  let length = 0;
  try {
    for (;;) {
      const part = await reader.read();
      if (part.done) break;
      length += part.value.byteLength;
      if (length > MAX_DOWNLOAD) throw new Error(serverMessage('dynamic_meta_source_size'));
      parts.push(part.value);
    }
  } finally {
    await reader.cancel().catch(() => {});
  }
  return Buffer.concat(parts, length);
}

const FENCE_LANGUAGES = new Map([
  ['formal-ai-markov', 'english'], ['lino', 'links'], ['javascript', 'javascript'], ['python', 'python'],
  ['formal-ai-link', 'link-substitution'],
]);

function convert(language, text) {
  if (language === 'link-substitution') {
    return { language, substitute: renderLinkSubstitution(parseLinkSubstitution(text)) };
  }
  const program = readRecipe(language, text);
  return { language, links: renderRecipe('links', program), instructions: renderRecipe('english', program) };
}

function convertBlock(language, text) {
  if (language !== 'links') return convert(language, text);
  try {
    return convert('links', text);
  } catch {
    return convert('link-substitution', text);
  }
}

function programs(text, language) {
  if (language !== 'markdown') return { candidates: [convert(language, text)], rejected: [] };
  const candidates = [];
  const rejected = [];
  // Track every fenced block so text embedded in unrelated examples is never
  // mistaken for a recipe heading.
  let fence = null;
  for (const [index, line] of text.split(/\r?\n/u).entries()) {
    if (fence) {
      if (line.trim() === fence.delimiter) {
        if (fence.language) {
          const source = fence.lines.join('\n');
          try {
            if (fence.tag || /^\s*\((?:\s*\(|\))/u.test(source)) {
              candidates.push({ ...convertBlock(fence.language, source), line: fence.line });
            }
          } catch (error) {
            if (fence.tag) rejected.push({ language: fence.language, line: fence.line, reason: error.message });
          }
          if (candidates.length + rejected.length >= 8) break;
        }
        fence = null;
      } else if (fence.language) fence.lines.push(line);
    } else {
      const match = /^\s*(`{3,}|~{3,})([^\s]*)\s*$/u.exec(line);
      if (match) {
        const [, delimiter, tag] = match;
        fence = { delimiter, tag, language: tag === '' ? 'link-substitution' : FENCE_LANGUAGES.get(tag), lines: [], line: index + 1 };
      }
    }
  }
  if (!candidates.length && !rejected.length) throw new Error(serverMessage('dynamic_meta_source_no_recipes'));
  return { candidates, rejected };
}

export function createRecipeDiscovery({ origins = [], env = process.env, fetchImpl } = {}) {
  const allowed = new Set(origins.map(originOf));
  const fetchSource = createReadOnlyFetch({ env, fetchImpl });
  async function sourceText(source, signal) {
    const url = new URL(source.url);
    if (url.href.length > 2048 || url.username || url.password || url.hash || !allowed.has(url.origin)) {
      throw new Error(serverMessage('dynamic_meta_origin_denied'));
    }
    const response = await fetchSource(url.href, { redirect: 'manual', signal, credentials: 'omit' });
    if (!response.ok) {
      await response.body?.cancel().catch(() => {});
      throw new Error(serverMessage('dynamic_meta_source_status', { status: response.status }));
    }
    const bytes = await boundedBody(response);
    const digest = sha256(bytes);
    if (source.sha256 !== undefined && source.sha256 !== digest) throw new Error(serverMessage('dynamic_meta_source_digest'));
    return { url: url.href, digest, text: new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(bytes) };
  }

  return async function discover(sources, signal) {
    if (!Array.isArray(sources) || sources.length < 1 || sources.length > 8) {
      throw new Error(serverMessage('dynamic_meta_source_count'));
    }
    const deadline = AbortSignal.timeout(30_000);
    const requestSignal = signal ? AbortSignal.any([signal, deadline]) : deadline;
    const results = [];
    for (const source of sources) {
      signal?.throwIfAborted();
      try {
        if (!['markdown', 'english', 'links', 'javascript', 'python', 'link-substitution', 'plugin'].includes(source.language)) {
          throw new Error(serverMessage('dynamic_meta_source_language'));
        }
        const { url, digest, text } = await sourceText(source, requestSignal);
        if (source.language === 'plugin') {
          const { id, origin } = boundedManifest(text);
          results.push({ url, sha256: digest, package: { id, origin }, manifest: text });
          continue;
        }
        const { candidates, rejected } = programs(text, source.language);
        results.push({
          url, sha256: digest, rejected,
          candidates: candidates.map(({ line, ...candidate }) => ({
            ...candidate, provenance: { url, sha256: digest, ...(line === undefined ? {} : { line }) },
          })),
        });
      } catch (error) {
        results.push({ url: typeof source?.url === 'string' ? source.url : null, error: error.message });
      }
    }
    return results;
  };
}
