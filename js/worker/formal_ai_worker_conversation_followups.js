// Conversation follow-ups from supplied history; mirrors try_how_it_works and
// extract_topic_from_prior_reply in rust/src/solver_handler_how.rs.
/**
 * The bare question's answer: the prior reply's topic through the concept
 * lookup, the seeded "no record yet" answer naming that topic, or — with no
 * prior reply — the seeded explanation of how to ask.
 * @param {object[]} history
 * @returns {object}
 */
function howItWorksFromHistory(prompt, history) {
  const solverEvents = [{ kind: "followup:how_it_works", payload: normalizePrompt(prompt) }];
  const prior = lastHistoryTurn(history, "assistant");
  if (prior) solverEvents.push({ kind: "followup:prior_turn", payload: "assistant" });
  const term = prior ? howItWorksPriorTopic(prior) : null;
  if (term) {
    const concept = tryConceptLookup(howPolicy("how_it_works", "concept-query", { term }));
    if (concept) {
      concept.evidence = [`followup:subject:prior_reply:${term}`, ...concept.evidence];
      concept.solverEvents = [
        ...solverEvents, { kind: "followup:subject", payload: `prior_reply:${term}` },
        ...(Array.isArray(concept.solverEvents) ? concept.solverEvents : []),
      ];
      return concept;
    }
    return {
      intent: "concept_elaboration_missing",
      content: nativeLaneRender(answerFor("how_it_works_prior_topic", "en"), { term }),
      confidence: 0.3,
      solverEvents: [...solverEvents, { kind: "followup:subject", payload: `prior_reply_no_record:${term}` }],
      evidence: ["followup:prior_turn:assistant", `followup:subject:prior_reply_no_record:${term}`, "response:concept_elaboration_missing"],
    };
  }
  return {
    intent: "meta_explanation",
    content: answerFor("how_it_works_no_context", "en"),
    confidence: 0.5,
    solverEvents: [...solverEvents, { kind: "how_it_works:refusal", payload: "no subject and no prior reply" }],
    evidence: ["how_it_works:refusal:no subject and no prior reply", "response:meta_explanation"],
  };
}

/**
 * The topic of a prior assistant reply: the term before "(" on its first line
 * ("Term (category): …"), else its first capitalised token of two or more
 * bytes that is not a topic_scan_stop_word. Mirrors extract_topic_from_prior_reply.
 * @param {string} reply
 * @returns {string|null}
 */
function howItWorksPriorTopic(reply) {
  const firstLine = String(reply).split(/\r?\n/u)[0].trim();
  const paren = firstLine.indexOf("(");
  if (paren >= 0 && firstLine.slice(0, paren).trim()) return firstLine.slice(0, paren).trim().toLowerCase();
  const stopWords = roleWordForms("topic_scan_stop_word").map((form) => form.text);
  for (const word of String(reply).split(/\s+/u).filter(Boolean)) {
    const clean = word.replace(/^[^\p{Alphabetic}\p{N}]+|[^\p{Alphabetic}\p{N}]+$/gu, "");
    if (new TextEncoder().encode(clean).length < 2 || !/^\p{Uppercase}/u.test(clean)) continue;
    if (!stopWords.includes(clean.toLowerCase())) return clean.toLowerCase();
  }
  return null;
}
