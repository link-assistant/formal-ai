// Reconcile an ambiguous GitHub write before a finite retry of the original publisher.
import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
import {boundedCommand} from '../js/server/bounded-command.mjs';

const successful = result => result.status === 0 && !result.error && !result.signal &&
  !result.timed_out && result.cleanup_complete === true;
const text = value => value?.toString('utf8') ?? '';

export async function createReleaseWithRecovery(args, {
  environment = process.env, transport = boundedCommand,
  pause = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds)),
  report = ({stdout, stderr, ...record}) => {
    console.log(JSON.stringify(record));
    if (stdout) process.stdout.write(stdout + (stdout.endsWith('\n') ? '' : '\n'));
    if (stderr) process.stderr.write(stderr + (stderr.endsWith('\n') ? '' : '\n'));
  },
} = {}) {
  const repository = environment.GITHUB_REPOSITORY, version = environment.RELEASE_VERSION;
  assert.match(repository ?? '', /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.match(version ?? '', /^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?$/u);
  assert(Array.isArray(args) && args.length <= 32 && args.every(arg => typeof arg === 'string'));
  const redact = value => environment.GH_TOKEN ? value.replaceAll(environment.GH_TOKEN, '***') : value;
  const detail = result => ({status: result.status, signal: result.signal, timedOut: result.timed_out,
    cleanupComplete: result.cleanup_complete, error: result.error?.code,
    stdout: redact(text(result.stdout)), stderr: redact(text(result.stderr))});
  const invoke = (file, argv, timeout) => transport(file, argv,
    {env: environment, timeout, stdoutLimitBytes: 1048576, stderrLimitBytes: 1048576});
  let source = environment.EXPECT_SOURCE;
  if (source === undefined) {
    const head = await invoke('git', ['rev-parse', 'HEAD'], 30000);
    assert(successful(head), 'local release source could not be authenticated');
    source = text(head.stdout).trim();
  }
  assert.match(source, /^[a-f0-9]{40}$/u, 'exact release source required');
  const tag = 'v' + version;
  for (let attempt = 1; attempt <= 3; attempt++) {
    const result = await invoke('rust-script', ['scripts/create-github-release.rs', ...args], 180000);
    report({phase: 'release-create', attempt, ...detail(result)});
    if (successful(result)) return {complete: true, attempt, reconciled: false};
    const failure = text(result.stderr) + text(result.stdout);
    const transient = /^Error creating release: unexpected end of JSON input\s*$/mu.test(failure);
    assert(result.cleanup_complete === true && !result.error && !result.signal && !result.timed_out && transient,
      'release creation failed without retryable transport evidence: ' + JSON.stringify(detail(result)));
    // POST may have committed even when its response was lost. Only an authenticated,
    // exact tag/source observation can credit that write or authorize another attempt.
    const observed = await invoke('gh', ['api', '--include', 'repos/' + repository + '/releases/tags/' + tag], 30000);
    report({phase: 'release-reconcile', attempt, ...detail(observed)});
    assert(observed.cleanup_complete === true && !observed.error && !observed.signal && !observed.timed_out,
      'release reconciliation transport failed');
    const response = /^HTTP\/[\d.]+ (\d{3})[^\r\n]*(?:\r?\n[^\r\n]+)*\r?\n\r?\n([\s\S]*)$/u.exec(text(observed.stdout));
    assert(response, 'release reconciliation returned no authenticated HTTP response');
    const body = JSON.parse(response[2]);
    if (response[1] === '200') {
      assert(successful(observed) && Number.isSafeInteger(body.id) && body.id > 0 &&
        body.tag_name === tag && body.draft === false && body.prerelease === false &&
        body.html_url === 'https://github.com/' + repository + '/releases/tag/' + tag,
      'existing release identity or publication state differs');
      const commit = await invoke('gh', ['api', 'repos/' + repository + '/commits/' + tag], 30000);
      assert(successful(commit), 'existing release source could not be authenticated');
      assert.equal(JSON.parse(text(commit.stdout)).sha, source, 'existing release source differs');
      return {complete: true, attempt, reconciled: true};
    }
    assert(response[1] === '404' && observed.status !== 0 && body.status === '404',
      'release absence could not be authenticated');
    assert(attempt < 3, 'release creation exhausted three attempts; exact release is still absent');
    await pause(attempt * 2000);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try { console.log(JSON.stringify(await createReleaseWithRecovery(process.argv.slice(2)))); }
  catch (error) { console.error(error.message); process.exitCode = 1; }
}
