// Deterministic text -> Links Notation knowledge base
// (rust/src/agentic_coding/formalize.rs). Every record is
// `id\n  key "value"` produced by `format_lino_record`.

import { cached, readText } from './host.mjs';
import { Lexicon, TermKind, predicateUse } from './lexicon.mjs';
import { ConceptGraph } from './crate/formalization_concept_links.mjs';
import { emitNeeds } from './crate/formalization_needs.mjs';
import { sentences } from './crate/formalization_segment.mjs';
import { detect } from './crate/language.mjs';
import { formatLinoRecord } from './crate/links_format.mjs';
import { byteOrder, isAlphanumeric, trim, trimEnd, trimMatches } from './crate/rust_str.mjs';

/** Mirrors `CANONICAL_FISHERMAN_SYNOPSIS`'s embedded path. */
export const FISHERMAN_SYNOPSIS_PATH = 'data/agentic-coding/fisherman-synopsis.txt';

/** Mirrors `const CANONICAL_FISHERMAN_SYNOPSIS` (read through the host). */
export function canonicalFishermanSynopsis() {
  return cached('agentic-fisherman-synopsis', () => readText(FISHERMAN_SYNOPSIS_PATH));
}

/** Mirrors `const FISHERMAN_DOC_ID`. */
export const FISHERMAN_DOC_ID = 'tale:fisherman-and-fish';

/** Mirrors `const PRIMITIVE_KINDS`. */
export const PRIMITIVE_KINDS = Object.freeze([
  'concept', 'entity', 'predicate', 'assertion', 'procedure', 'context', 'temporal', 'modal', 'annotation',
]);

const COUNT_FIELDS = {
  concept: 'concepts', entity: 'entities', predicate: 'predicates', assertion: 'assertions', procedure: 'procedures',
  context: 'contexts', temporal: 'temporals', modal: 'modals', annotation: 'annotations',
};

/** Mirrors `FormalizationSummary::covers_all_nine`. */
export function coversAllNine(summary) {
  return summary.covered.length === PRIMITIVE_KINDS.length && summary.needs_raised === summary.needs_grounded;
}

/** Mirrors `FormalizationSummary::total_records` (+1 for the header record). */
export function totalRecords(summary) {
  return 1 + PRIMITIVE_KINDS.reduce((sum, kind) => sum + summary[COUNT_FIELDS[kind]], 0);
}

/** Mirrors `fn record_count`. */
function recordCount(summary, kind) {
  return Object.prototype.hasOwnProperty.call(COUNT_FIELDS, kind) ? summary[COUNT_FIELDS[kind]] : 0;
}

const sortedEntries = (map) => [...map.entries()].sort(([left], [right]) => byteOrder(left, right));

/**
 * Mirrors `fn formalize_text_to_links`: `{links_notation, summary}`.
 * @param {string} text
 * @param {string} docId
 */
export function formalizeTextToLinks(text, docId) {
  const lexicon = Lexicon.standard();
  const work = lexicon.bestWorkFor(text);
  const resolvedDocId = resolveDocId(docId, work);
  const segmented = segmentSentences(text);
  const language = detect(text);

  const annotations = [];
  const assertions = [];
  const usedEntities = new Map();
  const usedPredicates = new Map();
  const usedConcepts = new Map();
  const temporals = new Map();
  const modals = new Map();
  const preserved = [];

  segmented.forEach((sentence, index) => {
    const annotationId = `ann:${index}`;
    annotations.push({ id: annotationId, doc: resolvedDocId, start: sentence.start, end: sentence.end, text: sentence.text, language });
    const provenance = `${resolvedDocId}@${sentence.start}:${sentence.end}`;
    const triple = work === null ? null : work.extract(sentence.text);
    if (triple === null) {
      preserved.push(index);
      return;
    }
    if (triple.subject.kind === TermKind.Entity) usedEntities.set(triple.subject.id, triple.subject.label);
    else if (triple.subject.kind === TermKind.Concept) usedConcepts.set(triple.subject.id, triple.subject.label);
    usedPredicates.set(triple.predicate.id, triple.predicate.label);
    if (triple.object.kind === TermKind.Entity) usedEntities.set(triple.object.id, triple.object.label);
    else if (triple.object.kind === TermKind.Concept) usedConcepts.set(triple.object.id, triple.object.label);

    let timeRef = null;
    if (triple.predicate.time !== null) {
      const temporal = temporalFromExpression(triple.predicate.time);
      temporals.set(temporal.id, temporal);
      timeRef = temporal.id;
    }
    let modalRef = null;
    if (triple.predicate.modal !== null) {
      const modal = modalFromRaw(triple.predicate.modal);
      modals.set(modal.id, modal);
      modalRef = modal.id;
    }
    const context = work === null ? null : (triple.predicate.id === 'pred:remain' ? work.finalContext() : work.primaryContext());
    assertions.push({
      id: `a:${index}`,
      subject: triple.subject,
      predicate: predicateUse(triple.predicate),
      object: triple.object,
      time: timeRef,
      modal: modalRef,
      context,
      annotation: annotationId,
      provenance,
      natural_language: null,
    });
  });

  const conceptRecords = [];
  const procedureRecords = [];
  const contextRecords = [];
  if (work !== null) {
    for (const concept of work.concepts) {
      conceptRecords.push({ id: concept.id, label: concept.label, kind: concept.kind, source: `lexicon:${work.id}` });
    }
    procedureRecords.push(...work.procedures);
    contextRecords.push(...work.contexts);
  }
  for (const [id, label] of sortedEntries(usedConcepts)) {
    if (conceptRecords.some((concept) => concept.id === id)) continue;
    conceptRecords.push({ id, label, kind: 'extracted', source: resolvedDocId });
  }
  conceptRecords.sort((left, right) => byteOrder(left.id, right.id));

  const grounded = groundedGraph(conceptRecords, usedEntities);
  const byteSegments = sentences(text);
  const unread = preserved.map((index) => byteSegments[index]).filter((segment) => segment !== undefined);
  const raised = emitNeeds(resolvedDocId, unread, grounded, 0);

  const summary = {
    doc_id: resolvedDocId,
    concepts: conceptRecords.length,
    entities: usedEntities.size,
    predicates: usedPredicates.size,
    assertions: assertions.length,
    procedures: procedureRecords.length,
    contexts: contextRecords.length,
    temporals: temporals.size,
    modals: modals.size,
    annotations: annotations.length,
    covered: [],
    needs_raised: raised.length,
    needs_grounded: 0,
    max_depth_reached: 0,
  };

  let document = '';
  const push = (record) => {
    if (document) document += '\n';
    document += `${trimEnd(record)}\n`;
  };
  push(formatLinoRecord('knowledge_base', [
    ['id', resolvedDocId],
    ['source', work === null ? resolvedDocId : work.title],
    ['primitive_scheme', PRIMITIVE_KINDS.join(' ')],
    ['generator', 'formal-ai/agentic-coding/formalize@links-v1'],
    ['concepts', String(summary.concepts)],
    ['entities', String(summary.entities)],
    ['predicates', String(summary.predicates)],
    ['assertions', String(summary.assertions)],
    ['procedures', String(summary.procedures)],
    ['contexts', String(summary.contexts)],
    ['temporals', String(summary.temporals)],
    ['modals', String(summary.modals)],
    ['annotations', String(summary.annotations)],
    ['preserved_spans', String(preserved.length)],
  ]));
  for (const concept of conceptRecords) {
    push(formatLinoRecord('concept', [['id', concept.id], ['label', concept.label], ['type', concept.kind], ['source', concept.source]]));
  }
  for (const [id, label] of sortedEntries(usedEntities)) {
    push(formatLinoRecord('entity', [['id', id], ['label', label], ['source', resolvedDocId]]));
  }
  for (const [id, label] of sortedEntries(usedPredicates)) {
    push(formatLinoRecord('predicate', [['id', id], ['label', label], ['source', resolvedDocId]]));
  }
  for (const procedure of procedureRecords) {
    push(formatLinoRecord('procedure', [
      ['id', procedure.id], ['signature', procedure.signature], ['description', procedure.description],
      ['trigger', procedure.trigger], ['source', `lexicon:${resolvedDocId}`],
    ]));
  }
  for (const context of contextRecords) {
    push(formatLinoRecord('context', [['id', context.id], ['label', context.label], ['description', context.description]]));
  }
  for (const [, temporal] of sortedEntries(temporals)) {
    push(formatLinoRecord('temporal', [['id', temporal.id], ['expression', temporal.expression], ['kind', temporal.kind]]));
  }
  for (const [, modal] of sortedEntries(modals)) {
    push(formatLinoRecord('modal', [['id', modal.id], ['kind', modal.kind], ['degree', modal.degree]]));
  }
  for (const annotation of annotations) {
    push(formatLinoRecord('annotation', [
      ['id', annotation.id], ['doc', annotation.doc], ['span', `${annotation.start}:${annotation.end}`],
      ['text', annotation.text], ['language', annotation.language],
    ]));
  }
  for (const index of preserved) {
    const annotation = annotations[index];
    push(formatLinoRecord('preserved_span', [
      ['id', `preserved:${index}`], ['doc', annotation.doc], ['span', `${annotation.start}:${annotation.end}`],
      ['text', annotation.text], ['language', annotation.language], ['annotation', annotation.id],
    ]));
  }
  for (const assertion of assertions) {
    const pairs = [
      ['id', assertion.id],
      ['subject', assertion.subject.id],
      ['subject_kind', assertion.subject.kind],
      ['predicate', assertion.predicate.id],
      ['object', assertion.object.id],
      ['object_kind', assertion.object.kind],
    ];
    if (assertion.time !== null) pairs.push(['time', assertion.time]);
    if (assertion.modal !== null) pairs.push(['modal', assertion.modal]);
    if (assertion.context !== null) pairs.push(['context', assertion.context]);
    if (assertion.natural_language !== null) pairs.push(['natural_language', assertion.natural_language]);
    pairs.push(['annotation', assertion.annotation]);
    pairs.push(['provenance', assertion.provenance]);
    push(formatLinoRecord('assertion', pairs));
  }

  summary.covered = PRIMITIVE_KINDS.filter((kind) => recordCount(summary, kind) > 0);
  return { links_notation: document, summary };
}

/** Mirrors `fn resolve_doc_id`. */
function resolveDocId(requested, work) {
  const trimmed = trim(requested);
  if (trimmed) return trimmed;
  return work === null ? 'doc:input' : work.doc_id;
}

/**
 * Mirrors `fn segment_sentences`: the segmenter's byte spans as character spans.
 * @param {string} text
 */
function segmentSentences(text) {
  const boundaries = [];
  let offset = 0;
  for (const character of text) {
    boundaries.push(offset);
    offset += new TextEncoder().encode(character).length;
  }
  boundaries.push(offset);
  const characters = (byteOffset) => {
    const position = boundaries.findIndex((boundary) => boundary >= byteOffset);
    return position >= 0 ? position : Math.max(boundaries.length - 1, 0);
  };
  return sentences(text).map((segment) => ({ start: characters(segment.start), end: characters(segment.end), text: segment.text }));
}

/** Mirrors `fn grounded_graph`: the work's concepts and entities as a graph. */
function groundedGraph(concepts, entities) {
  return new ConceptGraph({
    concepts: concepts.map((concept) => ({
      id: concept.id, label: concept.label, language: '', gloss: '', genus: null, differentiae: [], structures: [],
      source_id: concept.source, source_url: '', sha256: '', license_name: '', depth: 0,
    })),
    entities: sortedEntries(entities).map(([id, label]) => ({ id, label, language: '', source_span: '' })),
  });
}

/** Mirrors `fn slugify`. @param {string} text */
function slugify(text) {
  let slug = '';
  let lastWasDash = false;
  for (const character of trim(text)) {
    if (isAlphanumeric(character)) {
      slug += character.toLowerCase();
      lastWasDash = false;
    } else if (!lastWasDash) {
      slug += '-';
      lastWasDash = true;
    }
  }
  return trimMatches(slug, (character) => character === '-');
}

/** Mirrors `Temporal::from_expression`. */
function temporalFromExpression(expression) {
  return { id: `temporal:${slugify(expression)}`, expression, kind: 'relative' };
}

/** Mirrors `Modal::from_raw`. */
function modalFromRaw(raw) {
  const at = raw.indexOf(':');
  const kind = at < 0 ? raw : raw.slice(0, at);
  const degree = at < 0 ? '' : raw.slice(at + 1);
  return { id: `modal:${slugify(kind)}`, kind, degree };
}

/**
 * Mirrors `fn coverage_line`: the covered primitive kinds, comma-separated.
 * @param {{covered: Array<string>}} summary
 */
export function coverageLine(summary) {
  return summary.covered.join(', ');
}
