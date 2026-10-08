// A file operation followed by edits of the file it makes (PR #1188 G82):
// `Copy a.lino to b.lino. In b.lino replace 'x' with 'y', replace 'p' with
// 'q'.` was written as one literal file. When the first sentence is a seeded
// shell intent (copy, move) and every later sentence is an edit request whose
// file that sentence names, the sentences are planned one after another, each
// as the request it would be alone, over the same tool history; the answer
// states each in turn. The JavaScript twin of rust/src/agentic_coding/request_sequence.rs.

import { finalAnswer } from './plan.mjs';
import { pathsIn } from './module_function.mjs';
import { replaceList } from './replace_list.mjs';
import { semanticShellCommandForTask } from './shell_command.mjs';
import { sentences } from './shell_command_policy.mjs';
import { composeEditRequest } from './write_request.mjs';

/**
 * Mirrors `fn request_sequence`: the request's sentences when the first is a
 * seeded shell intent and every later one edits a file the first names, or
 * null.
 * @param {string} task
 */
export function requestSequence(task) {
  const parts = sentences(task).map((sentence) => sentence.text);
  if (parts.length < 2 || semanticShellCommandForTask(parts[0]) === null) return null;
  const named = pathsIn(parts[0]);
  for (const part of parts.slice(1)) {
    const target = replaceList(part)?.target ?? composeEditRequest(part)?.[0] ?? null;
    if (target === null || !named.includes(target)) return null;
  }
  return parts;
}

/** The conversation with its latest user turn asking `part` alone. */
function withRequest(messages, part) {
  let latest = -1;
  messages.forEach((message, index) => {
    if (String(message.role).toLowerCase() === 'user') latest = index;
  });
  return messages.map((message, index) => (index === latest ? { ...message, content: part } : message));
}

/**
 * Mirrors `fn plan_request_sequence_step`: the next step of the first
 * sentence not yet answered, planned by `planFor` as that sentence alone, or
 * the sentences' answers in order once each is answered.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 * @param {(messages: Array<object>, toolNames: Array<string>) => Promise<object|null>} planFor
 */
export async function planRequestSequenceStep(task, messages, toolNames, planFor) {
  const parts = requestSequence(task);
  if (parts === null) return null;
  const answers = [];
  for (const part of parts) {
    const plan = await planFor(withRequest(messages, part), toolNames);
    if (plan === null) return null;
    if (plan.kind !== 'final') return plan;
    answers.push(plan.answer);
  }
  return finalAnswer(answers.join('\n\n'));
}
