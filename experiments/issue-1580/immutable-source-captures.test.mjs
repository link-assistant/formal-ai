import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, existsSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {gunzipSync} from 'node:zlib';
const captures=JSON.parse(readFileSync(new URL('./fixtures/immutable-source-captures.json',import.meta.url)));
test('all eight historical source captures retain their original complete bytes',()=>{
 assert.equal(captures.length,8);
 assert.equal(new Set(captures.map(row=>row.originalPath)).size,8);
 for(const row of captures){
  assert.equal(row.archivePath,row.originalPath+'.gz');
  const raw=gunzipSync(readFileSync(row.archivePath));
  assert.equal(raw.length,row.rawBytes,row.originalPath);
  assert.equal(createHash('sha256').update(raw).digest('hex'),row.rawSha256,row.originalPath);
  assert.equal(raw.toString().split('\n').length-(raw.at(-1)===10?1:0),row.lines,row.originalPath);
  assert.equal(existsSync(row.originalPath),false,row.originalPath);
 }
});
