// Repository history as formal context: a port of rust/src/history_context.rs
// with its commits.rs and items.rs halves (issue #1180 R11).
//
// The pure core every root shares: the rules seed
// (data/seed/history-formalization.lino) read into the same tables, the
// `git log` record splitter, the trailer, co-author and merge scans, the
// named top-level item diff, and the commit-to-memory-event mapping. The
// subprocess half (running git and the store and cursor files) stays with the
// native importer; a JavaScript caller hands the `git log` text and the blobs
// in. The github-logs importers are js/agentic/crate/history_github.mjs and the
// lineage route's claim and answer are js/agentic/crate/history_lineage.mjs.

import { cached, readText } from '../host.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';

const SEED_PATH = 'data/seed/history-formalization.lino';
const NUMBER_PLACEHOLDER = '%number%';
const RECORD_SEPARATOR = '\u001e';
const UNIT_SEPARATOR = '\u001f';

/** The non-empty values of `node`'s children named `name`. */
function childValues(node, name) {
  return (node?.children || [])
    .filter((child) => child.name === name && child.value)
    .map((child) => child.value);
}

/**
 * Mirrors `HistoryRules::from_seed_text`: the rows sit under the
 * `history_formalization` root, so they are read one level down.
 * @param {string} text
 */
export function historyRulesFromSeedText(text) {
  const rules = {
    records: [],
    trailers: [],
    merge: { pattern: '', evidence: '', conversation: '' },
    pathEvidencePrefix: '',
    symbolEvidencePrefix: '',
    censusSuffixes: [],
    esSuffixes: [],
    coauthorPrefix: '',
    coauthorEvidencePrefix: '',
    itemKeywords: new Map(),
    itemSpanKeywords: new Map(),
    itemModifiers: [],
    stateTransitions: [],
    stateEvidencePrefix: '',
    requirement: { marker: '', leads: [], evidencePrefix: '' },
    lineageCues: [],
    repositoryQa: {
      statusCues: [],
      statusScript: '',
      statusLedger: '',
      statusTagMatch: '',
      definitionCues: [],
      censusDir: '',
      sourceRoot: '',
      docPrefix: '',
      attributePrefix: '',
    },
  };
  const nodes = [];
  for (const top of parseRoot(text).children || []) {
    nodes.push(top);
    for (const child of top.children || []) nodes.push(child);
  }
  for (const node of nodes) {
    if (node.name === 'record') {
      const kind = findChildValue(node, 'kind');
      if (!kind) continue;
      rules.records.push({
        kind,
        idPrefix: findChildValue(node, 'id_prefix'),
        role: findChildValue(node, 'role'),
        content: findChildValue(node, 'content'),
        timestamp: findChildValue(node, 'timestamp'),
      });
    } else if (node.name === 'trailer') {
      const pattern = findChildValue(node, 'pattern');
      if (!pattern) continue;
      rules.trailers.push({
        pattern,
        evidence: findChildValue(node, 'evidence'),
        conversation: findChildValue(node, 'conversation'),
      });
    } else if (node.name === 'merge') {
      rules.merge = {
        pattern: findChildValue(node, 'pattern'),
        evidence: findChildValue(node, 'evidence'),
        conversation: findChildValue(node, 'conversation'),
      };
    } else if (node.name === 'path_evidence') {
      rules.pathEvidencePrefix = findChildValue(node, 'prefix');
    } else if (node.name === 'symbol_evidence') {
      rules.symbolEvidencePrefix = findChildValue(node, 'prefix');
    } else if (node.name === 'coauthor_trailer') {
      rules.coauthorPrefix = findChildValue(node, 'prefix');
      rules.coauthorEvidencePrefix = findChildValue(node, 'evidence_prefix');
    } else if (node.name === 'state_transition') {
      const state = findChildValue(node, 'state');
      const fields = childValues(node, 'field');
      if (state && fields.length > 0) rules.stateTransitions.push({ state, fields });
    } else if (node.name === 'state_evidence') {
      rules.stateEvidencePrefix = findChildValue(node, 'prefix');
    } else if (node.name === 'requirement_statement') {
      rules.requirement = {
        marker: findChildValue(node, 'marker'),
        leads: childValues(node, 'lead'),
        evidencePrefix: findChildValue(node, 'evidence_prefix'),
      };
    } else if (node.name === 'lineage_cue') {
      const phrases = childValues(node, 'phrase');
      if (phrases.length > 0) rules.lineageCues.push([findChildValue(node, 'language'), phrases]);
    } else if (node.name === 'status_cue' || node.name === 'definition_cue') {
      const phrases = childValues(node, 'phrase');
      const cues = node.name === 'status_cue' ? rules.repositoryQa.statusCues : rules.repositoryQa.definitionCues;
      if (phrases.length > 0) cues.push([findChildValue(node, 'language'), phrases]);
    } else if (node.name === 'status_rule') {
      rules.repositoryQa.statusScript = findChildValue(node, 'script');
      rules.repositoryQa.statusLedger = findChildValue(node, 'ledger');
      rules.repositoryQa.statusTagMatch = findChildValue(node, 'tag_match');
    } else if (node.name === 'definition_source') {
      rules.repositoryQa.censusDir = findChildValue(node, 'census');
      rules.repositoryQa.sourceRoot = findChildValue(node, 'source_root');
      rules.repositoryQa.docPrefix = findChildValue(node, 'doc_prefix');
      rules.repositoryQa.attributePrefix = findChildValue(node, 'attribute_prefix');
    } else if (node.name === 'item_modifier' && node.value) {
      rules.itemModifiers.push(node.value);
    } else if (node.name === 'source') {
      const source = findChildValue(node, 'id');
      const suffixes = childValues(node, 'applies_to');
      if (source === 'ast_census') rules.censusSuffixes = suffixes;
      if (source === 'es_meta_extract') rules.esSuffixes = suffixes;
      const keywords = childValues(node, 'item_keyword');
      if (keywords.length > 0) rules.itemKeywords.set(source, keywords);
      const spanKeywords = childValues(node, 'item_span_keyword');
      if (spanKeywords.length > 0) rules.itemSpanKeywords.set(source, spanKeywords);
    }
  }
  return rules;
}

/** Mirrors `HistoryRules::load`: the registered seed's rules. */
export function historyRules() {
  return cached('history-formalization-rules', () => historyRulesFromSeedText(readText(SEED_PATH)));
}

/**
 * Mirrors `parse_log_output`: `git log --format=<LOG_FORMAT> --name-only`
 * text into raw commits, chronological.
 * @param {string} text
 */
export function parseLogOutput(text) {
  const commits = [];
  for (const chunk of String(text).split(RECORD_SEPARATOR)) {
    const fields = chunk.split(UNIT_SEPARATOR);
    if (fields.length < 6) continue;
    commits.push({
      sha: fields[0].trim(),
      author: fields[1].trim(),
      committerDate: fields[2].trim(),
      subject: fields[3].trim(),
      body: fields[4].trimEnd(),
      changedPaths: fields[5].split('\n').map((line) => line.trim()).filter((line) => line.length > 0),
    });
  }
  return commits.reverse();
}

/** Mirrors `pattern_number`: the `%number%` a pattern captures, or null. */
export function patternNumber(haystack, pattern) {
  const at = pattern.indexOf(NUMBER_PLACEHOLDER);
  if (at === -1) return null;
  const prefix = pattern.slice(0, at);
  const suffix = pattern.slice(at + NUMBER_PLACEHOLDER.length);
  let from = 0;
  for (;;) {
    const found = haystack.indexOf(prefix, from);
    if (found === -1) return null;
    const rest = haystack.slice(found + prefix.length);
    const digits = /^[0-9]*/u.exec(rest)[0];
    if (digits.length > 0 && rest.slice(digits.length).startsWith(suffix)) return digits;
    from = found + Math.max(prefix.length, 1);
  }
}

/**
 * Mirrors `state_transitions` (rust/src/history_context/statements.rs): the
 * dated lifecycle transitions of a captured issue or pull request,
 * chronological, as `<prefix><state>@<timestamp>` evidence.
 * @param {object} value the captured JSON record
 * @param {object} rules
 */
export function stateTransitions(value, rules) {
  const dated = [];
  for (const rule of rules.stateTransitions) {
    const field = rule.fields.find((name) => typeof value?.[name] === 'string');
    const stamp = field === undefined ? '' : value[field];
    if (stamp) dated.push([stamp, rule.state]);
  }
  // Array.prototype.sort is stable, as Rust's sort_by_key is.
  dated.sort((left, right) => (left[0] < right[0] ? -1 : left[0] > right[0] ? 1 : 0));
  return dated.map(([stamp, state]) => `${rules.stateEvidencePrefix}${state}@${stamp}`);
}

const STATEMENT_SEPARATORS = [' ', '.', ':', ')', '*'];

/**
 * Mirrors `requirement_statements`: each body line that opens (after any
 * seeded lead) with the marker, a number and a separator is one statement.
 * @param {string} body
 * @param {object} rules
 */
export function requirementStatements(body, rules) {
  const rule = rules.requirement;
  if (!rule.marker) return [];
  const statements = [];
  for (const line of String(body).split('\n')) {
    let rest = line.trim();
    for (;;) {
      const lead = rule.leads.find((candidate) => candidate && rest.startsWith(candidate));
      if (lead === undefined) break;
      rest = rest.slice(lead.length).trimStart();
    }
    if (!rest.startsWith(rule.marker)) continue;
    const afterMarker = rest.slice(rule.marker.length);
    const digits = /^[0-9]*/u.exec(afterMarker)[0];
    if (!digits) continue;
    const tail = afterMarker.slice(digits.length);
    if (tail && !STATEMENT_SEPARATORS.includes(tail[0])) continue;
    let start = 0;
    while (start < tail.length && STATEMENT_SEPARATORS.includes(tail[start])) start += 1;
    let end = tail.length;
    while (end > start && tail[end - 1] === '*') end -= 1;
    const statement = tail.slice(start, end).trim();
    if (!statement) continue;
    statements.push(`${rule.evidencePrefix}${rule.marker}${digits} ${statement}`);
  }
  return statements;
}

/** Mirrors `coauthors`: co-author names from the trailer lines. */
export function coauthors(body, rules) {
  const prefix = rules.coauthorPrefix.toLowerCase();
  if (prefix.length === 0) return [];
  const names = [];
  for (const line of String(body).split('\n').flatMap((part) => part.split('\\n'))) {
    const trimmed = line.trim();
    if (trimmed.slice(0, prefix.length).toLowerCase() !== prefix) continue;
    const value = trimmed.slice(prefix.length).trim();
    const name = value.split('<')[0].trim();
    if (name.length > 0 && !names.includes(name)) names.push(name);
  }
  return names;
}

/** Mirrors `bare_word`. */
function bareWord(token) {
  return token.split(/[(<:]/u)[0];
}

/** Mirrors `item_header`: the `keyword name` label a top-level line opens. */
function itemHeader(line, keywords, spanKeywords, modifiers) {
  if (line.length === 0 || /^\s/u.test(line)) return null;
  const tokens = line.split(/\s+/u).filter((token) => token.length > 0);
  const isKeyword = (token) => keywords.includes(bareWord(token));
  for (let index = 0; index < tokens.length; index += 1) {
    const word = bareWord(tokens[index]);
    if (spanKeywords.includes(word)) {
      const header = line.split(/[{;]/u)[0].split(/\s+/u).filter((token) => token.length > 0).join(' ');
      const start = header.indexOf(word);
      return start === -1 ? null : header.slice(start);
    }
    if (isKeyword(tokens[index])) {
      const next = tokens[index + 1];
      if (next === undefined) return null;
      if (isKeyword(next)) continue;
      const name = /^[\p{L}\p{N}_$]*/u.exec(next.replace(/^\*+/u, ''))[0];
      return name.length > 0 ? `${word} ${name}` : null;
    }
    if (!modifiers.includes(word)) return null;
  }
  return null;
}

/** Mirrors `named_items`: label → body text, in label order. */
export function namedItems(source, census, rules) {
  const keywords = rules.itemKeywords.get(census) || [];
  const spanKeywords = rules.itemSpanKeywords.get(census) || [];
  const items = new Map();
  let current = null;
  for (const line of String(source).split('\n')) {
    const label = itemHeader(line, keywords, spanKeywords, rules.itemModifiers);
    if (label !== null) {
      if (current !== null) items.set(current[0], current[1]);
      current = [label, ''];
    }
    if (current !== null) current[1] += `${line}\n`;
  }
  if (current !== null) items.set(current[0], current[1]);
  return new Map([...items].sort(([left], [right]) => (left < right ? -1 : Number(left > right))));
}

/** Mirrors `diff_named_items`: added (+1), removed (-1), modified (0). */
export function diffNamedItems(path, census, before, after, rules) {
  const old = namedItems(before, census, rules);
  const next = namedItems(after, census, rules);
  const change = (item, delta) => ({ path, source: census, item: `${path}#${item}`, delta });
  const changes = [];
  for (const item of [...next.keys()].sort()) {
    if (!old.has(item)) changes.push(change(item, 1));
    else if (old.get(item) !== next.get(item)) changes.push(change(item, 0));
  }
  for (const item of [...old.keys()].sort()) {
    if (!next.has(item)) changes.push(change(item, -1));
  }
  return changes;
}

/**
 * Mirrors `formalize_commit`: one raw commit as a memory event.
 * @param {object} raw a `parseLogOutput` record
 * @param {Array<object>} symbols the symbol changes of its paths
 * @param {string|null} merge the delivering merge commit's subject
 * @param {object} rules
 */
export function formalizeCommit(raw, symbols, merge, rules) {
  const rule = rules.records.find((record) => record.kind === 'commit') || rules.records[0];
  const evidence = raw.changedPaths.map((path) => `${rules.pathEvidencePrefix}${path}`);
  const push = (value) => {
    if (value.length > 0 && !evidence.includes(value)) evidence.push(value);
  };
  for (const change of symbols) push(`${rules.symbolEvidencePrefix}${change.item}`);
  let conversation = null;
  for (const trailer of rules.trailers) {
    const number = patternNumber(raw.body, trailer.pattern);
    if (number === null) continue;
    push(trailer.evidence.split(NUMBER_PLACEHOLDER).join(number));
    if (conversation === null && trailer.conversation) {
      conversation = trailer.conversation.split(NUMBER_PLACEHOLDER).join(number);
    }
  }
  for (const name of coauthors(raw.body, rules)) push(`${rules.coauthorEvidencePrefix}${name}`);
  if (merge !== null && merge !== undefined) {
    const number = patternNumber(merge, rules.merge.pattern);
    if (number !== null) push(rules.merge.evidence.split(NUMBER_PLACEHOLDER).join(number));
  }
  return {
    id: `${rule.idPrefix}${raw.sha}`,
    kind: rule.kind,
    role: rule.role,
    content: raw.body.length === 0 ? raw.subject : `${raw.subject}\n\n${raw.body}`,
    sentAt: raw.committerDate,
    conversationId: conversation,
    evidence,
  };
}
