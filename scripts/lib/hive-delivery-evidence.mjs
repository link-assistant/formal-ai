import assert from 'node:assert/strict';
export function requireExactProgramOutput(observation, expected) {
  assert.equal(observation.exitCode, 0, 'independent program failed');
  assert.equal(observation.signal, null, 'independent program was signalled');
  assert.equal(Buffer.from(observation.stdoutBase64, 'base64').toString('base64'),
    observation.stdoutBase64, 'noncanonical stdout encoding');
  assert.deepEqual(Buffer.from(observation.stdoutBase64, 'base64'), Buffer.from(expected),
    'original exactly-one-line stdout oracle');
  assert.equal(observation.stderrBase64, '', 'unexpected program stderr');
}
export function requireHarnessDelivery(evidence, tool) {
  assert.equal(evidence.tool, tool);
  assert.equal(evidence.model, 'formal-ai');
  assert.equal(evidence.passed, true, 'Hive harness did not deliver');
  assert.equal(evidence.solveExitCode, 0, 'actual solve did not exit successfully');
  for (const group of ['harness', 'evaluation']) {
    const rows = group === 'harness' ? evidence.harness : evidence.evaluation?.results;
    assert(Array.isArray(rows) && rows.length > 0, 'missing original ' + group + ' assertions');
    assert(rows.every(row => row.ok === true), 'original ' + group + ' assertion failed');
  }
  assert.equal(evidence.verdict?.harness, true);
  assert.equal(evidence.verdict?.model, true);
  const url = new URL(evidence.pullRequestUrl);
  assert.equal(url.origin, 'https://github.com');
  const parsed = /^\/([^/]+\/[^/]+)\/pull\/([1-9]\d*)$/u.exec(url.pathname);
  assert(parsed, 'actual pull-request URL required');
  return {repository: parsed[1], number: Number(parsed[2])};
}
export function requireReadyPullRequest(value, repository, number) {
  assert.equal(value.number, number);
  assert.equal(value.base?.repo?.full_name, repository);
  assert.equal(value.state, 'open');
  assert.equal(value.draft, false, 'delivered PR must be ready');
  assert.match(value.head?.sha, /^[a-f0-9]{40}$/u);
  assert.match(value.head?.ref, /^issue-[1-9]\d*-[A-Za-z0-9-]+$/u);
  assert.match(value.base?.ref, /^e2e\/hello-world\//u);
  return value.head.sha;
}
export function requireProgramMember(files, language) {
  const extension = {Kotlin: '.kt', Scala: '.scala', Rust: '.rs'}[language];
  assert(extension, 'unsupported original language');
  const programs = files.filter(file => file.filename.endsWith(extension));
  assert.equal(programs.length, 1, 'exactly one original-language program required');
  const file = programs[0].filename;
  assert(file.split('/').every(part => part && part !== '.' && part !== '..'));
  assert(!file.startsWith('/') && !file.includes('\\') && !file.includes('\0'));
  return file;
}
