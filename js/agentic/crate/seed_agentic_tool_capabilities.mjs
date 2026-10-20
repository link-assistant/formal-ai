// `crate::seed::agentic_tool_capabilities` (rust/src/seed/agentic_tool_capabilities.rs):
// the shared alias registry of data/seed/agentic-tool-capabilities.lino.
// An entry is `{id, aliases, command_aliases, cues}`.

import { lower as canonicalUnaryLower } from './rust_str.mjs';
import { cached, readText } from '../host.mjs';
import { splitPipeList } from './seed.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';

/** Mirrors `fn agentic_tool_capabilities`. */
export function agenticToolCapabilities() {
  return cached('agentic-tool-capabilities', () => {
    const root = parseRoot(readText('data/seed/agentic-tool-capabilities.lino')).children[0];
    if (!root) return [];
    return (root.children || []).filter((node) => node.name === 'capability').map((node) => {
      const group = (node.children || []).find((child) => child.name === 'cues');
      const cues = (group ? group.children || [] : [])
        .flatMap((language) => splitPipeList(language.value))
        .map(canonicalUnaryLower);
      return {
        id: node.value,
        aliases: splitPipeList(findChildValue(node, 'aliases')),
        command_aliases: splitPipeList(findChildValue(node, 'command_aliases')),
        cues,
      };
    });
  });
}
