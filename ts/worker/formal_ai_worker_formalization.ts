// Issue #1138 plan 04 browser mirror of the deep formalizer. The browser uses
// the same registry-driven concept lookup as the native path. It can preserve
// and attribute a procedure, but every browser step stays explicitly
// unverified because this runtime has no command executor.

async function formalizeDeeply(text, docId = "doc:input", preferences = {}, maxConceptDepth = 1) {
  const source = String(text || "");
  const language = detectLanguage(source);
  const graph = {
    docId: String(docId || "doc:input"),
    concepts: [],
    relations: [],
    procedures: [],
    entities: [],
    needs: [],
    segments: formalizationSegments(source),
    identity: "",
  };
  const seenNeeds = new Set();
  const seenConcepts = new Set();
  const pending = [];
  function queueNeeds(segments, raisedBy, sourceDocId, depth) {
    for (const segment of segments) for (const span of formalizationUnknownSpans(segment.text)) {
      if (graph.concepts.some((concept) => normalizePrompt(concept.label) === normalizePrompt(span.surface))) {
        continue;
      }
      const segmentLanguage = detectLanguage(segment.text);
      const needId = conceptStableId("need", `concept|${span.surface}|${segmentLanguage}`);
      if (seenNeeds.has(needId)) continue;
      seenNeeds.add(needId);
      pending.push({
        needId,
        kind: "concept",
        subject: span.surface,
        language: segmentLanguage,
        raisedBy,
        sourceSpan: `${sourceDocId}@${segment.start + span.start}:${segment.start + span.end}`,
        depth,
        state: "open",
        satisfiedBy: null,
        outcomes: [],
      });
    }
  }
  queueNeeds(graph.segments, graph.docId, graph.docId, 0);
  while (pending.length > 0) {
    const need = pending.shift();
    if (need.depth > maxConceptDepth) {
      need.state = "unsatisfiable";
      need.outcomes.push({ sourceId: "recursion_bound", status: "bounded", detail: String(maxConceptDepth) });
      graph.needs.push(need);
      continue;
    }
    // eslint-disable-next-line no-await-in-loop -- lookup order is part of the evidence trace.
    const lookup = await lookupConceptSurface(need.subject, need.language, preferences);
    need.outcomes = lookup.outcomes;
    if (lookup.items.length > 0) {
      need.state = "satisfied";
      need.satisfiedBy = lookup.items[0].contentId;
      for (const sense of lookup.items) {
        const groundedSense = { ...sense, depth: need.depth };
        const concept = formalizationConcept(groundedSense);
        if (!seenConcepts.has(concept.id)) {
          seenConcepts.add(concept.id);
          graph.concepts.push(concept);
        }
        if (need.depth < maxConceptDepth) {
          queueNeeds(
            formalizationSegments(sense.gloss),
            need.needId,
            sense.contentId,
            need.depth + 1,
          );
        }
      }
    } else {
      need.state = "unsatisfiable";
    }
    graph.needs.push(need);
  }
  const procedure = formalizationProcedure(source, graph.docId, language);
  if (procedure) graph.procedures.push(procedure);
  graph.identity = formalizationIdentity(graph);
  graph.maxDepthReached = Math.min(
    maxConceptDepth,
    graph.needs.reduce((depth, need) => Math.max(depth, need.depth), 0),
  );
  return graph;
}
