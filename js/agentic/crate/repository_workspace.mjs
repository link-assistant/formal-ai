// Named repository-protocol templates (rust/src/repository_workspace/mod.rs).
// Only `render_protocol_template` is ported; the protocol runner stays native.

import { readText } from '../host.mjs';
import { findChildValue, parseLinoRoot } from '../write_lino.mjs';

/** Mirrors `fn render_protocol_template_from`. */
export function renderProtocolTemplateFrom(document, id, values) {
  const node = parseLinoRoot(document).children.find((candidate) => candidate.name === 'repository_template' && candidate.id === id);
  if (!node) return null;
  return values.reduce((rendered, [name, value]) => rendered.split(`{${name}}`).join(value), findChildValue(node, 'text'));
}

/**
 * Mirrors `fn render_protocol_template`.
 * @param {string} id
 * @param {Array<[string, string]>} values
 */
export function renderProtocolTemplate(id, values) {
  return renderProtocolTemplateFrom(readText('data/meta/repository-workspace-protocol.lino'), id, values);
}
