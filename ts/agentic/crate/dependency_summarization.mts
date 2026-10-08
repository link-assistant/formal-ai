// Dependency summarization (R1188-U21): a summary chosen by what the
// statements of a text depend on, with no language model. The Rust twin is
// rust/src/summarization/dependency.rs.
//
// 1. The text is formalized into statements (text_formalization.mjs).
// 2. A statement whose formal content an earlier statement already holds is a
//    duplicate and is dropped. The comparison is by formal identity: the same
//    polarity, subject and terms, or a term set inside an earlier one's,
//    whatever the wording.
// 3. The statement graph: a later statement depends on an earlier one when it
//    mentions the earlier statement's subject, or shares at least
//    `ELABORATION_SHARED_TERMS` terms with it (it elaborates it).
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
// `{statements, duplicates, kept, text}` where `kept` lists node indexes.

import { compareStrings } from './summarization_dedup.mjs';
import { contentIds, formalizeSentences, joinSurfaces, statementIdentity } from './text_formalization.mjs';

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
  const out = [];
  formalizeSentences(text, language).forEach((sentence, index) => {
    const closing = closingOf(sentence.text);
    for (const statement of sentence.statements) out.push({ statement, sentence: index, closing });
  });
  return out;
}

/** Every member of `inner` is in `outer` (the Rust twin of `restates` checks it inline). */
function isSubset(inner, outer) {
  const held = new Set(outer);
  return inner.every((id) => held.has(id));
}

/**
 * Mirrors `fn restates` in rust/src/summarization/dependency.rs: `later` adds nothing to `earlier`, it has the same
 * formal identity or its terms all appear in `earlier`'s with the same polarity.
 */
export function restates(later, earlier) {
  if (statementIdentity(later) === statementIdentity(earlier)) return true;
  const ids = contentIds(later);
  return ids.length > 0 && later.polarity === earlier.polarity && isSubset(ids, contentIds(earlier));
}

/**
 * Mirrors `fn without_duplicates` in rust/src/summarization/dependency.rs: the statements no earlier kept statement
 * restates, and how many were dropped.
 */
export function withoutDuplicates(entries) {
  const unique = [];
  let duplicates = 0;
  for (const entry of entries) {
    if (unique.some((earlier) => restates(entry.statement, earlier.statement))) {
      duplicates += 1;
      continue;
    }
    unique.push(entry);
  }
  return { unique, duplicates };
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

/** Mirrors `fn keep_budget` in rust/src/summarization/dependency.rs: one statement in `KEEP_DIVISOR`, at least one. */
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
  const entries = sentenceStatements(text, language);
  const { unique, duplicates } = withoutDuplicates(entries);
  const nodes = statementGraph(unique);
  const kept = keptCore(nodes);
  return {
    statements: nodes,
    duplicates,
    kept,
    text: renderKept(nodes, kept),
  };
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
  return [...new Set(ids)].sort(compareStrings);
}

/** Mirrors `fn retained_facts` in rust/src/summarization/dependency.rs: how many of `facts` the kept statements still hold. */
export function retainedFacts(summary, facts) {
  const held = new Set(summary.kept.flatMap((index) => contentIds(summary.statements[index].statement)));
  return facts.filter((id) => held.has(id)).length;
}
