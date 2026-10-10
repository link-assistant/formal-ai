import { readFileSync, writeFileSync, mkdirSync, copyFileSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { digest, admitManifest, executeAdmittedCase } from './cohort-runner.mjs';
import { readBoundSource, extractExportedLiteral, extractArithmeticAssertions, validateHeldOutContract } from './cohort-source-contracts.mjs';
import { measureCodingRun, sourceDigest, summarizeCodingRuns } from './coding-amplification.mjs';
import { writeJournalUsageReport } from './cohort-usage-report.mjs';

export function discoverSourceContracts(root) {
  root=resolve(root);
  const binding=path=>({path,sha256:digest(readFileSync(path))});
  const archive=binding(join(root,'experiments/formal_ai_subagent/evidence/formal-ai-only-1188/module-observation/original-selected-summary.test.mjs'));
  const request=binding(join(root,'experiments/formal_ai_subagent/evidence/formal-ai-only-1188/g132/T1804-request.json'));
  const original=JSON.parse(readBoundSource(request));
  if(Buffer.byteLength(original.prompt)!==955)throw new Error('original955-byte request drift');
  const heldOut=extractExportedLiteral(archive,'heldOutCases');validateHeldOutContract(heldOut.value);
  const arithmetic=extractArithmeticAssertions(binding(join(root,'rust/tests/web/coding-amplification.test.mjs')),'a valid change can exceed concise input without counting explanation text');
  return {schemaVersion:1,original:{task:original.prompt,taskSHA256:digest(original.prompt),source:request},heldOut,arithmetic,
    independentFamilies:2,representativeBaseline:false,heldOutQualification:'archived predeclared variants, unseen-to-all-prior-hosts not established'};
}
export function compositionVariantTask(item) {
  const aliases=Object.entries(item.sourceBindings).map(([source,alias])=>source+' as '+alias).join(', ');
  return `Add exported ${item.exportName}(${item.parameterName}) in ${item.destination}: return the canonical summary of the selected candidate or null. Discover and compose observed solver_formalization.mjs and translation_formalization.mjs exports through imports using aliases ${aliases}; preserve inputs and reuse semantics. Run node --test selected-summary.test.mjs; keep source modules and tests unchanged.`;
}
export function checkSourceEvaluation(root, manifestPath) {
  const catalog=discoverSourceContracts(root),manifest=JSON.parse(readFileSync(manifestPath,'utf8'));
  if(manifest.catalogSHA256!==digest(JSON.stringify(catalog)))throw new Error('source catalog drift');
  const expected=[{runId:'g132-original',taskKind:'self-coding',task:catalog.original.task},
    ...catalog.heldOut.value.map(item=>({runId:'g132-'+item.id,taskKind:'self-coding',task:compositionVariantTask(item)})),
    {runId:'ordinary-arithmetic',taskKind:'coding',task:`Add ${catalog.arithmetic.exportName}(first,second) in result.mjs.`}];
  if(manifest.cases.length!==expected.length)throw new Error('missing declared source-derived task');
  for(const item of expected) {
    const observed=manifest.cases.find(candidate=>candidate.runId===item.runId);
    if(!observed||observed.task!==item.task||observed.taskSHA256!==digest(item.task)||observed.taskKind!==item.taskKind
        || observed.category!=='feature-implementation'||observed.expectedRelation!=='output-larger')throw new Error('source-derived declaration changed');
  }
  for(const binding of manifest.bindings)readBoundSource(binding);
  return {catalogSHA256:manifest.catalogSHA256,attemptedTasks:expected.length,representativeBaseline:false};
}

export async function evaluateSourceContracts(root, output) {
  root=resolve(root);output=resolve(output);mkdirSync(output);
  const binding=path=>({path,sha256:digest(readFileSync(path))});
  const catalog=discoverSourceContracts(root);
  const archive=join(root,'experiments/formal_ai_subagent/evidence/formal-ai-only-1188/module-observation/original-selected-summary.test.mjs');
  const originalPath=join(root,'experiments/formal_ai_subagent/evidence/formal-ai-only-1188/g132/T1804-request.json');
  const arithmeticPath=join(root,'rust/tests/web/coding-amplification.test.mjs');
  const original=JSON.parse(readBoundSource(binding(originalPath)));
  if(Buffer.byteLength(original.prompt)!==955)throw new Error('original955-byte request drift');
  const heldOut=catalog.heldOut.value;
  const arithmetic=catalog.arithmetic;
  const oracleDirectory=join(output,'oracles');mkdirSync(oracleDirectory);
  const sumOracle=join(oracleDirectory,'arithmetic.test.mjs');
  const arithmeticChecks=arithmetic.assertions.map(item=>`assert.equal(module.${item.exportName}(${item.first},${item.second}),${item.expected});`).join('\n');
  writeFileSync(sumOracle,"import assert from 'node:assert/strict';\nimport {pathToFileURL} from 'node:url';\nimport {join} from 'node:path';\nconst module=await import(pathToFileURL(join(process.env.COHORT_WORKSPACE,'result.mjs')).href);\n"+arithmeticChecks+'\n');
  const sources=['solver_formalization.mjs','translation_formalization.mjs'];
  const materialize=(id,target)=>{
    const workspace=join(output,id);mkdirSync(workspace);mkdirSync(dirname(join(workspace,target)),{recursive:true});
    for(const name of sources)copyFileSync(join(root,'js/agentic/crate',name),join(workspace,name));
    copyFileSync(archive,join(workspace,'selected-summary.test.mjs'));
    return workspace;
  };
  const cases=[];
  const originalWorkspace=materialize('g132-original','repair-summary.mjs');
  cases.push({runId:'g132-original',taskKind:'self-coding',category:'feature-implementation',expectedRelation:'output-larger',task:original.prompt,
    workspace:originalWorkspace,target:'repair-summary.mjs',exportName:'selectedSummary',oracle:binding(archive),originalRequest:binding(originalPath),heldOut:false});
  for(const item of heldOut) {
    const workspace=materialize('g132-'+item.id,item.destination);
    const task=compositionVariantTask(item);
    cases.push({runId:'g132-'+item.id,taskKind:'self-coding',category:'feature-implementation',expectedRelation:'output-larger',task,workspace,
      target:item.destination,exportName:item.exportName,oracle:binding(archive),heldOut:true,heldOutSource:binding(archive),heldOutContract:item});
  }
  const ordinaryWorkspace=join(output,'ordinary-arithmetic');mkdirSync(ordinaryWorkspace);
  cases.push({runId:'ordinary-arithmetic',taskKind:'coding',category:'feature-implementation',expectedRelation:'output-larger',task:`Add ${arithmetic.exportName}(first,second) in result.mjs.`,workspace:ordinaryWorkspace,
    target:'result.mjs',oracle:binding(sumOracle),sourceDerivedAssertions:arithmetic,heldOut:false});
  for(const item of cases) {
    item.taskSHA256=digest(item.task);item.timeoutMilliseconds=2000;
    item.allowedEffects=[join(item.workspace,item.target),join(item.workspace,'.formal-ai/general-change-plan.lino')];
  }
  const runtimePaths=['experiments/js_dogfood/drive.mjs','js/agentic/planner.mjs','js/agentic/host.mjs','js/server/worker-host.mjs',
    'js/agentic/node-host.mjs','js/agentic/command_reroute.mjs','js/agentic/content.mjs','scripts/lib/translation-blockers.mjs',
    'js/agentic/crate/solver_formalization.mjs','js/agentic/crate/translation_formalization.mjs'];
  const dependencyPaths=[...runtimePaths.map(path=>join(root,path)),archive,originalPath,arithmeticPath,
    fileURLToPath(import.meta.url),join(dirname(fileURLToPath(import.meta.url)),'cohort-source-contracts.mjs'),
    join(dirname(fileURLToPath(import.meta.url)),'cohort-runner.mjs'),
    join(dirname(fileURLToPath(import.meta.url)),'coding-amplification.mjs'),
    join(dirname(fileURLToPath(import.meta.url)),'cohort-usage-report.mjs'),
    join(dirname(fileURLToPath(import.meta.url)),'journal-provider-usage.mjs'),
    join(dirname(fileURLToPath(import.meta.url)),'task-relations.mjs'),sumOracle];
  for(const item of cases.filter(item=>item.taskKind==='self-coding'))for(const name of [...sources,'selected-summary.test.mjs'])dependencyPaths.push(join(item.workspace,name));
  const manifest={schemaVersion:1,cohortId:'source-derived-bounded-two-family-evaluation',catalogSHA256:digest(JSON.stringify(catalog)),heldOutQualification:catalog.heldOutQualification,bindings:[...new Set(dependencyPaths)].map(binding),cases,
    representativeBaseline:false,independentFamilies:2,originalBehavioralObligations:7,usagePolicy:'Unknown absent actual provider capture; no token or price fabrication',
    materializationAuthorship:'supplied source/test scaffold, zero autonomous credit',observedInputCompleteness:'bounded actualdriver observations; full coordinating-model context remainsUnknown'};
  manifest.cohortId+='-'+digest(JSON.stringify({output,bindings:manifest.bindings})).slice(0,24);
  writeFileSync(join(output,'manifest.json'),JSON.stringify(manifest,null,2)+'\n');
  checkSourceEvaluation(root,join(output,'manifest.json'));
  const admission=admitManifest(manifest,join(output,'journal.jsonl'));
  const load=path=>import(pathToFileURL(join(root,path)).href);
  const {drive}=await load('experiments/js_dogfood/drive.mjs');const {WorkerHost}=await load('js/server/worker-host.mjs');
  const {installNodeHost}=await load('js/agentic/node-host.mjs');await installNodeHost(new WorkerHost());
  const {planChatStep}=await load('js/agentic/planner.mjs');const {solve}=await load('js/agentic/host.mjs');
  const {planSymbolicCommandReroute}=await load('js/agentic/command_reroute.mjs');const {latestUserRequest}=await load('js/agentic/content.mjs');
  const fallthrough=async(messages,tools)=>{const symbolic=await solve(latestUserRequest(messages)??'',[]);return planSymbolicCommandReroute(messages,tools,symbolic)??{kind:'final',answer:symbolic.answer};};
  const measurements=[];
  for(const item of cases) {
    process.env.SELECTED_SUMMARY_MODULE=join(item.workspace,item.target);process.env.SELECTED_SUMMARY_EXPORT=item.exportName??'selectedSummary';
    const result=await executeAdmittedCase(admission,item.runId,'first-unchanged',
      (task,directory)=>drive(planChatStep,directory,task,{steps:24,fallthrough,allowedCommands:['node --test selected-summary.test.mjs']}));
    writeFileSync(join(output,item.runId+'-result.json'),JSON.stringify(result,null,2)+'\n');
    const sourceEffects=result.sourceEffects??[];
    const changes=sourceEffects.map(effect=>({path:effect.path,role:effect.path.includes('/.formal-ai/')?'evidence':'production',language:'javascript',before:effect.before?.content??'',after:effect.after?.content??''}));
    const repositoryContext=(result.transcript??[]).filter(entry=>entry.tool==='read').map(entry=>entry.result);
    const toolReceipts=(result.transcript??[]).map(entry=>JSON.stringify(entry.tool==='read'?{...entry,result:undefined}:entry));
    const run={runId:item.runId,taskKind:item.taskKind,origin:'autonomous',task:item.task,attemptInputs:[item.task],attemptIds:['first-unchanged'],repositoryContext,reviewedPatches:[],toolReceipts,changes};
    const measurement=await measureCodingRun(run,async actual=>({passed:result.accepted,bindings:actual.map(change=>({path:change.path,before:sourceDigest(change.before),after:sourceDigest(change.after)})),checks:[{name:'source-bound independent oracle',exitCode:result.check?.exitCode??null}]}));
    measurements.push({...measurement,observedInputCompleteness:manifest.observedInputCompleteness});
  }
  const providerUsage=writeJournalUsageReport(binding(join(output,'journal.jsonl')),join(output,'usage-receipts-index.json'));
  const report={cohortId:manifest.cohortId,manifestSHA256:admission.manifestSHA256,attemptedTasks:cases.length,independentFamilies:2,
    originalTaskBytes:Buffer.byteLength(original.prompt),originalBehavioralObligations:7,heldOutTasks:heldOut.length,representativeBaseline:false,
    summary:summarizeCodingRuns(measurements,cases.map(item=>item.runId)),measurements,providerUsage};
  writeFileSync(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');return report;
}
if(import.meta.url===pathToFileURL(resolve(process.argv[1]??'.')).href) {
  if(process.argv[2]==='--catalog'){console.log(JSON.stringify(discoverSourceContracts(process.argv[3]??process.cwd()),null,2));process.exit(0);}
  if(process.argv[2]==='--check'){console.log(JSON.stringify(checkSourceEvaluation(process.argv[3]??process.cwd(),process.argv[4])));process.exit(0);}
  const report=await evaluateSourceContracts(process.argv[2]??process.cwd(),process.argv[3]);
  console.log(JSON.stringify({attemptedTasks:report.attemptedTasks,summary:report.summary,representativeBaseline:false}));process.exit(0);
}
