// `crate::protocol::chat_prompt_and_history` (rust/src/protocol/recording.rs).

import { chatMessageToTurn, latestUserTurn } from '../content.mjs';

/**
 * Mirrors `fn chat_prompt_and_history` in rust/src/protocol/recording.rs:
 * `[prompt, history]` (prompt '' and history [] without a user turn).
 * @param {Array<object>} messages
 * @returns {[string, Array<{role: string, content: string}>]}
 */
export function chatPromptAndHistory(messages) {
  const latest = latestUserTurn(messages);
  if (!latest) return ['', []];
  const { index, prompt } = latest;
  const history = messages.slice(0, index).map(chatMessageToTurn).filter((turn) => turn !== null);
  return [prompt, history];
}
