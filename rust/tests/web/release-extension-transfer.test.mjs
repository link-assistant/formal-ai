// Controlled text and declared mock verification only; no production authority or publication proof.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createHash } from 'node:crypto';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { verifyAuthoritativeTransfer } from '../../../scripts/release-extension-policy.mjs';
import { selectCompletedSource } from '../../../scripts/release-extension-selection.mjs';
import { receiveExtension,
 resolveExtensionRelease } from '../../../scripts/release-extension-transfer.mjs';
import { resolvePackageRelease } from '../../../scripts/resolve-package-release.mjs';
test('signed artifact policy refuses source, subject and producer identity drift at the declared verifier boundary',
 () => {
  const expected = {
    repository: 'owner/repository',

    tag: 'v1.2.3',

    sourceCommit: 'a'.repeat(40),

    sourceTree: 'b'.repeat(40),

    protocolCommit: 'c'.repeat(40),

    runId: 11,

    attempt: 2,

    consumerCheckJobId: 44,

    consumerRunId: 10,

    consumerAttempt: 1
  };
  const sha = b => createHash('sha256').update(b).digest('hex');
  const artifactBytes = Buffer.from('CONTROLLED TEXT NOT REAL VSIX\n'),

    name = 'formal-ai-vscode-1.2.3.vsix';
  const sourceBytes = Buffer.from(JSON.stringify({
    version: 1,

    release_tag: expected.tag,

    source_commit: expected.sourceCommit,

    source_tree: expected.sourceTree,

    producer_run: '11'
  }));
  const evidence = {
    run: {
      id: 11,

      run_attempt: 2,

      repository: {
        full_name: expected.repository
      },

      head_repository: {
        full_name: expected.repository
      },

      head_branch: 'main',

      path: '.github/workflows/desktop-release.yml',

      event: 'workflow_run',

      status: 'completed',

      conclusion: 'success',

      head_sha: expected.protocolCommit
    },

    jobs: ['Package VS Code extension (.vsix)',
 'Publish SHA256SUMS.txt + provenance'].map((name,
 i) => ({
      name,

      id: 20 + i,

      run_id: 11,

      run_attempt: 2,

      status: 'completed',

      conclusion: 'success'
    })),

    consumerChecks: {
      id: 44,

      run_id: 10,

      run_attempt: 1,

      status: 'completed',

      conclusion: 'success'
    },

    release: {
      tag_name: expected.tag,

      draft: false,

      prerelease: false,

      assets: [{
        id: 33,

        name,

        state: 'uploaded',

        size: artifactBytes.length,

        digest: 'sha256:' + sha(artifactBytes)
      }]
    },

    tagCommit: {
      sha: expected.sourceCommit,

      commit: {
        tree: {
          sha: expected.sourceTree
        }
      }
    },

    artifactBytes,

    sourceBytes,

    checksumBytes: Buffer.from(sha(artifactBytes) + '  ' + name + '\n')
  };
  const verification = [];
  const fakeVerified = request => {
    verification.push(request);
    return [{
      verificationResult: {
        signature: {
          certificate: {
            runInvocationURI: 'https://github.com/owner/repository/actions/runs/11/attempts/2',

            buildSignerURI: 'https://github.com/owner/repository/.github/workflows/desktop-release.yml@refs/heads/main',

            buildSignerDigest: expected.protocolCommit,

            sourceRepositoryURI: 'https://github.com/owner/repository',

            issuer: 'https://token.actions.githubusercontent.com',

            runnerEnvironment: 'github-hosted'
          }
        },

        statement: {
          predicateType: 'https://slsa.dev/provenance/v1',

          subject: [{
            name: request.name,

            digest: {
              sha256: sha(request.bytes)
            }
          }],

          predicate: {
            runDetails: {
              metadata: {
                invocationId: 'https://github.com/owner/repository/actions/runs/11/attempts/2'
              }
            }
          }
        }
      }
    }];
  };
  const positive = verifyAuthoritativeTransfer(expected,
 evidence,
 fakeVerified);
  assert.ok(positive.bytes.equals(artifactBytes));
  assert.equal(verification.length,
 2);
  for (const call of verification) {
    assert.equal(call.signerWorkflow,
 'owner/repository/.github/workflows/desktop-release.yml');
    assert.equal(call.signerDigest,
 expected.protocolCommit);
  }
  let refusals = 0;
  for (const mutate of [e => e.run.repository.full_name = 'fork/repository',
 e => e.run.head_repository.full_name = 'fork/repository',
 e => e.run.head_branch = 'feature',
 e => e.run.path = '.github/workflows/foreign.yml',
 e => e.run.event = 'pull_request',
 e => e.run.status = 'in_progress',
 e => e.run.id = 12,
 e => e.run.run_attempt = 1,
 e => e.run.head_sha = 'd'.repeat(40),
 e => e.jobs[0].conclusion = 'skipped',
 e => e.jobs[1].conclusion = 'failure',
 e => e.jobs[0].run_attempt = 1,
 e => e.jobs.push(e.jobs[0]),
 e => e.consumerChecks.conclusion = 'failure',
 e => e.consumerChecks.run_id = 99,
 e => e.release.prerelease = true,
 e => e.release.tag_name = 'v2.0.0',
 e => e.tagCommit.sha = 'd'.repeat(40),
 e => e.tagCommit.commit.tree.sha = 'd'.repeat(40),
 e => e.sourceBytes = Buffer.from('{}'),
 e => e.artifactBytes = Buffer.from('different'),
 e => e.checksumBytes = Buffer.from(''),
 e => e.release.assets = [],
 e => e.release.assets[0].digest = 'sha256:' + '0'.repeat(64)]) {
    const changed = structuredClone(evidence);
    mutate(changed);
    assert.throws(() => verifyAuthoritativeTransfer(expected,
 changed,
 fakeVerified));
    refusals++;
  }
  for (const bad of [request => {
    const v = fakeVerified(request);
    v[0].verificationResult.signature.certificate.buildSignerDigest = 'd'.repeat(40);
    return v;
  },
 request => {
    const v = fakeVerified(request);
    v[0].verificationResult.signature.certificate.runInvocationURI = 'foreign';
    return v;
  },
 request => {
    const v = fakeVerified(request);
    v[0].verificationResult.signature.certificate.buildSignerURI = 'foreign';
    return v;
  },
 () => [],
 () => true,
 request => {
    const v = fakeVerified(request);
    v[0].verificationResult.statement.predicate.runDetails.metadata.invocationId = 'https://github.com/owner/repository/actions/runs/11/attempts/1';
    return v;
  },
 request => {
    const v = fakeVerified(request);
    v[0].verificationResult.statement.subject[0].digest.sha256 = '0'.repeat(64);
    return v;
  }]) {
    assert.throws(() => verifyAuthoritativeTransfer(expected,
 evidence,
 bad));
    refusals++;
  }
});
test('completed source selection preserves exact run/attempt/archive authority and refuses missing or ambiguous candidates',
 () => {
  const sha = b => createHash('sha256').update(b).digest('hex');
  const crc = bytes => {
    let value = 0xffffffff;
    for (const byte of bytes) {
      value ^= byte;
      for (let bit = 0; bit < 8; bit++) value = value >>> 1 ^ (value & 1 ? 0xedb88320 : 0);
    }
    return (value ^ 0xffffffff) >>> 0;
  };
  function archive(source) {
    const bytes = Buffer.from(JSON.stringify(source)),

      name = Buffer.from('source-selection.json'),

      checksum = crc(bytes);
    const local = Buffer.alloc(30);
    local.writeUInt32LE(0x04034b50);
    local.writeUInt16LE(20,
 4);
    local.writeUInt32LE(checksum,
 14);
    local.writeUInt32LE(bytes.length,
 18);
    local.writeUInt32LE(bytes.length,
 22);
    local.writeUInt16LE(name.length,
 26);
    const central = Buffer.alloc(46);
    central.writeUInt32LE(0x02014b50);
    central.writeUInt16LE(20,
 4);
    central.writeUInt16LE(20,
 6);
    central.writeUInt32LE(checksum,
 16);
    central.writeUInt32LE(bytes.length,
 20);
    central.writeUInt32LE(bytes.length,
 24);
    central.writeUInt16LE(name.length,
 28);
    const end = Buffer.alloc(22);
    end.writeUInt32LE(0x06054b50);
    end.writeUInt16LE(1,
 8);
    end.writeUInt16LE(1,
 10);
    end.writeUInt32LE(central.length + name.length,
 12);
    end.writeUInt32LE(local.length + name.length + bytes.length,
 16);
    return Buffer.concat([local,
 name,
 bytes,
 central,
 name,
 end]);
  }
  const expected = {
    repository: 'owner/repository',

    runId: 11,

    attempt: 2
  };
  const source = {
    version: 1,

    producer_run: '11',

    release_tag: 'v1.2.3',

    package_version: '1.2.3',

    source_commit: 'a'.repeat(40),

    source_tree: 'b'.repeat(40)
  };
  const run = {
    id: 11,

    run_attempt: 2,

    repository: {
      full_name: expected.repository
    },

    head_repository: {
      full_name: expected.repository
    },

    head_branch: 'main',

    path: '.github/workflows/desktop-release.yml',

    status: 'completed',

    event: 'workflow_run',

    conclusion: 'failure',

    head_sha: 'c'.repeat(40)
  };
  const jobs = {
    total_count: 2,

    jobs: ['Package VS Code extension (.vsix)',
 'Publish SHA256SUMS.txt + provenance'].map((name,
 i) => ({
      id: 20 + i,

      name,

      run_id: 11,

      run_attempt: 2,

      status: 'completed',

      conclusion: 'success'
    }))
  };
  function observed(mutate = () => {}) {
    const state = {
      run: structuredClone(run),

      jobs: structuredClone(jobs),

      source: structuredClone(source),

      list: {
        total_count: 1,

        artifacts: [{
          id: 33,

          name: 'native-release-source'
        }]
      },

      metadata: null
    };
    mutate(state);
    const bytes = archive(state.source);
    state.metadata ??= {
      id: 33,

      name: 'native-release-source',

      expired: false,

      workflow_run: {
        id: 11,

        head_sha: run.head_sha
      },

      size_in_bytes: bytes.length,

      digest: 'sha256:' + sha(bytes)
    };
    let repeated = 0;
    const github = {
      json: endpoint => {
        if (endpoint.endsWith('/jobs?per_page=100')) return state.jobs;
        if (endpoint.endsWith('/artifacts?per_page=100')) return state.list;
        if (endpoint.endsWith('/actions/artifacts/33')) {
          repeated++;
          const result = structuredClone(state.metadata);
          if (state.drift &&
 repeated === 2) result.expired = true;
          return result;
        }
        if (endpoint.endsWith('/attempts/2')) return state.run;
        throw Error('unknown API endpoint: ' + endpoint);
      },

      bytes: endpoint => {
        assert.ok(endpoint.endsWith('/actions/artifacts/33/zip'));
        return bytes;
      }
    };
    return selectCompletedSource(expected,
 github);
  }
  const positive = observed();
  assert.equal(positive.publicationAuthorized,
 false);
  assert.equal(positive.selectionOnly,
 true);
  assert.equal(positive.tag,
 'v1.2.3');
  let refusals = 0;
  for (const mutate of [s => s.run.repository.full_name = 'fork/repository',
 s => s.run.head_repository.full_name = 'fork/repository',
 s => s.run.head_branch = 'feature',
 s => s.run.path = '.github/workflows/foreign.yml',
 s => s.run.event = 'pull_request',
 s => s.run.status = 'queued',
 s => s.run.conclusion = 'cancelled',
 s => s.run.run_attempt = 1,
 s => s.jobs.total_count = 3,
 s => s.jobs.jobs[0].conclusion = 'failure',
 s => s.jobs.jobs[1].status = 'queued',
 s => s.jobs.jobs[0].run_attempt = 1,
 s => s.jobs.jobs.push(s.jobs.jobs[0]),
 s => s.list.artifacts = [],
 s => s.list.artifacts.push(s.list.artifacts[0]),
 s => s.list.total_count = 2,
 s => s.source.producer_run = '12',
 s => s.source.release_tag = null,
 s => s.source.package_version = '1.2.4',
 s => s.source.source_tree = 'foreign',
 s => s.drift = true,
 s => s.metadata = {
    id: 33,

    name: 'native-release-source',

    expired: true
  }]) {
    assert.throws(() => observed(mutate));
    refusals++;
  }
});
test('connected readonly transfer writes exact producer bytes once and preserves all original checks',
 () => {
  const {
    expected,

    evidence,

    fakeVerified
  } = (() => {
    const expected = {
      repository: 'owner/repository',

      tag: 'v1.2.3',

      sourceCommit: 'a'.repeat(40),

      sourceTree: 'b'.repeat(40),

      protocolCommit: 'c'.repeat(40),

      runId: 11,

      attempt: 2,

      consumerCheckJobId: 44,

      consumerRunId: 10,

      consumerAttempt: 1
    };
    const sha = b => createHash('sha256').update(b).digest('hex');
    const artifactBytes = Buffer.from('CONTROLLED TEXT NOT REAL VSIX\n'),

      name = 'formal-ai-vscode-1.2.3.vsix';
    const sourceBytes = Buffer.from(JSON.stringify({
      version: 1,

      release_tag: expected.tag,

      source_commit: expected.sourceCommit,

      source_tree: expected.sourceTree,

      producer_run: '11'
    }));
    const evidence = {
      run: {
        id: 11,

        run_attempt: 2,

        repository: {
          full_name: expected.repository
        },

        head_repository: {
          full_name: expected.repository
        },

        head_branch: 'main',

        path: '.github/workflows/desktop-release.yml',

        event: 'workflow_run',

        status: 'completed',

        conclusion: 'success',

        head_sha: expected.protocolCommit
      },

      jobs: ['Package VS Code extension (.vsix)',
 'Publish SHA256SUMS.txt + provenance'].map((name,
 i) => ({
        name,

        id: 20 + i,

        run_id: 11,

        run_attempt: 2,

        status: 'completed',

        conclusion: 'success'
      })),

      consumerChecks: {
        id: 44,

        run_id: 10,

        run_attempt: 1,

        status: 'completed',

        conclusion: 'success'
      },

      release: {
        tag_name: expected.tag,

        draft: false,

        prerelease: false,

        assets: [{
          id: 33,

          name,

          state: 'uploaded',

          size: artifactBytes.length,

          digest: 'sha256:' + sha(artifactBytes)
        }]
      },

      tagCommit: {
        sha: expected.sourceCommit,

        commit: {
          tree: {
            sha: expected.sourceTree
          }
        }
      },

      artifactBytes,

      sourceBytes,

      checksumBytes: Buffer.from(sha(artifactBytes) + '  ' + name + '\n')
    };
    const verification = [];
    const fakeVerified = request => {
      verification.push(request);
      return [{
        verificationResult: {
          signature: {
            certificate: {
              runInvocationURI: 'https://github.com/owner/repository/actions/runs/11/attempts/2',

              buildSignerURI: 'https://github.com/owner/repository/.github/workflows/desktop-release.yml@refs/heads/main',

              buildSignerDigest: expected.protocolCommit,

              sourceRepositoryURI: 'https://github.com/owner/repository',

              issuer: 'https://token.actions.githubusercontent.com',

              runnerEnvironment: 'github-hosted'
            }
          },

          statement: {
            predicateType: 'https://slsa.dev/provenance/v1',

            subject: [{
              name: request.name,

              digest: {
                sha256: sha(request.bytes)
              }
            }],

            predicate: {
              runDetails: {
                metadata: {
                  invocationId: 'https://github.com/owner/repository/actions/runs/11/attempts/2'
                }
              }
            }
          }
        }
      }];
    };
    return {
      expected,

      evidence,

      fakeVerified
    };
  })();
  const sha = b => createHash('sha256').update(b).digest('hex');
  const crc = bytes => {
    let value = 0xffffffff;
    for (const byte of bytes) {
      value ^= byte;
      for (let bit = 0; bit < 8; bit++) value = value >>> 1 ^ (value & 1 ? 0xedb88320 : 0);
    }
    return (value ^ 0xffffffff) >>> 0;
  };
  function archive(source) {
    const bytes = Buffer.from(JSON.stringify(source)),

      name = Buffer.from('source-selection.json'),

      checksum = crc(bytes);
    const local = Buffer.alloc(30);
    local.writeUInt32LE(0x04034b50);
    local.writeUInt16LE(20,
 4);
    local.writeUInt32LE(checksum,
 14);
    local.writeUInt32LE(bytes.length,
 18);
    local.writeUInt32LE(bytes.length,
 22);
    local.writeUInt16LE(name.length,
 26);
    const central = Buffer.alloc(46);
    central.writeUInt32LE(0x02014b50);
    central.writeUInt16LE(20,
 4);
    central.writeUInt16LE(20,
 6);
    central.writeUInt32LE(checksum,
 16);
    central.writeUInt32LE(bytes.length,
 20);
    central.writeUInt32LE(bytes.length,
 24);
    central.writeUInt16LE(name.length,
 28);
    const end = Buffer.alloc(22);
    end.writeUInt32LE(0x06054b50);
    end.writeUInt16LE(1,
 8);
    end.writeUInt16LE(1,
 10);
    end.writeUInt32LE(central.length + name.length,
 12);
    end.writeUInt32LE(local.length + name.length + bytes.length,
 16);
    return Buffer.concat([local,
 name,
 bytes,
 central,
 name,
 end]);
  }
  const source = {
    ...JSON.parse(evidence.sourceBytes),

    package_version: '1.2.3'
  };
  const sourceBytes = Buffer.from(JSON.stringify(source)),

    zip = archive(source);
  const metadata = {
    id: 36,

    name: 'native-release-source',

    expired: false,

    workflow_run: {
      id: 11,

      head_sha: expected.protocolCommit
    },

    size_in_bytes: zip.length,

    digest: 'sha256:' + sha(zip)
  };
  const state = {
    run: evidence.run,

    jobs: {
      total_count: 2,

      jobs: evidence.jobs
    },

    consumer: {
      id: 10,

      run_attempt: 1,

      path: '.github/workflows/publish-vscode.yml',

      event: 'workflow_run',

      head_branch: 'main',

      repository: {
        full_name: expected.repository
      },

      head_repository: {
        full_name: expected.repository
      }
    },

    release: {
      ...evidence.release,

      assets: [...evidence.release.assets,
 {
        id: 34,

        name: 'formal-ai-native-source-1.2.3.json'
      },
 {
        id: 35,

        name: 'SHA256SUMS.txt'
      }]
    }
  };
  const event = {
    workflow_run: {
      name: 'Desktop Release',

      id: 11,

      run_attempt: 2,

      head_branch: 'main',

      head_repository: {
        full_name: expected.repository
      }
    }
  };
  const temp = fs.mkdtempSync(join(fs.realpathSync(tmpdir()),
 'extension-transfer-controls-'));
  let calls = 0;
  function githubFor(mutate = () => {}) {
    const current = structuredClone(state);
    mutate(current);
    return {
      json: endpoint => {
        calls++;
        if (endpoint.endsWith('/actions/runs/10/attempts/1')) return current.consumer;
        if (endpoint.endsWith('/actions/runs/11') ||
 endpoint.endsWith('/actions/runs/11/attempts/2')) return current.run;
        if (endpoint.endsWith('/actions/runs/11/attempts/2/jobs?per_page=100')) return current.jobs;
        if (endpoint.endsWith('/actions/runs/11/artifacts?per_page=100')) return {
          total_count: 1,

          artifacts: [{
            id: 36,

            name: 'native-release-source'
          }]
        };
        if (endpoint.endsWith('/actions/artifacts/36')) return metadata;
        if (endpoint.endsWith('/actions/runs/10/attempts/1/jobs?per_page=100')) return {
          total_count: 1,

          jobs: [{
            ...evidence.consumerChecks,

            name: 'publish'
          }]
        };
        if (endpoint.endsWith('/actions/jobs/44')) return evidence.consumerChecks;
        if (endpoint.endsWith('/releases/tags/v1.2.3')) return current.release;
        if (endpoint.endsWith('/commits/v1.2.3')) return evidence.tagCommit;
        throw Error('unknown authenticated endpoint: ' + endpoint);
      },

      bytes: endpoint => {
        calls++;
        if (endpoint.endsWith('/actions/artifacts/36/zip')) return zip;
        if (endpoint.endsWith('/releases/assets/33')) return evidence.artifactBytes;
        if (endpoint.endsWith('/releases/assets/34')) return sourceBytes;
        if (endpoint.endsWith('/releases/assets/35')) return evidence.checksumBytes;
        throw Error('unknown asset endpoint');
      },

      verify: fakeVerified
    };
  }
  const environment = {
    GITHUB_REPOSITORY: expected.repository,

    PACKAGE_RELEASE_TAG: expected.tag,

    GITHUB_RUN_ID: '10',

    GITHUB_RUN_ATTEMPT: '1',

    GITHUB_EVENT_NAME: 'workflow_run',

    GITHUB_REF: 'refs/heads/main',

    EXTENSION_DESTINATION: temp
  };
  try {
    const result = receiveExtension(environment,
 event,
 githubFor(),
 temp);
    assert.deepEqual(fs.readFileSync(result.path),
 evidence.artifactBytes);
    assert.equal(result.sourceCommit,
 expected.sourceCommit);
    assert.equal(result.publicationCommandExecuted,
 false);
    assert.throws(() => receiveExtension(environment,
 event,
 githubFor(),
 temp),
 'no same-name overwrite');
    let refusals = 1;
    for (const mutate of [s => s.consumer.repository.full_name = 'fork/repository',
 s => s.consumer.head_repository.full_name = 'fork/repository',
 s => s.consumer.path = '.github/workflows/foreign.yml',
 s => s.consumer.event = 'pull_request',
 s => s.consumer.head_branch = 'feature',
 s => s.consumer.run_attempt = 2,
 s => s.run.head_repository.full_name = 'fork/repository',
 s => s.jobs.jobs[1].conclusion = 'failure',
 s => s.release.assets.pop()]) {
      assert.throws(() => receiveExtension(environment,
 event,
 githubFor(mutate),
 temp));
      refusals++;
    }
    assert.throws(() => receiveExtension({
      ...environment,

      GITHUB_EVENT_NAME: 'pull_request'
    },
 event,
 githubFor(),
 temp));
    refusals++;
    assert.throws(() => receiveExtension(environment,
 {
      workflow_run: {
        ...event.workflow_run,

        id: 12
      }
    },
 githubFor(),
 temp));
    refusals++;
    const decision = resolveExtensionRelease({
      EVENT: 'workflow_run',

      REPOSITORY: expected.repository
    },
 event,
 githubFor());
    assert.deepEqual(decision,
 {
      tag: expected.tag,

      publish: true,

      build: true
    });
    const ciEnvironment = {
      EVENT: 'workflow_run',

      REPOSITORY: expected.repository,

      RUN_HEAD: expected.protocolCommit,

      RUN_BRANCH: 'main',

      RUN_REPOSITORY: expected.repository,

      RUN_CONCLUSION: 'failure',

      RUN_ID: '22',

      RUN_ATTEMPT: '1',

      RUN_WORKFLOW_ID: '91'
    };
    const ciGithub = {
      json: endpoint => {
        if (endpoint.endsWith('/actions/runs/22')) return {
          ...evidence.run,

          id: 22,

          run_attempt: 1,

          head_sha: expected.protocolCommit,

          workflow_id: 91,

          path: '.github/workflows/release.yml',

          conclusion: 'failure'
        };
        if (endpoint.endsWith('/actions/workflows/release.yml')) return {
          id: 91,

          path: '.github/workflows/release.yml'
        };
        if (endpoint.endsWith('/releases?per_page=30')) return [state.release];
        if (endpoint.endsWith('/commits/v1.2.3')) return {
          ...evidence.tagCommit,

          parents: [{
            sha: expected.protocolCommit
          }]
        };
        throw Error(endpoint);
      }
    };
    assert.deepEqual(resolvePackageRelease(ciEnvironment,
 ciGithub.json),
 {
      tag: expected.tag,

      publish: true,

      build: true
    });
    assert.deepEqual(resolveExtensionRelease(ciEnvironment,
 {
      workflow_run: {
        name: 'CI/CD Pipeline'
      }
    },
 ciGithub),
 {
      tag: expected.tag,

      publish: false,

      build: true
    });
  } finally {
    fs.rmSync(temp,
 {
      recursive: true,

      force: true
    });
  }
});
