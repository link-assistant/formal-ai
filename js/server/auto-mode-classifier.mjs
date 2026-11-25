// Claude Code auto-mode classifier fast path (rust/src/auto_mode_classifier.rs,
// issue #1632 R4).
//
// `claude --permission-mode auto` checks every shell command and network
// action by sending the model a "security monitor" request: a large rule book
// as the system prompt, one user message holding the session `<transcript>`,
// `stop_sequences: ["</block>"]`, and a 60 s client timeout. The general solver
// cannot answer that in time, so the envelope is recognised by the seed markers
// in data/seed/auto-mode-classifier.lino and answered in the documented
// `<block>…</block>` format at once. Only the agent's own actions are matched;
// transcript entries written by the user never trip a rule.

import { childValue, childrenNamed, parseLino, readRepoFile } from './lino.mjs';

let policy = null;

/** The seed-backed policy, parsed once. */
export function classifierPolicy() {
  if (policy) return policy;
  const root = parseLino(readRepoFile('data/seed/auto-mode-classifier.lino'));
  policy = {
    markers: childrenNamed(root, 'marker').map((node) => node.value),
    transcriptOpen: childValue(root, 'transcript-open'),
    transcriptClose: childValue(root, 'transcript-close'),
    userEntryKey: childValue(root, 'user-entry-key'),
    allowText: childValue(root, 'allow-text'),
    blockTemplate: childValue(root, 'block-template'),
    rules: childrenNamed(root, 'rule').map((node) => ({
      name: node.value,
      reason: childValue(node, 'reason'),
      patterns: childrenNamed(node, 'pattern').map((pattern) => new RegExp(pattern.value, 'i')),
    })),
  };
  return policy;
}

/** Whether a system prompt is Claude Code's auto-mode security monitor. */
export function isClassifierPrompt(system) {
  const { markers } = classifierPolicy();
  return markers.length > 0 && markers.every((marker) => system.includes(marker));
}

function actionText(line, userEntryKey) {
  let entry;
  try {
    entry = JSON.parse(line);
  } catch {
    // A non-JSON line is not a user entry; keep it so it is still screened.
    return line;
  }
  if (entry === null || typeof entry !== 'object' || Array.isArray(entry)) return null;
  if (Object.prototype.hasOwnProperty.call(entry, userEntryKey)) return null;
  return Object.keys(entry)
    .sort()
    .map((key) => (typeof entry[key] === 'string' ? entry[key] : JSON.stringify(entry[key])))
    .join('\n');
}

/** The agent actions inside the `<transcript>` of a classifier request. */
export function agentActions(messages) {
  const { transcriptOpen, transcriptClose, userEntryKey } = classifierPolicy();
  let transcript = messages;
  const start = transcript.lastIndexOf(transcriptOpen);
  if (start >= 0) transcript = transcript.slice(start + transcriptOpen.length);
  const end = transcript.indexOf(transcriptClose);
  if (end >= 0) transcript = transcript.slice(0, end);
  return transcript
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line !== '')
    .map((line) => actionText(line, userEntryKey))
    .filter((action) => action !== null);
}

/** Judge the latest agent action: `{ blockedBy, text }`. */
export function judge(messages) {
  const { allowText, blockTemplate, rules } = classifierPolicy();
  const latest = agentActions(messages).at(-1);
  if (latest === undefined) return { blockedBy: null, text: allowText };
  const rule = rules.find((candidate) => candidate.patterns.some((pattern) => pattern.test(latest)));
  if (!rule) return { blockedBy: null, text: allowText };
  return {
    blockedBy: rule.name,
    text: blockTemplate.split('{name}').join(rule.name).split('{reason}').join(rule.reason),
  };
}

/** `classify`: the verdict, or null when this is not a classifier request. */
export function classify(system, messages) {
  return isClassifierPrompt(system) ? judge(messages) : null;
}

/** Cut a verdict at the first requested stop sequence, as the Anthropic API does. */
export function applyStopSequences(text, stopSequences) {
  let best = null;
  for (const stop of stopSequences || []) {
    if (typeof stop !== 'string' || stop === '') continue;
    const index = text.indexOf(stop);
    if (index >= 0 && (best === null || index < best.index)) best = { index, stop };
  }
  return best === null ? [text, null] : [text.slice(0, best.index), best.stop];
}
