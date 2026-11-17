import assert from 'node:assert/strict';
import {readFileSync, mkdtempSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {createWorkerContext, evaluate} from './support/browser-runtime.mjs';

const worker = createWorkerContext();
await evaluate(worker, 'loadSeed()');
const rows = JSON.parse(readFileSync(new URL('../../../data/parity/issue-1589-calendar-currency.json', import.meta.url), 'utf8'));
for (const row of rows) test(row.prompt, {timeout:15000}, async () => {
  const answer = await worker.solve(row.prompt, [], {}, {}, [], {});
  if (row.reject) return assert.notEqual(answer.intent, row.reject, JSON.stringify(answer));
  assert.equal(answer.intent, row.intent, JSON.stringify(answer));
  assert.equal(answer.content, row.answer, row.prompt);
  assert.ok(answer.content.includes(row.text), answer.content);
  assert.ok(answer.evidence.includes(row.evidence), JSON.stringify(answer));
  if (row.evidence.startsWith('currency:')) {
    for (const text of ['2026-11-17', 'https://www.cbr.ru/']) {
      assert.ok(answer.content.includes(text), answer.content);
    }
  }
});
test('fresh-home API exposes the answer and an actual derivation for every requirement', {timeout:60000}, async () => {
  const {startServer} = await import('../../../js/server/main.mjs');
  const home = mkdtempSync(join(tmpdir(), 'formal-ai-issue1589-'));
  const env = {...process.env, HOME:home, FORMAL_AI_API_BEARER_TOKEN:'issue1589',
    FORMAL_AI_MEMORY_PATH:join(home,'memory.lino'), FORMAL_AI_CACHE_DIR:join(home,'cache'),
    FORMAL_AI_DIALOG_LOG_DIR:join(home,'dialogs'), FORMAL_AI_RECORD_CHAT:'0',
    FORMAL_AI_LIVE_FETCH:'0', FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR:'false'};
  let running;
  try {
    running = await startServer({host:'127.0.0.1', port:0, env});
    for (const row of [...rows.slice(0,6), rows[10], rows[11]]) {
      const response = await fetch(running.url+'/v1/chat/completions', {
        method:'POST', headers:{authorization:'Bearer issue1589','content-type':'application/json'},
        body:JSON.stringify({model:'formal-ai', messages:[{role:'user',content:row.prompt}]}),
        signal:AbortSignal.timeout(10000)});
      assert.equal(response.status,200);
      const message = (await response.json()).choices[0].message;
      assert.ok(message.content.includes(row.text), JSON.stringify(message));
      assert.ok(message.thinking_steps?.length, JSON.stringify(message));
      assert.ok(message.thinking_steps.some(step => String(step.source_event).includes("derivation") && /[=×]/u.test(step.detail)), JSON.stringify(message.thinking_steps));
    }
  } finally {
    if (running) await new Promise(resolve => running.server.close(resolve));
    rmSync(home, {recursive:true,force:true});
  }
});

test('English, Russian, Hindi, Chinese and Spanish duration responses retain their language', {timeout:15000}, async () => {
  for(const [prompt,expected] of [
    ['Add 14 months and 2 years and 7 months. In months.','45 months'],
    ['Сложи 14 месяцев и 2 года и 7 месяцев. В месяцах.','45 месяцев'],
    ['जोड़ो 14 महीने और 2 वर्ष और 7 महीने. में महीने.','45 महीने'],
    ['相加 14 个月 和 2 年 和 7 个月. 以 个月.','45 个月'],
    ['Suma 14 meses y 2 años y 7 meses. En meses.','45 meses'],
    ['Suma 2 meses y 3 meses. En meses.','5 meses']
  ]) {
    const answer=await worker.solve(prompt,[],{},{},[],{});
    assert.equal(answer.intent,'calendar-duration-sum',JSON.stringify(answer));
    assert.ok(answer.content.includes(expected),answer.content);
  }
});
