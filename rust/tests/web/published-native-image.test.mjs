import test from 'node:test';
import assert from 'node:assert/strict';
import {verifyPublishedNativeImage,inspectPublishedNativeImage} from '../../../scripts/verify-published-native-image.mjs';

const expected={image:'ghcr.io/link-assistant/formal-ai@sha256:'+'a'.repeat(64),revision:'b'.repeat(40),version:'1.2.3',binarySha256:'c'.repeat(64)};
const observed=()=>({inspection:{RepoDigests:[expected.image],Config:{Labels:{
 'org.opencontainers.image.revision':expected.revision,'org.opencontainers.image.version':expected.version,
 'io.link-assistant.formal-ai.executable.sha256':expected.binarySha256}}},
 versionOutput:'formal-ai 1.2.3\n',binaryDigestOutput:expected.binarySha256+'  /usr/local/bin/formal-ai\n'});

test('a moving tag is never an immutable publication witness',()=>{
 assert.throws(()=>verifyPublishedNativeImage({...expected,image:'ghcr.io/link-assistant/formal-ai:1.2.3-slim'},observed()),/immutable/);
 const wrong=observed();wrong.inspection.RepoDigests=['ghcr.io/link-assistant/formal-ai@sha256:'+'d'.repeat(64)];
 assert.throws(()=>verifyPublishedNativeImage(expected,wrong),/repository digest differs/);
});
test('labels cannot replace actual binary bytes or executed version',()=>{
 for(const [field,value]of [['versionOutput','formal-ai 1.2.4\n'],['binaryDigestOutput','d'.repeat(64)+'  /usr/local/bin/formal-ai\n'],['binaryDigestOutput',expected.binarySha256+'  /tmp/another-file\n']]){
  assert.throws(()=>verifyPublishedNativeImage(expected,{...observed(),[field]:value}));
 }
 const labels=observed();labels.inspection.Config.Labels['org.opencontainers.image.revision']='e'.repeat(40);
 assert.throws(()=>verifyPublishedNativeImage(expected,labels),/source commit differs/);
 const version=observed();version.inspection.Config.Labels['org.opencontainers.image.version']='1.2.4';
 assert.throws(()=>verifyPublishedNativeImage(expected,version),/release version differs/);
 const binary=observed();binary.inspection.Config.Labels['io.link-assistant.formal-ai.executable.sha256']='f'.repeat(64);
 assert.throws(()=>verifyPublishedNativeImage(expected,binary),/binary label differs/);
});
test('independent source and exact executable receipts are preserved',()=>{
 assert.deepEqual(verifyPublishedNativeImage(expected,observed()),{version:1,image:expected.image,source_commit:expected.revision,
  package_version:'1.2.3',executable:{path:'/usr/local/bin/formal-ai',sha256:expected.binarySha256},version_output:'formal-ai 1.2.3'});
});
test('each actual Docker observation uses the same immutable pushed reference',()=>{
 const calls=[],data=observed(),outputs=[JSON.stringify([data.inspection]),data.versionOutput,data.binaryDigestOutput];
 const receipt=inspectPublishedNativeImage(expected,(command,args)=>{assert.equal(command,'docker');calls.push(args);return outputs.shift();});
 assert.equal(receipt.source_commit,expected.revision);assert.equal(calls.length,3);
 assert(calls.every(args=>args.includes(expected.image)));
 assert.throws(()=>inspectPublishedNativeImage(expected,()=>{throw Error('actual pull or execution failure');}),/actual pull or execution failure/);
});
