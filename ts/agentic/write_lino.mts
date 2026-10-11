// `crate::seed::parser::parse_lino` tree shape for the write-side modules
// (agent "write" helper; rust/src/seed/parser.rs).
//
// The shared JavaScript parser (host.mjs `parseLino`) returns a document's
// only top-level node itself, while the Rust parser always returns a nameless
// root whose children are the top-level nodes. Code ported from Rust that
// walks `root.children` reads through `parseLinoRoot` so it sees Rust's shape.

import { parseLino } from './host.mjs';

/** Mirrors `fn parse_lino`: a nameless root over the top-level nodes. */
export function parseLinoRoot(text) {
  const parsed = parseLino(text);
  if (parsed && parsed.name) return { name: '', id: '', value: '', children: [parsed] };
  return parsed ?? { name: '', id: '', value: '', children: [] };
}

/** Mirrors `LinoNode::find_child_value`: the first `name` child's id, or ''. */
export function findChildValue(node, name) {
  const child = (node?.children || []).find((candidate) => candidate.name === name);
  return child ? String(child.id ?? child.value ?? '') : '';
}
