#!/usr/bin/env node
// The Main producer retains only source coordinates from the original event, never the full payload.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
export function sourceEventRecord(environment, event) {
  assert.equal(environment.GITHUB_ACTIONS, 'true');
  assert.match(environment.GITHUB_REPOSITORY ?? '', /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.ok(environment.GITHUB_WORKFLOW_REF?.startsWith(environment.GITHUB_REPOSITORY + '/.github/workflows/release.yml@'));
  for (const key of ['GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT']) assert.match(environment[key] ?? '', /^[1-9]\d*$/u);
  assert.equal(event.repository.full_name, environment.GITHUB_REPOSITORY);
  assert.match(environment.GITHUB_SHA, /^[a-f0-9]{40}$/u);
  const source = {
    schema: 'formal-ai-hive-source-event/v1',
    repository: environment.GITHUB_REPOSITORY,
    run: Number(environment.GITHUB_RUN_ID),
    attempt: Number(environment.GITHUB_RUN_ATTEMPT),
    event: environment.GITHUB_EVENT_NAME,
    source: environment.GITHUB_SHA,
    ref: environment.GITHUB_REF
  };
  if (source.event === 'push') {
    assert.match(event.before, /^[a-f0-9]{40}$/u);
    assert.notEqual(event.before, '0'.repeat(40));
    assert.equal(event.after, source.source);
    assert.equal(event.ref, source.ref);
    assert.match(event.ref, /^refs\/heads\//u);
    return {
      ...source,
      before: event.before,
      after: event.after
    };
  }
  assert.ok(['pull_request', 'workflow_dispatch'].includes(source.event));
  return source;
}
export function main(environment = process.env) {
  const event = JSON.parse(fs.readFileSync(environment.GITHUB_EVENT_PATH, 'utf8'));
  const record = sourceEventRecord(environment, event);
  const destination = environment.HIVE_SOURCE_EVENT_OUTPUT;
  assert.ok(destination && path.isAbsolute(destination));
  fs.mkdirSync(path.dirname(destination), {
    recursive: true
  });
  fs.writeFileSync(destination, JSON.stringify(record, null, 2) + '\n', {
    flag: 'wx',
    mode: 0o600
  });
  console.log(JSON.stringify({
    schema: record.schema,
    source: record.source,
    event: record.event
  }));
}
if (process.argv[1] && import.meta.url === pathToFileURL(fs.realpathSync(process.argv[1])).href) main();
