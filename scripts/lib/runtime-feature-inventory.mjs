// Public entrypoint declarations and evidence coverage, never a parity verdict.
import {createHash} from 'node:crypto';
import {parse} from '@babel/parser';
import {parseRoutes} from '../../js/server/routes.mjs';
import {parseLino,childrenNamed,childValue} from '../../js/server/lino.mjs';
import {loadCorpus} from '../check-server-parity.mjs';
export const FEATURE_SOURCES=Object.freeze({
 nativeCli:'rust/src/main.rs',nodeCli:'js/server/main.mjs',
 agentCli:'js/agentic/orchestration_cli.mjs',integrityCli:'js/provenance/cli.mjs',
 memoryCli:'js/server/memory-upgrade.mjs',routes:'data/meta/server-routes.lino',
 routeBindings:'js/server/dispatch.mjs',corpus:'rust/tests/fixtures/server-parity/requests.lino',
 browserMessages:'js/worker/formal_ai_worker_solve.js',
 browserHandlers:'data/seed/browser-handler-precedence.lino',
 tools:'data/seed/tools.lino',toolCapabilities:'data/seed/agentic-tool-capabilities.lino',
});
const digest=text=>createHash('sha256').update(text).digest('hex');
function walk(value,visit){
 if(!value||typeof value!=='object')return;
 if(typeof value.type==='string')visit(value);
 for(const [key,child]of Object.entries(value)){
  if(['loc','start','end','extra','comments','tokens'].includes(key))continue;
  if(Array.isArray(child))for(const item of child)walk(item,visit);else walk(child,visit);
 }
}
function ast(text){return parse(text,{sourceType:'unambiguous',errorRecovery:false});}
function keyOf(property){return property.computed?null:property.key?.name??property.key?.value??null;}
export function objectMembers(text,name){
 const matches=[];walk(ast(text),node=>{
  if(node.type!=='VariableDeclarator'||node.id?.name!==name)return;
  const object=node.init?.type==='ObjectExpression'?node.init:
   node.init?.type==='CallExpression'?node.init.arguments[0]:null;
  if(object?.type==='ObjectExpression')matches.push(object);
 });
 if(matches.length!==1)throw Error('missing or ambiguous authoritative object '+name);
 return matches[0].properties.map(property=>{
  const key=keyOf(property);if(property.type==='SpreadElement'||typeof key!=='string')throw Error('dynamic authoritative object member '+name);
  return key;
 });
}
export function equalityChoices(text,selector){
 const choices=new Set();walk(ast(text),node=>{
  if(node.type!=='BinaryExpression'||!['===','!==','==','!='].includes(node.operator))return;
  for(const [member,literal]of [[node.left,node.right],[node.right,node.left]]){
   const match=selector==='action'?member.type==='Identifier'&&member.name==='action':
    member.type==='MemberExpression'&&member.object?.name===selector&&member.computed&&member.property?.value===0;
   if(match&&literal.type==='StringLiteral')choices.add(literal.value);
  }
 });return [...choices].sort();
}
export function browserProtocol(text){
 const handlers=[];walk(ast(text),node=>{
  if(node.type==='AssignmentExpression'&&node.left?.object?.name==='self'&&node.left?.property?.name==='onmessage')handlers.push(node.right);
 });if(handlers.length!==1)throw Error('missing or ambiguous browser message owner');
 const requests=new Set(),responses=new Set();walk(handlers[0],node=>{
  if(node.type==='BinaryExpression'&&node.operator==='==='&&node.left?.object?.name==='data'
   &&node.left?.property?.name==='kind'&&node.right?.type==='StringLiteral')requests.add(node.right.value);
 });
 walk(ast(text),node=>{
  if(node.type!=='CallExpression'||node.callee?.name!=='postMessage')return;
  const object=node.arguments[0];if(object?.type!=='ObjectExpression')throw Error('unmeasured dynamic browser response');
  const kinds=object.properties.filter(property=>keyOf(property)==='kind');
  if(kinds.length!==1||kinds[0].value.type!=='StringLiteral')throw Error('unmeasured dynamic browser response kind');
  responses.add(kinds[0].value.value);
 });return {requests:[...requests].sort(),responses:[...responses].sort(),defaultRequest:'default-solve'};
}
function state(status,source,detail){return {status,source,detail,evidence:[],behavior:'unmeasured'};}
function row(id,domain,authority,detail={}){
 return {id,domain,authority,detail,runtimes:{
  rust:state('unmeasured',null,'native command/behavior receipt not admitted'),
  javascript:state('unmeasured',null,'entrypoint-specific observation needed'),
  typescript:state('unmeasured',null,'generated source is not independent runtime evidence'),
  browser:state('unmeasured',null,'browser execution evidence not admitted'),
 },behavioralParity:'unmeasured'};
}
export function buildRuntimeFeatureInventory(read){
 const texts=new Map(),sources=[];for(const file of Object.values(FEATURE_SOURCES)){
  const text=read(file);texts.set(file,text);sources.push({path:file,sha256:digest(text)});
 }
 const features=[],gaps=[{domain:'native-cli',source:FEATURE_SOURCES.nativeCli,
  reason:'Full actual Args::command() graph not yet measured; main.rs subcommands and nested/flattened types must be captured by hosted existing binary probe.'}];
 const routeRows=parseRoutes(texts.get(FEATURE_SOURCES.routes));
 const handlers=new Set(objectMembers(texts.get(FEATURE_SOURCES.routeBindings),'ROUTE_HANDLERS'));
 const corpus=loadCorpus(texts.get(FEATURE_SOURCES.corpus));
 const routeIds=new Set(routeRows.map(route=>route.id));
 for(const route of routeRows){
  const feature=row('http:'+route.id,'http-route',FEATURE_SOURCES.routes,
   {method:route.method,paths:route.paths,auth:route.auth,deprecated:route.deprecated});
  feature.runtimes.javascript=state(handlers.has(route.id)?'source_bound':'missing_binding',
   FEATURE_SOURCES.routeBindings,'manifest dispatch binding; behavior unmeasured');
  feature.runtimes.typescript=state('unmeasured','ts/server/dispatch.mts','projection/runtime admission pending');
  feature.runtimes.rust=state('unmeasured','rust/src/server.rs','independent native matcher needs actual route evidence');
  const declared=corpus.requests.filter(request=>request.route===route.id).map(request=>request.id);
  feature.declaredEvidence=[{kind:'corpus_declared',source:FEATURE_SOURCES.corpus,cases:declared,
   authority:'test definitions only; no execution/pass receipt'}];
  if(!handlers.has(route.id))gaps.push({domain:'http-route',id:feature.id,reason:'manifest route missing JS dispatch binding'});
  if(!declared.length)gaps.push({domain:'http-route',id:feature.id,reason:'route has no declared finite corpus case'});
  features.push(feature);
 }
 for(const id of handlers)if(!routeIds.has(id)){
  features.push(row('http-unregistered-handler:'+id,'http-route-binding',FEATURE_SOURCES.routeBindings));
  gaps.push({domain:'http-route',id,reason:'dispatch handler missing from shared route manifest'});
 }
 for(const request of corpus.requests)if(request.route&&!routeIds.has(request.route))gaps.push({domain:'http-corpus',id:request.id,reason:'unknown route reference '+request.route});
 const cli=(path,source)=>{
  const feature=row('cli:'+path,'cli-command',source);feature.runtimes.javascript=state('source_bound',source,'dispatch/table declaration only');features.push(feature);
 };
 for(const name of equalityChoices(texts.get(FEATURE_SOURCES.nodeCli),'argv'))cli(name,FEATURE_SOURCES.nodeCli);
 // parseArgs owns the default server mode; this is a call-bound entry, not an inferred native command.
 cli('serve',FEATURE_SOURCES.nodeCli);
 for(const action of objectMembers(texts.get(FEATURE_SOURCES.agentCli),'ACTIONS'))cli('agent:'+action,FEATURE_SOURCES.agentCli);
 for(const action of equalityChoices(texts.get(FEATURE_SOURCES.memoryCli),'action'))cli('memory:'+action,FEATURE_SOURCES.memoryCli);
 for(const action of equalityChoices(texts.get(FEATURE_SOURCES.integrityCli),'argumentsList'))cli('provenance:'+action,FEATURE_SOURCES.integrityCli);
 const browser=browserProtocol(texts.get(FEATURE_SOURCES.browserMessages));
 for(const kind of [...browser.requests,browser.defaultRequest]){
  const feature=row('browser-message:'+kind,'browser-message',FEATURE_SOURCES.browserMessages);
  feature.runtimes.browser=state('source_bound',FEATURE_SOURCES.browserMessages,'request branch/default only; engine availability and behavior unmeasured');features.push(feature);
 }
 for(const kind of browser.responses){const feature=row('browser-response:'+kind,'browser-response',FEATURE_SOURCES.browserMessages);feature.runtimes.browser=state('source_bound',FEATURE_SOURCES.browserMessages,'postMessage kind declaration');features.push(feature);}
 for(const [domain,file,tag]of [
  ['browser-handler',FEATURE_SOURCES.browserHandlers,'handler'],
  ['tool',FEATURE_SOURCES.tools,'tool'],['tool-capability',FEATURE_SOURCES.toolCapabilities,'capability'],
 ])for(const node of childrenNamed(parseLino(texts.get(file)),tag)){
  const feature=row(domain+':'+node.value,domain,file,{name:childValue(node,'name')||node.value});
  feature.declaration='registry_declared';
  if(domain==='browser-handler')feature.runtimes.browser=state('registry_declared',file,'handler membership/order does not establish executable/runtime coverage');
  features.push(feature);
 }
 features.sort((a,b)=>a.id.localeCompare(b.id));
 const seen=new Set();for(const feature of features){if(seen.has(feature.id))throw Error('duplicate canonical feature ID '+feature.id);seen.add(feature.id);}
 return {schema:'formal-ai-runtime-feature-coverage-v1',sources,
  discovery:{complete:false,nativeCli:'pending_actual_clap_schema',browserActions:'handler registry and message kinds only; dynamic UI effects unmeasured',
   commandOptions:'not a complete option/default/alias census',entrypointRoots:'explicit authoritative product entrypoints; no glob/symbol denominator'},
  behavioralParity:'unmeasured',runtimeObservedEvidence:[],gaps,features};
}
