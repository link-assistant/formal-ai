#!/usr/bin/env node
// Source-derived output obligations; only the fixed authenticated observer issues proof.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {inflateRawSync} from 'node:zlib';
import {fileURLToPath, pathToFileURL} from 'node:url';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
export const LOADED_WRITER_SHA256 = digest(ordinaryBytes(fileURLToPath(import.meta.url),1024*1024));
export const OUTPUT_RECEIPT_PATH = 'producer-output-receipt.json';
export const OUTPUT_ARTIFACT_TEMPLATE = 'producer-output-${{ github.run_id }}-${{ github.run_attempt }}-${{ github.job }}';
const maximum = 65536;
const writerInvocation = 'node "$FORMAL_AI_NATIVE_PROTOCOL_DIR/release-producer-output-receipt.mjs" "$GITHUB_OUTPUT" "$FORMAL_AI_NATIVE_PROTOCOL_DIR/producer-output-workflow.yml" "$FORMAL_AI_NATIVE_PROTOCOL_DIR/protocol-selection.json" producer-output-receipt.json';
const sha = value => assert.match(value, /^[a-f0-9]{64}$/u);
const commit = value => assert.match(value, /^[a-f0-9]{40}$/u);
const atom = value => assert.match(value, /^[A-Za-z_][A-Za-z0-9_-]*$/u);
const positive = value => assert.ok(Number.isSafeInteger(value) && value > 0);
function ordinaryBytes(file, limit) {
  assert.equal(typeof file, 'string');
  const stat = fs.lstatSync(file);
  assert.ok(stat.isFile() && !stat.isSymbolicLink());
  assert.equal(fs.realpathSync(file), file);
  assert.ok(stat.size >= 0 && stat.size <= limit);
  const bytes = fs.readFileSync(file);
  assert.equal(bytes.length, stat.size);
  return bytes;
}
/** The only accepted shell guard is a closed literal echo and unconditional exit 1. */
export function deriveFailureOutputContract(job) {
  assert.ok(job && Array.isArray(job.steps) && job.steps.length <= 128);
  const ids = new Map(), guards = [];
  for (let index = 0; index < job.steps.length; index++) {
    const step = job.steps[index];
    if (step.id !== undefined) {
      atom(step.id); assert.ok(!ids.has(step.id), 'ambiguous source producer id');
    }
    const match = typeof step.if === 'string' && /^\$\{\{\s*steps\.([A-Za-z_][A-Za-z0-9_-]*)\.outputs\.([A-Za-z_][A-Za-z0-9_-]*)\s*!=\s*''\s*\}\}$/u.exec(step.if);
    if (match && typeof step.run === 'string') {
      const lines = step.run.trim().split('\n');
      const interpolation = new RegExp('\\$\\{\\{\\s*steps\\.' + match[1] + '\\.outputs\\.' + match[2] + '\\s*\\}\\}', 'gu');
      const echo = lines[0]?.replace(interpolation, 'SOURCE_OUTPUT');
      if (step['continue-on-error'] !== true && step.shell === undefined && lines.length === 2 && /^\s*echo "[^"\\`$\r\n]*"\s*$/u.test(echo) && lines[1].trim() === 'exit 1') {
        const producer = ids.get(match[1]);
        assert.ok(producer, 'source guard references missing or later producer');
        const run = producer.step.run;
        if (typeof run !== 'string' || run.trimEnd().split('\n').at(-1).trim() !== writerInvocation) {
          // Historical sources remain all-success-required until the receipt producer exists.
          if (step.id !== undefined) ids.set(step.id, {step, index});
          continue;
        }
        assert.equal(producer.step.env?.RELEASE_PRODUCER_OUTPUT_STEP, match[1]);
        assert.equal(producer.step.env?.RELEASE_PRODUCER_PROTOCOL_COMMIT, '${{ github.workflow_sha }}');
        const keys = producer.step.env?.RELEASE_PRODUCER_OUTPUT_KEYS;
        assert.equal(typeof keys, 'string');
        const declared = keys.split(',');
        assert.ok(declared.length > 0 && declared.length <= 16);
        declared.forEach(atom); assert.equal(new Set(declared).size, declared.length);
        assert.ok(declared.includes(match[2]), 'source guard output is undeclared');
        guards.push(Object.freeze({name:step.name, sourceSha256:digest(JSON.stringify(step)),
          producerId:match[1], producerName:producer.step.name,
          producerSourceSha256:digest(JSON.stringify(producer.step)),
          output:match[2], keys:Object.freeze([...declared]), index}));
      }
    }
    if (step.id !== undefined) ids.set(step.id, {step, index});
  }
  if (!guards.length) return null;
  assert.equal(new Set(guards.map(guard => guard.producerId)).size, 1, 'multiple output producers need a separate contract');
  const upload = job.steps.map((step,index) => ({step,index})).filter(({step}) =>
    step.uses === 'actions/upload-artifact@v7' && step.with?.name === OUTPUT_ARTIFACT_TEMPLATE && step.with?.path === OUTPUT_RECEIPT_PATH);
  const attest = job.steps.map((step,index) => ({step,index})).filter(({step}) =>
    step.uses === 'actions/attest@v4' && step.with?.['subject-path'] === OUTPUT_RECEIPT_PATH);
  assert.equal(upload.length, 1, 'canonical receipt upload missing or ambiguous');
  assert.equal(attest.length, 1, 'canonical receipt attestation missing or ambiguous');
  const producer = ids.get(guards[0].producerId);
  for (const entry of [upload[0],attest[0]]) {
    assert.equal(entry.step.if, undefined, 'output receipt must not be conditionally omitted');
    assert.ok(entry.index > producer.index && entry.index < Math.min(...guards.map(guard => guard.index)));
    assert.ok(typeof entry.step.name === 'string' && entry.step.name.length > 0 && !entry.step.name.includes('${{'));
  }
  assert.ok(attest[0].index < upload[0].index, 'receipt must be attested before upload');
  return Object.freeze({guards:Object.freeze(guards), producerId:guards[0].producerId,
    keys:guards[0].keys, requiredReceiptSteps:Object.freeze([attest[0].step.name,upload[0].step.name]),
    artifactTemplate:OUTPUT_ARTIFACT_TEMPLATE, member:OUTPUT_RECEIPT_PATH});
}
export function parseCanonicalOutputs(bytes, requiredKeys) {
  assert.ok(Buffer.isBuffer(bytes) && bytes.length <= maximum);
  assert.ok(Array.isArray(requiredKeys) && requiredKeys.length > 0 && requiredKeys.length <= 16);
  requiredKeys.forEach(atom); assert.equal(new Set(requiredKeys).size, requiredKeys.length);
  const content = new TextDecoder('utf-8', {fatal:true}).decode(bytes);
  assert.ok(content.endsWith('\n'), 'complete output framing required');
  const pairs = content.slice(0,-1).split('\n').map(line => {
    const match = /^([A-Za-z_][A-Za-z0-9_-]*)=([A-Za-z0-9_.,:/ -]*)$/u.exec(line);
    assert.ok(match, 'unsupported multiline, unsafe or noncanonical producer output');
    return [match[1],match[2]];
  });
  assert.ok(pairs.length > 0 && pairs.length <= 16);
  assert.equal(new Set(pairs.map(([key])=>key)).size,pairs.length,'duplicate producer output');
  const outputs = Object.fromEntries(pairs.sort(([left],[right])=>left.localeCompare(right)));
  for (const key of requiredKeys) assert.ok(Object.hasOwn(outputs,key),'explicit output key missing');
  return Object.freeze(outputs);
}
/** Fixture validation is deliberately unauthenticated and cannot discharge a skipped guard. */
export function validateProducerOutputReceipt(bytes, binding, contract) {
  assert.ok(Buffer.isBuffer(bytes) && bytes.length > 0 && bytes.length <= maximum);
  const record = JSON.parse(new TextDecoder('utf-8', {fatal:true}).decode(bytes));
  assert.deepEqual(Object.keys(record), ['schema','repository','runId','runAttempt','runHead','protocolCommit',
    'protocolTree','workflowPath','workflowSha256','writerSha256','jobId','stepId','outputs']);
  assert.equal(record.schema,'formal-ai-producer-outputs/v1');
  for (const key of ['repository','runId','runAttempt','runHead','protocolCommit','protocolTree','workflowPath',
    'workflowSha256','writerSha256','jobId']) assert.equal(record[key],binding[key],'foreign output receipt '+key);
  positive(record.runId); positive(record.runAttempt);
  for (const key of ['runHead','protocolCommit','protocolTree']) commit(record[key]);
  for (const key of ['workflowSha256','writerSha256']) sha(record[key]);
  assert.equal(record.stepId,contract.producerId);
  assert.ok(record.outputs && Object.getPrototypeOf(record.outputs) === Object.prototype);
  const keys = Object.keys(record.outputs);
  assert.ok(keys.length > 0 && keys.length <= 16);
  assert.deepEqual(keys,[...keys].sort((left,right)=>left.localeCompare(right)));
  for (const key of keys) {
    atom(key); assert.equal(typeof record.outputs[key],'string');
    assert.match(record.outputs[key], /^[A-Za-z0-9_.,:/ -]*$/u);
  }
  for (const key of contract.keys) assert.ok(Object.hasOwn(record.outputs,key),'explicit guard output missing');
  assert.deepEqual(bytes,Buffer.from(JSON.stringify(record,null,2)+'\n'),'canonical receipt bytes required');
  return Object.freeze({outputs:Object.freeze({...record.outputs}), authenticated:false,
    skippedAdmission:false, publicationComplete:false});
}
export function writeProducerOutputReceipt(outputFile, workflowFile, selectionFile, destination, environment=process.env) {
  for (const file of [outputFile,workflowFile,selectionFile,destination]) assert.equal(typeof file,'string');
  const identity = environment.GITHUB_WORKFLOW_REF;
  const match = /^([A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+)\/(\.github\/workflows\/[A-Za-z0-9_-]+\.yml)@[^\r\n]+$/u.exec(identity);
  assert.ok(match,'canonical workflow source identity required');
  assert.equal(environment.GITHUB_REPOSITORY,match[1]);
  atom(environment.GITHUB_JOB); atom(environment.RELEASE_PRODUCER_OUTPUT_STEP);
  const runId = Number(environment.GITHUB_RUN_ID), runAttempt = Number(environment.GITHUB_RUN_ATTEMPT);
  positive(runId); positive(runAttempt);
  assert.equal(String(runId),environment.GITHUB_RUN_ID); assert.equal(String(runAttempt),environment.GITHUB_RUN_ATTEMPT);
  commit(environment.GITHUB_SHA); commit(environment.RELEASE_PRODUCER_PROTOCOL_COMMIT);
  const selection = JSON.parse(ordinaryBytes(selectionFile,maximum));
  assert.equal(selection.commit,environment.RELEASE_PRODUCER_PROTOCOL_COMMIT); commit(selection.tree);
  const outputs = parseCanonicalOutputs(ordinaryBytes(outputFile,maximum),environment.RELEASE_PRODUCER_OUTPUT_KEYS.split(','));
  const own = ordinaryBytes(fileURLToPath(import.meta.url),1024*1024);
  assert.equal(digest(own),LOADED_WRITER_SHA256,'loaded output writer source drift');
  const record = {schema:'formal-ai-producer-outputs/v1',repository:match[1],runId,runAttempt,
    runHead:environment.GITHUB_SHA,protocolCommit:selection.commit,protocolTree:selection.tree,
    workflowPath:match[2],workflowSha256:digest(ordinaryBytes(workflowFile,1024*1024)),writerSha256:digest(own),
    jobId:environment.GITHUB_JOB,stepId:environment.RELEASE_PRODUCER_OUTPUT_STEP,outputs};
  const bytes = Buffer.from(JSON.stringify(record,null,2)+'\n');
  assert.ok(bytes.length <= maximum);
  fs.writeFileSync(destination,bytes,{flag:'wx',mode:0o600});
  return Object.freeze({bytes:bytes.length,sha256:digest(bytes),authenticated:false,publicationComplete:false});
}
/** Enforce the tiny actual inflation bound before the unchanged ZIP/CRC decoder. */
export function preflightProducerOutputArchive(bytes) {
  assert.ok(Buffer.isBuffer(bytes) && bytes.length >= 22 && bytes.length <= 256*1024);
  let end = -1;
  for (let offset=bytes.length-22;offset>=Math.max(0,bytes.length-65557);offset--) {
    if (bytes.readUInt32LE(offset)===0x06054b50 && offset+22+bytes.readUInt16LE(offset+20)===bytes.length) {
      end=offset; break;
    }
  }
  assert.ok(end>=0,'missing output ZIP boundary');
  assert.equal(bytes.readUInt16LE(end+8),1); assert.equal(bytes.readUInt16LE(end+10),1);
  const central=bytes.readUInt32LE(end+16);
  assert.ok(central+46<=end); assert.equal(bytes.readUInt32LE(central),0x02014b50);
  const length=bytes.readUInt32LE(central+24), compressed=bytes.readUInt32LE(central+20);
  assert.ok(length>0 && length<=maximum,'bounded output member length required');
  const local=bytes.readUInt32LE(central+42), method=bytes.readUInt16LE(central+10);
  assert.ok(local+30<=central); assert.equal(bytes.readUInt32LE(local),0x04034b50);
  const begin=local+30+bytes.readUInt16LE(local+26)+bytes.readUInt16LE(local+28);
  assert.ok(begin<=central && begin+compressed<=central);
  assert.ok(method===0 || method===8,'unsupported output compression');
  const encoded=bytes.subarray(begin,begin+compressed);
  const decoded=method===0 ? encoded : inflateRawSync(encoded,{maxOutputLength:maximum});
  assert.equal(decoded.length,length); assert.ok(decoded.length<=maximum,'actual output inflation exceeded');
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  assert.equal(process.argv.length,6);
  console.log(JSON.stringify(writeProducerOutputReceipt(...process.argv.slice(2))));
}
