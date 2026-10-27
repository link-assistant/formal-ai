import test from 'node:test';
import assert from 'node:assert/strict';
import {installSelectedToolchain,transientRustupFailure} from '../../../scripts/install-selected-rust-toolchain.mjs';
const toolchain='1.99.1',target='x86_64-apple-darwin';
const good=stdout=>({status:0,stdout:Buffer.from(stdout??''),stderr:Buffer.alloc(0),cleanup_complete:true,signal:null,error:null});
const bad=(message,extra={})=>({...good(),status:1,stderr:Buffer.from(message),...extra});
const identity='release: '+toolchain+'\ncommit-hash: '+'a'.repeat(40)+'\n';
function fixture(results){const calls=[];return{calls,run:async(program,args,options)=>{calls.push({program,args,options});return results.shift()??good(identity);}};}
async function install(f){return installSelectedToolchain({toolchain,target,run:f.run,delay:async()=>{},report:()=>{},environment:{}});}
test('one unchanged installation retry recovers the captured macOS DNS error',async()=>{
 const f=fixture([bad('dns error: failed to lookup address information: nodename nor servname provided'),good(),good(),good(identity)]),result=await install(f);
 assert.equal(result.installAttempts,2);assert.equal(result.calls,4);assert.equal(result.compiled,false);assert.deepEqual(f.calls[0].args,f.calls[1].args);assert(f.calls[0].args.includes('--no-self-update'));assert.equal(f.calls[3].options.env.RUSTUP_TOOLCHAIN,toolchain);
});
test('repeated DNS failures stop after exactly two attempts',async()=>{const f=fixture([bad('dns error'),bad('dns error')]);await assert.rejects(install(f),/no further attempts/u);assert.equal(f.calls.length,2);});
for(const message of ['checksum mismatch','signature verification failed','HTTP 404 not found','toolchain does not exist'])test('permanent installation failure is never retried: '+message,async()=>{const f=fixture([bad(message)]);await assert.rejects(install(f));assert.equal(f.calls.length,1);});
test('uncertain process cleanup cannot be retried',async()=>{const f=fixture([bad('dns error',{cleanup_complete:false})]);await assert.rejects(install(f));assert.equal(f.calls.length,1);});
test('wrong compiler identity is refused despite successful setup',async()=>{const f=fixture([good(),good(),good('release: 1.98.0\ncommit-hash: '+'b'.repeat(40))]);await assert.rejects(install(f),/release mismatch/u);});
test('only transient transport diagnostics qualify for recovery',()=>{assert.equal(transientRustupFailure('HTTP 503 temporarily unavailable'),true);assert.equal(transientRustupFailure('checksum mismatch after dns error'),false);assert.equal(transientRustupFailure('unknown compiler failure'),false);});
