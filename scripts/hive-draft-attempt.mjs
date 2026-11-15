#!/usr/bin/env node
// Issue #1484: the Formal AI side of the Hive Mind draft attempt identity.
//   --select          build job: admit the once-resolved sources and the built binary.
//   --verify-custody  draft job, before any task effect: the downloaded binary and checkouts are the admitted ones.
//   --verify-hive     draft job, after the shared attempt: Hive Mind ran and served exactly that selection.
// Missing artifacts end as deferred-infrastructure and wrong ones as identity-mismatch, with zero credit.
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {appendFileSync, existsSync, readFileSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {AttemptIdentityError, ATTEMPT_OUTCOMES, FILES, parseAttemptIdentity, selectAttemptIdentity, summarizeAttempt,
  verifyAttemptCustody, verifyHiveAttempt, versionOf} from './lib/hive-draft-attempt-identity.mjs';

const readJson = path => existsSync(path) ? JSON.parse(readFileSync(path, 'utf8')) : null;
const gitHead = dir => execFileSync('git', ['-C', dir, 'rev-parse', 'HEAD'], {encoding: 'utf8', timeout: 15000}).trim();
const inspectBinary = path => {
  if (!path || !existsSync(path)) return null;
  const version = execFileSync(path, ['--version'], {encoding: 'utf8', timeout: 60000}).trim();
  const bytes = readFileSync(path);
  return {sha256: createHash('sha256').update(bytes).digest('hex'), bytes: bytes.length, version};
};
const annotation = (level, title, message) => '::' + level + ' title=' + title + '::' + String(message).replace(/%/gu, '%25').replace(/\r/gu, '%0D').replace(/\n/gu, '%0A');

// `io.log` receives every line meant for the job log, `::error`/`::warning` workflow commands included. Tests pass a
// collector, so the commands their fixtures provoke are asserted instead of becoming annotations on a green job (#1569).
export function main(env = process.env, argv = process.argv.slice(2), io = {inspectBinary, gitHead}) {
  const log = io.log ?? console.log;
  const logDir = env.FORMAL_AI_DRAFT_LOG_DIR;
  if (!logDir) throw Error('FORMAL_AI_DRAFT_LOG_DIR is required');
  const write = (name, value) => writeFileSync(join(logDir, name), JSON.stringify(value, null, 2) + '\n');
  const output = text => env.GITHUB_OUTPUT && appendFileSync(env.GITHUB_OUTPUT, text);
  const summary = text => env.GITHUB_STEP_SUMMARY && appendFileSync(env.GITHUB_STEP_SUMMARY, text + '\n');
  let identity = null;
  try {
    if (argv.includes('--select')) {
      const hivePackage = readJson(join(env.HIVE_MIND_DIR ?? '.hive-mind', 'package.json'));
      const binary = io.inspectBinary(env.FORMAL_AI_BINARY);
      identity = selectAttemptIdentity({
        repository: env.GITHUB_REPOSITORY, issue: env.FORMAL_AI_DRAFT_ISSUE,
        formalAi: {sourceSha: env.FORMAL_AI_SELECTED_SHA, checkedOutSha: io.gitHead(env.FORMAL_AI_SOURCE_DIR ?? '.formal-ai-source'),
          version: versionOf(binary?.version), binarySha256: binary?.sha256, binaryBytes: binary?.bytes ?? null, binarySource: env.FORMAL_AI_BINARY_SOURCE || 'build'},
        hive: {sourceSha: env.HIVE_MIND_SOURCE_SHA, checkedOutSha: io.gitHead(env.HIVE_MIND_DIR ?? '.hive-mind'), version: hivePackage?.version, image: env.HIVE_MIND_IMAGE},
        selected: {at: env.FORMAL_AI_SELECTED_AT || new Date().toISOString(), runId: env.GITHUB_RUN_ID ?? null, runAttempt: env.GITHUB_RUN_ATTEMPT ?? null,
          runUrl: env.GITHUB_RUN_ID ? (env.GITHUB_SERVER_URL ?? 'https://github.com') + '/' + env.GITHUB_REPOSITORY + '/actions/runs/' + env.GITHUB_RUN_ID : null},
      });
      write(FILES.identity, identity);
      output('attempt_id=' + identity.id + '\nattempt_mode=' + identity.mode + '\nattempt_identity=' + JSON.stringify(identity) + '\n');
      summary(summarizeAttempt(identity));
      log('Admitted attempt ' + identity.id + ': Formal AI ' + identity.formalAi.sourceSha + ' (' + identity.formalAi.version + ', sha256 ' + identity.formalAi.binarySha256 + '), Hive Mind ' + identity.hive.sourceSha + ' (' + identity.hive.image + ').');
    } else if (argv.includes('--verify-custody')) {
      if (!(env.FORMAL_AI_ATTEMPT_IDENTITY ?? '').trim())
        throw new AttemptIdentityError(ATTEMPT_OUTCOMES.deferred, 'no attempt identity was admitted by the build job; nothing is run on an unselected backend');
      identity = parseAttemptIdentity(env.FORMAL_AI_ATTEMPT_IDENTITY);
      const custody = verifyAttemptCustody({identity, binary: io.inspectBinary(env.FORMAL_AI_DRAFT_BINARY), hiveCheckoutSha: io.gitHead(env.HIVE_MIND_DIR ?? '.hive-mind'), issue: env.FORMAL_AI_DRAFT_ISSUE});
      write(FILES.identity, identity);
      log('Custody verified for attempt ' + identity.id + ': binary sha256 ' + custody.binarySha256 + ', Hive Mind ' + custody.hiveSourceSha + '.');
    } else if (argv.includes('--verify-hive')) {
      identity = parseAttemptIdentity(env.FORMAL_AI_ATTEMPT_IDENTITY);
      const receipt = verifyHiveAttempt({identity, hiveIdentity: readJson(join(logDir, FILES.hiveIdentity)), hiveRecord: readJson(join(logDir, FILES.hiveRecord)), hiveOutcome: readJson(join(logDir, FILES.hiveOutcome))});
      write(FILES.receipt, receipt);
      output('attempt_outcome=' + receipt.outcome + '\ncurrent_source_credit=' + receipt.currentSourceCredit + '\n');
      summary(summarizeAttempt(identity, receipt));
      if (receipt.outcome === ATTEMPT_OUTCOMES.mismatch) throw new AttemptIdentityError(receipt.outcome, receipt.reason, {differences: receipt.differences ?? []});
      if (receipt.outcome !== ATTEMPT_OUTCOMES.verified) log(annotation('warning', 'Formal AI draft ' + receipt.outcome, receipt.reason + '; zero current-source credit'));
      else log('Hive Mind served the admitted selection: ' + receipt.servingExeSha256 + ' in ' + receipt.imageReference + '.');
    } else throw Error('expected --select, --verify-custody or --verify-hive');
  } catch (error) {
    if (!(error instanceof AttemptIdentityError)) throw error;
    const record = {schema: 'formal-ai/hive-draft-attempt-outcome/v1', attemptId: identity?.id ?? null, mode: identity?.mode ?? null,
      outcome: error.outcome, message: error.message, evidence: error.evidence, capabilityCredit: 0, at: new Date().toISOString()};
    write(FILES.outcome, record);
    output('attempt_outcome=' + error.outcome + '\n');
    log(annotation('error', 'Formal AI draft ' + error.outcome, error.message));
    process.exitCode = 1;
    return record;
  }
  return identity;
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
