#!/usr/bin/env node
// Collects the project owner's own messages from the Claude Code session logs
// of this repository and writes them, deduplicated and verbatim, to
// docs/case-studies/pull-request-1188/user-messages.md.
//
// The session logs are large JSON-lines files, so they are streamed line by
// line. A message is kept when it is human text: a `user` entry that is not
// meta, not a compaction summary, not a tool result, or a queued command whose
// origin is a human. Harness wrappers (system reminders, task notifications,
// local command echoes, subagent hand-backs, interruption markers,
// continuation summaries) are dropped.
//
// Usage:
//   node experiments/formal_ai_subagent/collect-user-messages.mjs [--write|--check] [--source <dir>] [--output <file>]
// --write   regenerate the file (the default when neither flag is given)
// --check   regenerate in memory and fail when the file differs
// --source  the session directory (default: the Claude Code project directory
//           derived from the repository path)

import { createReadStream, existsSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { createInterface } from 'node:readline';
import { fileURLToPath } from 'node:url';

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const argumentsList = process.argv.slice(2);

function optionValue(name, fallback) {
  const position = argumentsList.indexOf(name);
  return position >= 0 && argumentsList[position + 1] ? argumentsList[position + 1] : fallback;
}

const projectDirectoryName = repositoryRoot.replace(/[^A-Za-z0-9]/g, '-');
const sourceDirectory = optionValue('--source', join(homedir(), '.claude', 'projects', projectDirectoryName));
const outputPath = resolve(repositoryRoot, optionValue('--output', 'docs/case-studies/pull-request-1188/user-messages.md'));
const checkMode = argumentsList.includes('--check');

const droppedPrefixes = [
  '<task-notification>',
  '<local-command-',
  '<command-name>',
  '<command-message>',
  '<agent-message',
  '[Subagent hand-back]',
  '[Request interrupted',
  '[Usage limit',
  'This session is being continued',
  '/compact',
  'Caveat: The messages below',
];

const secretPatterns = [
  /gh[pousr]_[A-Za-z0-9]{20,}/g,
  /github_pat_[A-Za-z0-9_]{20,}/g,
  /sk-[A-Za-z0-9_-]{20,}/g,
  /xox[abpr]-[A-Za-z0-9-]{10,}/g,
  /dckr_pat_[A-Za-z0-9_-]{10,}/g,
];

function humanText(text) {
  const withoutReminders = text.replace(/<system-reminder>[\s\S]*?<\/system-reminder>/g, '').trim();
  if (!withoutReminders) return null;
  if (droppedPrefixes.some((prefix) => withoutReminders.startsWith(prefix))) return null;
  return secretPatterns.reduce((current, pattern) => current.replace(pattern, '[redacted]'), withoutReminders);
}

function textsOfUserEntry(entry) {
  if (entry.type !== 'user' || entry.isMeta || entry.isCompactSummary || entry.isVisibleInTranscriptOnly) return [];
  const content = entry.message?.content;
  if (typeof content === 'string') return [content];
  if (!Array.isArray(content)) return [];
  if (content.some((block) => block.type === 'tool_result')) return [];
  return content.filter((block) => block.type === 'text').map((block) => block.text);
}

function textsOfQueuedCommand(entry) {
  const attachment = entry.attachment;
  if (entry.type !== 'attachment' || attachment?.type !== 'queued_command') return [];
  if (attachment.isMeta || attachment.origin?.kind !== 'human') return [];
  const prompt = attachment.prompt;
  if (typeof prompt === 'string') return [prompt];
  if (Array.isArray(prompt)) return prompt.filter((block) => block.type === 'text').map((block) => block.text);
  return [];
}

async function collectMessages() {
  const files = readdirSync(sourceDirectory).filter((name) => name.endsWith('.jsonl')).sort();
  const messages = new Map();
  for (const file of files) {
    const lines = createInterface({ input: createReadStream(join(sourceDirectory, file)), crlfDelay: Infinity });
    for await (const line of lines) {
      if (!line.includes('"user"') && !line.includes('queued_command')) continue;
      let entry;
      try {
        entry = JSON.parse(line);
      } catch {
        continue;
      }
      const texts = [...textsOfUserEntry(entry), ...textsOfQueuedCommand(entry)];
      const timestamp = entry.timestamp ?? entry.attachment?.timestamp ?? '';
      const sessionIdentifier = entry.sessionId ?? file.replace(/\.jsonl$/, '');
      for (const rawText of texts) {
        const text = humanText(rawText);
        if (!text) continue;
        const key = text.replace(/\s+/g, ' ');
        const known = messages.get(key);
        if (!known || timestamp < known.timestamp) messages.set(key, { text, timestamp, sessionIdentifier });
      }
    }
  }
  return [...messages.values()].sort((left, right) => left.timestamp.localeCompare(right.timestamp));
}

function fenceFor(text) {
  const longestRun = Math.max(2, ...(text.match(/`+/g) ?? []).map((run) => run.length));
  return '`'.repeat(longestRun + 1);
}

function render(messages) {
  const lines = [
    '# The project owner\'s messages in the Claude Code sessions for this repository',
    '',
    'Generated by `node experiments/formal_ai_subagent/collect-user-messages.mjs --write`;',
    '`--check` regenerates and compares. Do not edit by hand.',
    '',
    'One entry per distinct message, verbatim, oldest first, with the session it was',
    'first sent in and its timestamp. Tool results, system reminders, task',
    'notifications, local command echoes, subagent hand-backs and continuation',
    'summaries are left out; token-shaped strings are redacted.',
    '',
    `Messages: ${messages.length}.`,
    '',
  ];
  messages.forEach((message, index) => {
    const fence = fenceFor(message.text);
    lines.push(`## ${index + 1}. ${message.timestamp}`, '', `Session \`${message.sessionIdentifier}\`.`, '', `${fence}text`, message.text, fence, '');
  });
  return `${lines.join('\n').trimEnd()}\n`;
}

if (!existsSync(sourceDirectory)) {
  console.log(`collect-user-messages: skipped, no session directory at ${sourceDirectory}`);
  process.exit(0);
}

const rendered = render(await collectMessages());
if (checkMode) {
  const current = existsSync(outputPath) ? readFileSync(outputPath, 'utf8') : '';
  if (current !== rendered) {
    console.error(`collect-user-messages: ${outputPath} is stale; run with --write`);
    process.exit(1);
  }
  console.log('collect-user-messages: up to date');
} else {
  writeFileSync(outputPath, rendered);
  console.log(`collect-user-messages: wrote ${outputPath}`);
}
