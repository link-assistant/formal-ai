import {before,test} from 'node:test';
import assert from 'node:assert/strict';
import {responseArgumentsForTool} from '../../../js/agentic/crate/protocol_responses.mjs';
import {invalidArguments} from '../../../js/agentic/crate/tool_call_validation.mjs';
import {chatCompletionFromPlan,responseFromPlan} from '../../../js/server/agentic.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
before(async()=>{await installNodeHost(new WorkerHost());});
const edit={type:'object',properties:{old_string:{type:'string'},new_string:{type:'string'}},required:['old_string','new_string'],additionalProperties:false};
const multi={type:'object',properties:{paths:{type:'array',items:{type:'string'}},edits:{type:'array',minItems:1,items:edit}},required:['paths','edits'],additionalProperties:false};
const status={type:'string',enum:['pending','in_progress','completed']};
const todo={type:'object',properties:{plan:{type:'array',minItems:1,items:{type:'object',properties:{step:{type:'string'},status},required:['step','status'],additionalProperties:false}}},required:['plan'],additionalProperties:false};
function fixture(schema,source,name='transform_records') {
 const tools=[{type:'function',name,parameters:schema}],argumentsText=JSON.stringify(source);
 const projected=JSON.parse(responseArgumentsForTool(tools,name,argumentsText,'explicit user task',null));
 const request={tools,messages:[{role:'user',content:'explicit user task'}]};
 const plan={kind:'tool_calls',calls:[{tool:name,arguments:argumentsText}]};
 const chat=chatCompletionFromPlan({},request,'explicit user task',plan,0);
 const responses=responseFromPlan({},request,request,'explicit user task',plan,0);
 return {projected,chat,responses};
}
function accepted(result,expected) {
 assert.deepEqual(result.projected,expected);
 assert.deepEqual(JSON.parse(result.chat.choices[0].message.tool_calls[0].function.arguments),expected);
 assert.deepEqual(JSON.parse(result.responses.output.find(row=>row.type==='function_call').arguments),expected);
}
function refused(result,schema) {
 assert.ok(invalidArguments(result.projected,schema));
 assert.deepEqual(result.chat.choices[0].message.tool_calls,[]);
 assert.equal(result.chat.choices[0].finish_reason,'stop');
 assert.ok(!result.responses.output.some(row=>row.type==='function_call'));
}
test('strict nested edit aliases are projected generically with array order and explicit precedence preserved',()=>{
 const source={paths:['b.txt','a.txt'],edits:[{old:'alpha',old_string:'authoritative',new:'beta',new_string:'beta'},{old:'gamma',new:'delta'}]};
 const result=fixture(multi,source);
 const expected={paths:['b.txt','a.txt'],edits:[{old_string:'authoritative',new_string:'beta'},{old_string:'gamma',new_string:'delta'}]};
 accepted(result,expected);assert.equal(invalidArguments(result.projected,multi),null);
 assert.deepEqual(source.edits[1],{old:'gamma',new:'delta'},'projection owns its output without mutating caller values');
});
test('nested todo status values stay exact rather than choosing the first enum',()=>{
 const source={plan:[{step:'First',status:'completed',capability_alias:'discard'},{step:'Second',status:'in_progress'},{step:'Third',status:'pending'}]};
 accepted(fixture(todo,source),{plan:source.plan.map(({step,status})=>({step,status}))});
});
test('declared nested objects and arrays of arrays use the same alias contract',()=>{
 const schema={type:'object',properties:{payload:{type:'object',properties:{groups:{type:'array',items:{type:'array',items:edit}}},required:['groups'],additionalProperties:false}},required:['payload'],additionalProperties:false};
 accepted(fixture(schema,{payload:{groups:[[{old:'a',new:'b'}],[{old:'c',new:'d'}]],unused:'alias'}}),{payload:{groups:[[{old_string:'a',new_string:'b'}],[{old_string:'c',new_string:'d'}]]}});
});
for(const [label,source] of [['wrong scalar',{paths:['a'],edits:'not an array'}],['null item',{paths:['a'],edits:[null]}],['explicit invalid alias target',{paths:['a'],edits:[{old_string:7,old:'alpha',new:'beta'}]}],['null explicit value',{paths:['a'],edits:[{old_string:null,old:'alpha',new:'beta'}]}],['missing edit text',{paths:['a'],edits:[{old:'alpha'}]}],['missing array',{paths:['a']}],['empty array',{paths:['a'],edits:[]}]] )test('invalid nested edit remains a boundary refusal: '+label,()=>{
 const result=fixture(multi,source);refused(result,multi);
 if(label==='explicit invalid alias target')assert.equal(result.projected.edits[0].old_string,7);
 if(label==='null explicit value')assert.equal(result.projected.edits[0].old_string,null);
 if(label==='missing array')assert.equal(Object.hasOwn(result.projected,'edits'),false);
});
for(const entry of [{step:'task'},{step:'task',status:'done'},{step:'task',status:null},{status:'pending'}])test('todo projection never invents or coerces missing required task/status: '+JSON.stringify(entry),()=>{
 const result=fixture(todo,{plan:[entry]});refused(result,todo);
 assert.deepEqual(result.projected,{plan:[entry]});
});
test('unresolved nested references remain visible and refuse on both full boundaries',()=>{
 const item={...edit,$ref:'#/$defs/unknown'},schema={...multi,properties:{...multi.properties,edits:{...multi.properties.edits,items:item}}};
 const source={paths:['a'],edits:[{old:'alpha',new:'beta'}]};
 const result=fixture(schema,source);assert.deepEqual(result.projected,source);refused(result,schema);
 const root={...multi,$ref:'#/$defs/unknown'};assert.deepEqual(fixture(root,source).projected,source);refused(fixture(root,source),root);
});
test('projection cutoff keeps the deep value so validation cannot escape its depth refusal',()=>{
 let source={old:'alpha',new:'beta'},schema=edit;
 for(let depth=0;depth<36;depth++){source={nested:source};schema={type:'object',properties:{nested:schema},required:['nested'],additionalProperties:false};}
 const result=fixture(schema,source);assert.deepEqual(result.projected,source);refused(result,schema);
 assert.match(invalidArguments(result.projected,schema),/depth/i);
});
for(const [name,prompt,schema] of [['multi_edit','replace alpha with beta in a.txt and b.txt',multi],['update_plan','create a todo list for this change',todo]])test('actual maintained planner reaches the original strict '+name+' schema on both wire surfaces',async()=>{
 const plan=await planChatStep([{role:'user',content:prompt}],[name]);
 assert.equal(plan.kind,'tool_calls');
 const request={messages:[{role:'user',content:prompt}],tools:[{type:'function',name,parameters:schema}]};
 const chat=chatCompletionFromPlan({},request,prompt,plan,0),responses=responseFromPlan({},request,request,prompt,plan,0);
 const chatArguments=JSON.parse(chat.choices[0].message.tool_calls[0].function.arguments);
 const responseArguments=JSON.parse(responses.output.find(row=>row.type==='function_call').arguments);
 assert.equal(invalidArguments(chatArguments,schema),null);assert.deepEqual(responseArguments,chatArguments);
 if(name==='update_plan')assert.equal(chatArguments.plan[0].status,'pending');
 else assert.deepEqual(chatArguments.edits,[{old_string:'alpha',new_string:'beta'}]);
});


test('declared reserved keys retain distinct own values through nested projection and both wire surfaces',()=>{
 const reservedSchema=JSON.parse('{"type":"object","properties":{"__proto__":{"type":"string"},"constructor":{"type":"string"},"prototype":{"type":"string"},"toString":{"type":"string"}},"required":["__proto__","constructor","prototype","toString"],"additionalProperties":false}');
 const schema={type:'object',properties:{records:{type:'array',items:reservedSchema}},required:['records'],additionalProperties:false};
 const source=JSON.parse('{"records":[{"__proto__":"first prototype value","constructor":"first constructor value","prototype":"first ordinary value","toString":"first string value"},{"__proto__":"second prototype value","constructor":"second constructor value","prototype":"second ordinary value","toString":"second string value"}]}');
 const result=fixture(schema,source);accepted(result,source);
 for(const record of result.projected.records) {
  assert.equal(Object.hasOwn(record,'__proto__'),true);
  assert.equal(Object.getPrototypeOf(record),Object.prototype,'serialized records remain ordinary JSON data');
 }
 assert.equal(Object.prototype.polluted,undefined);
});
test('declared reserved object members cannot mutate prototypes or erase distinct nested keys',()=>{
 const source=JSON.parse('{"__proto__":{"polluted":"retained as data","constructor":"nested constructor"},"constructor":"outer constructor"}');
 const schema=JSON.parse('{"type":"object","properties":{"__proto__":{"type":"object","properties":{"polluted":{"type":"string"},"constructor":{"type":"string"}},"required":["polluted","constructor"],"additionalProperties":false},"constructor":{"type":"string"}},"required":["__proto__","constructor"],"additionalProperties":false}');
 accepted(fixture(schema,source),source);
 assert.equal(Object.prototype.polluted,undefined);
 const invalid=JSON.parse('{"__proto__":{"polluted":9,"constructor":"nested constructor"},"constructor":"outer constructor"}');
 const result=fixture(schema,invalid);assert.deepEqual(result.projected,invalid);refused(result,schema);
 assert.equal(Object.prototype.polluted,undefined);
});


test('required shell description is grounded in the exact user prompt on both wire surfaces',()=>{
 const schema={type:'object',properties:{command:{type:'string'},description:{type:'string'}},required:['command','description'],additionalProperties:false};
 accepted(fixture(schema,{cmd:'printf verified'},'run_shell_command'),{command:'printf verified',description:'explicit user task'});
 accepted(fixture(schema,{command:'printf verified',description:'caller supplied detail'},'run_shell_command'),{command:'printf verified',description:'caller supplied detail'});
 for(const description of [null,7,'']) {
  const result=fixture(schema,{command:'printf verified',description},'run_shell_command');
  assert.equal(result.projected.description,description);refused(result,schema);
 }
});
test('declared description defaults and enum restrictions retain precedence over prompt metadata',()=>{
 const base={type:'object',properties:{command:{type:'string'},description:{type:'string',default:'advertised default'}},required:['command','description'],additionalProperties:false};
 accepted(fixture(base,{command:'printf verified'},'run_shell_command'),{command:'printf verified',description:'advertised default'});
 accepted(fixture(base,{command:'printf verified',description:'explicit description'},'run_shell_command'),{command:'printf verified',description:'explicit description'});
 const restricted={...base,properties:{...base.properties,description:{type:'string',enum:['first operation','second operation']}}};
 const unbound=fixture(restricted,{command:'printf verified'},'run_shell_command');
 assert.equal(Object.hasOwn(unbound.projected,'description'),false);refused(unbound,restricted);
 accepted(fixture(restricted,{command:'printf verified',description:'second operation'},'run_shell_command'),{command:'printf verified',description:'second operation'});
 const explicitInvalid=fixture(restricted,{command:'printf verified',description:'outside enum'},'run_shell_command');
 assert.equal(explicitInvalid.projected.description,'outside enum');refused(explicitInvalid,restricted);
});


test('the original held-out unmade visibility choice refuses while explicit defaults and valid choices remain executable',async()=>{
 const prompt='write backoff=250ms to notes/scheduler.md file';
 const parameters={type:'object',required:['path','content','visibility'],properties:{path:{type:'string'},content:{type:'string'},visibility:{type:'string',enum:['public','internal','private']}}};
 const tools=[{type:'function',function:{name:'local_file_write',parameters}}];
 const messages=[{role:'user',content:prompt}],plan=await planChatStep(messages,['local_file_write']);
 assert.equal(plan.kind,'tool_calls');
 const projected=JSON.parse(responseArgumentsForTool(tools,'local_file_write',plan.calls[0].arguments,prompt,null));
 assert.equal(Object.hasOwn(projected,'visibility'),false,'the request never makes this choice');
 const request={messages,tools};
 const chat=chatCompletionFromPlan({},request,prompt,plan,0),responses=responseFromPlan({},request,request,prompt,plan,0);
 assert.deepEqual(chat.choices[0].message.tool_calls,[]);assert.equal(chat.choices[0].finish_reason,'stop');
 assert.match(chat.choices[0].message.content,/No call was sent.*incomplete/);
 assert.ok(!responses.output.some(row=>row.type==='function_call'));
 assert.ok(!JSON.stringify(chat).includes('"visibility":'));
 const withDefault={...parameters,properties:{...parameters.properties,visibility:{...parameters.properties.visibility,default:'internal'}}};
 accepted(fixture(withDefault,{path:'notes/scheduler.md',content:'backoff=250ms'},'local_file_write'),{path:'notes/scheduler.md',content:'backoff=250ms',visibility:'internal'});
 accepted(fixture(parameters,{path:'notes/scheduler.md',content:'backoff=250ms',visibility:'private'},'local_file_write'),{path:'notes/scheduler.md',content:'backoff=250ms',visibility:'private'});
});
