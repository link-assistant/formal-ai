// Convert a publisher feed to reviewable multilingual prompt and answer records.

import { cached, host, readText, solve } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { compactFeedText, GOOGLE_TRENDS_SOURCE, loadPromptTemplates, parseGoogleTrendsRss, requestedGeo } from './crate/google_trends_feed.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { trimEnd } from './crate/rust_str.mjs';
import { findChildValue, parseRoot } from './crate/seed_parser.mjs';

export { parseGoogleTrendsRss };

/** Mirrors `GOOGLE_TRENDS_CATALOG_PATH` in rust/src/agentic_coding/google_trends_catalog.rs. */
export const GOOGLE_TRENDS_CATALOG_PATH = 'google-trends-catalog.lino';

/** Mirrors `GOOGLE_TRENDS_CATALOG_TASK` in rust/src/agentic_coding/google_trends_catalog.rs. */
export function googleTrendsCatalogTask() {
  return agenticMessage('google_trends_catalog_task');
}

const GOOGLE_TRENDS_KEYWORDS = ['google trends', 'trending searches', 'top searches', 'trends catalog'];

/**
 * Mirrors `fn is_google_trends_catalog_task` in rust/src/agentic_coding/google_trends_catalog.rs.
 * @param {string} prompt
 */
export function isGoogleTrendsCatalogTask(prompt) {
  const lower = prompt.toLowerCase();
  return GOOGLE_TRENDS_KEYWORDS.some((keyword) => lower.includes(keyword))
    && (lower.includes('prompt') || lower.includes('answer') || lower.includes('catalog') || lower.includes('test'));
}

function trendsSource() {
  return cached(GOOGLE_TRENDS_SOURCE, () => parseRoot(readText('data/seed/sources-registry.lino')).children
    .flatMap(({ children }) => children)
    .find(({ name, id }) => name === 'source' && id === GOOGLE_TRENDS_SOURCE));
}

function promptTemplates() {
  return cached('google-trends-prompts', () =>
    loadPromptTemplates(readText('data/seed/google-trends-prompts.lino')));
}

/** Mirrors `feed_for_task` and `answer_google_trends_catalog`. */
export async function loadCatalog(task = '', fetchSource = host().sourceFetch?.({ ttlSeconds: 900 })) {
  const source = trendsSource();
  if (!source || !fetchSource) return null;
  const geo = requestedGeo(task);
  const endpoint = findChildValue(source, 'api');
  if (!geo || !endpoint.includes('{geo}')) return null;
  const url = endpoint.replace('{geo}', geo);
  const locale = findChildValue(source, 'api_language');
  const capture = await fetchSource(url);
  if (!capture?.text || capture.source_url !== url || sha256Hex(capture.text) !== capture.sha256) return null;
  const topics = parseGoogleTrendsRss(capture.text);
  if (!topics.length) return null;
  for (const topic of topics) {
    topic.answered = [];
    for (const { language, variation, template } of promptTemplates()) {
      const prompt = template.replaceAll('{query}', () => topic.query);
      topic.answered.push({ language, variation, prompt, answer: await solve(prompt) });
    }
  }
  return { source: GOOGLE_TRENDS_SOURCE, url, geo, locale, collected: capture.fetched_at, sha256: capture.sha256, topics };
}

/** Mirrors `field`. */
export function linoField(indent, key, value) {
  const escaped = compactFeedText(value).replaceAll('\\', '\\\\').replaceAll('"', "'");
  return ' '.repeat(indent) + key + ' "' + escaped + '"\n';
}

function sourceFields(catalog) {
  return linoField(2, 'source', catalog.source) + linoField(2, 'source_url', catalog.url)
    + linoField(2, 'geo', catalog.geo) + linoField(2, 'locale', catalog.locale)
    + linoField(2, 'collected_at', catalog.collected) + linoField(2, 'source_sha256', catalog.sha256);
}

/** Render linkable findings from the exact captured response. */
export function renderCatalog(catalog) {
  const count = catalog.topics.reduce((sum, topic) => sum + topic.answered.length, 0);
  let out = 'google_trends_catalog\n' + linoField(2, 'record_type', 'google_trends_catalog')
    + sourceFields(catalog) + linoField(2, 'topic_count', catalog.topics.length)
    + linoField(2, 'prompt_count', count) + linoField(2, 'answered_count', count);
  for (const { rank, query, traffic, published, news, answered } of catalog.topics) {
    out += '  topic\n' + linoField(4, 'rank', rank) + linoField(4, 'query', query);
    if (traffic) out += linoField(4, 'approx_traffic', traffic);
    if (published) out += linoField(4, 'pub_date', published);
    if (news) out += '    source_news\n' + Object.entries(news).map(([key, value]) => linoField(6, key, value)).join('');
    for (const { language, variation, prompt } of answered) {
      out += '    prompt\n' + linoField(6, 'prompt_language', language)
        + linoField(6, 'variation', variation) + linoField(6, 'text', prompt);
    }
    for (const { language, variation, prompt, answer } of answered) {
      out += '    answered\n' + linoField(6, 'prompt_language', language)
        + linoField(6, 'variation', variation) + linoField(6, 'prompt', prompt)
        + linoField(6, 'intent', answer.intent)
        + linoField(6, 'confidence', Number(answer.confidence).toFixed(3)) + linoField(6, 'answer', answer.answer);
    }
  }
  return out;
}

/** Mirrors `document_for_task`. */
export async function renderDocument(task) {
  const catalog = await loadCatalog(task);
  return catalog && renderCatalog(catalog);
}

/**
 * Mirrors `fn final_answer` in rust/src/agentic_coding/google_trends_catalog.rs.
 * @param {string} document
 */
export function finalAnswer(document) {
  const catalog = parseRoot(document).children[0];
  return agenticMessage('google_trends_catalog_final_answer', {
    topic_count: findChildValue(catalog, 'topic_count'),
    geo: findChildValue(catalog, 'geo'),
    prompt_count: findChildValue(catalog, 'prompt_count'),
    answer_count: findChildValue(catalog, 'answered_count'),
    source_url: findChildValue(catalog, 'source_url'),
    path: GOOGLE_TRENDS_CATALOG_PATH,
    document: trimEnd(document),
  });
}

/** Mirrors `fn verification_command` in rust/src/agentic_coding/google_trends_catalog.rs. */
export function verificationCommand() {
  const program = `p='${GOOGLE_TRENDS_CATALOG_PATH}';s=open(p).read().splitlines();print(len(s));print('\\n'.join(s[:12]))`;
  return "python3 -c '" + program.replaceAll("'", String.fromCharCode(39, 92, 39, 39)) + "'";
}
