// Structural helpers for the issue #1138 deep-formalization browser mirror.

function formalizationSlug(value) {
  return String(value || "")
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, "-")
    .replace(/^-|-$/gu, "");
}

let cachedFormalizationPunctuation = null;

function formalizationPunctuation() {
  if (cachedFormalizationPunctuation) return cachedFormalizationPunctuation;
  const punctuation = { terminators: new Set(), clauseSeparators: new Set() };
  const raw = seedRawText(SEED_RAW, "sentence-punctuation.lino");
  if (raw && self.FormalAiSeed) {
    const root = self.FormalAiSeed.parse(raw);
    const registry = (root.children || []).find((node) => node.name === "sentence_punctuation") || root;
    for (const script of registry.children || []) {
      for (const item of script.children || []) {
        if (item.name === "terminator") punctuation.terminators.add(String(item.value || ""));
        if (item.name === "clause_separator") punctuation.clauseSeparators.add(String(item.value || ""));
      }
    }
  }
  cachedFormalizationPunctuation = punctuation;
  return punctuation;
}

function formalizationByteOffset(text, codeUnitOffset) {
  return new TextEncoder().encode(String(text || "").slice(0, codeUnitOffset)).length;
}

function formalizationSegments(text) {
  const source = String(text || "");
  const segments = [];
  const terminators = formalizationPunctuation().terminators;
  let start = 0;
  for (let index = 0; index < source.length; index += 1) {
    if (!terminators.has(source[index])) continue;
    const end = index + 1;
    const raw = source.slice(start, end);
    const leading = raw.search(/\S/u);
    if (leading >= 0) {
      const trimmed = raw.trimEnd();
      segments.push({
        text: trimmed.slice(leading),
        start: formalizationByteOffset(source, start + leading),
        end: formalizationByteOffset(source, start + trimmed.length),
      });
    }
    start = end;
  }
  const tail = source.slice(start);
  const leading = tail.search(/\S/u);
  if (leading >= 0) {
    const trimmed = tail.trimEnd();
    segments.push({
      text: trimmed.slice(leading),
      start: formalizationByteOffset(source, start + leading),
      end: formalizationByteOffset(source, start + trimmed.length),
    });
  }
  return segments;
}

function formalizationUnknownSpans(text) {
  const known = conceptKnownSurfaces();
  const spans = [];
  const pattern = /[\p{L}\p{N}_]+/gu;
  let match = pattern.exec(String(text || ""));
  while (match) {
    const surface = match[0];
    const folded = normalizePrompt(surface);
    if (Array.from(surface).length >= 3 && !known.has(folded)) {
      spans.push({
        surface,
        start: formalizationByteOffset(text, match.index),
        end: formalizationByteOffset(text, match.index + surface.length),
      });
    }
    match = pattern.exec(String(text || ""));
  }
  return spans;
}

function formalizationConcept(sense) {
  return {
    id: `concept:${formalizationSlug(sense.lemma)}`,
    label: sense.lemma,
    language: sense.language,
    gloss: sense.gloss,
    structures: [],
    sourceId: sense.sourceId,
    sourceUrl: sense.sourceUrl,
    sha256: sense.sha256,
    licenseName: sense.licenseName,
    depth: sense.depth,
  };
}

function formalizationProcedureSeparators() {
  return formalizationPunctuation().clauseSeparators;
}

function formalizationProcedureLeadSurfaces(language) {
  return meaningsWithRole("skill_procedure_clause_separator")
    .flatMap((meaning) => meaning.lexemes || [])
    .filter((lexeme) => lexeme.language === language)
    .flatMap((lexeme) => lexeme.words || [])
    .sort((left, right) => right.length - left.length);
}

function formalizationTrimProcedureLead(part, language) {
  for (const surface of formalizationProcedureLeadSurfaces(language)) {
    const escaped = surface.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    const match = String(part).match(new RegExp(`^${escaped}(?:\\s+|$)`, "iu"));
    if (match) return String(part).slice(match[0].length).trim();
  }
  return String(part).trim();
}

function formalizationProcedure(text, docId, language) {
  const source = String(text || "");
  const colon = source.indexOf(":");
  const bodyOffset = colon >= 0 ? colon + 1 : 0;
  const body = source.slice(bodyOffset).trim();
  const lead = bodyOffset + source.slice(bodyOffset).indexOf(body);
  const separators = Array.from(formalizationProcedureSeparators())
    .filter(Boolean)
    .map((separator) => separator.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&"));
  const parts = body
    .split(new RegExp(`\\s*(?:${separators.join("|")})\\s*`, "u"))
    .map((part) => part.trim())
    .filter(Boolean);
  const firstInstruction = parts.length >= 3 ? 1 : -1;
  const instructions = firstInstruction >= 0
    ? parts.slice(firstInstruction).map((part) => formalizationTrimProcedureLead(part, language))
    : [];
  if (instructions.length < 2) return null;
  let searchFrom = lead;
  const steps = instructions.map((instruction, index) => {
    const start = source.indexOf(instruction, searchFrom);
    const safeStart = start >= 0 ? start : searchFrom;
    const end = safeStart + instruction.length;
    searchFrom = end;
    const words = instruction.split(/\s+/u);
    return {
      position: index + 1,
      imperative: words[0] || instruction,
      object: words.slice(1).join(" ") || null,
      sourceSpan: `${docId}@${formalizationByteOffset(source, safeStart)}:${formalizationByteOffset(source, end)}`,
      verified: false,
    };
  });
  const goal = firstInstruction > 0 ? parts.slice(0, firstInstruction).join(" ") : body;
  const identity = `${goal}\u001f${language}\u001f${steps.map((step) => step.imperative).join("\u001e")}`;
  return {
    id: conceptStableId("extracted_procedure", identity),
    goal,
    language,
    steps,
    preconditions: [],
    postconditions: [],
    sourceId: docId,
    sourceUrl: docId,
    sha256: conceptStableId("document", source).replace("document_", ""),
    licenseName: "user-provided",
  };
}

function formalizationIdentity(graph) {
  const complete = graph.needs.every((need) => need.state === "satisfied");
  const structures = complete
    ? Array.from(new Set(graph.concepts.flatMap((concept) => concept.structures || []))).sort()
    : [];
  const relations = Array.from(new Set(graph.relations.map((relation) => relation.kind))).sort();
  const procedureShapes = graph.procedures.map((procedure) => String(procedure.steps.length)).sort();
  return conceptStableId(
    "concept_graph",
    `structures=${structures.join(",")};relations=${relations.join(",")};procedure_shapes=${procedureShapes.join(",")}`,
  );
}
