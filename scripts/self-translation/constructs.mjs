// The construct map (data/meta/self-translation/constructs.lino) as the
// tables the frontends and emitters consult. Nothing here names a construct
// the data file does not: the spellings, precedences, operand types and
// refusal reasons are all read from it.

import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { field, parseLinks, textOf } from './lino.mjs';

export const CONSTRUCTS_PATH = 'data/meta/self-translation/constructs.lino';

const REPOSITORY = join(dirname(fileURLToPath(import.meta.url)), '..', '..');

let cached = null;

/**
 * The construct map, read once.
 * @returns {{
 *   rows: Array<{kind: string, id: string}>,
 *   types: Map<string, {javascript: string, rust: string, rustOwned: string}>,
 *   operators: Map<string, {javascript: string, rust: string, precedence: number, operands: string, yields: string}>,
 *   unaries: Map<string, {javascript: string, rust: string, operand: string, yields: string}>,
 *   builtins: Map<string, {javascript: string, rust: string, arguments: Array<string>, yields: string}>,
 *   methods: Map<string, {javascript: string, rust: string, receiver: string, arguments: Array<string>, yields: string}>,
 *   carried: Map<string, string>,
 * }}
 */
export function constructMap() {
  if (cached) return cached;
  const text = readFileSync(join(REPOSITORY, CONSTRUCTS_PATH), 'utf8');
  const map = {
    rows: [],
    types: new Map(),
    operators: new Map(),
    unaries: new Map(),
    builtins: new Map(),
    methods: new Map(),
    carried: new Map(),
  };
  const value = (link, head) => {
    const found = field(link, head);
    if (!found) throw new Error(`${CONSTRUCTS_PATH}: (${link[0]} ${link[1]}) has no (${head} ...)`);
    return textOf(found[1]);
  };
  const values = (link, head) => (field(link, head) ?? [head]).slice(1).map(textOf);
  for (const link of parseLinks(text)) {
    const [kind, id] = link;
    map.rows.push({ kind, id });
    if (kind === 'type') {
      map.types.set(id, { javascript: value(link, 'javascript'), rust: value(link, 'rust'), rustOwned: value(link, 'rust-owned') });
    } else if (kind === 'operator') {
      map.operators.set(id, {
        javascript: value(link, 'javascript'),
        rust: value(link, 'rust'),
        precedence: Number(value(link, 'precedence')),
        operands: value(link, 'operands'),
        yields: value(link, 'yields'),
      });
    } else if (kind === 'unary') {
      map.unaries.set(id, { javascript: value(link, 'javascript'), rust: value(link, 'rust'), operand: value(link, 'operand'), yields: value(link, 'yields') });
    } else if (kind === 'builtin') {
      map.builtins.set(id, { javascript: value(link, 'javascript'), rust: value(link, 'rust'), arguments: values(link, 'arguments'), yields: value(link, 'yields') });
    } else if (kind === 'method') {
      map.methods.set(id, {
        javascript: value(link, 'javascript'),
        rust: value(link, 'rust'),
        receiver: value(link, 'receiver'),
        arguments: values(link, 'arguments'),
        yields: value(link, 'yields'),
      });
    } else if (kind === 'carried') {
      map.carried.set(id, textOf(link[2]));
    } else if (kind !== 'construct') {
      throw new Error(`${CONSTRUCTS_PATH}: unknown row kind ${kind}`);
    }
  }
  cached = map;
  return map;
}

/**
 * The first entry of `table` whose `language` spelling is `spelling`.
 * @param {Map<string, object>} table
 * @param {string} language 'javascript' or 'rust'
 * @param {string} spelling
 * @returns {string | undefined} the meta id
 */
export function idBySpelling(table, language, spelling) {
  for (const [id, row] of table) {
    if (row[language] === spelling) return id;
  }
  return undefined;
}

/**
 * Throw the refusal `slug` names: an Error carrying `slug` and the row's
 * `reason`, the way es_tokenizer.mjs carries its error kind (no classes).
 * @param {string} slug a `carried` row of the construct map
 * @param {string} [detail]
 * @returns {never}
 */
export function refuse(slug, detail) {
  const reason = constructMap().carried.get(slug);
  if (!reason) throw new Error(`no carried row ${slug} in ${CONSTRUCTS_PATH}`);
  throw Object.assign(new Error(detail ? `${reason}: ${detail}` : reason), { slug, reason });
}

/**
 * Whether `error` is a refusal `refuse` threw.
 * @param {unknown} error
 * @returns {boolean}
 */
export function isRefusal(error) {
  return Boolean(error && typeof error === 'object' && 'slug' in error && 'reason' in error);
}
