import {readFileSync,writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {installTextHost} from '/Users/konard/Code/Archive/link-assistant/formal-ai/scripts/lib/text-capability-measures.mjs';
import {WorkerHost} from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/server/worker-host.mjs';
import {constantsOf} from './typed-reader-next/typed-spec-reader.mjs';
import {compileSkill} from './typed-reader-next/compiled-skill-prototype.mjs';
const root='/Users/konard/Code/Archive/link-assistant/formal-ai';const nativeFile=root+'/rust/tests/unit/specification/natural_language_skill_compilation.rs';const source=readFileSync(nativeFile,'utf8');const constants=constantsOf(source,nativeFile,root);const skill=constants.get('SKILL').value;const trigger=constants.get('TRIGGER').value;installTextHost();const p=compileSkill(skill);const host=new WorkerHost();const history=[];const observations=[];
for(const [phase,prompt]of[['teach',skill],['count','How many behavior rules are there?'],['list','Show rules'],['replay',trigger]]){
 const answer=await host.solve(prompt,history);observations.push({phase,prompt,history:structuredClone(history),answer});history.push({role:'user',content:prompt},{role:'assistant',content:answer.content});
}
const teach=observations[0].answer;const count=observations[1].answer;const list=observations[2].answer;const replay=observations[3].answer;
const facts={teachHasCompiledPackage:teach.content.includes('type "compiled_skill_package"')&&teach.content.includes(p.id),teachLogsActualPackage:teach.solverEvents.some(e=>e.kind==='skill_compile:package'&&e.payload===p.id),countActualDialogLocalOne:count.content.includes('dialog_local_rules "1"'),countNativeClosingNewline:count.content.endsWith('\n'),listHasActualPackageAndLegacyId:list.content.includes(p.id)&&list.content.includes(p.legacy_behavior_rule_id),replayContentCorrect:replay.content===p.response,replayHasCompiledPackageEvent:replay.solverEvents.some(e=>e.kind==='compiled_skill:package'&&e.payload===p.linksNotation()),replayHasCacheHit:replay.solverEvents.some(e=>e.kind==='cache_hit'&&e.payload===p.id)};
const packet={sourceHead:execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim(),scope:'actual unmodified production WorkerHost; genuine sequential user/assistant history',productionSourceMutations:0,nativeFixture:nativeFile,packageExpected:{id:p.id,rule_id:p.rule_id,handler_id:p.handler_id,legacy_behavior_rule_id:p.legacy_behavior_rule_id},facts,observations};
writeFileSync('/private/tmp/spec-recurrence-1188/compiled-consumers-production-baseline.json',JSON.stringify(packet,null,2)+'\n');console.log(JSON.stringify({sourceHead:packet.sourceHead,scope:packet.scope,facts},null,2));
