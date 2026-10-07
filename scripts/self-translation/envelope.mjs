// Self-translation between JavaScript and Rust, with the output format and
// the round-trip rule of link-foundation/meta-language PR #196
// (docs/self-translation.md, js/src/self-translation.js; public domain under
// the Unlicense), adapted to this repository's portable fragment:
//
// * a header records the source language and the SHA-256 and UTF-8 length
//   of the source;
// * every top-level item becomes one block: `translated` (its source kept
//   above the code as `// |` lines, with the hash of the code), `carried`
//   (the source lines only, with the construct map's reason) or a copied
//   comment; the definitions the code needs sit once in a prelude;
// * translating back restores every block whose code still matches its hash
//   to its source text, re-translates an edited block, and returns the
//   original byte for byte when the header's hash matches the result.

import { createHash } from 'node:crypto';

import { isRefusal } from './constructs.mjs';
import { emitJavaScript, emitRust, rustAllow } from './emitters.mjs';
import { itemTerm, parseJavaScriptItem, parseRustItem } from './frontends.mjs';
import { check, normalizeBody, signatures, toLino } from './ir.mjs';
import { lex, topLevelItems } from './lexer.mjs';

export const LANGUAGES = Object.freeze(['JavaScript', 'Rust']);

const HEADER = '// formal-ai:self-translation:v1 ';
const CARRIED = '// formal-ai:carried ';
const TRANSLATED = '// formal-ai:translated ';
const SOURCE_LINE = '// |';
const PRELUDE_BEGIN = '// formal-ai:prelude begin';
const PRELUDE_END = '// formal-ai:prelude end';
const ALIASES = new Map([
  ['javascript', 'JavaScript'], ['js', 'JavaScript'], ['mjs', 'JavaScript'],
  ['rust', 'Rust'], ['rs', 'Rust'],
]);

/**
 * The language `name` names (a name or a file extension), or null.
 * @param {string} name
 * @returns {string | null}
 */
export function languageOf(name) {
  return ALIASES.get(String(name).toLowerCase()) ?? null;
}

function required(name) {
  const language = languageOf(name);
  if (!language) throw new Error(`self-translation reads and writes ${LANGUAGES.join(', ')}, not ${name}`);
  return language;
}

const sha256 = (text) => createHash('sha256').update(Buffer.from(text, 'utf8')).digest('hex');
const lineBreak = (layout) => /^\r?\n$/u.test(layout);
const sourceLine = (line) => (line === '' ? SOURCE_LINE : `${SOURCE_LINE} ${line}`);
const sourceLines = (items) => items.map(({ text }) => text.slice(SOURCE_LINE.length).replace(/^ /u, ''));
const isMarker = (item) => item.comment && [HEADER, CARRIED, TRANSLATED, PRELUDE_BEGIN].some((marker) => item.text.startsWith(marker));

/**
 * The top-level items of `text` with their text, the layout after each and
 * UTF-8 byte offsets.
 * @param {string} text
 * @param {string} language
 * @returns {Array<object>}
 */
function sourceItems(text, language) {
  const items = topLevelItems(lex(text, language));
  const byte = (offset) => Buffer.byteLength(text.slice(0, offset), 'utf8');
  return items.map((item, index) => ({
    ...item,
    text: text.slice(item.start, item.end),
    after: text.slice(item.end, items[index + 1]?.start ?? text.length),
    byteStart: byte(item.start),
    byteEnd: byte(item.end),
    term: item.comment ? 'comment' : itemTerm(item.tokens, language),
  }));
}

/**
 * Group the items: a header, a prelude, a carried or translated block with
 * its provenance, a comment run, and an item with the comments directly
 * before it are one group each.
 */
function groupItems(items, text) {
  const groups = [];
  for (let index = 0; index < items.length; index += 1) {
    const item = items[index];
    const take = (end, fields) => {
      groups.push({ ...fields, items: items.slice(index, end + 1), after: items[end].after, text: text.slice(item.start, items[end].end) });
      index = end;
    };
    const linesAfter = (at) => {
      let end = at;
      while (end + 1 < items.length && items[end + 1].comment && items[end + 1].text.startsWith(SOURCE_LINE) && lineBreak(items[end].after)) end += 1;
      return end;
    };
    if (item.comment && item.text.startsWith(HEADER)) {
      take(index, { kind: 'provenance' });
    } else if (item.comment && item.text === PRELUDE_BEGIN) {
      let end = index;
      while (end + 1 < items.length && items[end].text !== PRELUDE_END) end += 1;
      take(end, { kind: 'provenance' });
    } else if (item.comment && item.text.startsWith(CARRIED)) {
      const end = linesAfter(index);
      take(end, { kind: 'restore', lines: sourceLines(items.slice(index + 1, end + 1)) });
    } else if (item.comment && item.text.startsWith(TRANSLATED)) {
      const fields = Object.fromEntries(item.text.slice(TRANSLATED.length).split(' ').slice(2).map((pair) => pair.split('=')));
      const linesEnd = linesAfter(index);
      const count = Number(fields.items);
      const last = linesEnd + count;
      if (Number.isSafeInteger(count) && count > 0 && last < items.length) {
        const code = text.slice(items[linesEnd + 1].start, items[last].end);
        if (sha256(code) === fields.sha256) {
          take(last, { kind: 'restore', lines: sourceLines(items.slice(index + 1, linesEnd + 1)) });
          continue;
        }
        // An edited translation is translated again; its provenance is dropped.
        take(linesEnd, { kind: 'provenance' });
        continue;
      }
      take(index, { kind: 'comment' });
    } else {
      // Comments directly before an item document it and travel with it.
      let end = index;
      while (items[end].comment && end + 1 < items.length && lineBreak(items[end].after) && !isMarker(items[end + 1])) end += 1;
      take(end, { kind: items[end].comment ? 'comment' : 'item' });
    }
  }
  return groups;
}

/**
 * Parse every translatable group, then check them together until no
 * refusal removes a definition another item calls.
 * @returns {Map<object, {ir?: object, refusal?: object}>}
 */
function translateGroups(groups, from) {
  const results = new Map();
  for (const group of groups) {
    if (group.kind !== 'item') continue;
    const item = group.items[group.items.length - 1];
    const doc = group.items.slice(0, -1).map(({ text }) => text).join('\n');
    try {
      const ir = from === 'Rust' ? parseRustItem(item.tokens) : parseJavaScriptItem(item.tokens, doc);
      results.set(group, { ir: ir.kind === 'function' ? { ...ir, body: normalizeBody(ir.body) } : ir });
    } catch (error) {
      if (!isRefusal(error)) throw error;
      results.set(group, { refusal: error });
    }
  }
  for (let changed = true; changed;) {
    changed = false;
    const table = signatures([...results.values()].filter((result) => result.ir).map((result) => result.ir));
    for (const [group, result] of results) {
      if (!result.ir) continue;
      try {
        result.checked = check(result.ir, table);
      } catch (error) {
        if (!isRefusal(error)) throw error;
        results.set(group, { refusal: error });
        changed = true;
      }
    }
  }
  return results;
}

/**
 * A comment keeps its text when the target reads it as an ordinary comment:
 * a Rust doc comment needs an item after it, and Rust block comments nest.
 */
function commentFits(text, language) {
  if (text.startsWith('/*') && /\/\*|\*\//u.test(text.slice(2, -2))) return false;
  return language !== 'Rust' || !/^(\/\/[/!]|\/\*[*!])/u.test(text);
}

/**
 * Translate `source` from `sourceLanguage` to `targetLanguage`.
 *
 * The result is `{ sourceLanguage, targetLanguage, code, items }`; each item
 * `{ term, start, end, status, reason }` is a top-level item of the source
 * (UTF-8 byte offsets) with its status: `kept` (same language), `translated`,
 * `restored`, `comment`, `provenance` or `carried`, with the reason.
 * @param {string} source
 * @param {string} sourceLanguage
 * @param {string} targetLanguage
 * @returns {{sourceLanguage: string, targetLanguage: string, code: string, items: Array<object>}}
 */
export function selfTranslate(source, sourceLanguage, targetLanguage) {
  const from = required(sourceLanguage);
  const to = required(targetLanguage);
  const text = String(source);
  const items = sourceItems(text, from);
  const record = (item, status, reason = null) => ({ term: item.term, start: item.byteStart, end: item.byteEnd, status, reason });
  if (from === to) return { sourceLanguage: from, targetLanguage: to, code: text, items: items.map((item) => record(item, 'kept')) };
  const groups = groupItems(items, text);
  const results = translateGroups(groups, from);
  const lints = new Set();
  const blocks = [];
  const recorded = [];
  let gap = '';
  for (const group of groups) {
    const out = block(group, results.get(group), from, to, lints);
    for (const item of group.items) recorded.push(record(item, out.status, out.reason ?? null));
    if (out.code !== null) {
      blocks.push(`${blocks.length ? '\n'.repeat(Math.max(1, (gap.match(/\n/gu) ?? []).length)) : ''}${out.code}`);
      gap = group.after;
    } else if (!gap) {
      gap = group.after;
    }
  }
  const body = `${blocks.join('')}\n`;
  const header = items.find((item) => item.comment && item.text.startsWith(HEADER));
  if (header && header.text.includes(` source=${to} `) && header.text.includes(` sha256=${sha256(body)} `)) {
    return { sourceLanguage: from, targetLanguage: to, code: body, items: recorded };
  }
  const lines = [`${HEADER}source=${from} target=${to} sha256=${sha256(text)} bytes=${Buffer.byteLength(text, 'utf8')}`, ''];
  const allow = to === 'Rust' ? rustAllow(lints) : null;
  if (allow) lines.push(PRELUDE_BEGIN, allow, PRELUDE_END, '');
  return { sourceLanguage: from, targetLanguage: to, code: `${lines.join('\n')}\n${body}`, items: recorded };
}

function block(group, result, from, to, lints) {
  if (group.kind === 'provenance') return { code: null, status: 'provenance' };
  if (group.kind === 'restore') return { code: group.lines.join('\n'), status: 'restored' };
  const term = group.items[group.items.length - 1].term;
  if (group.kind === 'comment') {
    if (group.items.every((item) => commentFits(item.text, from) && commentFits(item.text, to))) return { code: group.text, status: 'comment' };
    return carry(group, from, term, 'comment', 'comment the target cannot hold');
  }
  if (result.refusal) return carry(group, from, term, result.refusal.slug, result.refusal.message);
  let code;
  if (to === 'Rust') {
    const emitted = emitRust(result.checked);
    for (const lint of emitted.lints) lints.add(lint);
    code = emitted.code;
  } else {
    // A Rust doc comment (`///`) becomes the JSDoc description.
    const description = group.items.filter((item) => item.comment && item.text.startsWith('///')).map((item) => item.text.slice(3).replace(/^ /u, ''));
    code = emitJavaScript(result.ir, description);
  }
  const count = topLevelItems(lex(code, to)).length;
  const marker = `${TRANSLATED}${from} ${term} items=${count} sha256=${sha256(code)}`;
  return { code: [marker, ...group.text.split(/\r?\n/u).map(sourceLine), code].join('\n'), status: 'translated' };
}

function carry(group, from, term, slug, reason) {
  return {
    code: [`${CARRIED}${from} ${term} (${reason})`, ...group.text.split(/\r?\n/u).map(sourceLine)].join('\n'),
    status: 'carried',
    reason,
    slug,
  };
}

/**
 * The meta-language document of `source`: one portable-pure-v1 link per
 * translated item and one `(carried term "reason")` link per other item.
 * @param {string} source
 * @param {string} sourceLanguage
 * @returns {string}
 */
export function toMeta(source, sourceLanguage) {
  const from = required(sourceLanguage);
  const text = String(source);
  const groups = groupItems(sourceItems(text, from), text);
  const results = translateGroups(groups, from);
  const lines = [`# portable-pure-v1 meta document of a ${from} source, sha256=${sha256(text)}`];
  for (const group of groups) {
    const result = results.get(group);
    if (!result) continue;
    const term = group.items[group.items.length - 1].term;
    lines.push(result.ir ? toLino(result.ir) : `(carried ${term} "${result.refusal.message.replace(/"/gu, "'")}")`);
  }
  return `${lines.join('\n')}\n`;
}

/**
 * The items of a meta document emitted in `targetLanguage`, with the Rust
 * prelude when one is needed.
 * @param {Array<object>} irItems from `fromLino`
 * @param {string} targetLanguage
 * @returns {string}
 */
export function fromMeta(irItems, targetLanguage) {
  const to = required(targetLanguage);
  const table = signatures(irItems);
  const lints = new Set();
  const codes = irItems.map((item) => {
    if (to === 'JavaScript') return emitJavaScript(item);
    const emitted = emitRust(check(item, table));
    for (const lint of emitted.lints) lints.add(lint);
    return emitted.code;
  });
  const allow = to === 'Rust' ? rustAllow(lints) : null;
  return `${[...(allow ? [allow] : []), ...codes].join('\n\n')}\n`;
}

/**
 * The checked IR of every item of `source` that translates, by name: the
 * reimport a round-trip test compares across languages.
 * @param {string} source
 * @param {string} sourceLanguage
 * @returns {Map<string, string>} name to its meta link
 */
export function translatedItems(source, sourceLanguage) {
  const from = required(sourceLanguage);
  const text = String(source);
  const results = translateGroups(groupItems(sourceItems(text, from), text), from);
  const out = new Map();
  for (const result of results.values()) if (result.ir) out.set(result.ir.name, toLino(result.ir));
  return out;
}
