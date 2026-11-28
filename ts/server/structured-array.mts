import { lookupConceptQuery } from '../agentic/crate/concepts_lookup.mjs';
import { cached } from '../agentic/host.mjs';
import { agenticMessage } from '../agentic/messages.mjs';
import { schemaReference } from './structured-schema.mjs';

const BULLET = /^(?:[-*•–—]|[0-9]+[.)])[ \t]+(.+)$/u;
const WORDS = /[ \t]+/u;
const NAME = /^(?:[.\p{Lu}\p{Lt}\p{N}][\p{L}\p{M}\p{N}_.+#'-]*)(?:[ \t]+[.\p{Lu}\p{Lt}\p{N}][\p{L}\p{M}\p{N}_.+#'-]*)*$/u;
const QUOTES = [['"', '"'], ["'", "'"], ['«', '»'], ['“', '”'], ['\x60', '\x60']];

function grammar() {
  return cached('structured-array-grammar', () => {
    const pattern = key => new RegExp(agenticMessage('structured-array-' + key), 'iu');
    return {
      separator: pattern('separator'), label: pattern('label'), lead: pattern('lead'), tail: pattern('tail'),
      topics: agenticMessage('structured-array-topics').split(';').map(group => group.split('|')),
    };
  });
}

function allowsType(rule, type) {
  return rule?.type === type || (Array.isArray(rule?.type) && rule.type.includes(type));
}

export function stringArrayItems(field, root) {
  if (!allowsType(field, 'array')) return undefined;
  const item = schemaReference(root, field.items?.$ref) ?? field.items;
  if (allowsType(item, 'string') || (item?.type === undefined && Array.isArray(item?.enum)
      && item.enum.every(value => typeof value === 'string'))) return item;
  return undefined;
}

function clauses(source, rules) {
  const result = [];
  let heading = '';
  let bullets;
  for (const line of source.split(/[\r\n]+/u)) {
    const bullet = BULLET.exec(line.trim());
    if (bullet) {
      if (!bullets) {
        bullets = [];
        result.push({ context: heading, labeled: Boolean(heading), items: bullets });
      }
      bullets.push(bullet[1]);
      continue;
    }
    heading = '';
    bullets = undefined;
    const label = rules.label.exec(line.trim());
    if (label) {
      const labeled = { context: label[1], labeled: true, text: label[2] };
      const literal = literalArray(labeled);
      if (literal) {
        result.push({ ...labeled, literal });
        continue;
      }
    }
    for (const clause of line.split(/[.!?]+[ \t]+|;/u)) {
      const text = clause.trim();
      const label = rules.label.exec(text);
      if (label) {
        heading = label[2] ? '' : label[1];
        if (label[2]) result.push({ context: label[1], labeled: true, text: label[2] });
      } else if (text) {
        heading = '';
        const lead = rules.lead.exec(text);
        const cropped = lead ? text.slice(lead.index + lead[0].length) : text;
        const clean = cropped.replace(/[.!?]+$/u, '');
        const body = clean.replace(rules.tail, '');
        const before = lead ? text.slice(0, lead.index + lead[0].length) : '';
        const after = clean.slice(body.length);
        result.push({ context: before + ' ' + after, scoped: Boolean(lead || after), text: body });
      }
    }
  }
  return result;
}

function containsPhrase(text, part) {
  return part && (' ' + text + ' ').includes(' ' + part + ' ');
}

function topicalRank(context, name, description, groups, normalizeKey) {
  if (!context) return 0;
  const label = normalizeKey(context);
  if (label === name) return 4;
  if (containsPhrase(label, name)) return 3;
  if (containsPhrase(description, label) || containsPhrase(label, description)) return 2;
  const words = label.split(' ');
  if (words.some(word => word.length >= 4 && containsPhrase(description, word))) return 1;
  return groups.some(group => group.some(word => words.includes(word))
    && group.some(word => containsPhrase(name, word) || containsPhrase(description, word))) ? 1 : 0;
}

function trimItem(text) {
  const item = text.trim().replace(/[.!?]+$/u, '').trimEnd();
  for (const [start, end] of QUOTES) {
    if (item.startsWith(start) && item.endsWith(end)) return item.slice(1, -1).trim();
  }
  return item;
}

const labelKey = value => value.trim().split(WORDS).join(' ').toLowerCase();

function exactLabel(item, labels) {
  if (labels.includes(item)) return item;
  const matches = labels.filter(label => labelKey(label) === labelKey(item));
  return matches.length === 1 ? matches[0] : undefined;
}

function edgeItem(text, edge, known) {
  if (known(text)) return text;
  const tokens = text.split(WORDS);
  for (let size = Math.min(tokens.length - 1, 6); size > 0; size -= 1) {
    const part = (edge === 0 ? tokens.slice(-size) : tokens.slice(0, size)).join(' ');
    if (known(part)) return part;
  }
  return text;
}

function candidateItems(candidate, rules, recognized) {
  if (candidate.items) return candidate.items.map(trimItem).filter(Boolean);
  const { text, labeled, scoped } = candidate;
  const items = text.split(rules.separator).map(trimItem).filter(Boolean);
  if (items.length < 2) return undefined;
  if (labeled) return items;
  items[0] = edgeItem(items[0], 0, recognized);
  items[items.length - 1] = edgeItem(items.at(-1), 1, recognized);
  return scoped || items.every(item => !/\s/u.test(item) || NAME.test(item) || recognized(item))
    ? items : undefined;
}

function literalArray(candidate) {
  if (candidate.labeled && candidate.text?.startsWith('[')) {
    try {
      const value = JSON.parse(candidate.text.trimEnd().replace(/[.!?]+$/u, ''));
      if (Array.isArray(value)) return value;
    } catch { /* Continue with prose extraction. */ }
  }
  return undefined;
}

// A missing or ambiguous list is distinct from an explicitly supplied empty array.
export function stringArrayEvidence(source, name, description, rule, normalizeKey) {
  const rules = grammar();
  const labels = (rule.enum ?? []).filter(label => typeof label === 'string');
  const key = normalizeKey(name);
  const described = typeof description === 'string' ? normalizeKey(description) : '';
  const knownItems = new Map();
  const known = item => {
    if (!knownItems.has(item)) {
      knownItems.set(item, Boolean(lookupConceptQuery({ term: item, context: null })));
    }
    return knownItems.get(item);
  };
  let answer;
  let bestTopic = -1;
  let bestLinks = -1;
  const recognized = item => known(item) || exactLabel(item, labels) !== undefined;
  for (const candidate of clauses(source, rules)) {
    const topic = topicalRank(candidate.context, key, described, rules.topics, normalizeKey);
    if ((candidate.labeled && !topic) || topic < bestTopic) continue;
    const literal = candidate.literal ?? literalArray(candidate);
    const items = literal ?? candidateItems(candidate, rules, recognized);
    if (!items) continue;
    const value = Array.isArray(rule.enum) && !literal
      ? items.map(item => exactLabel(item, labels)).filter(item => item !== undefined) : items;
    if (!value.length && !literal) continue;
    const links = value.filter(item => typeof item === 'string' && known(item)).length;
    if (topic > bestTopic || links > bestLinks) {
      bestTopic = topic;
      bestLinks = links;
      answer = value;
    } else if (links === bestLinks && JSON.stringify(value) !== JSON.stringify(answer)) {
      answer = undefined;
    }
  }
  return answer;
}
