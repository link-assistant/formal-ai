// Verify immutable published image identity against independent selected source and binary bytes.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {writeFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';

export function verifyPublishedNativeImage(expected,observed){
 assert.match(expected.image,/^[^\s]+@sha256:[a-f0-9]{64}$/u,'immutable image digest required');
 assert.match(expected.revision,/^[a-f0-9]{40}$/u,'source commit required');
 assert.match(expected.version,/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/u,'release version required');
 assert.match(expected.binarySha256,/^[a-f0-9]{64}$/u,'independent executable digest required');
 const image=observed.inspection;
 assert.ok(Array.isArray(image.RepoDigests)&&image.RepoDigests.includes(expected.image),'pulled repository digest differs');
 const labels=image.Config?.Labels??{};
 assert.equal(labels['org.opencontainers.image.revision'],expected.revision,'image source commit differs');
 assert.equal(labels['org.opencontainers.image.version'],expected.version,'image release version differs');
 assert.equal(labels['io.link-assistant.formal-ai.executable.sha256'],expected.binarySha256,'image binary label differs');
 assert.equal(observed.versionOutput.trim(),'formal-ai '+expected.version,'executed release version differs');
 const digest=/^([a-f0-9]{64})[ \t]+\*?\/usr\/local\/bin\/formal-ai\n?$/u.exec(observed.binaryDigestOutput);
 assert.ok(digest,'actual executable hash receipt malformed');
 assert.equal(digest[1],expected.binarySha256,'actual image executable bytes differ');
 return {version:1,image:expected.image,source_commit:expected.revision,package_version:expected.version,
  executable:{path:'/usr/local/bin/formal-ai',sha256:digest[1]},version_output:observed.versionOutput.trim()};
}

export function inspectPublishedNativeImage(expected,run=execFileSync){
 const invoke=args=>run('docker',args,{encoding:'utf8',timeout:120000,maxBuffer:4*1024*1024});
 const inspections=JSON.parse(invoke(['image','inspect',expected.image]));
 assert.equal(inspections.length,1,'exactly one immutable image required');
 return verifyPublishedNativeImage(expected,{inspection:inspections[0],
  versionOutput:invoke(['run','--rm',expected.image,'--version']),
  binaryDigestOutput:invoke(['run','--rm','--entrypoint','sha256sum',expected.image,'/usr/local/bin/formal-ai'])});
}

if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
 const [image,revision,version,binarySha256,output]=process.argv.slice(2);
 const receipt=inspectPublishedNativeImage({image,revision,version,binarySha256});
 receipt.producer_run=String(process.env.GITHUB_RUN_ID??'');
 receipt.producer_attempt=String(process.env.GITHUB_RUN_ATTEMPT??'');
 writeFileSync(output,JSON.stringify(receipt,null,2)+'\n');console.log(JSON.stringify(receipt));
}
