// The shell steps and file templates a work item ends with, read from
// data/meta/work-item-steps.lino (rust/src/agentic_coding/work_item_steps.rs).

import { cached, readText } from './host.mjs';
import { rustLines } from './content.mjs';

const STEPS_FILE = 'data/meta/work-item-steps.lino';

/** Mirrors `fn unquote`. */
function unquote(value) {
  const chars = Array.from(value);
  if (chars.length >= 2 && chars[0] === chars[chars.length - 1] && (chars[0] === '"' || chars[0] === "'")) {
    return value.slice(1, -1);
  }
  return value;
}

/** Mirrors `fn template`. @param {string} key @returns {string|null} */
export function template(key) {
  const lines = cached('work-item-steps', () => rustLines(readText(STEPS_FILE)));
  for (const line of lines) {
    if (!line.startsWith('  ')) continue;
    const rest = line.slice(2);
    const space = rest.indexOf(' ');
    if (space < 0) continue;
    if (rest.slice(0, space) !== key) continue;
    return unquote(rest.slice(space + 1).trim()).replaceAll('\\n', '\n').replaceAll('\\t', '\t');
  }
  return null;
}

/**
 * Mirrors `fn fill`: the template under `key` with each `[placeholder, value]`
 * replaced in order. Throws when the ledger lacks the key.
 * @param {string} key
 * @param {Array<[string, string]>} values
 */
export function fill(key, values) {
  let text = template(key);
  if (text === null) throw new Error(`${STEPS_FILE} lacks ${key}`);
  for (const [placeholder, value] of values) text = text.split(placeholder).join(value);
  return text;
}
