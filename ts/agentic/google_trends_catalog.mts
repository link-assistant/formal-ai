// Convert a publisher feed to reviewable multilingual prompt and answer records.

import { cached, host, readText, solve } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { trimEnd } from './crate/rust_str.mjs';
import { findChildValue, parseRoot } from './crate/seed_parser.mjs';

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

const SOURCE_ID = 'google_trends_rss';
const TOP_LIMIT = 10;
const XML_ENTITIES = { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'" };
const compact = (value) => String(value ?? '').trim().replace(/\s+/gu, ' ');

function blocks(xml, tag) {
  const pattern = new RegExp('<' + tag + '>([\\s\\S]*?)</' + tag + '>', 'gu');
  return Array.from(xml.matchAll(pattern), (match) => match[1]);
}

function text(xml, tag) {
  const raw = blocks(xml, tag)[0]?.trim() ?? '';
  if (raw.startsWith('<![CDATA[') && raw.endsWith(']]>')) return compact(raw.slice(9, -3));
  return compact(raw.replace(/&(amp|lt|gt|quot|apos);/gu, (_, name) => XML_ENTITIES[name]));
}

/** Mirrors `parse_google_trends_rss`. */
export function parseGoogleTrendsRss(xml) {
  if (!xml.includes('<rss') || !xml.includes('<channel>')) return [];
  const topics = [];
  for (const item of blocks(xml, 'item')) {
    const query = text(item, 'title');
    if (!query) continue;
    const news = blocks(item, 'ht:news_item').map((entry) => ({
      title: text(entry, 'ht:news_item_title'),
      source: text(entry, 'ht:news_item_source'),
      url: text(entry, 'ht:news_item_url'),
    })).find(({ title }) => title);
    topics.push({
      rank: topics.length + 1,
      query,
      traffic: text(item, 'ht:approx_traffic'),
      published: text(item, 'pubDate'),
      news,
    });
    if (topics.length === TOP_LIMIT) break;
  }
  return topics;
}

function trendsSource() {
  return cached(SOURCE_ID, () => parseRoot(readText('data/seed/sources-registry.lino')).children
    .flatMap(({ children }) => children)
    .find(({ name, id }) => name === 'source' && id === SOURCE_ID));
}

function promptTemplates() {
  return cached('google-trends-prompts', () => parseRoot(readText('data/seed/google-trends-prompts.lino')).children
    .filter((row) => findChildValue(row, 'record_type') === 'google_trend_prompt_template')
    .map((row) => ({
      language: findChildValue(row, 'language'),
      variation: findChildValue(row, 'variation'),
      template: findChildValue(row, 'text'),
    }))
    .filter(({ language, variation, template }) => language && variation && template));
}

/** Mirrors `feed_for_task` and `answer_google_trends_catalog`. */
export async function loadCatalog(task = '', fetchSource = host().sourceFetch?.({ ttlSeconds: 900 })) {
  const source = trendsSource();
  if (!source || !fetchSource) return null;
  const words = task.toLowerCase().split(/[\s?&#]+/u);
  const argument = words.findIndex((word) => word === 'geo' || word.startsWith('geo='));
  const geo = (argument < 0 ? 'US' : words[argument] === 'geo' ? words[argument + 1] ?? '' : words[argument].slice(4)).toUpperCase();
  if (!/^[A-Z]{2}$/u.test(geo)) return null;
  const locale = findChildValue(source, 'api_language');
  const url = findChildValue(source, 'api').replace('{geo}', geo);
  const capture = await fetchSource(url);
  if (!capture?.text || capture.source_url !== url || sha256Hex(capture.text) !== capture.sha256) return null;
  const topics = parseGoogleTrendsRss(capture.text);
  if (!topics.length) return null;
  for (const topic of topics) {
    topic.answered = [];
    for (const { language, variation, template } of promptTemplates()) {
      const prompt = template.replaceAll('{query}', topic.query);
      topic.answered.push({ language, variation, prompt, answer: await solve(prompt) });
    }
  }
  return { source: SOURCE_ID, url, geo, locale, collected: capture.fetched_at, sha256: capture.sha256, topics };
}

/** Mirrors `field`. */
export function linoField(indent, key, value) {
  const escaped = compact(value).replaceAll('\\', '\\\\').replaceAll('"', "'");
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
