// The running-conversation summary the native solver answers a summarize
// request with (the summary body of `try_summarize_conversation` in
// rust/src/solver_handlers/conversation_memory/conversation_summary.rs):
// `Conversation summary: <dialog summary>`, its title, and every user turn.
// The Agent CLI stores this answer as its compaction summary, and the
// planner's `compactedAgentTask` recovers the task from its `User turns:`
// list, so a compacted session keeps its task (PR #1188 dogfooding).

import { localizedResponse } from './seed.mjs';
import { fillSlots } from './seed_reports.mjs';
import { detect } from './language.mjs';
import { SummarizationMode, defaultConfig, withLanguage, withMode } from './summarization.mjs';
import { assistantTurn, generateChatTitle, summarizeDialog, userTurn } from './summarization_dialog.mjs';

const ENVELOPE_LANGUAGES = new Set(['ru', 'zh']);

/**
 * Mirrors the summary body of `fn try_summarize_conversation`: the envelope
 * over `history` (`{role, content}` turns), or null without a user turn.
 * Mirrors `fn conversation_summary_envelope` in rust/src/solver_handlers/conversation_memory/conversation_summary.rs.
 * @param {string} prompt
 * @param {Array<{role: string, content: unknown}>} history
 */
export function conversationSummaryRecord(prompt, history) {
  const turns = (history || [])
    .filter((turn) => (turn?.role === 'user' || turn?.role === 'assistant') && typeof turn.content === 'string' && turn.content.trim() !== '')
    .map((turn) => (turn.role === 'user' ? userTurn(turn.content) : assistantTurn(turn.content)));
  if (!turns.length) {
    const at = prompt.indexOf(':');
    const content = at < 0 ? '' : prompt.slice(at + 1).trim();
    if (content) turns.push(userTurn(content));
  }
  if (!turns.length && prompt.trim()) turns.push(userTurn(prompt.trim()));
  const users = turns.filter((turn) => turn.role === 'user');
  if (!users.length) return null;
  const language = detect(prompt);
  const config = withLanguage(withMode(defaultConfig(), SummarizationMode.Standard), language);
  const summary = summarizeDialog(turns, config);
  const title = generateChatTitle(turns, language);
  let body = fillSlots(
    localizedResponse('conversation-summary-envelope', ENVELOPE_LANGUAGES.has(language) ? language : 'en') ?? '',
    [['summary', summary], ['title', title]],
  );
  users.forEach((turn, index) => {
    body += `  ${index + 1}. ${turn.text}\n`;
  });
  return { content: body.trimEnd(), title, language };
}

/** Mirrors the body of `fn conversation_summary_envelope` for existing server callers. */
export function conversationSummaryEnvelope(prompt, history) {
  return conversationSummaryRecord(prompt, history)?.content ?? null;
}
