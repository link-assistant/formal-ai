/**
 * Structured search-provider capture (issue #130).
 *
 * Turns a query + provider into a normalized, machine-readable result set so
 * that browser, CLI, and server callers all consume one consistent contract
 * instead of each reimplementing provider-specific scraping. Server-side and
 * CLI callers fetch provider pages directly (no CORS restriction), so this
 * module defaults to the `fetch` capture mode. Providers that expose a native
 * CORS/JSON API (Wikipedia) are preferred; HTML search engines are parsed
 * best-effort and report CAPTCHA/blocking through `diagnostics`.
 *
 * Normalized result shape:
 * {
 *   query, provider, captureMode, capturedAt,
 *   results: [{ rank, title, url, snippet }],
 *   diagnostics: { status, blockedByCors, blockedByCaptcha, sourceUrl }
 * }
 *
 * @module search
 */

import fetch from 'node-fetch';
import * as cheerio from 'cheerio';
import he from 'he';
import { URL } from 'url';

/** Providers understood by the search contract. */
export const SEARCH_PROVIDERS = [
  'wikipedia',
  'duckduckgo',
  'google',
  'bing',
  'brave',
];

/** Default provider when none is supplied. */
export const DEFAULT_PROVIDER = 'wikipedia';

/** Default number of results requested/returned. */
export const DEFAULT_LIMIT = 10;

const USER_AGENT =
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 ' +
  '(KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36';

/**
 * Normalize whitespace and decode HTML entities in extracted text.
 *
 * @param {string} text - Raw text, possibly with markup/entities
 * @returns {string} Cleaned single-line text
 */
function cleanText(text) {
  if (!text) {
    return '';
  }
  // Strip any residual tags, decode entities, collapse whitespace.
  const stripped = String(text).replace(/<[^>]*>/g, ' ');
  return he.decode(stripped).replace(/\s+/g, ' ').trim();
}

/**
 * Build the provider-native source URL for a query.
 *
 * @param {string} provider - One of {@link SEARCH_PROVIDERS}
 * @param {string} query - Search query
 * @param {number} limit - Desired result count (used by APIs that support it)
 * @returns {string} Absolute source URL
 */
export function buildSearchUrl(provider, query, limit = DEFAULT_LIMIT) {
  const q = encodeURIComponent(query);
  switch (provider) {
    case 'wikipedia':
      return `https://en.wikipedia.org/w/rest.php/v1/search/page?q=${q}&limit=${limit}`;
    case 'duckduckgo':
      return `https://html.duckduckgo.com/html/?q=${q}`;
    case 'google':
      return `https://www.google.com/search?q=${q}&num=${limit}`;
    case 'bing':
      return `https://www.bing.com/search?q=${q}&count=${limit}`;
    case 'brave':
      return `https://search.brave.com/search?q=${q}`;
    default:
      throw new Error(
        `Unknown search provider "${provider}". Supported: ${SEARCH_PROVIDERS.join(', ')}`
      );
  }
}

/**
 * Detect provider CAPTCHA / bot-block interstitials in an HTML body.
 *
 * @param {string} html - Raw HTML response body
 * @returns {boolean} True when the page looks like a block/CAPTCHA wall
 */
export function looksLikeCaptcha(html) {
  if (!html) {
    return false;
  }
  return /captcha|unusual traffic|are you a robot|verify you('|&#39;)?re a human|\/sorry\/index|automated queries/i.test(
    html
  );
}

/**
 * Decode a DuckDuckGo redirect href (`//duckduckgo.com/l/?uddg=...`) back to
 * the real destination URL. Non-redirect hrefs are returned as-is.
 *
 * @param {string} href - Raw anchor href
 * @returns {string} The resolved destination URL
 */
function resolveDuckDuckGoHref(href) {
  if (!href) {
    return '';
  }
  let normalized = href;
  if (normalized.startsWith('//')) {
    normalized = `https:${normalized}`;
  }
  try {
    const parsed = new URL(normalized, 'https://duckduckgo.com');
    const uddg = parsed.searchParams.get('uddg');
    if (uddg) {
      return uddg;
    }
    return parsed.toString();
  } catch {
    return href;
  }
}

/** Parse the Wikipedia REST search JSON into normalized results. */
function parseWikipedia(body, limit) {
  let data;
  try {
    data = typeof body === 'string' ? JSON.parse(body) : body;
  } catch {
    return [];
  }
  const pages = Array.isArray(data?.pages) ? data.pages : [];
  return pages.slice(0, limit).map((page, i) => ({
    rank: i + 1,
    title: cleanText(page.title || page.key || ''),
    url: `https://en.wikipedia.org/wiki/${encodeURIComponent(page.key || page.title || '')}`,
    snippet: cleanText(page.excerpt || page.description || ''),
  }));
}

/** Parse DuckDuckGo HTML (html.duckduckgo.com/html) into normalized results. */
function parseDuckDuckGo($, limit) {
  const results = [];
  // `.result__body` wraps each organic result; fall back to `.web-result`
  // for older markup. Selecting a single container avoids counting the same
  // result twice when both classes are present on nested elements.
  let containers = $('.result__body');
  if (containers.length === 0) {
    containers = $('.web-result');
  }
  containers.each((_, el) => {
    if (results.length >= limit) {
      return;
    }
    const anchor = $(el).find('a.result__a').first();
    const title = cleanText(anchor.text());
    const url = resolveDuckDuckGoHref(anchor.attr('href'));
    const snippet = cleanText($(el).find('.result__snippet').first().text());
    if (title && url) {
      results.push({ rank: results.length + 1, title, url, snippet });
    }
  });
  return results;
}

/** Parse Google results HTML into normalized results (best-effort). */
function parseGoogle($, limit) {
  const results = [];
  $('div.g, div.tF2Cxc, div.MjjYud').each((_, el) => {
    if (results.length >= limit) {
      return;
    }
    const anchor = $(el).find('a[href^="http"]').first();
    const url = anchor.attr('href');
    const title = cleanText($(el).find('h3').first().text());
    const snippet = cleanText(
      $(el).find('div[data-sncf], .VwiC3b, .IsZvec').first().text()
    );
    if (title && url) {
      results.push({ rank: results.length + 1, title, url, snippet });
    }
  });
  return results;
}

/** Parse Bing results HTML into normalized results. */
function parseBing($, limit) {
  const results = [];
  $('li.b_algo').each((_, el) => {
    if (results.length >= limit) {
      return;
    }
    const anchor = $(el).find('h2 a').first();
    const url = anchor.attr('href');
    const title = cleanText(anchor.text());
    const snippet = cleanText($(el).find('.b_caption p, p').first().text());
    if (title && url) {
      results.push({ rank: results.length + 1, title, url, snippet });
    }
  });
  return results;
}

/** Parse Brave Search results HTML into normalized results. */
function parseBrave($, limit) {
  const results = [];
  $('div.snippet[data-type="web"], div.snippet').each((_, el) => {
    if (results.length >= limit) {
      return;
    }
    const anchor = $(el).find('a[href^="http"]').first();
    const url = anchor.attr('href');
    const title = cleanText(
      $(el).find('.snippet-title, .title').first().text() || anchor.text()
    );
    const snippet = cleanText(
      $(el).find('.snippet-description, .snippet-content').first().text()
    );
    if (title && url) {
      results.push({ rank: results.length + 1, title, url, snippet });
    }
  });
  return results;
}

/**
 * Parse a provider response body into normalized result rows. Pure function
 * (no network) so it can be unit-tested against fixtures.
 *
 * @param {string} provider - One of {@link SEARCH_PROVIDERS}
 * @param {string} body - Raw response body (JSON for wikipedia, HTML otherwise)
 * @param {{limit?: number}} [options] - Parse options
 * @returns {{results: Array, blockedByCaptcha: boolean}}
 */
export function parseSearchResults(
  provider,
  body,
  { limit = DEFAULT_LIMIT } = {}
) {
  if (provider === 'wikipedia') {
    return { results: parseWikipedia(body, limit), blockedByCaptcha: false };
  }
  const blockedByCaptcha = looksLikeCaptcha(body);
  const $ = cheerio.load(body || '');
  let results;
  switch (provider) {
    case 'duckduckgo':
      results = parseDuckDuckGo($, limit);
      break;
    case 'google':
      results = parseGoogle($, limit);
      break;
    case 'bing':
      results = parseBing($, limit);
      break;
    case 'brave':
      results = parseBrave($, limit);
      break;
    default:
      throw new Error(
        `Unknown search provider "${provider}". Supported: ${SEARCH_PROVIDERS.join(', ')}`
      );
  }
  return { results, blockedByCaptcha };
}

/**
 * Render a normalized search result as Markdown.
 *
 * @param {Object} result - Normalized search result object
 * @returns {string} Markdown document
 */
export function formatSearchAsMarkdown(result) {
  const lines = [];
  lines.push(`# Search results for "${result.query}"`);
  lines.push('');
  lines.push(`- Provider: \`${result.provider}\``);
  lines.push(`- Capture mode: \`${result.captureMode}\``);
  lines.push(`- Captured at: ${result.capturedAt}`);
  lines.push(`- Source: ${result.diagnostics.sourceUrl}`);
  if (result.diagnostics.blockedByCaptcha) {
    lines.push('- ⚠️ Provider returned a CAPTCHA / bot-block page.');
  }
  lines.push('');
  if (!result.results.length) {
    lines.push('_No results._');
    return lines.join('\n');
  }
  for (const item of result.results) {
    lines.push(`${item.rank}. [${item.title}](${item.url})`);
    if (item.snippet) {
      lines.push(`   ${item.snippet}`);
    }
  }
  return lines.join('\n');
}

/**
 * Capture structured search results for a query from a provider.
 *
 * @param {Object} options - Search options
 * @param {string} options.query - Search query (required)
 * @param {string} [options.provider=wikipedia] - Provider id
 * @param {number} [options.limit=10] - Max results to return
 * @param {string} [options.captureMode=fetch] - Reported capture mode
 * @param {Function} [options.fetchImpl=fetch] - Fetch implementation (injectable for tests)
 * @param {Function} [options.now] - Clock returning an ISO timestamp (injectable for tests)
 * @returns {Promise<Object>} Normalized search result object
 */
export async function search({
  query,
  provider = DEFAULT_PROVIDER,
  limit = DEFAULT_LIMIT,
  captureMode = 'fetch',
  fetchImpl = fetch,
  now = () => new Date().toISOString(),
} = {}) {
  if (!query || !String(query).trim()) {
    throw new Error('Missing `query` parameter');
  }
  if (!SEARCH_PROVIDERS.includes(provider)) {
    throw new Error(
      `Unknown search provider "${provider}". Supported: ${SEARCH_PROVIDERS.join(', ')}`
    );
  }

  const sourceUrl = buildSearchUrl(provider, query, limit);
  const capturedAt = now();
  const diagnostics = {
    status: 0,
    blockedByCors: false,
    blockedByCaptcha: false,
    sourceUrl,
  };

  let results = [];
  try {
    const response = await fetchImpl(sourceUrl, {
      headers: {
        'User-Agent': USER_AGENT,
        Accept:
          provider === 'wikipedia'
            ? 'application/json'
            : 'text/html,application/xhtml+xml',
        'Accept-Language': 'en-US,en;q=0.9',
      },
    });
    diagnostics.status = response.status;
    const body = await response.text();
    const parsed = parseSearchResults(provider, body, { limit });
    results = parsed.results;
    diagnostics.blockedByCaptcha = parsed.blockedByCaptcha;
  } catch (err) {
    // A network/transport failure from a server context is reported as an
    // error status; browser CORS failures surface the same way and are flagged.
    diagnostics.status = diagnostics.status || 0;
    diagnostics.error = err.message;
  }

  return {
    query,
    provider,
    captureMode,
    capturedAt,
    results,
    diagnostics,
  };
}

/**
 * Express handler for `GET /search?q=<query>&provider=<p>&format=json|markdown`.
 *
 * @param {import('express').Request} req
 * @param {import('express').Response} res
 */
export async function searchHandler(req, res) {
  const query = req.query.q || req.query.query;
  if (!query) {
    return res.status(400).send('Missing `q` parameter');
  }
  const provider = req.query.provider || DEFAULT_PROVIDER;
  if (!SEARCH_PROVIDERS.includes(provider)) {
    return res
      .status(400)
      .send(
        `Unknown provider "${provider}". Supported: ${SEARCH_PROVIDERS.join(', ')}`
      );
  }
  const limit = Number.parseInt(req.query.limit, 10) || DEFAULT_LIMIT;
  const format = (req.query.format || 'json').toLowerCase();
  try {
    const result = await search({ query, provider, limit });
    if (format === 'markdown' || format === 'md') {
      res.type('text/markdown').send(formatSearchAsMarkdown(result));
    } else {
      res.json(result);
    }
  } catch (err) {
    console.error('Search error:', err);
    res.status(500).send(`Error performing search: ${err.message}`);
  }
}
