// Issue #1162: pick the shortest real body per own-status pattern from the
// three 2026-11 Hello World test pull requests (fetched with
// `gh pr view <url> --json comments,reviews > <dir>/pr-<repo>.json`) and write
// rust/tests/fixtures/issue-1162/own-status-notices.json, which the Rust and
// JS feedback tests both assert.
//
// Usage: node experiments/issue-1162/collect-notice-samples.mjs <dir> [--write]
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { fill } from '../../js/agentic/work_item_steps.mjs';
import { feedbackNeedsChanges } from '../../js/agentic/restart_feedback.mjs';

await installNodeHost(new WorkerHost());
const dir = process.argv[2];
const keys = [...readFileSync('data/meta/work-item-steps.lino', 'utf8').matchAll(/^ {2}(pull-request-[a-z-]+-pattern) /gmu)].map(match => match[1]);
const best = new Map();
let total = 0, unmatched = 0;
for (const file of readdirSync(dir).filter(name => /^pr-test-.*\.json$/u.test(name)).sort()) {
  for (const comment of JSON.parse(readFileSync(join(dir, file), 'utf8')).comments) {
    total += 1;
    const body = comment.body.trim();
    const key = keys.find(candidate => new RegExp(fill(candidate, []), 'u').test(body));
    if (!key) { unmatched += 1; continue; }
    if (!best.has(key) || body.length < best.get(key).body.length) best.set(key, { key, source: file.replace(/^pr-|\.json$/gu, ''), body: comment.body });
  }
}
const own = keys.map(key => best.get(key)).filter(Boolean);
const missing = keys.filter(key => !best.has(key));
const item = body => ({ author: { login: 'konard' }, viewerDidAuthor: true, includesCreatedEdit: false, body });
const ready = 'pull-request-ready-to-merge-pattern';
const readyBody = best.get(ready)?.body ?? '';
const fixture = {
  issue: 1162,
  purpose: 'Real orchestrator notices from the three 2026-11 Hello World test pull requests (shortest body per own-status pattern). Each alone must not count as feedback; every negative must.',
  generated_by: 'experiments/issue-1162/collect-notice-samples.mjs',
  observed: { comments: total, unmatched },
  own_notices: own,
  negatives: [
    { id: 'viewer-plain-request', item: item('Please rename the workflow to ci.yml') },
    { id: 'other-author-notice', item: { ...item(readyBody), author: { login: 'reviewer' }, viewerDidAuthor: false } },
    { id: 'edited-notice', item: { ...item(readyBody), includesCreatedEdit: true } },
    { id: 'notice-with-request-appended', item: item(`${readyBody.trim()}\n\nAlso fix the README.`) },
  ],
};
for (const notice of own) {
  if (feedbackNeedsChanges(JSON.stringify({ comments: [item(notice.body)], reviews: [] }))) throw new Error(`notice counted as feedback: ${notice.key}`);
}
for (const negative of fixture.negatives) {
  if (!feedbackNeedsChanges(JSON.stringify({ comments: [negative.item], reviews: [] }))) throw new Error(`negative ignored: ${negative.id}`);
}
console.log(`comments=${total} unmatched=${unmatched} keys=${own.length}/${keys.length} missing=${missing.join(',') || 'none'}`);
if (process.argv.includes('--write')) writeFileSync('rust/tests/fixtures/issue-1162/own-status-notices.json', `${JSON.stringify(fixture, null, 2)}\n`);
