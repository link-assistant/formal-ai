// `crate::protocol::chat_prompt_and_history` (rust/src/protocol/recording.rs).

import { chatMessageToTurn, stripSystemEcho, systemPromptText, userRequestText } from '../content.mjs';

/**
 * Mirrors `fn chat_prompt_and_history` in rust/src/protocol/recording.rs:
 * `[prompt, history]` (prompt '' and history [] without a user turn).
 * @param {Array<object>} messages
 * @returns {[string, Array<{role: string, content: string}>]}
 */
export function chatPromptAndHistory(messages) {
  let latest = -1;
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (messages[index].role.toLowerCase() === 'user') {
      latest = index;
      break;
    }
  }
  if (latest < 0) return ['', []];
  const prompt = stripSystemEcho(userRequestText(messages[latest].content), systemPromptText(messages));
  const history = messages.slice(0, latest).map(chatMessageToTurn).filter((turn) => turn !== null);
  return [prompt, history];
}
