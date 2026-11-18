#!/usr/bin/env node
// Review a bounded plan before explicitly retiring inactive generated test fixtures.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {setTimeout as delay} from 'node:timers/promises';
import {createEvidenceClient} from './collect-test-replays.mjs';
import {retirementPlan, retireGeneratedIssues} from './lib/generated-test-retirement.mjs';

export async function main(args = process.argv.slice(2)) {
  if (args.length !== 2 || !['--plan', '--apply'].includes(args[0])) throw Error('Usage: retire-stale-generated-test-issues.mjs --plan PLAN.json | --apply PLAN.json');
  const [mode, file] = args, apply = mode === '--apply';
  const client = createEvidenceClient({maxRequests: 700, aggregateBytes: 4194304, bounds: {retries: 0, timeoutMs: 20000}});
  if (!apply) {
    const issues = await client.json('repos/link-assistant/formal-ai/issues?state=open&labels=e2e-task&per_page=100');
    const plan = retirementPlan(issues);
    fs.writeFileSync(file, JSON.stringify(plan, null, 2) + '\n', {flag: 'wx', mode: 0o600});
    console.log(JSON.stringify({plan: file, eligible: plan.eligible.length, retained: plan.retained, apply: false}));
    return;
  }
  const receiptPath = file + '.applied.json';
  if (fs.existsSync(receiptPath) || fs.statSync(file).size > 1048576) throw Error('Existing receipt or oversized plan; inspect before another apply');
  const lock = path.join(os.tmpdir(), 'formal-ai-generated-test-retirement.lock');
  const descriptor = fs.openSync(lock, 'wx', 0o600);
  try {
    const result = await retireGeneratedIssues({plan: JSON.parse(fs.readFileSync(file)), client, apply: true,
      pause: () => delay(3000)});
    fs.writeFileSync(receiptPath, JSON.stringify(result, null, 2) + '\n', {flag: 'wx', mode: 0o600});
    console.log(JSON.stringify({receipt: receiptPath, complete: result.complete,
      retired: result.results.filter(r => r.status === 'retired').length, results: result.results}));
    if (!result.complete) process.exitCode = 1;
  } finally { fs.closeSync(descriptor); fs.unlinkSync(lock); }
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main().catch(error => {
  console.error(error.message); process.exitCode = 1;
});
