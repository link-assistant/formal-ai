// Anonymous, bounded public release downloads. Returned hashes are data, not publication authority.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {open,unlink,lstat} from 'node:fs/promises';
import {resolve,dirname,basename} from 'node:path';
const hosts=new Set(['github.com','release-assets.githubusercontent.com','objects.githubusercontent.com','github-releases.githubusercontent.com']);
/** Validate a stable GitHub release asset and an explicit streaming byte budget. */
export function publicReleaseAsset({asset,repository,tag,maxBytes}) {
  assert.match(repository,/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.match(tag,/^v[0-9]+\.[0-9]+\.[0-9]+$/u);
  assert.ok(Number.isSafeInteger(maxBytes)&&maxBytes>0&&maxBytes<=2*1024**3,'explicit public download budget required');
  assert.ok(asset&&Number.isSafeInteger(asset.id)&&asset.id>0);
  assert.match(asset.name,/^[A-Za-z0-9][A-Za-z0-9._-]{0,199}$/u);
  assert.equal(asset.state,'uploaded');
  assert.ok(Number.isSafeInteger(asset.size)&&asset.size>0&&asset.size<=maxBytes,'release asset exceeds byte budget');
  assert.match(asset.digest,/^sha256:[a-f0-9]{64}$/u);
  const canonical='https://github.com/'+repository+'/releases/download/'+tag+'/'+encodeURIComponent(asset.name);
  assert.equal(new URL(canonical).href,canonical,'noncanonical repository or tag');
  assert.equal(asset.browser_download_url,canonical,'asset URL is not the selected public release');
  return canonical;
}
/** Reject credential-bearing or non-GitHub redirects without sending authorization headers. */
export function publicReleaseRedirect(location,previous) {
  const next=new URL(location,previous);
  assert.equal(next.protocol,'https:');assert.ok(hosts.has(next.hostname),'untrusted public release redirect');
  assert.equal(next.username,'');assert.equal(next.password,'');assert.equal(next.port,'');assert.equal(next.hash,'');
  return next.href;
}
/** Stream independently downloaded public bytes to one new file, then require exact size and SHA256. */
export async function downloadPublicReleaseAsset({asset,repository,tag,destination,maxBytes,deadline}) {
  const canonical=publicReleaseAsset({asset,repository,tag,maxBytes});
  assert.ok(Number.isSafeInteger(deadline)&&deadline>Date.now(),'absolute download deadline required');
  destination=resolve(destination);assert.equal(basename(destination),asset.name);
  const parent=await lstat(dirname(destination));assert.ok(parent.isDirectory()&&!parent.isSymbolicLink());
  let current=canonical,response;
  for(let redirects=0;redirects<=5;redirects++) {
    const remaining=deadline-Date.now();assert.ok(remaining>0,'public download deadline exhausted');
    response=await fetch(current,{redirect:'manual',signal:AbortSignal.timeout(Math.min(remaining,2147483647)),
      headers:{Accept:'application/octet-stream','Accept-Encoding':'identity'}});
    if([301,302,303,307,308].includes(response.status)) {
      assert.ok(redirects<5,'too many public release redirects');
      const location=response.headers.get('location');assert.ok(location,'missing public redirect location');
      if(response.body)await response.body.cancel();
      current=publicReleaseRedirect(location,current);continue;
    }
    assert.equal(response.status,200,'public release download did not return complete bytes');break;
  }
  assert.ok(response?.body,'public asset response body missing');
  const encoding=response.headers.get('content-encoding');assert.ok(encoding===null||encoding==='identity','encoded transfer cannot prove original asset bytes');
  const length=response.headers.get('content-length');if(length!==null){assert.match(length,/^[0-9]+$/u);assert.equal(Number(length),asset.size,'public response size differs from selected asset');}
  const handle=await open(destination,'wx',0o600);let complete=false,total=0;
  const hash=createHash('sha256');
  try {
    for await(const value of response.body) {
      assert.ok(Date.now()<deadline,'public download deadline exhausted');
      const bytes=Buffer.from(value);total+=bytes.length;
      assert.ok(total<=asset.size&&total<=maxBytes,'public asset stream exceeds declared size');hash.update(bytes);
      let offset=0;while(offset<bytes.length){const written=await handle.write(bytes,offset,bytes.length-offset);assert.ok(written.bytesWritten>0);offset+=written.bytesWritten;}
    }
    assert.equal(total,asset.size,'truncated public release asset');
    const sha256=hash.digest('hex');assert.equal('sha256:'+sha256,asset.digest,'public release asset digest mismatch');
    await handle.sync();complete=true;
    return {id:asset.id,name:asset.name,size:total,sha256,repository,tag,transport:'anonymous-public-release-download',completed:true,publicationAuthority:false};
  } finally {
    await handle.close();if(!complete)await unlink(destination);
  }
}
