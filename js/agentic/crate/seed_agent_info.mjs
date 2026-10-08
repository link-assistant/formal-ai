// `crate::seed::agent_info` (rust/src/seed.rs): the key/value fields of
// data/seed/agent-info.lino.

import { cached, childrenNamed, childValue, parseLino, readText } from '../host.mjs';

/**
 * Mirrors `fn agent_info` in rust/src/seed.rs: a Map of field id to value.
 * @returns {Map<string, string>}
 */
export function agentInfo() {
  return cached('agent-info', () => {
    const out = new Map();
    const root = parseLino(readText('data/seed/agent-info.lino')).children?.[0];
    for (const entry of childrenNamed(root, 'field')) {
      if (entry.id) out.set(entry.id, childValue(entry, 'value'));
    }
    return out;
  });
}

/**
 * Mirrors `fn agent_info_value` in rust/src/seed.rs: the field value, or null.
 * @param {string} key
 */
export function agentInfoValue(key) {
  return agentInfo().has(key) ? agentInfo().get(key) : null;
}
