import test from 'node:test';import assert from 'node:assert/strict';import {createHash} from 'node:crypto';
import {verifyAnonymousImageManifest} from '../../../scripts/verify-anonymous-image-manifest.mjs';
const manifest=JSON.stringify({schemaVersion:2,mediaType:'application/vnd.oci.image.manifest.v1+json',config:{digest:'sha256:'+'b'.repeat(64),size:123},layers:[]});
const digest='sha256:'+createHash('sha256').update(manifest).digest('hex'),image='ghcr.io/link-assistant/formal-ai@'+digest;
const fake=(token,body=manifest,status=200,header=digest)=>{let calls=0;return async()=>++calls===1?
 new Response(JSON.stringify(token),{status:200}):new Response(body,{status,headers:{'Docker-Content-Digest':header}});};
test('successful anonymous manifest access verifies exact bytes without publishing the token',async()=>{
 const calls=[];const receipt=await verifyAnonymousImageManifest(image,async(url,options)=>{
  calls.push({url,options});return calls.length===1?new Response('{"token":"anonymous-pull-only"}'):
   new Response(manifest,{headers:{'Docker-Content-Digest':digest}});
 });
 assert.equal(receipt.manifest_sha256,digest);assert.equal(receipt.observation,'AnonymousImmutableManifestBytesVerified');
 assert(!JSON.stringify(receipt).includes('anonymous-pull-only'));assert.equal(calls[0].options.headers,undefined);
 assert.equal(calls[1].options.headers.Authorization,'Bearer anonymous-pull-only');assert(calls[1].url.endsWith('/manifests/'+digest));
});
test('a token endpoint HTTP200 alone cannot establish anonymous image access',async()=>{
 await assert.rejects(verifyAnonymousImageManifest(image,fake({})),/no valid anonymous/);
 await assert.rejects(verifyAnonymousImageManifest(image,fake({token:'anon'},manifest,403)),/anonymous manifest pull denied/);
 await assert.rejects(verifyAnonymousImageManifest(image,fake({token:'anon'},manifest,404)),/anonymous manifest pull denied/);
 await assert.rejects(verifyAnonymousImageManifest(image,async()=>new Response('',{status:401})),/anonymous token request denied/);
});
test('successful transport with wrong manifest bytes or digest is refused',async()=>{
 await assert.rejects(verifyAnonymousImageManifest(image,fake({token:'anon'},manifest+'\n')),/bytes differ/);
 await assert.rejects(verifyAnonymousImageManifest(image,fake({token:'anon'},manifest,200,'sha256:'+'f'.repeat(64))),/reported manifest digest differs/);
});
test('moving tags, malformed anonymous tokens and transport failures remain failures',async()=>{
 await assert.rejects(verifyAnonymousImageManifest('ghcr.io/link-assistant/formal-ai:latest',fake({token:'anon'})),/immutable/);
 await assert.rejects(verifyAnonymousImageManifest(image,fake({token:'bad\r\nheader'})),/no valid anonymous/);
 await assert.rejects(verifyAnonymousImageManifest(image,async()=>{throw Error('actual network failure');}),/actual network failure/);
});
