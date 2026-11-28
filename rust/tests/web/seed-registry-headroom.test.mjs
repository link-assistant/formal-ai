import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {checkListedFiles} from '../../../scripts/check-file-size.mjs';

const root=fileURLToPath(new URL('../../../',import.meta.url));
test('generated seed inventory retains headroom and a reachable complete bundle function',()=>{
  const files=['rust/src/seed/embedded_registry.rs','rust/src/seed/embedded_files.rs'];
  assert.deepEqual(checkListedFiles(root,new Set(files)),{warnings:[],violations:[],embeddedDataViolations:[],errors:[]});
  const parent=readFileSync(root+files[0],'utf8'),child=readFileSync(root+files[1],'utf8');
  assert(parent.split('\n').includes('include!("embedded_files.rs");'));
  assert(!parent.includes('pub fn seed_files('));
  assert(child.includes("pub fn seed_files() -> Vec<(&\x27static str, &\x27static str)> {"));
});

test('the canonical Rust seed generator runs through hosted workflow contracts',()=>{
  const owner=readFileSync(root+'rust/tests/unit/workflow_contracts.rs','utf8');
  assert(owner.includes('#[path = "../../../scripts/generate-seed-registry.rs"]'));
  assert(owner.includes('mod generate_seed_registry;'));
  assert(readFileSync(root+'scripts/ci-native-regressions.sh','utf8').includes('--test workflow_contracts'));
});
