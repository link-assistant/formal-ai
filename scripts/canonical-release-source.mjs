// Assemble a live canonical release source from explicitly bound physical parts.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {lstatSync,readFileSync,realpathSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
export const CANONICAL_PREFIX='experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
export const CANONICAL_SOURCE=CANONICAL_PREFIX+'original-release-workflow.yml';
export const CANONICAL_PARTS=Object.freeze([1,2].map(n=>CANONICAL_PREFIX+'original-release-workflow.part-'+n+'.txt'));
export const CANONICAL_FILES=Object.freeze([CANONICAL_SOURCE,...CANONICAL_PARTS]);
const MARKER='# canonical-release-source-parts/v1\n';
const digest=text=>createHash('sha256').update(text).digest('hex');
function index(text) {
 if(!text.startsWith(MARKER)){assert(!text.startsWith('# canonical-release-source-'),'unknown canonical source layout');return null;}
 const declaration=JSON.parse(text.slice(MARKER.length));
 assert.deepEqual(Object.keys(declaration).sort(),['parts','sha256']);
 assert.match(declaration.sha256,/^[a-f0-9]{64}$/u);
 assert(Array.isArray(declaration.parts));assert.equal(declaration.parts.length,2);
 for(const [n,part] of declaration.parts.entries()){
  assert.deepEqual(Object.keys(part).sort(),['bytes','path','sha256']);
  assert.equal(part.path,CANONICAL_PARTS[n]);assert(Number.isSafeInteger(part.bytes)&&part.bytes>0);
  assert.match(part.sha256,/^[a-f0-9]{64}$/u);
 }
 return declaration;
}
/** Validate complete physical byte declarations; never hash assembled text as the index file. */
export function assembleCanonicalSource(files) {
 assert(Array.isArray(files));assert(files.length===1||files.length===3);
 assert.deepEqual(files.map(file=>file.path),files.length===1?[CANONICAL_SOURCE]:CANONICAL_FILES);
 for(const file of files){assert.deepEqual(Object.keys(file).sort(),['path','text']);assert.equal(typeof file.text,'string');}
 const declaration=index(files[0].text);
 if(!declaration){assert.equal(files.length,1);return files[0].text;}
 assert.equal(files.length,3,'complete canonical source parts required');
 for(const [n,part] of declaration.parts.entries()){
  assert.equal(Buffer.byteLength(files[n+1].text),part.bytes,'canonical part size differs');
  assert.equal(digest(files[n+1].text),part.sha256,'canonical part bytes differ');
 }
 const text=files.slice(1).map(file=>file.text).join('');
 assert.equal(digest(text),declaration.sha256,'assembled canonical source differs');return text;
}
export function readCanonicalSource(root=process.cwd()) {
 if(root instanceof URL)root=fileURLToPath(root);
 const read=path=>{const file=resolve(root,path),status=lstatSync(file);assert(status.isFile()&&!status.isSymbolicLink(),'regular canonical source required');assert.equal(realpathSync(file),file,'canonical source redirects');return {path,text:readFileSync(file,'utf8')};};
 const first=read(CANONICAL_SOURCE),declaration=index(first.text);
 const files=[first,...(declaration?CANONICAL_PARTS.map(read):[])];
 return {text:assembleCanonicalSource(files),files};
}
export function partitionCanonicalSource(text) {
 assert.equal(typeof text,'string');const lines=text.match(/[^\n]*\n|[^\n]+$/gu)??[];
 assert(lines.length>=2&&lines.length<=2000,'canonical source needs two bounded parts');
 const half=Math.ceil(lines.length/2),parts=[lines.slice(0,half).join(''),lines.slice(half).join('')];
 const declaration={parts:parts.map((part,n)=>({path:CANONICAL_PARTS[n],bytes:Buffer.byteLength(part),sha256:digest(part)})),sha256:digest(text)};
 const files=[{path:CANONICAL_SOURCE,text:MARKER+JSON.stringify(declaration,null,2)+'\n'},...parts.map((part,n)=>({path:CANONICAL_PARTS[n],text:part}))];
 assert.equal(assembleCanonicalSource(files),text);return files;
}
