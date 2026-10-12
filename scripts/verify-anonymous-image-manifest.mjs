// Verify anonymous access to the exact manifest returned by a real registry push.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';

const MEDIA_TYPES=[
 'application/vnd.oci.image.index.v1+json','application/vnd.oci.image.manifest.v1+json',
 'application/vnd.docker.distribution.manifest.list.v2+json','application/vnd.docker.distribution.manifest.v2+json',
];

export async function verifyAnonymousImageManifest(image,fetcher=fetch,observeManifest=null){
 const match=/^ghcr\.io\/([a-z0-9][a-z0-9._/-]*)@(sha256:[a-f0-9]{64})$/u.exec(image);
 assert.ok(match,'exact immutable GHCR image required');
 const [,repository,digest]=match;
 assert(repository.split('/').every(part=>part&&!['.','..'].includes(part)));
 const query=new URLSearchParams({service:'ghcr.io',scope:'repository:'+repository+':pull'});
 const tokenResponse=await fetcher('https://ghcr.io/token?'+query,{redirect:'error',signal:AbortSignal.timeout(15000)});
 assert.equal(tokenResponse.status,200,'anonymous token request denied');
 const tokenRecord=await tokenResponse.json(),token=tokenRecord.token??tokenRecord.access_token;
 assert.ok(typeof token==='string'&&token.length>0&&!/\s/u.test(token),'no valid anonymous pull token');
 const response=await fetcher('https://ghcr.io/v2/'+repository+'/manifests/'+digest,{
  redirect:'error',signal:AbortSignal.timeout(15000),headers:{Authorization:'Bearer '+token,Accept:MEDIA_TYPES.join(', ')},
 });
 assert.equal(response.status,200,'anonymous manifest pull denied');
 const bytes=Buffer.from(await response.arrayBuffer());assert.ok(bytes.length>0&&bytes.length<=16*1024*1024);
 const actual='sha256:'+createHash('sha256').update(bytes).digest('hex');
 assert.equal(actual,digest,'anonymous manifest bytes differ from pushed digest');
 const reported=response.headers.get('Docker-Content-Digest');if(reported!==null)assert.equal(reported,digest,'reported manifest digest differs');
 const manifest=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(bytes));
 assert.equal(manifest.schemaVersion,2);assert.ok(MEDIA_TYPES.includes(manifest.mediaType));
 const index=Array.isArray(manifest.manifests);
 const descriptors=index?manifest.manifests:[manifest.config,...(manifest.layers??[])];
 assert.ok(descriptors.length>0,'empty image manifest');
 for(const descriptor of descriptors){assert.match(descriptor?.digest??'',/^sha256:[a-f0-9]{64}$/u);assert.ok(Number.isSafeInteger(descriptor.size)&&descriptor.size>0);}
 if(observeManifest!==null)await observeManifest(manifest);
 return {version:1,image,manifest_sha256:actual,media_type:manifest.mediaType,descriptor_count:descriptors.length,
  observation:'AnonymousImmutableManifestBytesVerified'};
}

if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
 try{console.log(JSON.stringify(await verifyAnonymousImageManifest(process.argv[2])));}
 catch(error){console.error(error.message);process.exitCode=1;}
}
