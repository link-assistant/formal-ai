import assert from 'node:assert/strict';
import test from 'node:test';
import {observeRustDesktopChecksums,observeFormalDesktopChecksums} from './probe-rust-desktop-checksums.mjs';

function assertAssets(observation){
  assert.equal(observation.exitCode,0,observation.stderr);
  for(const [name,digest]of Object.entries(observation.expectedAssetDigests)){
    const entry=observation.entries.find(row=>row.name===name);
    assert(entry,`Missing produced asset: ${name}`);
    assert.equal(entry.digest,digest);assert.equal(entry.matchesCurrentBytes,true);
  }
}
test('pinned Rust desktop checksum step hashes its own incomplete output on both branches',{timeout:10000},()=>{
  const {observations}=observeRustDesktopChecksums();assert.equal(observations.length,4);
  for(const observation of observations){
    assertAssets(observation);
    const self=observation.entries.find(row=>row.name===observation.manifestName);
    assert(self,'unchanged pinned command includes its own output');
    assert.equal(self.matchesCurrentBytes,false,'published fragment bytes differ from their recorded digest');
    assert.equal(observation.entries.some(row=>row.name==='SHA256SUMS-previous.partial'),observation.staleFragment);
  }
});
test('candidate selection emits only exact produced-asset digests with clean and reused directories',{timeout:10000},()=>{
  const candidate=process.env.ISSUE_1580_RUST_DISABLE_CHECKSUM_CANDIDATE!=='1';
  const {observations}=observeRustDesktopChecksums({candidate});assert.equal(observations.length,4);
  for(const observation of observations){
    assertAssets(observation);
    assert.deepEqual(observation.entries.map(row=>row.name).sort(),Object.keys(observation.expectedAssetDigests).sort());
    assert(observation.entries.every(row=>row.matchesCurrentBytes));
  }
});
test('Formal AI desktop checksum step excludes current and previous fragments',{timeout:10000},()=>{
  const {observations}=observeFormalDesktopChecksums();assert.equal(observations.length,2);
  for(const observation of observations){
    assertAssets(observation);
    assert.deepEqual(observation.entries.map(row=>row.name).sort(),Object.keys(observation.expectedAssetDigests).sort());
    assert(observation.entries.every(row=>row.matchesCurrentBytes));
  }
});
