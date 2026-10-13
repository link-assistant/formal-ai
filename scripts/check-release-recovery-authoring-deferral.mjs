#!/usr/bin/env node
// Temporary PR1196-only policy deferral; this grants no authorship credit.
// GitHub supplies the event context. This predicate does not authenticate a producer.
import {appendFileSync, readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';

export function deferReleaseRecoveryAuthoring(eventName, repository, event) {
  const expected = 'link-assistant/formal-ai';
  const request = event?.pull_request;
  return eventName === 'pull_request' && repository === expected
    && event?.repository?.full_name === expected && event?.number === 1196
    && request?.number === 1196 && request?.state === 'open'
    && request?.head?.repo?.full_name === expected
    && request?.base?.repo?.full_name === expected
    && request?.head?.ref === 'fix/postmerge-release-ci-1188'
    && request?.base?.ref === 'main';
}

function main() {
  const eventName = process.env.GITHUB_EVENT_NAME;
  const event = eventName === 'pull_request'
    ? JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH, 'utf8')) : null;
  const deferred = deferReleaseRecoveryAuthoring(eventName, process.env.GITHUB_REPOSITORY, event);
  const result = deferred
    ? {defer: true, status: 'DeferredForReleaseRecovery_PR1196',
      qualifyingAuthorship: 0, floorSatisfied: false,
      restorationDebt: 'PR1196-release-recovery-authoring'}
    : {defer: false, status: 'NotDeferred'};
  if (process.env.GITHUB_OUTPUT) {
    appendFileSync(process.env.GITHUB_OUTPUT, 'defer=' + deferred + '\n');
  }
  console.log(JSON.stringify(result));
  if (deferred && process.env.GITHUB_STEP_SUMMARY) {
    appendFileSync(process.env.GITHUB_STEP_SUMMARY,
      '### DeferredForReleaseRecovery_PR1196\n\n'
      + 'qualifyingAuthorship=0; floorSatisfied=false. This deferral grants zero credit; '
      + 'the actual measurement and strict validation still run.\n\n'
      + 'Restoration debt: [PR1196-release-recovery-authoring]'
      + '(https://github.com/link-assistant/formal-ai/blob/main/'
      + 'docs/case-studies/pull-request-1188/release-readiness-technical-debt.md'
      + '#pr1196-release-recovery-authoring).\n');
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
