import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {createMemory} from '../../../js/server/memory.mjs';
import {ensureNodeHost} from '../../../js/server/node-host-install.mjs';
import {createChatCompletion} from '../../../js/server/openai.mjs';
import {chatPromptAndHistory, parseChatRequest} from '../../../js/server/chat-request.mjs';
import {parseResponsesRequest, toChatCompletionRequest} from '../../../js/server/responses-input.mjs';
import {parseAnthropicRequest, anthropicToChatCompletionRequest} from '../../../js/server/anthropic.mjs';
import {predictionHitEvent} from '../../../js/server/anticipation.mjs';

test('ordinary coding history does not invoke a prediction classifier', () => {
  const body=fs.readFileSync(new URL('../fixtures/released-client-requests/agent.json',import.meta.url),'utf8');
  const request=parseChatRequest(body);
  const {prompt}=chatPromptAndHistory(request.messages);
  const unexpectedClassifier=()=>{throw new Error('no prediction exists to classify');};
  for(const events of [[],[{id:'ordinary-history',kind:'message',content:prompt}]]) {
    assert.equal(predictionHitEvent(events,prompt,'actual-request',unexpectedClassifier),null);
  }
});

test('captured released-client requests produce the grounded first step without tool effects', {timeout:30000}, async () => {
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-released-request-'));
  try {
    const ctx={worker:new WorkerHost(),agentMode:true,memory:createMemory({FORMAL_AI_MEMORY_PATH:path.join(directory,'memory.lino')})};
    await ensureNodeHost(ctx);
    for(const [client,tool,issue] of [['agent','bash','1411'],['codex','exec_command','1413'],['claude','Bash','1415']]) {
      const body=fs.readFileSync(new URL('../fixtures/released-client-requests/'+client+'.json',import.meta.url),'utf8');
      const request=client==='codex'?toChatCompletionRequest(parseResponsesRequest(body))
        :client==='claude'?anthropicToChatCompletionRequest(parseAnthropicRequest(body)):parseChatRequest(body);
      const started=performance.now();
      const completion=await createChatCompletion(ctx,request);
      const calls=completion.choices[0].message.tool_calls;
      if(client==='codex') {
        // Its public export removed the properties object (the schema declares
        // a shell "login" option). Preserve that loss and refuse the call.
        assert.equal(calls.length,0);
        assert.match(completion.choices[0].message.content,/required tool arguments could not be bound/u);
      } else {
        assert.equal(calls.length,1,client);
        assert.equal(calls[0].function.name,tool);
        assert(calls[0].function.arguments.includes('https://github.com/link-assistant/formal-ai/issues/'+issue));
      }
      assert(performance.now()-started<15000,client+' exceeded its pure-planning latency budget');
    }
  } finally {fs.rmSync(directory,{recursive:true,force:true});}
});
