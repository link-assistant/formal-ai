// Maintained storage must retain source-size headroom without deleting records.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {checkListedFiles} from '../../scripts/check-file-size.mjs';
import {loadRenameMap} from '../formal_ai_subagent/rename-by-rule.mjs';
import {readLedger} from '../../scripts/self-hosting-ledger.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url)).replace(/\/$/u,'');
test('the complete maintained rename map retains source-size headroom',()=>{
 const map=loadRenameMap(root);
 const files=new Set(map.exclude.filter(path=>path==='data/meta/rename-map.lino'||path.startsWith('data/meta/rename-map/')));
 assert(files.size>=3);
 assert.deepEqual(checkListedFiles(root,files),{warnings:[],violations:[],embeddedDataViolations:[]});
 assert(map.renames.some(row=>row.tree==='browser-worker-modules'));
 assert(map.renames.some(row=>row.tree==='shared-application-constants'));
});
test('the complete maintained release ledger retains source-size headroom',()=>{
 const file='data/meta/self-hosting-ledger.lino',source=readFileSync(root+'/'+file,'utf8');
 const parts=[...source.matchAll(/^  archive "([^"\n]+)"$/gmu)].map(match=>'data/meta/'+match[1]);
 assert(parts.length>0);
 assert.deepEqual(checkListedFiles(root,new Set([file,...parts])),{warnings:[],violations:[],embeddedDataViolations:[]});
 assert(readLedger(root+'/'+file).includes('tag "v0.296.0"'));
});
