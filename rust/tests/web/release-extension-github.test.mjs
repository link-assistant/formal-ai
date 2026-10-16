// Controlled CLI adapter tests only; no GitHub authority or real package execution.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {createReadonlyGithub} from '../../../scripts/release-extension-github.mjs';

function withAdapter(run, check) {
  const directory=fs.mkdtempSync(join(tmpdir(),'formal-ai-extension-github-test-'));
  try { check(createReadonlyGithub(directory,{run})); }
  finally { fs.rmSync(directory,{recursive:true,force:true}); }
}
const success=stdout=>({status:0,signal:null,stdout:Buffer.from(stdout),stderr:Buffer.alloc(0)});

test('Actions artifact ZIP uses default API negotiation and preserves exact bytes',()=>{
  const bytes=Buffer.from('CONTROLLED ZIP BYTES');
  withAdapter((command,args)=>{
    assert.equal(command,'gh');assert.deepEqual(args,['api','repos/owner/repo/actions/artifacts/123/zip']);
    return success(bytes);
  },github=>assert.deepEqual(github.bytes('repos/owner/repo/actions/artifacts/123/zip'),bytes));
});
test('release asset keeps the required octet-stream negotiation',()=>{
  withAdapter((command,args)=>{
    assert.equal(command,'gh');assert.deepEqual(args,['api','-H','Accept: application/octet-stream','repos/owner/repo/releases/assets/456']);
    return success('CONTROLLED RELEASE BYTES');
  },github=>assert.equal(github.bytes('repos/owner/repo/releases/assets/456').toString(),'CONTROLLED RELEASE BYTES'));
});
test('metadata requests remain JSON API requests',()=>{
  withAdapter((command,args)=>{
    assert.deepEqual(args,['api','repos/owner/repo/actions/artifacts/123']);return success('{"id":123}');
  },github=>assert.deepEqual(github.json('repos/owner/repo/actions/artifacts/123'),{id:123}));
});
test('HTTP failure reports the actual bounded diagnostic and still refuses',()=>{
  withAdapter(()=>({status:1,signal:null,stdout:Buffer.alloc(0),stderr:Buffer.from("gh: Unsupported Accept header (HTTP 415)\n" )}),github=>{
    assert.throws(()=>github.bytes('repos/owner/repo/actions/artifacts/123/zip'),/GitHub read-only verification failed: gh: Unsupported Accept header \(HTTP 415\)/u);
  });
});
test('failure diagnostics redact known tokens and signed URL queries',()=>{
  const saved=process.env.GH_TOKEN;process.env.GH_TOKEN='CONTROLLED_SECRET_VALUE';
  try { withAdapter(()=>({status:1,signal:null,stdout:Buffer.alloc(0),stderr:Buffer.from('CONTROLLED_SECRET_VALUE ghp_FAKE_PUBLIC_TEST_TOKEN https://example.invalid/archive?signature=CONTROLLED_QUERY_VALUE')}),github=>{
    assert.throws(()=>github.bytes('repos/owner/repo/actions/artifacts/123/zip'),error=>{
      assert(!error.message.includes('CONTROLLED_SECRET_VALUE'));assert(!error.message.includes('ghp_FAKE_PUBLIC_TEST_TOKEN'));assert(!error.message.includes('CONTROLLED_QUERY_VALUE'));assert(error.message.includes('[redacted]'));return true;
    });
  }); } finally { if(saved===undefined)delete process.env.GH_TOKEN;else process.env.GH_TOKEN=saved; }
});
test('failure diagnostics are bounded',()=>{
  withAdapter(()=>({status:1,signal:null,stdout:Buffer.alloc(0),stderr:Buffer.from('x'.repeat(10000))}),github=>{
    assert.throws(()=>github.bytes('repos/owner/repo/actions/artifacts/123/zip'),error=>{assert(error.message.length<2400);return true;});
  });
});
