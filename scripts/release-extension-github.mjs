// GET/verification only. Neither package execution nor publication is permitted here.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { verifyAuthoritativeTransfer } from './release-extension-policy.mjs';
export function createReadonlyGithub(directory,
 {
  deadline = Date.now() + 15 * 60 * 1000,

  run = spawnSync
} = {}) {
  fs.mkdirSync(directory,
 {
    recursive: true
  });
  let sequence = 0;
  const execute = (args,
 maxBuffer = 32 * 1024 * 1024) => {
    assert.ok(args[0] === 'api' ||
 args.slice(0,
 2).join(' ') === 'attestation verify');
    const remaining = deadline - Date.now();
    assert.ok(remaining > 1000,
 'bounded verification deadline expired');
    const result = run('gh',
 args,
 {
      timeout: Math.min(120000,
 remaining),

      maxBuffer,

      env: {
        ...process.env,

        GH_HOST: 'github.com'
      }
    });
    const prefix = path.join(directory,
 String(sequence++).padStart(3,
 '0'));
    fs.writeFileSync(prefix + '-stdout.bin',
 result.stdout ?? Buffer.alloc(0));
    fs.writeFileSync(prefix + '-stderr.bin',
 result.stderr ?? Buffer.alloc(0));
    fs.writeFileSync(prefix + '-process.json',
 JSON.stringify({
      args,

      status: result.status,

      signal: result.signal,

      error: result.error?.message
    },
 null,
 2));
    assert.equal(result.status,
 0);
    assert.equal(result.signal,
 null);
    assert.equal(result.error,
 undefined);
    return result.stdout;
  };
  return {
    json: endpoint => JSON.parse(execute(['api',
 endpoint]).toString('utf8')),

    bytes: endpoint => execute(['api',
 '-H',
 'Accept: application/octet-stream',
 endpoint],
 400 * 1024 * 1024),

    verify: request => {
      assert.match(request.name,
 /^[A-Za-z0-9_.-]+$/u);
      const file = path.join(directory,
 request.name);
      fs.writeFileSync(file,
 request.bytes);
      return JSON.parse(execute(['attestation',
 'verify',
 file,
 '--repo',
 request.repository,
 '--signer-workflow',
 request.signerWorkflow,
 '--signer-digest',
 request.signerDigest,
 '--deny-self-hosted-runners',
 '--format',
 'json']).toString('utf8'));
    }
  };
}
export function transferFromAuthenticatedGithub(expected,
 consumerCheckJobId,
 directory,
 github = createReadonlyGithub(directory)) {
  const base = 'repos/' + expected.repository;
  const run = github.json(base + '/actions/runs/' + expected.runId + '/attempts/' + expected.attempt);
  const jobs = github.json(base + '/actions/runs/' + expected.runId + '/attempts/' + expected.attempt + '/jobs?per_page=100');
  assert.equal(jobs.total_count,
 jobs.jobs.length,
 'producer job pagination must be complete');
  const release = github.json(base + '/releases/tags/' + expected.tag);
  const filename = 'formal-ai-vscode-' + expected.tag.slice(1) + '.vsix';
  const asset = name => {
    const matches = release.assets.filter(a => a.name === name);
    assert.equal(matches.length,
 1,
 'missing/duplicate authoritative asset: ' + name);
    return github.bytes(base + '/releases/assets/' + matches[0].id);
  };
  const evidence = {
    run,

    jobs: jobs.jobs,

    release,

    tagCommit: github.json(base + '/commits/' + expected.tag),

    consumerChecks: github.json(base + '/actions/jobs/' + consumerCheckJobId),

    sourceBytes: asset('formal-ai-native-source-' + expected.tag.slice(1) + '.json'),

    checksumBytes: asset('SHA256SUMS.txt'),

    artifactBytes: asset(filename)
  };
  return verifyAuthoritativeTransfer(expected,
 evidence,
 github.verify);
}
