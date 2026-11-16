// Issue #1562: turn the recorded chat request of a hive-mind solve.log (the
// request that received the stored Hello World program) into a draft-replay
// fixture (schema draft-request-replay/v1, as scripts/extract-draft-replay.mjs
// writes from a Formal AI dialog).
// Usage: node experiments/issue-1562/draft-replay-of-log.mjs <solve.log> <request index> <issue> <pull> <log url> <out.json>
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { DRAFT_REPLAY_SCHEMA, requireDraftReplay } from '../../scripts/lib/draft-replay.mjs';

const [log, index, issue, pull, source, out] = process.argv.slice(2);
const previews = [];
for (const line of readFileSync(log, 'utf8').split('\n')) {
  const match = /"bodyPreview": (".*"),?$/u.exec(line);
  if (!match) continue;
  let text;
  try { text = JSON.parse(match[1]); } catch { continue; }
  if (text.startsWith('{"model":"formal-ai"')) previews.push(text);
}
const text = previews[Number(index)];
const request = JSON.parse(text);
const fixture = requireDraftReplay({
  schema: DRAFT_REPLAY_SCHEMA,
  issue: Number(issue),
  repository: 'link-assistant/hive-mind',
  pull: Number(pull),
  run: null,
  sourceCommit: null,
  source,
  family: 'work-item-cited-program',
  captured: {
    dialogId: null,
    requestSha256: createHash('sha256').update(text).digest('hex'),
    reason: null,
    kind: 'tool_calls',
    calls: 'write main.*, run it, commit, gh pr ready'
  },
  tools: request.tools.map(tool => tool?.function?.name ?? tool?.name),
  messages: request.messages
});
writeFileSync(out, JSON.stringify(fixture) + '\n');
console.log(out, fixture.messages.length, fixture.captured.requestSha256);
