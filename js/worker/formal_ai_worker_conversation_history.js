// Mirrors recognize_recall_query, term_from_form and recall_matches in
// rust/src/solver_handlers/conversation_memory/mod.rs; only supplied dialog turns are read.
function historyRecallTerm(form, normalized) {
  let raw;
  if (form.slot === "prefix" && normalized.startsWith(form.before)) {
    raw = normalized.slice(form.before.length);
  } else if (form.slot === "suffix" && normalized.endsWith(form.after)) {
    raw = normalized.slice(0, normalized.length - form.after.length);
  } else if (form.slot === "circumfix" && normalized.startsWith(form.before)) {
    const rest = normalized.slice(form.before.length);
    if (!rest.endsWith(form.after)) return null;
    raw = rest.slice(0, rest.length - form.after.length);
  } else {
    return null;
  }
  return raw.trim().replace(/^[\s`"':\-_.,?!()]+|[\s`"':\-_.,?!()]+$/gu, "")
    .split(/\s+/u).filter(Boolean).join(" ") || null;
}

// Mirrors recognize_recall_query: slot forms and scopes come from the seed roles.
function historyRecallQuery(normalized) {
  const roles = [
    [ROLE_CONVERSATION_RECALL_QUERY, "conversation"],
    [ROLE_CONVERSATION_RECALL_OTHER_QUERY, "other_conversations"],
  ];
  for (const [role, scope] of roles) {
    for (const form of roleWordForms(role)) {
      const term = historyRecallTerm(form, normalized);
      if (term) return { term, scope };
    }
  }
  return null;
}

// Mirrors try_conversation_recall and render_recall_report: native typed events
// record actual matching turn indices; no persisted store is inferred from dialog history.
function tryConversationRecall(prompt, normalized, history) {
  const query = historyRecallQuery(normalized);
  if (!query) return null;
  const needle = normalizePrompt(query.term);
  const matches = [];
  for (const [index, turn] of (Array.isArray(history) ? history : []).entries()) {
    if (!turn || !["user", "assistant"].includes(turn.role)) continue;
    const content = String(turn.content ?? "");
    if (needle && normalizePrompt(content).includes(needle)) {
      matches.push({ turn: index + 1, role: turn.role, content });
    }
  }
  const solverEvents = [
    { kind: "filter:memory_query", payload: query.term },
    { kind: "filter:memory_scope", payload: query.scope },
    { kind: "filter:memory_matches", payload: String(matches.length) },
    ...matches.map((matched) => ({
      kind: "memory_match",
      payload: `turn=${matched.turn} role=${matched.role} content=${matched.content}`,
    })),
  ];
  const language = detectLanguage(prompt);
  const values = { term: query.term, count: String(matches.length) };
  const name = matches.length ? "conversation-recall-header" : "conversation-recall-empty";
  const lines = [handlerRulesFillOnce(answerFor(name, language), values)];
  for (const matched of matches) {
    lines.push(handlerRulesFillOnce(answerFor("conversation-recall-turn", language), matched));
  }
  return { intent: "conversation_recall", content: lines.join("\n").trimEnd(), confidence: 0.9, solverEvents };
}
