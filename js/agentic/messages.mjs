// The planner's wording, read from data/meta/agentic-messages.lino.
//
// The Rust planner (rust/src/agentic_coding/) still spells some sentences
// inline with `format!`; their JavaScript twins keep no prose (R1010,
// `scripts/check-js-literals.mjs` holds js/agentic/ at zero), so each such
// sentence is a keyed template here, copied byte for byte from the Rust
// literal it mirrors. `{name}` is a placeholder.

import { cached, childValue, childrenNamed, parseLino, readText } from './host.mjs';

export const MESSAGES_FILE = 'data/meta/agentic-messages.lino';

function table() {
  return cached('agentic-messages', () => {
    const out = new Map();
    for (const node of childrenNamed(parseLino(readText(MESSAGES_FILE)), 'message')) {
      out.set(node.value, childValue(node, 'text'));
    }
    return out;
  });
}

/**
 * The template under `key` with `{name}` placeholders filled from `params`.
 * Throws for an unknown key: the file ships with the code, so a missing key
 * is a defect of this repository, not of a request.
 * Rust built-in `format!`: the Rust planner spells each sentence inline.
 * @param {string} key
 * @param {Record<string, unknown>} [params]
 * @returns {string}
 */
export function agenticMessage(key, params = {}) {
  if (!table().has(key)) throw new Error(`${MESSAGES_FILE} lacks ${key}`);
  return table().get(key).replace(/\{([a-z_]+)\}/g, (whole, name) =>
    Object.prototype.hasOwnProperty.call(params, name) ? String(params[name]) : whole,
  );
}
