// A usage-weighted, associative persistence store for meta-language
// expressions: the part of rust/src/associative_persistence.rs
// (`AssociativeMemory`, `PersistedExpression`, `from_memory_events`,
// `recall_related`, `retention_score`, `retention_ranking`) that
// `agentic_coding::learning_report` reaches. The embedded
// `SubstitutionGraph` (rust/src/substitution.rs) is a set of directed
// `from -> to` links, kept here as a `Set` of keys.

import { agenticMessage } from '../messages.mjs';
import { stableId } from './engine_stable_id.mjs';
import { byteOrder } from './rust_str.mjs';

const LINK_SEPARATOR = '\u0000';

/** Mirrors `struct AssociativeMemory` in rust/src/associative_persistence.rs. */
export class AssociativeMemory {
  /** Mirrors `AssociativeMemory::new`. */
  constructor() {
    /** `from\0to` keys of the association graph. */
    this.associations = new Set();
    /** id -> `PersistedExpression` (iterate through `expressions()` for `BTreeMap` order). */
    this.expressions_ = new Map();
  }

  /** Mirrors `AssociativeMemory::persist`: the content-addressed id. */
  persist(text) {
    const id = stableId('expression', text);
    this.persistIdentified(id, text);
    return id;
  }

  /** Mirrors `AssociativeMemory::persist_identified`. */
  persistIdentified(id, text) {
    const existing = this.expressions_.get(id);
    if (existing) {
      existing.text = text;
      existing.writes += 1;
      return;
    }
    this.expressions_.set(id, { id, text, reads: 0, writes: 1, qualifiers: new Map(), validation_issues: [] });
  }

  /** Mirrors `AssociativeMemory::note_read`. */
  noteRead(id) {
    const expression = this.expressions_.get(id);
    if (!expression) return false;
    expression.reads += 1;
    return true;
  }

  /** Mirrors `AssociativeMemory::associate`. */
  associate(from, to) {
    if (from === to || !this.expressions_.has(from) || !this.expressions_.has(to)) return false;
    const key = `${from}${LINK_SEPARATOR}${to}`;
    if (this.associations.has(key)) return false;
    this.associations.add(key);
    return true;
  }

  /** Mirrors `SubstitutionGraph::links`: `{from, to}` pairs. */
  links() {
    return [...this.associations].map((key) => {
      const [from, to] = key.split(LINK_SEPARATOR);
      return { from, to };
    });
  }

  /** Mirrors `AssociativeMemory::contains`. */
  contains(id) {
    return this.expressions_.has(id);
  }

  /** Mirrors `AssociativeMemory::get`: the expression or null. */
  get(id) {
    return this.expressions_.get(id) ?? null;
  }

  /** Mirrors `AssociativeMemory::expressions`: `[id, expression]` pairs in id order. */
  expressions() {
    return [...this.expressions_.entries()].sort(([left], [right]) => byteOrder(left, right));
  }

  /** Mirrors `AssociativeMemory::len`. */
  len() {
    return this.expressions_.size;
  }

  /** Mirrors `AssociativeMemory::out_degree`. */
  outDegree(id) {
    return this.links().filter((link) => link.from === id).length;
  }

  /** Mirrors `AssociativeMemory::in_degree`. */
  inDegree(id) {
    return this.links().filter((link) => link.to === id).length;
  }

  /** Mirrors `AssociativeMemory::recall_related`: breadth-first, each recalled id counted as read. */
  recallRelated(id, maxHops) {
    if (!this.contains(id)) return [];
    const queue = [[id, 0]];
    const seen = new Set();
    const recalled = [];
    while (queue.length) {
      const [current, hops] = queue.shift();
      if (seen.has(current)) continue;
      seen.add(current);
      this.noteRead(current);
      recalled.push(current);
      if (hops >= maxHops) continue;
      const neighbors = [];
      for (const link of this.links()) {
        if (link.from === current) neighbors.push(link.to);
        else if (link.to === current) neighbors.push(link.from);
      }
      const unique = [...new Set(neighbors)].sort(byteOrder);
      for (const neighbor of unique) queue.push([neighbor, hops + 1]);
    }
    return recalled;
  }

  /** Mirrors `AssociativeMemory::retention_score` (uniform weights). */
  retentionScore(id) {
    const expression = this.expressions_.get(id);
    if (!expression) return 0;
    return expression.reads + expression.writes + this.inDegree(id) + this.outDegree(id);
  }

  /** Mirrors `AssociativeMemory::retention_score_map`. */
  retentionScoreMap() {
    const scores = new Map(this.expressions().map(([id, expression]) => [id, expression.reads + expression.writes]));
    for (const link of this.links()) {
      if (scores.has(link.from)) scores.set(link.from, scores.get(link.from) + 1);
      if (scores.has(link.to)) scores.set(link.to, scores.get(link.to) + 1);
    }
    return scores;
  }

  /** Mirrors `AssociativeMemory::retention_scores`: `{id, score}`, most retained first. */
  retentionScores() {
    return [...this.retentionScoreMap()].map(([id, score]) => ({ id, score }))
      .sort((left, right) => (right.score - left.score) || byteOrder(left.id, right.id));
  }

  /** Mirrors `AssociativeMemory::retention_ranking`. */
  retentionRanking() {
    return this.retentionScores().map((scored) => scored.id);
  }

  /**
   * Mirrors `AssociativeMemory::from_memory_events`.
   * @param {Array<object>} events `MemoryEvent`s (js/agentic/crate/memory.mjs)
   */
  static fromMemoryEvents(events) {
    const memory = new AssociativeMemory();
    for (const event of events) {
      if (event.id === '') continue;
      memory.persistIdentified(event.id, eventExpressionText(event));
      const expression = memory.expressions_.get(event.id);
      expression.reads = event.access_count;
      expression.writes = Math.max(event.write_count, 1);
      addQualifier(expression.qualifiers, 'kind', event.kind);
      addQualifier(expression.qualifiers, 'role', event.role);
      addQualifier(expression.qualifiers, 'intent', event.intent);
      addQualifier(expression.qualifiers, 'tool', event.tool);
      addQualifier(expression.qualifiers, 'sent_at', event.sent_at);
      addQualifier(expression.qualifiers, 'conversation_id', event.conversation_id);
      addQualifier(expression.qualifiers, 'conversation_title', event.conversation_title);
    }
    const identified = events.filter((event) => event.id !== '');
    const references = (reference, id) => reference === id
      || (reference.endsWith(id) && reference.slice(0, reference.length - id.length).endsWith(':'));
    for (const source of identified) {
      const searchable = eventExpressionText(source);
      for (const target of identified) {
        if (source.id === target.id) continue;
        const explicitlyLinked = source.evidence.some((reference) => references(reference, target.id));
        if (explicitlyLinked || searchable.includes(target.id)) memory.associate(source.id, target.id);
      }
      for (const reference of source.evidence) {
        const resolved = identified.some((target) => references(reference, target.id));
        if (!resolved) memory.expressions_.get(source.id).validation_issues.push(agenticMessage('associative_persistence_unresolved_evidence', { reference }));
      }
    }
    return memory;
  }
}

/** Mirrors `fn add_qualifier` in rust/src/associative_persistence.rs (a `BTreeMap` insert). */
function addQualifier(qualifiers, name, value) {
  if (value !== null && value !== undefined && value !== '') qualifiers.set(name, value);
}

/** The qualifiers of an expression as sorted `[name, value]` pairs: Rust built-in `BTreeMap::iter` order. */
export function sortedQualifiers(expression) {
  return [...expression.qualifiers.entries()].sort(([left], [right]) => byteOrder(left, right));
}

/** Mirrors `fn event_expression_text` in rust/src/associative_persistence.rs. */
function eventExpressionText(event) {
  return [event.kind, event.role, event.intent, event.tool, event.inputs, event.outputs, event.content,
    event.conversation_title, event.demo_label]
    .filter((value) => value !== null && value !== undefined)
    .concat(event.evidence)
    .join('\n');
}
