import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { WorkerHost, REPO_ROOT, createStorage } from '../../../js/server/worker-host.mjs';
const blocked = {
  WebAssembly: {
    compile() {
      throw Error('LocalProductWasmForbidden');
    },
    instantiate() {
      throw Error('LocalProductWasmForbidden');
    }
  }
};
test('constructor captures actual compiled scripts and seed reads before the first request', async () => {
  const worker = new WorkerHost(blocked, {
    provenance: true
  });
  assert.equal(worker.provenanceGenesis(), null);
  await worker.boot();
  const original = worker.provenanceGenesis();
  assert.equal(original.schema, 'formal-ai-worker-genesis-v1');
  assert.equal(original.scope, 'constructor-owned-worker-state-before-first-request');
  assert.equal(original.completeness, 'custom-overrides-not-reconstructible');
  assert.deepEqual(original.overrides, ['WebAssembly']);
  assert(original.sources.length > 20);
  assert(original.inputs.length > 20);
  assert.equal(new Set(original.sources.map(row => row.path)).size, original.sources.length);
  for (const source of original.sources) assert.equal(source.text, fs.readFileSync(path.join(REPO_ROOT, source.path), 'utf8'), source.path);
  for (const input of original.inputs) {
    assert.equal(input.status, 200, input.path);
    if (input.textState === 'delivered') {
      assert(input.deliveries > 0, input.path);
      assert.equal(input.text, fs.readFileSync(path.join(REPO_ROOT, input.path), 'utf8'), input.path);
    } else {
      assert.equal(input.textState, 'not-delivered');
      assert.equal(input.text, undefined);
      assert.equal(input.deliveries, 0);
    }
  }
  assert(original.inputs.some(row => row.path.startsWith('data/seed/') && row.textState === 'delivered'));
  assert(original.inputs.some(row => row.path.endsWith('.wasm') && row.textState === 'not-delivered'));
  assert(original.inputs.some(row => row.path.startsWith('data/seed/')));
  const before = JSON.stringify(original);
  worker.context.localStorage.setItem('later-memory', 'changed');
  original.sources[0].text = 'forged';
  original.memory.localStorage.push(['forged', 'value']);
  assert.equal(JSON.stringify(worker.provenanceGenesis()), before);
});
test('custom fetch cannot be represented as actual default seed-read custody', async () => {
  const worker = new WorkerHost({
    ...blocked,
    fetch: async () => ({
      ok: false,
      status: 404,
      text: async () => ''
    })
  }, {
    provenance: true
  });
  await worker.boot();
  const genesis = worker.provenanceGenesis();
  assert.equal(genesis.completeness, 'custom-overrides-not-reconstructible');
  assert.deepEqual(genesis.inputs, []);
  assert(genesis.overrides.includes('fetch'));
});
import os from 'node:os';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { decodeTyped, encodeTyped } from '../../../js/logging/typed-codec.mjs';
import { rawBytes } from '../../../js/provenance/session.mjs';
import { verifyDriveEvidence, retainedEvidenceBytes } from '../../../js/provenance/verify-drive.mjs';
test('actual drive binds genesis once and verifies every retained source/seed byte', async t => {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), 'worker-genesis-drive-'));
  t.after(() => fs.rmSync(temp, {
    recursive: true,
    force: true
  }));
  const work = path.join(temp, 'work');
  fs.mkdirSync(work);
  const initialNote = 'actual initial memory\n'.repeat(20000);
  const worker = new WorkerHost({
    ...blocked,
    localStorage: createStorage({
      'genesis-note': initialNote
    })
  }, {
    provenance: true
  });
  await installNodeHost(worker);
  const original = worker.provenanceGenesis.bind(worker);
  let observations = 0;
  worker.provenanceGenesis = () => {
    observations++;
    return original();
  };
  const result = await drive(planChatStep, work, 'Create file custody.mjs containing export const custody = 47;', {
    steps: 12,
    provenance: {
      directory: path.join(temp, 'evidence'),
      environment: {}
    }
  });
  assert.equal(result.stop, 'final');
  assert.equal(observations, 1);
  const directory = result.provenance.directory;
  const rows = fs.readFileSync(path.join(directory, 'chain.lino'), 'utf8').trimEnd().split('\n').map(decodeTyped);
  const records = rows.filter(row => row.kind === 'engine_genesis');
  assert.equal(records.length, 1);
  const genesis = JSON.parse(retainedEvidenceBytes(directory, records[0].payload.evidence));
  const reference = [records[0].payload.evidence, ...genesis.sources.map(row => row.evidence), ...genesis.inputs.map(row => row.evidence)].find(row => row.format === 'chunks-v1');
  assert(reference, 'actual retained worker body must exercise bounded chunking');
  const exact = retainedEvidenceBytes(directory, reference);
  assert(exact.length > 256 * 1024);
  assert.throws(() => retainedEvidenceBytes(directory, {
    ...reference,
    chunks: reference.chunks.slice(1)
  }), /invalid_evidence_chunks/);
  assert.throws(() => retainedEvidenceBytes(directory, {
    ...reference,
    chunks: [...reference.chunks].reverse()
  }), /evidence_hash|invalid_evidence_chunks/);
  assert.equal(genesis.completeness, 'custom-overrides-not-reconstructible');
  assert(genesis.memory.localStorage.some(([key, value]) => key === 'genesis-note' && value === initialNote));
  for (const row of [...genesis.sources, ...genesis.inputs]) {
    if (row.textState === 'not-delivered') {
      assert.equal(row.evidence, undefined);
      continue;
    }
    const body = JSON.parse(retainedEvidenceBytes(directory, row.evidence));
    assert.equal(body.text, fs.readFileSync(path.join(REPO_ROOT, row.path), 'utf8'), row.path);
  }
  assert.equal(verifyDriveEvidence(directory, work, result.provenance.anchor).integrity, true);
  const changed = path.join(directory, genesis.sources[0].evidence.file);
  fs.writeFileSync(changed, encodeTyped(rawBytes(Buffer.from(JSON.stringify({
    text: 'forged'
  })))) + '\n');
  assert.throws(() => verifyDriveEvidence(directory, work, result.provenance.anchor), /evidence_hash/);
});
test('ordinary worker construction retains no provenance source or memory copies', async () => {
  const worker = new WorkerHost(blocked);
  await worker.boot();
  assert.equal(worker.provenanceGenesis(), null);
  assert.equal(worker.provenanceInputs(), null);
});
test('actual missing-file response records its empty text delivery without claiming HTTP success', async () => {
  const worker = new WorkerHost(blocked, {
    provenance: true
  });
  await worker.boot();
  const missing = 'seed/custody-held-out-missing-298451.lino';
  assert.equal(fs.existsSync(path.join(REPO_ROOT, 'data', missing)), false);
  const response = await worker.context.fetch(missing);
  assert.equal(response.ok, false);
  assert.equal(response.status, 404);
  const prepared = worker.provenanceInputs().at(-1);
  assert.equal(prepared.textState, 'not-delivered');
  assert.equal(prepared.deliveries, 0);
  assert.equal(await response.text(), '');
  const consumed = worker.provenanceInputs().at(-1);
  assert.equal(consumed.status, 404);
  assert.equal(consumed.textState, 'delivered');
  assert.equal(consumed.deliveries, 1);
  assert.equal(consumed.text, '');
});
