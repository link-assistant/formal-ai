// Desktop release manifests retain the workflow-selected protocol and source-size headroom.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import YAML from 'yaml';
import {checkListedFiles} from '../../scripts/check-file-size.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url)).replace(/\/$/u,'');
test('desktop release retains unchanged source-size headroom',()=>{
 assert.deepEqual(checkListedFiles(root,new Set(['.github/workflows/desktop-release.yml','scripts/desktop-release-manifest.sh','scripts/desktop-release-evidence.sh'])),{warnings:[],violations:[],embeddedDataViolations:[]});
});
test('the manifest helper is captured before selected-source import',()=>{
 const workflow=YAML.parse(readFileSync(root+'/.github/workflows/desktop-release.yml','utf8'));
 const steps=workflow.jobs.finalize.steps;
 const capture=steps.findIndex(step=>step.name==='Capture release protocol before selecting binary source');
 const source=steps.findIndex(step=>step.name==='Import selected source for durable worker verification');
 const manifest=steps.find(step=>step.name==='Build consolidated SHA256SUMS.txt and BUILD-PROVENANCE.txt');
 assert(capture>=0&&capture<source);
 assert.match(steps[capture].run,/cp scripts\/desktop-release-manifest\.sh "\$protocol_dir\/"/u);
 assert.deepEqual(manifest.run.trim().split('\n'),["bash \"$FORMAL_AI_NATIVE_PROTOCOL_DIR/desktop-release-manifest.sh\"","node \"$FORMAL_AI_NATIVE_PROTOCOL_DIR/release-producer-output-receipt.mjs\" \"$GITHUB_OUTPUT\" \"$FORMAL_AI_NATIVE_PROTOCOL_DIR/producer-output-workflow.yml\" \"$FORMAL_AI_NATIVE_PROTOCOL_DIR/protocol-selection.json\" producer-output-receipt.json"]);
 assert.equal(manifest.env.RELEASE_PRODUCER_OUTPUT_STEP,'manifest');
 assert.equal(manifest.env.RELEASE_PRODUCER_OUTPUT_KEYS,'incomplete');
 const evidence=steps.find(step=>step.name==='Validate and prepare exact-byte durable release evidence');
 assert.match(steps[capture].run,/cp scripts\/desktop-release-evidence\.sh "\$protocol_dir\/"/u);
 assert.equal(evidence.run.trim(),'bash "$FORMAL_AI_NATIVE_PROTOCOL_DIR/desktop-release-evidence.sh"');
});
