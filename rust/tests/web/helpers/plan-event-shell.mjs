// Execute the canonical plan append against a bounded temporary event store.
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { template } from '../../../../js/agentic/work_item_steps.mjs';

/** Return null for unsupported commands; preserve real stdout, failures and event bytes. */
export function runPlanEvent(files, command) {
  const prefix = template('plan-event-append-command')?.split('{lock_path}')[0];
  if (!prefix || !command.startsWith(prefix)) return null;
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-plan-event-fixture-'));
  const eventPath = '.formal-ai/general-change-plan.lino';
  const destination = join(directory, eventPath);
  try {
    if (files.has(eventPath)) {
      mkdirSync(join(directory, '.formal-ai'), { recursive: true });
      writeFileSync(destination, files.get(eventPath));
    }
    try {
      return execFileSync('/bin/sh', ['-c', command], {
        cwd: directory, encoding: 'utf8', timeout: 5000, stdio: ['ignore', 'pipe', 'pipe'],
      });
    } catch (error) {
      return ['Output: ' + (error.stdout ?? ''), 'Error: ' + (error.stderr ?? ''),
        'Exit Code: ' + (error.status ?? 1)].join(String.fromCharCode(10));
    } finally {
      if (existsSync(destination)) files.set(eventPath, readFileSync(destination, 'utf8'));
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}
