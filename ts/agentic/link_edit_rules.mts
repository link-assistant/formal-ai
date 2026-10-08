// Structural edits applied through a links network
// (rust/src/agentic_coding/link_edit_rules.rs, issue #1085).
//
// native-only: rust/src/agentic_coding/link_edit_rules.rs apply_link_edit
// (feature "meta-language"); the JavaScript runtime has no meta-language CST
// engine, so `applyLinkEdit` always reports `engine_unavailable` -- the
// behaviour of a Rust build without that feature -- and
// `insertMembersViaLinks` therefore returns null, leaving structured edits on
// their byte-level path.

import { readText } from './host.mjs';
import { lines, trim, trimMatches, trimStart } from './write_str.mjs';

/** Mirrors `LinkEditRule::name`. A rule is `{kind, ...fields}`. */
export function ruleName(rule) {
  return rule.kind;
}

/** Mirrors `fn rule_shapes`. */
export function ruleShapes() {
  return parseRuleShapes(readText('data/meta/link-edit-rules.lino'));
}

/** Mirrors `fn parse_rule_shapes`. */
export function parseRuleShapes(text) {
  const shapes = [];
  for (const line of lines(text)) {
    const trimmed = trim(line);
    if (trimmed.startsWith('rule ')) {
      shapes.push({ rule: trim(trimmed.slice(5)), node_kinds: [], anchor_kind: null, list_kind: null, element_kind: null });
      continue;
    }
    const shape = shapes[shapes.length - 1];
    if (!shape) continue;
    if (trimmed.startsWith('node_kind ')) shape.node_kinds.push(trim(trimmed.slice('node_kind '.length)));
    else if (trimmed.startsWith('anchor_kind ')) shape.anchor_kind = trim(trimmed.slice('anchor_kind '.length));
    else if (trimmed.startsWith('list_kind ')) shape.list_kind = trim(trimmed.slice('list_kind '.length));
    else if (trimmed.startsWith('element_kind ')) shape.element_kind = trim(trimmed.slice('element_kind '.length));
  }
  return shapes;
}

/**
 * Mirrors `fn apply_link_edit` without the meta-language engine.
 * @returns {{ok: [string, object]}|{error: {kind: string}}}
 */
export function applyLinkEdit(_source, _language, _rule) {
  return { error: { kind: 'engine_unavailable' } };
}

/** Mirrors `fn insert_members_via_links`: `[text, inserted]` or null. */
export function insertMembersViaLinks(source, list, values) {
  let text = source;
  const inserted = [];
  for (const value of values) {
    const result = applyLinkEdit(text, 'rust', { kind: 'insert_member', list, member: value });
    if (result.ok) {
      [text] = result.ok;
      inserted.push(value);
    } else if (result.error.kind !== 'member_present') return null;
  }
  return [text, inserted];
}

/** Mirrors `fn parse_rule_document`: `{ok: RuleDocument}` or `{error: {kind: 'malformed_rule', reason}}`. */
export function parseRuleDocument(text) {
  let leaf = '';
  let path = '';
  let language = 'rust';
  const expect = [];
  let ruleNameText = '';
  const fields = [];
  for (const raw of lines(text)) {
    const trimmed = trimStart(raw);
    if (trimmed === '' || trimmed.startsWith('#') || trimmed === 'link_edit') continue;
    const indent = new TextEncoder().encode(raw).length - new TextEncoder().encode(trimmed).length;
    const space = trimmed.indexOf(' ');
    const name = space < 0 ? trimmed : trimmed.slice(0, space);
    const value = trimMatches(space < 0 ? '' : trim(trimmed.slice(space + 1)), (character) => character === '"');
    if (indent === 2 && name === 'leaf') leaf = value;
    else if (indent === 2 && name === 'path') path = value;
    else if (indent === 2 && name === 'language') language = value;
    else if (indent === 2 && name === 'expect') expect.push(value);
    else if (indent === 2 && name === 'rule') ruleNameText = value;
    else if (indent === 4) fields.push([name, value]);
  }
  const malformed = (reason) => ({ error: { kind: 'malformed_rule', reason } });
  if (path === '') return malformed('path');
  const field = (name) => fields.find(([candidate]) => candidate === name)?.[1] ?? null;
  const pairs = { insert_member: ['list', 'member'], replace_literal: ['old', 'new'], rename_identifier: ['old', 'new'] }[ruleNameText];
  if (!pairs) return malformed(ruleNameText);
  const rule = { kind: ruleNameText };
  for (const key of pairs) {
    const value = field(key);
    if (value === null) return malformed(key);
    rule[key] = value;
  }
  return { ok: { leaf, path, language, rule, expect } };
}
