// Everything one formalization grounded, plus everything it could not
// (rust/src/formalization/concept_links.rs).

import { emitNeeds, satisfy } from './formalization_needs.mjs';
import { clauses, sentences } from './formalization_segment.mjs';
import { conceptFromSense, conceptSenseContentId, relationsIn } from './formalization_concepts.mjs';
import { stableId } from './engine_stable_identifier.mjs';
import { pushLinoNode } from './links_format.mjs';
import { NeedState, needLinksNotation } from './needs.mjs';
import { byteOrder, eqIgnoreAsciiCase, trimEnd } from './rust_str.mjs';

const sortedUnique = (values) => [...new Set(values)].sort(byteOrder);

/** Mirrors `struct ConceptGraph` and its methods. */
export class ConceptGraph {
  constructor(fields = {}) {
    this.doc_id = fields.doc_id ?? '';
    this.concepts = fields.concepts ?? [];
    this.relations = fields.relations ?? [];
    this.procedures = fields.procedures ?? [];
    this.entities = fields.entities ?? [];
    this.needs = fields.needs ?? [];
    this.segments = fields.segments ?? [];
  }

  /** Mirrors `ConceptGraph::identity`. */
  identity() {
    const structures = this.unresolved().length === 0 ? this.structureIds() : [];
    const relations = sortedUnique(this.relations.map((relation) => relation.kind));
    const shapes = this.procedures.map((procedure) => String(procedure.steps.length)).sort(byteOrder);
    return stableId('concept_graph', `structures=${structures.join(',')};relations=${relations.join(',')};procedure_shapes=${shapes.join(',')}`);
  }

  /** Mirrors `ConceptGraph::to_links_notation`. */
  toLinksNotation() {
    let out = '';
    out = pushLinoNode(out, 0, 'concept_graph', this.identity());
    out = pushLinoNode(out, 2, 'document', this.doc_id);
    for (const concept of [...this.concepts].sort((left, right) => byteOrder(left.id, right.id))) {
      out = pushLinoNode(out, 2, 'concept', concept.id);
      out = pushLinoNode(out, 4, 'label', concept.label);
      out = pushLinoNode(out, 4, 'language', concept.language);
      out = pushLinoNode(out, 4, 'gloss', concept.gloss);
      if (concept.genus !== null && concept.genus !== undefined) out = pushLinoNode(out, 4, 'genus', concept.genus);
      for (const differentia of concept.differentiae) out = pushLinoNode(out, 4, 'differentia', differentia);
      for (const structure of concept.structures) out = pushLinoNode(out, 4, 'structure', structure);
      out = pushLinoNode(out, 4, 'source', concept.source_id);
      out = pushLinoNode(out, 4, 'source_url', concept.source_url);
      out = pushLinoNode(out, 4, 'sha256', concept.sha256);
      out = pushLinoNode(out, 4, 'license', concept.license_name);
      out = pushLinoNode(out, 4, 'depth', String(concept.depth));
    }
    for (const relation of [...this.relations].sort((left, right) => byteOrder(left.id, right.id))) {
      out = pushLinoNode(out, 2, 'relation', relation.id);
      out = pushLinoNode(out, 4, 'kind', relation.kind);
      out = pushLinoNode(out, 4, 'subject', relation.subject);
      out = pushLinoNode(out, 4, 'object', relation.object);
      out = pushLinoNode(out, 4, 'cue', relation.cue);
      out = pushLinoNode(out, 4, 'source_span', relation.source_span);
    }
    for (const need of [...this.needs].sort((left, right) => byteOrder(left.need_id, right.need_id))) {
      out += needLinksNotation(need);
    }
    return trimEnd(out);
  }

  /** Mirrors `ConceptGraph::structure_ids`. */
  structureIds() {
    return sortedUnique(this.concepts.flatMap((concept) => concept.structures));
  }

  /** Mirrors `ConceptGraph::grounds`. @param {string} surface */
  grounds(surface) {
    return this.concepts.some((concept) => eqIgnoreAsciiCase(concept.label, surface))
      || this.entities.some((entity) => eqIgnoreAsciiCase(entity.label, surface));
  }

  /** Mirrors `ConceptGraph::unresolved`. */
  unresolved() {
    return this.needs.filter((need) => need.state !== NeedState.Satisfied);
  }

  /** Mirrors `ConceptGraph::grounded_ratio`: `[grounded, total]`. */
  groundedRatio() {
    return [this.needs.filter((need) => need.state === NeedState.Satisfied).length, this.needs.length];
  }

  /** Mirrors `ConceptGraph::max_depth_reached`. */
  maxDepthReached() {
    return this.needs.reduce((max, need) => Math.max(max, need.depth), 0);
  }
}

/** Recursively ground retrieved senses with the native visited-need and depth rules. */
export function formalizeDeeply(text, docId, lookup, bounds, maxConceptDepth) {
  if (!Number.isSafeInteger(maxConceptDepth) || maxConceptDepth < 0) throw new RangeError('invalid_concept_depth');
  const graph = new ConceptGraph({ doc_id: docId, segments: sentences(text) });
  let pending = emitNeeds(docId, graph.segments, graph, 0);
  const seen = new Set(pending.map(need => need.need_id));
  while (pending.length) {
    const satisfaction = satisfy(pending, lookup, bounds, maxConceptDepth);
    for (const need of satisfaction.needs) {
      const index = graph.needs.findIndex(existing => existing.need_id === need.need_id);
      if (index >= 0) graph.needs[index] = need;
      else graph.needs.push(need);
    }
    const next = [];
    for (const sense of satisfaction.senses) {
      const parent = graph.needs.find(need => need.state === NeedState.Satisfied
        && eqIgnoreAsciiCase(need.subject, sense.surface));
      if (!parent) continue;
      const concept = conceptFromSense({ ...sense, depth: parent.depth });
      if (concept && !graph.concepts.some(existing => existing.id === concept.id)) graph.concepts.push(concept);
      if (parent.depth >= maxConceptDepth) continue;
      const senseDoc = conceptSenseContentId(sense);
      for (const need of emitNeeds(senseDoc, sentences(sense.gloss), graph, parent.depth + 1)) {
        need.raised_by = parent.need_id;
        if (!seen.has(need.need_id)) { seen.add(need.need_id); next.push(need); }
      }
    }
    pending = next;
  }
  graph.concepts.sort((left, right) => byteOrder(left.id, right.id));
  graph.needs.sort((left, right) => (left.depth - right.depth)
    || byteOrder(left.source_span, right.source_span)
    || byteOrder(left.need_id, right.need_id));
  const relations = new Map(graph.segments.flatMap(clauses).flatMap(clause => relationsIn(clause, graph))
    .map(relation => [relation.id, relation]));
  graph.relations = [...relations.values()].sort((left, right) => byteOrder(left.id, right.id));
  return graph;
}
