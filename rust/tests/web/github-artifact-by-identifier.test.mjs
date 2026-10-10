import {tmpdir} from 'node:os';
import {join} from 'node:path';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import test from 'node:test';
import {readArtifactZip,verifyArtifactArchive,downloadArtifact} from '../../../scripts/github-artifact-by-identifier.mjs';
// Golden ZIP bytes were independently generated with Python zipfile.
const fixtures={
  "normal": "UEsDBBQAAAAIAEM+Sl2O4o48FQAAABMAAAAMAAAAd2l0bmVzcy5qc29uS87PKynKz8lJTVEozyzJSy0u5gIAUEsBAhQDFAAAAAgAQz5KXY7ijjwVAAAAEwAAAAwAAAAAAAAAAAAAAIABAAAAAHdpdG5lc3MuanNvblBLBQYAAAAAAQABADoAAAA/AAAAAAA=",
  "traversal": "UEsDBBQAAAAIAEM+Sl37OSuCBQAAAAMAAAAMAAAALi4vZXZpbC5qc29uS0pMAQBQSwECFAMUAAAACABDPkpd+zkrggUAAAADAAAADAAAAAAAAAAAAAAAgAEAAAAALi4vZXZpbC5qc29uUEsFBgAAAAABAAEAOgAAAC8AAAAAAA==",
  "case": "UEsDBBQAAAAIAEM+Sl3xhmx6BQAAAAMAAAAFAAAAQS50eHTLz0sFAFBLAwQUAAAACABDPkpdZorKEQUAAAADAAAABQAAAGEudHh0KynPBwBQSwECFAMUAAAACABDPkpd8YZsegUAAAADAAAABQAAAAAAAAAAAAAAgAEAAAAAQS50eHRQSwECFAMUAAAACABDPkpdZorKEQUAAAADAAAABQAAAAAAAAAAAAAAgAEoAAAAYS50eHRQSwUGAAAAAAIAAgBmAAAAUAAAAAAA",
  "symlink": "UEsDBBQAAAAAAAAAIQD8L29GBgAAAAYAAAAEAAAAbGlua3RhcmdldFBLAQIUAxQAAAAAAAAAIQD8L29GBgAAAAYAAAAEAAAAAAAAAAAAAAD/oQAAAABsaW5rUEsFBgAAAAABAAEAMgAAACgAAAAAAA=="
};
test('exact-ID artifact downloads verify metadata, full archive, safe paths and physical bytes', () => {
const bytes=Buffer.from(fixtures.normal,'base64');
const expected={repository:'owner/repository',id:12,run:13,head:'a'.repeat(40),name:'witness'};
const metadata={id:12,name:'witness',expired:false,size_in_bytes:bytes.length,digest:'sha256:'+createHash('sha256').update(bytes).digest('hex'),workflow_run:{id:13,head_sha:expected.head}};
assert.equal(verifyArtifactArchive(expected,metadata,bytes)[0].bytes.toString(),'controlled witness\n');
let refusals=0;
for(const mutate of [m=>m.id=99,m=>m.name='foreign',m=>m.expired=true,m=>m.workflow_run.id=99,m=>m.workflow_run.head_sha='b'.repeat(40),m=>m.digest='sha256:'+'0'.repeat(64)]){
 const changed=structuredClone(metadata);mutate(changed);assert.throws(()=>verifyArtifactArchive(expected,changed,bytes));refusals++;
}
for(const name of ['traversal','case','symlink']){assert.throws(()=>readArtifactZip(Buffer.from(fixtures[name],'base64')));refusals++;}
for(const mutate of [b=>b.writeUInt16LE(1,6),b=>b.writeUInt16LE(99,8),b=>b.writeUInt32LE(0,14),b=>b.writeUInt32LE(0xffffffff,b.length-6),b=>b[40]^=1]){
 const changed=Buffer.from(bytes);mutate(changed);assert.throws(()=>readArtifactZip(changed));refusals++;
}
assert.throws(()=>readArtifactZip(Buffer.alloc(17*1024*1024)));refusals++;
const destination=fs.mkdtempSync(join(fs.realpathSync(tmpdir()), 'fixture-download-controls-'))+'/verified';let calls=0;
const mock=(command,args,options)=>{
 assert.equal(command,'gh');assert.deepEqual(args.slice(0,3),['api','--hostname','github.com']);
 assert.ok(options.timeout<=60000&&options.maxBuffer<=17*1024*1024);
 calls++;return {status:0,signal:null,stdout:args.at(-1).endsWith('/zip')?bytes:Buffer.from(JSON.stringify(metadata))};
};
assert.equal(downloadArtifact(expected,destination,mock).files,1);assert.equal(calls,3);
assert.equal(fs.readFileSync(destination+'/witness.json','utf8'),'controlled witness\n');
assert.throws(()=>downloadArtifact({...expected,repository:'owner/../foreign'},destination,()=>{throw Error('unexpected network');}));refusals++;
  const changedMetadata = (command, args) => ({ status: 0, signal: null, stdout: args.at(-1).endsWith('/zip') ? bytes : Buffer.from(JSON.stringify({ ...metadata, expired: true })) });
  assert.throws(() => downloadArtifact(expected, destination + '-expired', changedMetadata));
  assert.equal(fs.existsSync(destination + '-expired'), false);
});
