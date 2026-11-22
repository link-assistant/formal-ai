import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {Parser} from 'links-notation';
import {parseLino} from '../../../js/server/lino.mjs';

const read=path=>readFileSync(new URL('../../../'+path,import.meta.url),'utf8');

test('client integration seed parses with the native canonical notation grammar',()=>{
  assert.doesNotThrow(()=>new Parser().parse(read('data/seed/client-integrations.lino').trim()));
});

test('Qwen provider configuration retains its complete JSON value and correlation header',()=>{
  const nodes=[];
  const visit=node=>{nodes.push(node);for(const child of node.children??[])visit(child);};
  visit(parseLino(read('data/seed/client-integrations.lino')));
  const settings=nodes.filter(node=>node.name==='temp_home_json_set'&&node.value?.startsWith('modelProviders.openai=json:'));
  assert.equal(settings.length,1);
  const json=settings[0].value.slice('modelProviders.openai=json:'.length)
    .replaceAll('{context_window_tokens}','32768').replaceAll('{max_output_tokens}','8192');
  assert.deepEqual(JSON.parse(json),[{
    id:'{model}',baseUrl:'{endpoint_base_url}',envKey:'OPENAI_API_KEY',
    generationConfig:{contextWindowSize:32768,maxTokens:8192,
      customHeaders:{'x-formal-ai-dialog-id':'${session_id}'}},
  }]);
  assert.equal(read('rust/embedded/data/seed/client-integrations.lino'),read('data/seed/client-integrations.lino'));
});
