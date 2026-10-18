// The server's wording, read from data/meta/server-messages.lino.
//
// Every sentence a response carries (error messages, network labels) is data,
// keyed by a language-neutral id, so this code holds no prose (R1010) and the
// Rust literals it mirrors are pinned in one reviewable file.

import { childValue, childrenNamed, parseLino, readRepoFile } from './lino.mjs';

export const MESSAGES_FILE = 'data/meta/server-messages.lino';

let table = null;

function messages() {
  if (table) return table;
  table = new Map();
  for (const node of childrenNamed(parseLino(readRepoFile(MESSAGES_FILE)), 'message')) {
    table.set(node.value, childValue(node, 'text'));
  }
  return table;
}

/**
 * The text for `key`, with `{name}` placeholders filled from `params`.
 * An unknown key renders as the key itself, like the Rust `config` lookup.
 * @param {string} key
 * @param {Record<string, unknown>} [params]
 * @returns {string}
 */
export function serverMessage(key, params = {}) {
  const template = messages().has(key) ? messages().get(key) : key;
  return template.replace(/\{([a-z_]+)\}/g, (whole, name) =>
    Object.prototype.hasOwnProperty.call(params, name) ? String(params[name]) : whole,
  );
}

/** Every message key, for tests. @returns {Array<string>} */
export function serverMessageKeys() {
  return [...messages().keys()];
}
