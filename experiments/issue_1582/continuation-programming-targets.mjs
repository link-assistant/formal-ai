// Supplied exhaustive code-target census; no target compilers are invoked.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import {installDefaultNodeSourceHost} from '../../js/server/default-node-source-bootstrap.mjs';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {parseRoot,findChildValue} from '../../js/agentic/crate/seed_parser.mjs';
await installDefaultNodeSourceHost(new WorkerHost());
const catalog=parseRoot(fs.readFileSync('data/meta/agentic-coding-catalog.lino','utf8')).children.find(row=>row.name==='coding_catalog');
const targets=catalog.children.filter(row=>row.name==='language').map(row=>({slug:row.id,name:findChildValue(row,'name'),frameworkOf:findChildValue(row,'framework_of')||null}));
assert.equal(targets.length,16);
const source=fs.readFileSync('rust/src/coding/catalog/languages.rs','utf8');
assert.deepEqual(targets.map(row=>row.slug),[...source.matchAll(/^        slug: "([^"]+)",$/gmu)].map(match=>match[1]));
const path='data/parity/issue-1582-permissions.json';
const fixtures=JSON.parse(fs.readFileSync(path,'utf8')).filter(row=>!row.programmingTarget);
for(const target of targets)fixtures.push({language:'en',programmingTarget:target.slug,prompt:`Write a ${target.name} program that displays "enable internet access".`,guidance:false});
fs.writeFileSync(path,JSON.stringify(fixtures,null,2)+'\n');
const test='rust/tests/web/issue-1582-permissions.test.mjs';
let text=fs.readFileSync(test,'utf8');
const marker="  assert(helper,'the TypeScript projection must include the shared request recognizer');";
assert(text.includes(marker));
text=text.replace(marker,marker+`
  // Every declared coding target is covered, including framework targets.
  const catalog=readFileSync(new URL('../../../data/meta/agentic-coding-catalog.lino',import.meta.url),'utf8');
  const targets=[...catalog.matchAll(/^  language (\\S+)$/gmu)].map(match=>match[1]);
  const native=readFileSync(new URL('../../../rust/src/coding/catalog/languages.rs',import.meta.url),'utf8');
  assert.deepEqual(targets,[...native.matchAll(/^        slug: "([^"]+)",$/gmu)].map(match=>match[1]));
  assert.deepEqual(fixtures.filter(row=>row.programmingTarget).map(row=>row.programmingTarget),targets);`);
fs.writeFileSync(test,text);
const matrix='docs/case-studies/issue-1582/raw-data/local/continuation-support-matrix.json';
const census=JSON.parse(fs.readFileSync(matrix,'utf8'));
census.programmingTargets={kind:'permission-route exclusions, not compilation or execution of generated programs',targets};
fs.writeFileSync(matrix,JSON.stringify(census,null,2)+'\n');
console.log(JSON.stringify({targets:targets.map(row=>row.slug),fixtureCases:fixtures.length,nodeTests:fixtures.length+1}));
