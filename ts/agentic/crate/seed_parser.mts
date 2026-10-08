// `crate::seed::parser::parse_lino` root shape.
//
// The Rust parser always returns a synthetic, unnamed root whose children are
// the document's top-level nodes; the shared JavaScript parser collapses a
// document with a single top-level node onto that node. `parseRoot` restores
// the Rust shape so `tree.children.first()` reads the same node in both.

import { parseLino } from '../host.mjs';

/** Mirrors `parse_lino`: always the unnamed root. @param {string} text */
export function parseRoot(text) {
  const tree = parseLino(text);
  if (tree.indent === -1 || (tree.name === '' && tree.value === '' && tree.indent === undefined)) return tree;
  return { name: '', id: '', value: '', children: [tree], indent: -1 };
}

/** Mirrors `LinoNode::find_child_value`. */
export function findChildValue(node, name) {
  return (node?.children || []).find((child) => child.name === name)?.value ?? '';
}
