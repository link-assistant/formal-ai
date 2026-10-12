#!/usr/bin/env node
// Refresh the Wikidata P2370 capture both runtimes read at answer time
// (issue #1176 R1176-1).
//
// data/seed/wikidata-conversion-to-si.lino keeps, for every unit grounded in
// data/seed/meanings-units.lino, its item, the amount of its P2370
// ("conversion to SI unit") statement and the SI unit item it converts into.
// The unit converter derives each linear factor from two of these amounts and
// cites the items, so the factor is Wikidata's, not hand-typed. This script
// re-reads every listed item through the wbgetentities API and reports (or,
// with --write, records) any amount or target that moved, plus the date.
//
// Usage:
//   node scripts/refresh-wikidata-conversion-to-si.mjs           # report drift
//   node scripts/refresh-wikidata-conversion-to-si.mjs --write   # rewrite the capture

import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const CAPTURE = join(dirname(fileURLToPath(import.meta.url)), '..', 'data', 'seed', 'wikidata-conversion-to-si.lino');
const API = 'https://www.wikidata.org/w/api.php';

/**
 * The P2370 statement of one entity: its amount (leading `+` dropped) and the
 * Q-id of its unit, or null when the entity states none.
 * @param {object} entity wbgetentities entity
 * @returns {{amount: string, target: string}|null}
 */
function conversionToSi(entity) {
  const claims = (entity && entity.claims && entity.claims.P2370) || [];
  const preferred = claims.find((claim) => claim.rank === 'preferred') || claims.find((claim) => claim.rank === 'normal');
  const value = preferred && preferred.mainsnak && preferred.mainsnak.datavalue && preferred.mainsnak.datavalue.value;
  if (!value || typeof value.amount !== 'string') return null;
  const target = String(value.unit || '').split('/').pop();
  return { amount: value.amount.replace(/^\+/u, ''), target: target };
}

async function main(argv) {
  const write = argv.includes('--write');
  const text = readFileSync(CAPTURE, 'utf8');
  const items = [...text.matchAll(/^ {4}item (Q\d+)$/gmu)].map((match) => match[1]);
  const url = `${API}?action=wbgetentities&format=json&props=claims&ids=${items.join('|')}`;
  const response = await fetch(url, { headers: { 'user-agent': 'formal-ai capture refresh (https://github.com/link-assistant/formal-ai)' } });
  if (!response.ok) throw new Error(`wbgetentities answered ${response.status}`);
  const entities = (await response.json()).entities || {};
  let updated = text;
  const drift = [];
  for (const item of items) {
    const statement = conversionToSi(entities[item]);
    if (statement === null) {
      drift.push(`${item}: no P2370 statement`);
      continue;
    }
    const row = new RegExp(`( {4}item ${item}\\n {4}amount )([^\\n]+)(\\n {4}target )(Q\\d+)`, 'u');
    const match = updated.match(row);
    if (!match) continue;
    if (match[2] !== statement.amount || match[4] !== statement.target) {
      drift.push(`${item}: amount ${match[2]} -> ${statement.amount}, target ${match[4]} -> ${statement.target}`);
      updated = updated.replace(row, `$1${statement.amount}$3${statement.target}`);
    }
  }
  const today = new Date().toISOString().slice(0, 10);
  if (write) {
    updated = updated.replace(/^ {2}fetched \S+$/mu, `  fetched ${today}`).replace(/captured \d{4}-\d{2}-\d{2}/u, `captured ${today}`);
    writeFileSync(CAPTURE, updated);
  }
  console.log(drift.length === 0 ? `P2370 capture matches Wikidata for ${items.length} items` : drift.join('\n'));
  return drift.length === 0 || write ? 0 : 1;
}

main(process.argv.slice(2)).then((code) => {
  process.exitCode = code;
}, (error) => {
  console.error(String(error && error.message ? error.message : error));
  process.exitCode = 2;
});
