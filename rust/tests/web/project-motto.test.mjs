import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import {checkProjectMotto,MOTTO,FILES} from '../../../scripts/check-project-motto.mjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const sources=new Map(FILES.map(file=>[file,readFileSync(path.join(root,file),'utf8')]));
const check=map=>checkProjectMotto(file=>{if(!map.has(file))throw Error('missing');return map.get(file);});
test('source surfaces carry the exact motto and retain historical context',()=>{assert.deepEqual(check(sources),[]);});
for(const file of FILES)test('removing the motto from '+file+' fails the source gate',()=>{
 const changed=new Map(sources);changed.set(file,changed.get(file).replaceAll(MOTTO,'Removed project slogan'));
 assert.ok(check(changed).length>0,file);
});
for(const method of ['random or evolutionary search','exhaustive search'])test('prescribing '+method+' fails even when every motto remains',()=>{
 const changed=new Map(sources);changed.set('VISION.md',changed.get('VISION.md')+'\nUse '+method+' to generate the next draft.\n');
 assert.ok(check(changed).some(problem=>problem.includes('candidate selection')));
});
test('a missing public surface and malformed package fail independently',()=>{
 const changed=new Map(sources);changed.delete('js/cli/index.html');changed.set('desktop/package.json','{');
 const problems=check(changed);assert.ok(problems.some(problem=>problem.includes('source unavailable')));assert.ok(problems.some(problem=>problem.includes('invalid package JSON')));
});
test('the historical architect note remains linked',()=>{
 const changed=new Map(sources);changed.set('VISION.md',changed.get('VISION.md').replaceAll('docs/architect-notes/2026-09-14-know-how-to-get-to-know-anything.md','history-removed.md'));
 assert.ok(check(changed).some(problem=>problem.includes('historical')));
});

test('altering an architect word fails the verbatim quote contract',()=>{
 const changed=new Map(sources),file='docs/architect-notes/2026-10-15-know-just-enough.md';
 changed.set(file,changed.get(file).replace('self conding','self coding'));
 assert.ok(check(changed).some(problem=>problem.includes('primary quote')));
});
