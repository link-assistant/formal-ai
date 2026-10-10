// A bounded structural execution review, not a general usefulness proof.
import {parse} from '@babel/parser';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
const hash=value=>createHash('sha256').update(value).digest('hex');
export const oracleReviewerIdentity=()=>hash(readFileSync(fileURLToPath(import.meta.url)));
const assertionMethods=new Set(['equal','strictEqual','notEqual','notStrictEqual','deepEqual','deepStrictEqual','notDeepEqual','notDeepStrictEqual','ok','ifError']);
const pureMethods={ 'node:fs':new Set(['readFileSync']), 'node:path':new Set(['join','resolve']) };
export function reviewClosedOracleSource(source,expectedSHA256){
 const base={oracleSHA256:hash(source),reviewerSHA256:oracleReviewerIdentity(),scope:'source-owned closed structural execution only',qualityRequiresReview:true,semanticCompleteness:'Unknown'};
 try{
  if(Buffer.byteLength(source)>1048576||base.oracleSHA256!==expectedSHA256)throw Error('oracle source bound or identity mismatch');
  const ast=parse(source,{sourceType:'module'}),environment=new Map(),assertions=[],tests=[],names=new Set();let nodes=0;
  function fail(reason){throw Error(reason);}
  function bind(name,value,env){if(env.has(name))fail('shadowed oracle binding');env.set(name,value);}
  function property(node){if(node.computed||node.optional||node.property?.type!=='Identifier')fail('unknown oracle member');return node.property.name;}
  function callOwner(node,env){
   if(node.type==='Identifier')return env.get(node.name);
   if(node.type==='MemberExpression'&&node.object.type==='Identifier'){
    const owner=env.get(node.object.name),member=property(node);
    if(owner?.kind==='assert'&&assertionMethods.has(member))return {kind:'assertion',method:member};
    if(owner?.kind==='pureNamespace'&&pureMethods[owner.module].has(member))return {kind:'pure',method:member};
   }
   fail('unknown imported oracle callable');
  }
  function pure(node,env){
   if(!node||++nodes>65536)fail('oracle expression bound');
   if(['StringLiteral','NumericLiteral','BooleanLiteral','NullLiteral'].includes(node.type))return;
   if(node.type==='Identifier'){if(env.get(node.name)?.kind!=='value')fail('unknown pure oracle binding');return;}
   if(node.type==='UnaryExpression'&&['+','-','!','~'].includes(node.operator)){pure(node.argument,env);return;}
   if(node.type==='BinaryExpression'&&['+','-','*','/','%','**','===','!==','==','!=','<','<=','>','>=','&','|','^','<<','>>','>>>'].includes(node.operator)){pure(node.left,env);pure(node.right,env);return;}
   if(node.type==='ArrayExpression'){for(const value of node.elements)pure(value,env);return;}
   if(node.type==='ObjectExpression'){for(const value of node.properties){if(value.type!=='ObjectProperty'||value.computed||value.method||!['Identifier','StringLiteral'].includes(value.key.type)||['__proto__','constructor','prototype'].includes(value.key.name??value.key.value))fail('unknown oracle object');pure(value.value,env);}return;}
   if(node.type==='MemberExpression'&&!node.computed&&node.property.type==='Identifier'&&node.property.name==='COHORT_WORKSPACE'&&node.object.type==='MemberExpression'&&!node.object.computed&&node.object.property.name==='env'&&node.object.object.type==='Identifier'&&node.object.object.name==='process'&&!env.has('process'))return;
   if(node.type==='CallExpression'&&!node.optional){const owner=callOwner(node.callee,env);if(owner?.kind!=='pure')fail('unknown pure oracle call');for(const value of node.arguments)pure(value,env);return;}
   fail('unsupported pure oracle expression '+node.type);
  }
  function statement(node,env,allowTest=true){
   if(++nodes>65536)fail('oracle statement bound');
   if(node.type==='VariableDeclaration'&&node.kind==='const'){for(const declaration of node.declarations){if(declaration.id.type!=='Identifier')fail('unknown oracle declaration');pure(declaration.init,env);bind(declaration.id.name,{kind:'value'},env);}return;}
   if(node.type!=='ExpressionStatement'||node.expression.type!=='CallExpression'||node.expression.optional)fail('unsupported oracle statement '+node.type);
   const call=node.expression;
   if(call.callee.type==='MemberExpression'&&call.callee.object.type==='Identifier'&&call.callee.object.name==='console'&&!env.has('console')&&property(call.callee)==='log'){
    if(!call.arguments.length||call.arguments.some(value=>!['StringLiteral','NumericLiteral','BooleanLiteral','NullLiteral'].includes(value.type)))fail('unknown diagnostic expression');return;
   }
   const owner=callOwner(call.callee,env);
   if(owner?.kind==='assertion'||owner?.kind==='assert'){
    if(!call.arguments.length)fail('missing assertion argument');for(const value of call.arguments)pure(value,env);
    assertions.push({start:call.start,end:call.end,method:owner.kind==='assert'?'ok':owner.method});if(assertions.length>4096)fail('oracle assertion bound');return;
   }
   if(owner?.kind==='test'&&allowTest){
    if(call.arguments.length<2||call.arguments.length>3||call.arguments[0].type!=='StringLiteral'||!call.arguments[0].value||names.has(call.arguments[0].value))fail('unknown or duplicate test identity');
    const callback=call.arguments.at(-1);
    if(call.arguments.length===3){const options=call.arguments[1];if(options.type!=='ObjectExpression')fail('unknown test options');for(const option of options.properties){const name=option.key?.name??option.key?.value;if(option.type!=='ObjectProperty'||option.computed||!['skip','todo'].includes(name)||option.value.type!=='BooleanLiteral'||option.value.value)fail('barred or unknown test option');}}
    if(!['ArrowFunctionExpression','FunctionExpression'].includes(callback.type)||callback.async||callback.generator||callback.params.length)fail('unknown test callback');
    const count=assertions.length,scope=new Map(env);if(callback.body.type==='BlockStatement'){for(const child of callback.body.body)statement(child,scope,false);}else statement({type:'ExpressionStatement',expression:callback.body},scope,false);
    if(assertions.length===count)fail('test has no source-owned assertion');names.add(call.arguments[0].value);tests.push({name:call.arguments[0].value,start:call.start,end:call.end,sourceAssertions:assertions.length-count});if(tests.length>256)fail('test count bound');return;
   }
   fail('unknown statement callable');
  }
  for(const item of ast.program.body){
   if(item.type==='ImportDeclaration'){
    const module=item.source.value;
    if(!['node:assert','node:assert/strict','node:test',...Object.keys(pureMethods)].includes(module)||item.assertions?.length||item.attributes?.length)fail('unknown independent oracle import');
    if(!item.specifiers.length)fail('unknown side-effect import');
    for(const value of item.specifiers){let owner;
     if(['node:assert','node:assert/strict'].includes(module)){if(['ImportDefaultSpecifier','ImportNamespaceSpecifier'].includes(value.type))owner={kind:'assert'};else if(assertionMethods.has(value.imported?.name))owner={kind:'assertion',method:value.imported.name};}
     else if(module==='node:test'){if(value.type==='ImportDefaultSpecifier'||value.type==='ImportSpecifier'&&value.imported?.name==='test')owner={kind:'test'};}
     else if(value.type==='ImportNamespaceSpecifier')owner={kind:'pureNamespace',module};else if(value.type==='ImportSpecifier'&&pureMethods[module].has(value.imported?.name))owner={kind:'pure',method:value.imported.name};
     if(!owner||['process','console','globalThis'].includes(value.local.name))fail('unknown independent oracle import identity');bind(value.local.name,owner,environment);
    }
   }else statement(item,environment);
  }
  if(!assertions.length)fail('zero source-owned assertions');
  return {...base,complete:true,sourceAssertions:assertions.length,assertionObligations:assertions,testObligations:tests,expectedProcessTests:tests.length||1};
 }catch(error){return {...base,complete:false,reason:String(error.message)};}
}
export function oracleProcessEvidence(check,expectedTests){
 if(check.status!==0||check.error||check.signal)return {complete:false,reason:'oracle process failed'};
 const output=String(check.stdout??''),counts=Object.create(null);
 const headers=[...output.matchAll(/^TAP version 13$/gm)],plans=[...output.matchAll(/^1\.\.([0-9]+)$/gm)],durations=[...output.matchAll(/^# duration_ms ([0-9]+(?:\.[0-9]+)?)$/gm)];
 if(headers.length!==1||plans.length!==1||Number(plans[0][1])!==expectedTests||durations.length!==1||!Number.isFinite(Number(durations[0][1])))return {complete:false,reason:'malformed TAP envelope'};
 for(const name of ['tests','suites','pass','fail','cancelled','skipped','todo']){const matches=[...output.matchAll(new RegExp('^# '+name+' ([0-9]+)$','gm'))];if(matches.length!==1||!Number.isSafeInteger(Number(matches[0][1])))return {complete:false,reason:'malformed TAP footer'};counts[name]=Number(matches[0][1]);}
 if(!Number.isSafeInteger(expectedTests)||expectedTests<1||counts.tests!==expectedTests||counts.pass!==expectedTests||counts.suites||counts.fail||counts.cancelled||counts.skipped||counts.todo)return {complete:false,reason:'nonexecuted or incomplete tests',counts};
 return {complete:true,counts,scope:'ordinary TAP process completion of closed source obligations; semantic quality requires independent review'};
}
