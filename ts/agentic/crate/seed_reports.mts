// The English report templates of the multilingual responses seed
// (rust/src/seed/reports.rs).

import { cached } from '../host.mjs';
import { multilingualResponses } from './seed.mjs';

function reports() {
  return cached('write:seed_reports', () => {
    const map = new Map();
    for (const record of multilingualResponses()) {
      if (record.language === 'en' && !map.has(record.intent)) map.set(record.intent, record.text);
    }
    return map;
  });
}

/**
 * Mirrors `fn report_text` in rust/src/seed/reports.rs: the English template
 * for `intent` with its named slots filled, or the intent name itself.
 * @param {string} intent
 * @param {Array<[string, string]>} values
 */
export function reportText(intent, values) {
  const template = reports().get(intent);
  return template === undefined ? intent : fillSlots(template, values);
}

/** Mirrors `fn fill_slots` in rust/src/seed/reports.rs. */
export function fillSlots(template, values) {
  let out = '';
  let rest = template;
  for (let open = rest.indexOf('{'); open >= 0; open = rest.indexOf('{')) {
    out += rest.slice(0, open);
    const after = rest.slice(open + 1);
    const close = after.indexOf('}');
    const found = close >= 0 ? values.find(([slot]) => slot === after.slice(0, close)) : undefined;
    if (found) {
      out += found[1];
      rest = after.slice(close + 1);
    } else {
      out += '{';
      rest = after;
    }
  }
  return out + rest;
}
