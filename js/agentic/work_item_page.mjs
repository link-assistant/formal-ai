// A fetched work-item page is a document around the work item: keep its title
// and body, drop the site chrome, and reject a page that has no body so the
// API reads run instead (issue #1560). The markers are data in
// data/meta/work-item-steps.lino.
// Native twin: rust/src/agentic_coding/work_item_page.rs.
import { fill } from './work_item_steps.mjs';
import { rustLines } from './content.mjs';
import { trim } from './write_str.mjs';

const NAMED_ENTITIES = { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'", nbsp: '\u00a0', copy: '\u00a9' };

function alternatives(key) {
  return fill(key, []).split('|').filter(marker => marker !== '');
}

/** Mirrors `fn decode_entities`: one pass, so `&amp;lt;` stays `&lt;`. */
export function decodeEntities(text) {
  return text.replace(/&(#[0-9]{1,7}|#[xX][0-9a-fA-F]{1,6}|[a-z]+);/gu, (whole, name) => {
    if (!name.startsWith('#')) return Object.hasOwn(NAMED_ENTITIES, name) ? NAMED_ENTITIES[name] : whole;
    const hex = name[1] === 'x' || name[1] === 'X';
    const code = Number.parseInt(name.slice(hex ? 2 : 1), hex ? 16 : 10);
    return code > 0 && code <= 0x10ffff && (code < 0xd800 || code > 0xdfff) ? String.fromCodePoint(code) : whole;
  });
}

/** Mirrors `fn is_work_item_page`: the title suffix or a chrome line marks a site page. */
export function isWorkItemPage(raw) {
  const lines = rustLines(raw).map(line => trim(line));
  const first = lines.find(line => line !== '') ?? '';
  const chrome = alternatives('page_chrome_lines');
  return first.endsWith(fill('page_title_suffix', [])) || lines.some(line => chrome.includes(line));
}

function pageTitle(first) {
  const suffix = fill('page_title_suffix', []);
  const separator = fill('page_title_separator', []);
  const named = first.endsWith(suffix) ? first.slice(0, first.length - suffix.length) : first;
  const parts = named.split(separator);
  // "<title> · Issue #N · owner/repo": the last two parts are the page's address.
  return trim(parts.length >= 3 ? parts.slice(0, -2).join(separator) : named);
}

function earliest(text, markers, from) {
  let found = null;
  for (const marker of markers) {
    const at = text.indexOf(marker, from);
    if (at >= 0 && (found === null || at < found)) found = at;
  }
  return found;
}

function bodyLines(text) {
  const kept = [];
  const endLines = alternatives('page_body_end_lines');
  for (const raw of rustLines(text)) {
    const line = trim(raw);
    if (endLines.includes(line)) break;
    if (line === '' && (kept.length === 0 || kept[kept.length - 1] === '')) continue;
    kept.push(line);
  }
  while (kept.length && kept[kept.length - 1] === '') kept.pop();
  return kept.join('\n');
}

/**
 * Mirrors `fn work_item_page_text`. Text that is not a site page (gh, REST or
 * a connector's extracted record) is returned unchanged. A page yields its
 * title and body; a page without a body yields `null`.
 */
export function workItemPageText(raw) {
  if (!isWorkItemPage(raw)) return raw;
  const text = decodeEntities(raw);
  const startMarkers = alternatives('page_body_start');
  const start = earliest(text, startMarkers, 0);
  if (start === null) return null;
  const marker = startMarkers.find(candidate => text.startsWith(candidate, start));
  const from = start + marker.length;
  const end = earliest(text, alternatives('page_body_end'), from);
  const body = bodyLines(text.slice(from, end ?? text.length));
  if (body === '') return null;
  const first = rustLines(text).map(line => trim(line)).find(line => line !== '') ?? '';
  const title = pageTitle(first);
  return title === '' ? body : title + '\n\n' + body;
}
