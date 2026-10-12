// Conversation summary request ownership follows declared text and loaded meaning roles.
// Mirrors asks_for_conversation_summary in rust/src/solver_handlers/conversation_memory/conversation_summary.rs.

// Issue #386: a conversation-summary request is recognised by composing
// meaning roles, not by matching raw words per language. The universal
// algorithm is identical for every language: the prompt either carries a
// complete standalone conversation-summary phrasing, an objectless courtesy
// frame ("can you summarize", "подведи итог"), a summary directive together
// with an explicit conversation reference, or it is itself a bare summary
// directive. The prompt is re-normalised first so the boundary-aware matcher
// sees punctuation collapsed to spaces (idempotent here, since `normalized`
// is already normalised). Mirror of asks_for_conversation_summary in
// src/solver_handlers/mod.rs.
function isSummarizePrompt(normalized, prompt = normalized) {
  const cleaned = normalizePrompt(normalized);
  const head = normalizePrompt(textTransformCommandHead(prompt));
  if (textTransformFreeTextPayload(prompt) !== null &&
      !lexiconMentionsRole(ROLE_CONVERSATION_REFERENCE, head) &&
      !lexiconMentionsRole(ROLE_CONVERSATION_SUMMARY_PHRASE, head)) return false;
  return (
    isReturnRecapPrompt(cleaned) ||
    lexiconMentionsRole(ROLE_CONVERSATION_SUMMARY_PHRASE, cleaned) ||
    (lexiconMentionsRole(ROLE_CONVERSATION_SUMMARY_COURTESY, cleaned) &&
      courtesyFrameLeavesNoObject(cleaned)) ||
    (lexiconMentionsRole(ROLE_CONVERSATION_SUMMARY_DIRECTIVE, cleaned) &&
      lexiconMentionsRole(ROLE_CONVERSATION_REFERENCE, cleaned)) ||
    summaryDirectiveLeads(cleaned)
  );
}
