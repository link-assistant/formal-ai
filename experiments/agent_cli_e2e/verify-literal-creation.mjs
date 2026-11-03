// Literal completion uses actual wire custody, typed readback and independent bytes.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {completionExchangeTrace, readCompletionRecords, relativeFile, observedFile} from './verify-output-project.mjs';
import {parseChatRequest} from '../../js/server/chat-request.mjs';
import {currentDirectory} from '../../js/agentic/host.mjs';
import {Progress, qualifiedToolAttempt} from '../../js/agentic/progress.mjs';
import {Capability} from '../../js/agentic/capability.mjs';
import {classifyTool} from '../../js/agentic/capability_router.mjs';
import {readOnlyCapability} from '../../js/agentic/progress_command_readback.mjs';
import {ownedCreationObservation} from '../../js/agentic/general_execution.mjs';
import {GeneralPlanMode} from '../../js/agentic/general_planner.mjs';
import {renderSeededOutcome} from '../../js/agentic/code_task.mjs';
const digest=value=>createHash('sha256').update(value).digest('hex');

export function verifyLiteralCreation({records,workspace,task,target,expectedContent}) {
  const evidence={schema:'formal-ai-literal-creation-completion/v1',mode:'literal-creation',
    target,exchanges:0,writeCallId:null,readCallId:null,physicalSha256:null,
    sourceSha256:null,terminalCompleted:false,verified:false,stage:'configuration'};
  try {
    relativeFile(target);
    assert.equal(typeof task,'string','configured task missing');
    assert.equal(typeof expectedContent,'string','exact expected content missing');
    assert.ok(Buffer.byteLength(expectedContent)<=1024*1024,'configured literal too large');
    evidence.taskSha256=digest(task);
    evidence.bytes=Buffer.byteLength(expectedContent);
    const actualRoot=currentDirectory();
    assert.equal(typeof actualRoot,'string','trusted workspace root unavailable');
    assert.equal(fs.lstatSync(workspace).isSymbolicLink(),false,'workspace symlink');
    assert.equal(fs.realpathSync(workspace),fs.realpathSync(actualRoot),'trusted workspace differs');

    evidence.stage='wire-custody';
    const trace=completionExchangeTrace(records,{task,workspace});
    evidence.exchanges=trace.exchanges.length;
    assert.ok(trace.terminal,'terminal completion missing');
    evidence.terminalEvidence={finish:trace.terminal.response.finish,
      contentSha256:digest(trace.terminal.response.content),kind:'unverified-terminal'};
    const writes=[...trace.calls.values()].filter(call=>classifyTool(call.name)===Capability.Write);
    assert.equal(writes.length,1,'one declared literal Write required');
    for(const call of trace.calls.values()) assert.ok(
      classifyTool(call.name)===Capability.Write||readOnlyCapability(classifyTool(call.name)),
      'unexpected potentially mutating call');
    evidence.writeCallId=writes[0].id;
    assert.ok(trace.returned.has(writes[0].id),'Write receipt missing');

    evidence.stage='owned-write-read';
    const chat=parseChatRequest(JSON.stringify(trace.terminal.request));
    const progress=Progress.scan(chat.messages);
    const mutation=progress.creationMutationFor(target);
    assert.ok(mutation?.qualified&&mutation.succeeded,'qualified target Write unavailable');
    assert.equal(mutation.frame.callId,writes[0].id,'declared target Write differs');
    const plan={mode:GeneralPlanMode.LiteralFile,goal:task,target,content:expectedContent};
    const proof=ownedCreationObservation(plan,progress);
    assert.ok(proof&&proof.method==='read-tool','fresh complete exact target Read unavailable');
    assert.equal(proof.source,expectedContent,'returned source bytes differ');
    const read=qualifiedToolAttempt(progress,Capability.Read,JSON.parse(proof.arguments));
    assert.ok(read?.binding==='exact'&&read.receiptPresent,'qualified Read receipt unavailable');
    assert.ok(trace.calls.has(read.callId)&&trace.returned.has(read.callId),'actual Read custody missing');
    evidence.readCallId=read.callId;
    evidence.method=proof.method;
    evidence.sourceSha256=digest(proof.source);

    evidence.stage='physical-bytes';
    const actual=observedFile(workspace,target);
    assert.deepEqual(actual,Buffer.from(expectedContent),'physical literal bytes differ');
    evidence.physicalSha256=digest(actual);
    evidence.sha256=evidence.physicalSha256;

    evidence.stage='terminal';
    const expected=renderSeededOutcome('coding_workspace_effect_observed',task,target);
    assert.equal(typeof expected,'string','owned completion template unavailable');
    assert.equal(trace.terminal.response.content.trim(),expected.trim(),
      'terminal did not carry the verified effect finding');
    evidence.terminalEvidence={finish:trace.terminal.response.finish,
      contentSha256:digest(trace.terminal.response.content),kind:'exact-effect-observed-template'};
    evidence.terminalCompleted=true;
    evidence.verified=true;
    evidence.stage='complete';
    return evidence;
  } catch(error) {
    error.literalCreationEvidence={...evidence,reason:'literal-completion-evidence-rejected'};
    throw error;
  }
}

if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const [directory,workspace,task,target,expectedContent,report]=process.argv.slice(2);
  let evidence;
  try {
    assert.ok(report,'completion report path missing');
    const {installNativeCaptureObservationHost}=await import('../../scripts/lib/native-capture-observation.mjs');
    await installNativeCaptureObservationHost();
    evidence=verifyLiteralCreation({records:readCompletionRecords(directory),workspace,task,target,expectedContent});
  } catch(error) {
    evidence=error.literalCreationEvidence??{schema:'formal-ai-literal-creation-completion/v1',
      mode:'literal-creation',target,verified:false,reason:'literal-completion-input-rejected'};
    process.exitCode=1;
  }
  if(report) fs.writeFileSync(report,JSON.stringify(evidence,null,2)+'\n',{flag:'wx'});
  console.log(JSON.stringify(evidence));
}
