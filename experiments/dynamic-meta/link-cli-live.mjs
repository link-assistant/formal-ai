import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const repository = 'link-foundation/link-cli';
const revision = 'cc6008f06f19cf4aa81823025add2fec7dfe4ba5';
const sourceOrigin = 'https://raw.githubusercontent.com';
const source = {
  url: `${sourceOrigin}/${repository}/${revision}/README.md`,
  language: 'markdown',
  sha256: 'da1fd7b688fa1450601a96ea59c6c1197e75d09dded8a9036572baf63e726755',
};
const cli = fileURLToPath(new URL('../../js/server/main.mjs', import.meta.url));
const workspace = mkdtempSync(join(tmpdir(), 'formal-ai-link-cli-'));
const packageName = 'link-cli-guide';
const env = {
  ...process.env,
  FORMAL_AI_DYNAMIC_META_STORE: join(workspace, 'meta.sqlite'),
  FORMAL_AI_DYNAMIC_META_SOURCE_ORIGINS: sourceOrigin,
};

function request(action) {
  return JSON.parse(execFileSync(process.execPath, [cli, 'meta'], { env, input: JSON.stringify(action), encoding: 'utf8' }));
}

try {
  const { sources: [discovered] } = request({ action: 'discover', sources: [source], confirm: true });
  assert.equal(discovered.error, undefined);
  assert.equal(discovered.sha256, source.sha256);
  const datasets = Object.entries({ read: 114, create: 128, delete: 136, update: 144 }).map(([name, line]) => {
    const candidate = discovered.candidates.find(({ provenance }) => provenance.line === line);
    assert.equal(candidate?.language, 'link-substitution', `usable ${name} candidate at line ${line}`);
    return { id: name, substitute: candidate.substitute, provenance: candidate.provenance };
  });
  const manifest = JSON.stringify({
    schema: 'formal-ai-plugin/v1', id: packageName,
    origin: { kind: 'github', locator: repository, revision }, datasets,
  });
  const install = request({ action: 'install', manifest, confirm: true });
  const datasetReceipts = [];
  const setupReceipts = [];

  function applyDataset(name, events) {
    const result = request({ action: 'dataset', dataset: `${packageName}/${name}`, confirm: true });
    const stored = request({ action: 'receipt', revision: result.revision });
    assert.deepEqual(stored, { action: 'dataset', ...result });
    assert.deepEqual(stored.trace.flatMap(({ events }) => events.map(({ kind }) => kind)), events);
    datasetReceipts.push(stored);
  }

  applyDataset('create', ['create']);
  applyDataset('read', []);
  setupReceipts.push(request({ action: 'substitute', query: '() ((2: 2 2))', confirm: true }).revision);
  applyDataset('update', ['update']);
  setupReceipts.push(request({ action: 'substitute', query: '((1: 1 2)) ((1: 1 1))', confirm: true }).revision);
  applyDataset('delete', ['delete']);
  const { graph: finalGraph } = request({ action: 'inspect' });
  assert.deepEqual(finalGraph, { links: [{ id: 2, from_id: 2, to_id: 2 }], names: [] });
  console.log(JSON.stringify({ source, datasets, install, setupReceipts, datasetReceipts, finalGraph }, null, 2));
} finally {
  rmSync(workspace, { recursive: true, force: true });
}
