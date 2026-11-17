// Dependency summarization (R1188-U21): a summary chosen by what the
// statements of a text depend on, with no language model. The Rust twin is
// rust/src/summarization/dependency.rs.
//
// 1. The text is formalized into statements (text_formalization.mjs).
// 2. A statement whose formal content an earlier statement already holds is a
//    duplicate and is dropped. Only a checked relation certifies that
//    (statement_relation.mjs): the same formal identity, the same ordered
//    relations, or the conjuncts of an asserted conjunctive statement.
//    Operand order, connectives, quantifiers, negation scope and explicit
//    counts keep statements apart (issue #1487).
// 3. The statement graph: a later statement depends on an earlier one when it
//    mentions the earlier statement's subject, or shares at least
//    `ELABORATION_SHARED_TERMS` terms with it (it elaborates it). Shared
//    terms are retrieval: they relate statements, they never certify one.
// 4. The kept core: the roots (statements that depend on nothing earlier)
//    that something depends on, then the most depended-on statements, up to
//    one statement in `KEEP_DIVISOR`, in text order.
// 5. Each kept statement is deformalized with the surfaces it was read from,
//    so the summary stays in the text's own words; a fragment that ends its
//    sentence in the summary is closed with that sentence's terminator.
//
// Representation: an entry is `{statement, sentence, closing}` (the index of
// its sentence and that sentence's closing punctuation, or ''); a node adds
// `dependsOn` and `dependents`, lists of node indexes; a summary is
// `{statements, duplicates, removed, kept, text}` where `removed` lists the
// duplicate entries and `kept` lists node indexes.

import { byteOrder } from './rust_str.mjs';
import { contentIds, formalizeSentenceSegments, formalizeSentences, joinSurfaces, Polarity, roleWordsIn, tokens } from './text_formalization.mjs';
import { sentences } from './formalization_segment.mjs';
import { quoteFault, quotedSegmentSpans } from './normal_markov.mjs';
import { certifiesRestatement, checkedRelation } from './statement_relation.mjs';

/** Mirrors `ELABORATION_SHARED_TERMS` in rust/src/summarization/dependency.rs: shared terms that make one statement elaborate another. */
export const ELABORATION_SHARED_TERMS = 2;

/** Mirrors `KEEP_DIVISOR` in rust/src/summarization/dependency.rs: the summary keeps at most one statement in this many. */
export const KEEP_DIVISOR = 3;

const WORD_CHARACTER = /^[\p{L}\p{M}\p{N}]$/u;

/** Mirrors `fn closing_of` in rust/src/summarization/dependency.rs: the last character of a sentence when it is punctuation, else ''. */
export function closingOf(sentence) {
  const last = Array.from(sentence).pop();
  return last === undefined || WORD_CHARACTER.test(last) ? '' : last;
}

/**
 * Mirrors `fn sentence_statements` in rust/src/summarization/dependency.rs: every statement of `text` with the index
 * of the sentence it came from and that sentence's closing punctuation.
 * @param {string} text
 * @param {string} language
 */
export function sentenceStatements(text, language) {
  return sentenceEntries(formalizeSentences(text, language));
}

function sentenceEntries(formalized) {
  const out = [];
  formalized.forEach((sentence, index) => {
    const closing = closingOf(sentence.text);
    for (const statement of sentence.statements) out.push({ statement, sentence: index, closing });
  });
  return out;
}

/**
 * Mirrors `fn restates` in rust/src/summarization/dependency.rs: `later` adds nothing to `earlier`, a checked
 * relation finds it equivalent to or entailed by `earlier` (never a bare term overlap).
 */
export function restates(later, earlier) {
  return certifiesRestatement(checkedRelation(later, earlier));
}

/**
 * Mirrors `fn split_duplicates` in rust/src/summarization/dependency.rs: the statements no earlier kept statement
 * restates, and the ones that restate one (the duplicates removed), each in text order.
 */
export function splitDuplicates(entries) {
  const unique = [];
  const removed = [];
  for (const entry of entries) {
    if (unique.some((earlier) => restates(entry.statement, earlier.statement))) {
      removed.push(entry);
      continue;
    }
    unique.push(entry);
  }
  return { unique, removed };
}

/**
 * Mirrors `fn without_duplicates` in rust/src/summarization/dependency.rs: the statements no earlier kept statement
 * restates, and how many were dropped.
 */
export function withoutDuplicates(entries) {
  const { unique, removed } = splitDuplicates(entries);
  return { unique, duplicates: removed.length };
}

/** Mirrors `fn shared_terms` in rust/src/summarization/dependency.rs: how many distinct term ids two statements share. */
export function sharedTerms(left, right) {
  const held = new Set(contentIds(right));
  return contentIds(left).filter((id) => held.has(id)).length;
}

/**
 * Mirrors `fn depends_on` in rust/src/summarization/dependency.rs: `later` mentions `earlier`'s subject, or shares
 * enough terms with it to elaborate it.
 */
export function dependsOn(later, earlier) {
  if (earlier.subject !== null && contentIds(later).includes(earlier.subject.id)) return true;
  return sharedTerms(later, earlier) >= ELABORATION_SHARED_TERMS;
}

/**
 * Mirrors `fn statement_graph` in rust/src/summarization/dependency.rs: for every statement, the earlier statements
 * it depends on and the later ones that depend on it.
 */
export function statementGraph(entries) {
  const nodes = entries.map((entry) => ({ ...entry, dependsOn: [], dependents: [] }));
  nodes.forEach((node, later) => {
    for (let earlier = 0; earlier < later; earlier += 1) {
      if (!dependsOn(node.statement, nodes[earlier].statement)) continue;
      node.dependsOn.push(earlier);
      nodes[earlier].dependents.push(later);
    }
  });
  return nodes;
}

/** Mirrors `fn keep_budget` in rust/src/summarization/dependency.rs: one statement in `KEEP_DIVISOR`, at least one. @param {number} count @returns {number} */
export function keepBudget(count) {
  return count === 0 ? 0 : Math.max(1, Math.ceil(count / KEEP_DIVISOR));
}

/**
 * Mirrors `fn kept_core` in rust/src/summarization/dependency.rs: the indexes of the kept statements, in text order.
 * Roots something depends on come first, then the rest, each group by
 * dependents (most first) and text order.
 */
export function keptCore(nodes) {
  const byDependents = (left, right) => (nodes[right].dependents.length - nodes[left].dependents.length) || (left - right);
  const indexes = nodes.map((_, index) => index);
  const roots = indexes.filter((index) => nodes[index].dependsOn.length === 0 && nodes[index].dependents.length > 0);
  const rest = indexes.filter((index) => !roots.includes(index));
  const ranked = [...roots.sort(byDependents), ...rest.sort(byDependents)];
  return ranked.slice(0, keepBudget(nodes.length)).sort((left, right) => left - right);
}

/**
 * Mirrors `fn render_kept` in rust/src/summarization/dependency.rs: the kept statements in their own words, a
 * fragment that ends its sentence here closed with the sentence's punctuation.
 */
export function renderKept(nodes, kept) {
  const pieces = kept.map((index, position) => {
    const node = nodes[index];
    const next = position + 1 < kept.length ? nodes[kept[position + 1]] : null;
    const endsSentence = next === null || next.sentence !== node.sentence;
    const text = node.statement.text;
    return endsSentence && node.closing !== '' && !text.endsWith(node.closing) ? `${text}${node.closing}` : text;
  });
  return joinSurfaces(pieces);
}

/**
 * Mirrors `fn summarize_by_dependency` in rust/src/summarization/dependency.rs: the kept core of `text` and its
 * rendering in the text's own words.
 * @param {string} text
 * @param {string} language
 */
export function summarizeByDependency(text, language) {
  return summarizeEntries(sentenceStatements(text, language));
}

function summarizeEntries(entries) {
  const { unique, removed } = splitDuplicates(entries);
  const nodes = statementGraph(unique);
  const kept = keptCore(nodes);
  return {
    statements: nodes,
    duplicates: removed.length,
    removed,
    kept,
    text: renderKept(nodes, kept),
  };
}

const PHATIC_ROLES = ['social_greeting', 'social_farewell', 'social_courtesy_response', 'text-summary-phatic'];

function renderCausalSentence(statement, causePattern, causes) {
  const { text } = statement;
  if (!causePattern || statement.polarity === Polarity.Denied || statement.dialogue.question
      || /[()[\]{}]/u.test(text)
      || quoteFault(text) !== null || quotedSegmentSpans(text).length > 0) return text;
  const [cause, repeated] = text.matchAll(causePattern);
  if (!cause || repeated || cause.index === 0 || !causes.has(cause[1].toLowerCase().replace(/\s+/gu, ' '))) return text;
  const main = text.slice(0, cause.index);
  const reasons = text.slice(cause.index + cause[0].length);
  return `${main}: ${reasons}`;
}

/** Rank complete substantive sentences so a selection keeps its governed clauses. */
export function summarizeFreeText(text, language) {
  const source = sentences(text);
  const phatic = new Set(PHATIC_ROLES.flatMap((role) => [...roleWordsIn(role, language)]));
  const content = source.filter((sentence) => !phatic.has(
    tokens(sentence.text).map((token) => token.surface.toLowerCase()).join(' '),
  ));
  const summary = summarizeEntries(sentenceEntries(formalizeSentenceSegments(content, language, true)));
  if (summary.kept.length === 0) return null;

  const causes = roleWordsIn('text-summary-cause', language);
  const cues = [...roleWordsIn('text-summary-cause-phrase', language), ...causes].map((cue) => cue.split(' ')
    .map((word) => word.replace(/[.*+?^${}()|[\]\\]/gu, '\\$&')).join('\\s+'));
  const causePattern = cues.length ? new RegExp(`[\\s,]+(${cues.join('|')})\\s+`, 'giu') : null;
  summary.text = joinSurfaces(summary.kept.map((index) => renderCausalSentence(summary.statements[index].statement, causePattern, causes)));
  return source.length === 1 && summary.text === source[0].text ? null : summary;
}

/**
 * Mirrors `fn key_facts` in rust/src/summarization/dependency.rs: the gold facts of a text, the distinct term ids of
 * the first sentence that yields a statement (its subject and the facts that
 * define it).
 */
export function keyFacts(text, language) {
  const entries = sentenceStatements(text, language);
  if (entries.length === 0) return [];
  const first = entries[0].sentence;
  const ids = entries.filter((entry) => entry.sentence === first).flatMap((entry) => contentIds(entry.statement));
  return [...new Set(ids)].sort(byteOrder);
}

/** Mirrors `fn retained_facts` in rust/src/summarization/dependency.rs: how many of `facts` the kept statements still hold. */
export function retainedFacts(summary, facts) {
  const held = new Set(summary.kept.flatMap((index) => contentIds(summary.statements[index].statement)));
  return facts.filter((id) => held.has(id)).length;
}
