import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {createWorkerContext, evaluate} from './support/browser-runtime.mjs';

const fixtures = JSON.parse(readFileSync(
  new URL('../fixtures/issue-1580-calendar-clock.json', import.meta.url), 'utf8'));
const worker = createWorkerContext();
await evaluate(worker, 'loadSeed()');

for (const row of fixtures.events) {
  test('calendar preserves the clock in ' + row.prompt, {timeout:15000}, async () => {
    const answer = await worker.solve(row.prompt, [], {}, {}, [], {});
    assert.equal(answer.intent, 'calendar_create_event', JSON.stringify(answer));
    assert.ok(answer.evidence.includes('calendar:parsed_time:' + row.time), JSON.stringify(answer));
    assert.ok(answer.evidence.includes('calendar:parsed_time_zone:' + row.zone), JSON.stringify(answer));
    const compact = row.time.replace(':', '') + '00';
    assert.match(answer.content, new RegExp('DTSTART[^\\r\\n]*T' + compact));
    assert.ok(answer.content.includes(row.time), answer.content);
    if (row.participant) assert.ok(answer.content.toLowerCase().includes(row.participant), answer.content);
  });
}

for (const prompt of fixtures.documents) {
  test('clock text cannot authorize a calendar action: ' + prompt, {timeout:15000}, async () => {
    assert.equal(evaluate(worker, 'calendarCreateRequestSpan(' + JSON.stringify(prompt) + ')'), null);
    const answer = await worker.solve(prompt, [], {}, {}, [], {});
    assert.notEqual(answer.intent, 'calendar_create_event', JSON.stringify(answer));
    assert.doesNotMatch(answer.content, /BEGIN:VEVENT/);
  });
}


test('chat API preserves the requested clock and declines a background clock', {timeout:30000}, async () => {
  const {mkdtempSync, rmSync} = await import('node:fs');
  const {tmpdir} = await import('node:os');
  const {join} = await import('node:path');
  const {startServer} = await import('../../../js/server/main.mjs');
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-calendar-clock-'));
  const env = {...process.env, FORMAL_AI_API_BEARER_TOKEN:'calendar-clock-control',
    FORMAL_AI_MEMORY_PATH:join(directory, 'memory.lino'),
    FORMAL_AI_CACHE_DIR:join(directory, 'cache'),
    FORMAL_AI_DIALOG_LOG_DIR:join(directory, 'dialogs'), FORMAL_AI_RECORD_CHAT:'0',
    FORMAL_AI_LIVE_FETCH:'0', FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR:'false'};
  let running;
  try {
    running = await startServer({host:'127.0.0.1', port:0, env});
    for (const row of [fixtures.events[0], fixtures.events[1], {prompt:fixtures.documents[1]}]) {
      const response = await fetch(running.url + '/v1/chat/completions', {
        method:'POST', headers:{authorization:'Bearer calendar-clock-control', 'content-type':'application/json'},
        body:JSON.stringify({model:'formal-ai', messages:[{role:'user', content:row.prompt}]}),
        signal:AbortSignal.timeout(10000),
      });
      assert.equal(response.status, 200);
      const body = await response.json();
      const answer = body.choices[0].message.content;
      if (row.time) {
        assert.ok(answer.includes(row.time), answer);
        assert.ok(answer.includes(row.zone), answer);
        assert.ok(answer.toLowerCase().includes(row.participant), answer);
        assert.match(answer, new RegExp('DTSTART[^\\r\\n]*T' + row.time.replace(':', '') + '00'));
      } else {
        assert.doesNotMatch(answer, /BEGIN:VEVENT/);
      }
    }
  } finally {
    if (running) await new Promise(resolve => running.server.close(resolve));
    rmSync(directory, {recursive:true, force:true});
  }
});
