// Unsupported-language fallback consults actual local, seeded and captured stores.
// Mirrors the terminal unknown-language branch of rust/src/solver_unknown_reasoning.rs.
function solverUnknownLanguage(prompt, language, history, memory, preferences) {
  if (language !== "unknown" || !String(prompt || "").trim()) return null;
  const focus = String(prompt).trim(), folded = normalizePrompt(focus);
  const local = [...(Array.isArray(history) ? history : []), ...(Array.isArray(memory) ? memory : [])]
    .filter((row) => normalizePrompt(String(row?.content || row?.text || "")) === folded);
  const publicRecords = CONCEPTS.filter((record) => (record.terms || [])
    .some((term) => normalizePrompt(term) === folded));
  const captures = Array.from(sourceWalkCaptureCache.values()).filter((capture) => {
    if (!capture.ok) return false;
    try {
      const value = JSON.parse(capture.text);
      return normalizePrompt(value?.title || value?.word || "") === folded
        && conceptReadGlosses("mediawiki_summary_v1", capture.text, focus).length > 0;
    } catch { return false; }
  });
  // Matching text alone cannot certify an answer to a request in an unsupported grammar.
  const events = [
    { kind: "reasoning:known", payload: "language=unknown local_search=complete prompt_state=unmatched_prompt" },
    { kind: "reasoning:unknown", payload: "missing_answer_for:" + focus },
  ];
  for (const [source, records] of [["link_memory", local], ["public_knowledge_cache", publicRecords], ["source_cache", captures]]) {
    events.push({ kind: "reasoning:candidate_source", payload: source });
    events.push({ kind: "reasoning:gather_attempt", payload: source + ":" + focus });
    events.push({ kind: "reasoning:gather_result", payload: source + (records.length ? ":no_verified_answer" : ":miss") });
  }
  events.push({ kind: "reasoning:candidate_source", payload: preferences?.offline ? "allowed_external_api:skipped_offline" : "allowed_external_api:no_verified_value" });
  const body = wordDefinitionRender("unknown_reasoning_trace", language, { focus });
  const text = String(wordDefinitionRender("detected_failure_report_invitation", language, {})).trim();
  const invitation = text ? body.trimEnd() + "\n\n" + text : body.trimEnd();
  return { content: invitation, events };
}
