import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {parseJson,toCompactJson} from '../../../js/server/json.mjs';
import {encodeDialogRecord,decodeDialogRecord} from '../../../js/server/dialog-codec.mjs';
import {createPublicEvidenceExport,publicEvidenceValue} from '../../../scripts/lib/issue-1364-public-evidence.mjs';
function directory(){return fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-1364-public-export-')));}
test('publication removes accounts, credentials and recursively encoded profile metadata',()=>{
  const value=publicEvidenceValue({account_id:'private-account',access_token:'private-token',
    owner:{login:'private-login',name:'Private Person'},source:'a'.repeat(40),
    message:'person@example.invalid known-secret',
    nested:'{"accountId":"another-private-account","api_key":"private-key","call_id":"valid-call"}'},
    {GH_TOKEN:'known-secret'});
  const output=toCompactJson(value);
  assert.doesNotMatch(output,/private-account|private-token|private-login|Private Person|example.invalid|known-secret|private-key/u);
  assert(output.includes('valid-call'));assert(output.includes('a'.repeat(40)));
});
test('source identity bytes and typed numeric values remain independently auditable',()=>{
  const root=directory(),raw=path.join(root,'raw');fs.mkdirSync(raw);
  try{
    const typed=parseJson('{"counter":9007199254740993,"fraction":1.0,"negative":-0.0,"account_id":"private-account"}');
    fs.writeFileSync(path.join(raw,'dialog.lino'),encodeDialogRecord(typed)+'\n');
    fs.writeFileSync(path.join(raw,'result.json'),'{"passed":true,"actor":{"login":"private-login"}}\n');
    const target=path.join(root,'publish'),writer=createPublicEvidenceExport(target,{});
    const identity=Buffer.from('{"source":"'+'b'.repeat(40)+'"}\n');writer.originalIdentity(identity);
    writer.collect(raw,'pass-1');const manifest=writer.finish();
    assert.deepEqual(fs.readFileSync(path.join(target,'source-binary-identity.json')),identity);
    const actual=decodeDialogRecord(fs.readFileSync(path.join(target,'pass-1/dialog.lino'),'utf8').trim());
    assert.equal(actual.counter,typed.counter);assert.deepEqual(actual.fraction,typed.fraction);
    assert.deepEqual(actual.negative,typed.negative);assert(!Object.hasOwn(actual,'account_id'));
    assert.equal(manifest.members.length,3);assert.equal(manifest.rawPrivateLogsPublished,false);
    assert(manifest.members.some(row=>row.changedByRedactionOrCanonicalEncoding));
    assert.doesNotMatch(fs.readFileSync(path.join(target,'pass-1/result.json'),'utf8'),/private-login/u);
  }finally{fs.rmSync(root,{recursive:true,force:true});}
});
for(const kind of ['symlink','partial-lino','binary'])test('refuse '+kind+' instead of publishing unreviewed raw bytes',()=>{
  const root=directory(),raw=path.join(root,'raw');fs.mkdirSync(raw);
  try{
    if(kind==='symlink')fs.symlinkSync('/not-read',path.join(raw,'external'));
    if(kind==='partial-lino')fs.writeFileSync(path.join(raw,'partial.lino'),'unfinished');
    if(kind==='binary')fs.writeFileSync(path.join(raw,'opaque.log'),Buffer.from([255,254]));
    const writer=createPublicEvidenceExport(path.join(root,'publish'),{});
    assert.throws(()=>writer.collect(raw,'pass-1'));
  }finally{fs.rmSync(root,{recursive:true,force:true});}
});
