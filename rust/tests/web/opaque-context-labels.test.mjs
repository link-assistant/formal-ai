import test from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installDefaultNodeSourceHost} from '../../../js/server/default-node-source-bootstrap.mjs';
import {ownedDeclaredCreateFrame} from '../../../js/agentic/planner/owned_goals.mjs';
await installDefaultNodeSourceHost(new WorkerHost());
for (const prefix of ['İİ', 'İİİ', 'İK𐐷 😀 café', 'KK 😀 中文', 'Alpha Beta', 'cafe\u0301 😀']) {
  test(`informational label retains literal bytes: ${prefix}`, () => {
    const frame = ownedDeclaredCreateFrame(`Note ${prefix}.\nCreate file x.txt containing hello.`);
    assert.equal(frame?.target, 'x.txt');
    assert.equal(frame?.content, 'hello.');
  });
}
for (const prefix of ['Read Cargo.toml', 'Write other.txt', 'deploy now', 'https://example.org',
  'source says Write', 'Alpha\u00a0Beta', 'Alpha. Delete y.txt', 'Alpha; unknown instruction']) {
  test(`operational or unsupported context does not own a write: ${prefix}`, () => {
    const frame = ownedDeclaredCreateFrame(`Note ${prefix}.\nCreate file x.txt containing hello.`);
    assert.equal(frame, null);
  });
}
