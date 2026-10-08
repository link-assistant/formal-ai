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
// Moved from formal_ai_worker_20.js (issue #999 warning band): the
// formalization fold-back, interpretation, deformalization and thinking-level
// helpers finalize() applies to every answer.
// Fold a handler's concrete entity back into its initial formalization trace.
function applyResolvedFormalization(events, steps, formalizationContext, answer) {
  if (!formalizationContext || !answer || !answer.formalizedObject) return;
  const resolved = resolveFormalizationWithId(
    formalizationContext.initial,
    answer.formalizedObject,
  );
  if (!resolved) return;
  // Cache hits may already contain the resolved id.
  if (resolved.tuple === formalizationContext.initial.tuple) return;
  formalizationContext.resolved = resolved;
  events.push(`formalization:resolved:${resolved.tuple}`);
  steps.push({
    step: "formalize_resolved",
    detail: formalizationDetail(resolved),
    formalization: {
      raw: resolved.raw,
      subject: resolved.subject,
      verb: resolved.verb,
      object: resolved.object,
      tuple: resolved.tuple,
    },
  });
}
function collectInterpretations(formalizationContext, answer) {
  const combined = [];
  const pushAll = (items) => {
    if (!Array.isArray(items)) return;
    for (const item of items) {
      if (!item || !item.original || !item.corrected) continue;
      combined.push({
        original: String(item.original),
        corrected: String(item.corrected),
      });
    }
  };
  pushAll(
    formalizationContext &&
      formalizationContext.initial &&
      formalizationContext.initial.interpretations,
  );
  pushAll(answer && answer.interpretations);
  const seen = new Set();
  return combined.filter((item) => {
    const key = `${item.original.toLowerCase()}\u0000${item.corrected.toLowerCase()}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}
function interpretationStatements(interpretations) {
  return interpretations
    .map((item) => `Interpreted "${item.original}" as "${item.corrected}".`)
    .join("\n");
}
function applyVisibleInterpretations(answer, interpretations) {
  if (!answer || interpretations.length === 0) return answer;
  const statements = interpretationStatements(interpretations);
  return Object.assign({}, answer, {
    content: `${statements}\n\n${String(answer.content || "")}`,
    evidence: [
      ...(Array.isArray(answer.evidence) ? answer.evidence : []),
      ...interpretations.map((item) => `interpretation:${item.original}->${item.corrected}`),
    ],
  });
}
function deformalizeProjection(formalizationContext, answer) {
  const tuple =
    (formalizationContext &&
      ((formalizationContext.resolved && formalizationContext.resolved.tuple) ||
        (formalizationContext.initial && formalizationContext.initial.tuple))) ||
    "(@USER OP:express ?)";
  const evidence = Array.isArray(answer.evidence) ? answer.evidence : [];
  const content = String(answer.content || "");
  const firstLine = content.split(/\r?\n/, 1)[0] || "";
  const projection = firstLine.length > 96 ? `${firstLine.slice(0, 96)}…` : firstLine;
  return {
    tuple,
    intent: answer.intent || "unknown",
    contentChars: content.length,
    evidenceCount: evidence.length,
    language:
      (formalizationContext && formalizationContext.language) ||
      answer.language ||
      "",
    summary: `${tuple} ⇒ ${answer.intent || "unknown"}: ${projection}`,
  };
}
// Issue #488: classify each reasoning step into a granularity tier so the
// thinking preview can show only the high-level universal-algorithm phases at
// the default ("standard") granularity and fold the mechanical sub-steps
// (the symbolic formalization tuple, tool probes, calculator reductions, memory
// scans, rule bookkeeping) into the opt-in "detailed" view. This mirrors the
// Rust solver's `ThinkingStep::level` classification (see src/engine.rs and
// src/event_log.rs) so the browser and native engines curate the trace
// identically — the thinking is fully applied to the logic, not just the UI.
const HIGH_LEVEL_THINKING_STEPS = new Set([
  "impulse",
  "detect_language",
  "resolve_response_language",
  "dispatch_handler",
  "match_rule",
  "clarify_formalization",
  "program_plan",
  "compute",
  "deformalize",
  "user_context",
  "fallback",
]);
function thinkingStepLevel(step) {
  const raw = String(step || "");
  // Nested agent sub-reasoning always folds under its composite agent task.
  if (/^agent_\d+_/i.test(raw)) return "detailed";
  return HIGH_LEVEL_THINKING_STEPS.has(raw) ? "high" : "detailed";
}
function withThinkingLevels(steps) {
  if (!Array.isArray(steps)) return [];
  return steps.map((step) =>
    step && typeof step === "object" && !step.level
      ? Object.assign({}, step, { level: thinkingStepLevel(step.step) })
      : step,
  );
}
