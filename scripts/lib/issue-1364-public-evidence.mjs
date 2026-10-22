// Publish bounded public-task observations after account and configured-secret redaction.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {sanitizeEvidence} from '../collect-test-replays.mjs';
import {makeRedactor} from '../../js/server/structured-log.mjs';
import {parseJson, toCompactJson} from '../../js/server/json.mjs';
import {decodeDialogRecord, encodeDialogRecord} from '../../js/server/dialog-codec.mjs';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
export function publicEvidenceValue(value, environment) {
  return makeRedactor(environment,{preserveJsonNumbers:true})(sanitizeEvidence(value));
}
export function createPublicEvidenceExport(directory, environment=process.env) {
  assert.equal(fs.realpathSync(path.dirname(directory)),path.resolve(path.dirname(directory)));
  fs.mkdirSync(directory,{mode:0o700});
  const members=[];
  let aggregateBytes=0;
  const safe = value => publicEvidenceValue(value,environment);
  function write(relative,bytes,original=bytes,scope='sanitized-public-task-observation') {
    assert(relative && relative.split('/').every(part=>/^[A-Za-z0-9_.-]+$/u.test(part)&&part!=='.'&&part!=='..'));
    const file=path.join(directory,relative);
    fs.mkdirSync(path.dirname(file),{recursive:true,mode:0o700});
    fs.writeFileSync(file,bytes,{flag:'wx',mode:0o600});
    members.push({path:relative,sourceBytes:Buffer.byteLength(original),sourceSha256:digest(original),
      publishedBytes:Buffer.byteLength(bytes),publishedSha256:digest(bytes),
      changedByRedactionOrCanonicalEncoding:digest(original)!==digest(bytes),scope});
  }
  function collect(source,label) {
    assert(/^[A-Za-z0-9_-]+$/u.test(label));
    assert.equal(fs.realpathSync(source),path.resolve(source));
    const pending=[''];let count=0;
    while(pending.length) {
      const parent=pending.pop();
      assert(parent.split('/').length<=5);
      for(const entry of fs.readdirSync(path.join(source,parent),{withFileTypes:true})) {
        assert(++count<=512);
        assert(!entry.isSymbolicLink()&&/^[A-Za-z0-9_.-]+$/u.test(entry.name),'unpublishable evidence member');
        const relative=parent?parent+'/'+entry.name:entry.name;
        if(entry.isDirectory()){pending.push(relative);continue;}
        assert(entry.isFile());
        const file=path.join(source,relative),descriptor=fs.openSync(file,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW);
        let bytes;
        try{
          const before=fs.fstatSync(descriptor);assert(before.isFile()&&before.size<=33554432);
          aggregateBytes+=before.size;assert(aggregateBytes<=134217728);
          bytes=fs.readFileSync(descriptor);
          const after=fs.fstatSync(descriptor);assert.equal(after.size,before.size);assert.equal(after.mtimeMs,before.mtimeMs);
        }finally{fs.closeSync(descriptor);}
        assert.deepEqual(Buffer.from(bytes.toString('utf8')),bytes,'non-text public workload evidence');
        const text=bytes.toString('utf8');let output;
        if(relative.endsWith('.lino')){
          assert(text===''||text.endsWith('\n'),'partial structured log');
          output=text===''?'':text.slice(0,-1).split('\n').map(line=>encodeDialogRecord(safe(decodeDialogRecord(line)))+'\n').join('');
        }else if(relative.endsWith('.json'))output=toCompactJson(safe(parseJson(text)))+'\n';
        else output=text.split('\n').map(line=>String(safe(line))).join('\n');
        write(label+'/'+relative,output,bytes);
      }
    }
  }
  return {safe,collect,
    json(relative,value){write(relative,toCompactJson(safe(value))+'\n');},
    originalIdentity(bytes){JSON.parse(bytes);write('source-binary-identity.json',bytes,bytes,'reviewed-public-source-and-workflow-identity');},
    text(relative,value){write(relative,String(safe(value)),value);},
    finish({omitted=[]}={}){
      const manifest={schema:'formal-ai-public-task-evidence/v1',members,
        rawPrivateLogsPublished:false,personalSessionExportsRead:false,
        scope:'controlled public test workload; known account fields and configured credentials removed',
        redactionPolicy:'shared production evidence sanitizer plus configured-secret redactor',
        wholePlannerSemanticEquivalence:'not-claimed',omitted,complete:omitted.length===0};
      fs.writeFileSync(path.join(directory,'privacy-manifest.json'),JSON.stringify(manifest,null,2)+'\n',{flag:'wx',mode:0o600});
      return manifest;
    }};
}
