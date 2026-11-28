// Rebuild the unchanged-operation staged release projection after a layout-only repair.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {gunzipSync} from 'node:zlib';
import YAML from 'yaml';
import {readCanonicalSource} from '../../scripts/canonical-release-source.mjs';
import {buildStagedReleaseProjection} from '../../scripts/generate-staged-release.mjs';
const evidence='experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
const packet=JSON.parse(readFileSync(evidence+'stage-source-coverage.json','utf8'));
const retained=readCanonicalSource(process.cwd());
const generated=buildStagedReleaseProjection(retained.text,packet,{partitioned:retained.files.length===3});
const before=gunzipSync(readFileSync(process.argv[2])).toString();
const after=generated.outputs.get('.github/workflows/release-staged.yml');
assert.deepEqual(YAML.parse(after),YAML.parse(before),'every workflow job, input, step and script must retain its parsed value');
for(const [path,bytes] of generated.outputs)writeFileSync(path,bytes);
console.log(JSON.stringify({operations:generated.operations,bindings:generated.binding.length,outputs:generated.outputs.size,beforeLines:before.trimEnd().split('\n').length,afterLines:after.trimEnd().split('\n').length,parsedWorkflowExactlyEqual:true}));
