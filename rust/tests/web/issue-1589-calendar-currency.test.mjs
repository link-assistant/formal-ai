import assert from 'node:assert/strict';
import {readFileSync, readdirSync, mkdtempSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {createWorkerContext, evaluate} from './support/browser-runtime.mjs';
import {currencyExamples, currencyFetch, quoteDate} from './support/currency-publisher-captures.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';

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
});
test('fresh-home API exposes the answer and an actual derivation for every requirement', {timeout:60000}, async () => {
  const {startServer} = await import('../../../js/server/main.mjs');
  const home = mkdtempSync(join(tmpdir(), 'formal-ai-issue1589-'));
  const env = {...process.env, HOME:home, FORMAL_AI_API_BEARER_TOKEN:'issue1589',
    FORMAL_AI_MEMORY_PATH:join(home,'memory.lino'), FORMAL_AI_CACHE_DIR:join(home,'cache'),
    FORMAL_AI_DIALOG_LOG_DIR:join(home,'dialogs'), FORMAL_AI_RECORD_CHAT:'0',
    FORMAL_AI_LIVE_FETCH:'1', FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR:'false'};
  let running;
  try {
    const workerHost = new WorkerHost({}, { networkFetch: currencyFetch });
    running = await startServer({host:'127.0.0.1', port:0, env, worker:workerHost});
    assert.ok(running.ctx.worker === workerHost, 'the API uses the captured-publisher worker');
    const currencyRows = currencyExamples.map(([prompt, text]) => ({prompt, text}));
    for (const row of [...rows.slice(0,5), rows[9], rows[10], ...currencyRows]) {
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

// Replays main run 44188508650: calendar-duration-sum was defined as both
// a handler cue and a response intent; alias strings also used in-band pipes.
function duplicateMeaningSlugs(documents) {
  const owners = new Map();
  for (const [name, content] of documents) {
    content.split('\n').forEach((line, index) => {
      const header = /^ {2}([^\s#]+)(?:\s+#.*)?$/u.exec(line.trimEnd());
      if (!header || header[1] === 'meanings') return;
      const locations = owners.get(header[1]) || [];
      locations.push(`${name}:${index + 1}`);
      owners.set(header[1], locations);
    });
  }
  return [...owners].filter(([, locations]) => locations.length > 1);
}

test('calendar meanings stay globally unique across all seed meaning documents', () => {
  const seed = new URL('../../../data/seed/', import.meta.url);
  const documents = readdirSync(seed, {recursive:true})
    .filter(name => /(?:^|\/)meanings[^/]*\.lino$/u.test(name))
    .map(name => [name, readFileSync(new URL(name, seed), 'utf8')]);
  assert.ok(documents.length > 0, 'seed meaning documents were inspected');
  assert.deepEqual(duplicateMeaningSlugs(documents), []);
});

test('meaning uniqueness replay detects duplicates within and across documents', () => {
  const first = 'meanings\n  same\n    defined-by time\n  distinct\n';
  assert.deepEqual(duplicateMeaningSlugs([['first', first]]), []);
  assert.deepEqual(duplicateMeaningSlugs([['first', first + '  same\n']]),
    [['same', ['first:2', 'first:5']]]);
  assert.deepEqual(duplicateMeaningSlugs([['first', first], ['second', 'meanings\n  same # second definition\n']]),
    [['same', ['first:2', 'second:2']]]);
});

// Calendar/currency vocabulary rows hold individual linguistic surfaces.
// Provider configuration uses pipes for structured fields, including ISO lists.
function packedSurfaceAliases(tables) {
  return Object.entries(tables)
    .filter(([name]) => /^(?:calendar-|currency-conversion-)/u.test(name))
    .flatMap(([name, table]) => table.rows
      .filter(([, value]) => value.includes('|')).map(row => ({name, row})));
}

test('calendar and currency vocabulary rows carry one surface each without pipe-packed aliases', () => {
  assert.deepEqual(packedSurfaceAliases(evaluate(worker, 'handlerRulesTables()')), []);
});

test('surface replay catches packed aliases while allowing structured provider fields', () => {
  const calendar = {rows: [['en', 'add|sum']]};
  const currency = {rows: [['en', 'to|into']]};
  const provider = {rows: [['iso-codes', 'EUR|USD']]};
  assert.deepEqual(packedSurfaceAliases({'currency-quote-provider': provider}), []);
  assert.deepEqual(packedSurfaceAliases({'calendar-duration-addition-cue': calendar}),
    [{name: 'calendar-duration-addition-cue', row: calendar.rows[0]}]);
  assert.deepEqual(packedSurfaceAliases({'currency-conversion-connector': currency}),
    [{name: 'currency-conversion-connector', row: currency.rows[0]}]);
});

test('separate vocabulary rows retain alternative addition, interval and currency surfaces', {timeout:15000}, async () => {
  for (const [prompt, intent, expected] of [
    ['Sum 12 months + 1 year in years.', 'calendar-duration-sum', '2 years'],
    ['Суммируй 1 год и 2 месяца. В месяцах.', 'calendar-duration-sum', '14 месяцев'],
    ['Сложите 1 год и 2 месяца. Ответ в месяцах.', 'calendar-duration-sum', '14 месяцев'],
    ['How many whole years are from March 2021 until June 2024?', 'calendar-full-periods', '3 years'],
    ['How many complete months are from March 2021 to June 2024?', 'calendar-full-periods', '39 months'],
  ]) {
    const answer = await worker.solve(prompt, [], {}, {}, [], {});
    assert.equal(answer.intent, intent, JSON.stringify(answer));
    assert.ok(answer.content.includes(expected), answer.content);
  }
});
test('dated publisher captures support known and held-out pairs', {timeout: 15000}, async () => {
  const sourcedWorker = createWorkerContext({}, {networkFetch: currencyFetch});
  await evaluate(sourcedWorker, 'loadSeed()');
  for (const [prompt, result, pair, rate] of currencyExamples) {
    const answer = await sourcedWorker.solve(prompt, [], {}, {}, [], {});
    assert.equal(answer.intent, 'calculation', JSON.stringify(answer));
    assert.ok(answer.content.includes(`${rate} ${result.split(' ')[1]}`), answer.content);
    assert.ok(answer.content.includes(`/rate/${pair}?date=${quoteDate}`), answer.content);
    assert.ok(answer.evidence.includes(`currency:result:${result}`), JSON.stringify(answer));
    assert.ok(answer.evidence.some(value => /^source:sha256:[a-f0-9]{64}$/u.test(value)), JSON.stringify(answer));
  }
});

test('same-currency rounding does not need a publisher', async () => {
  for (const [prompt, expected] of [
    ['Calculate 1.005 USD in USD', '1.01 USD'],
    ['Calculate -1.005 USD in USD', '-1.01 USD'],
    ['Посчитай 0,004 рублей в рублях', '0.00 RUB'],
  ]) {
    const answer = await worker.solve(prompt, [], {offline: true}, {}, [], {});
    assert.equal(answer.intent, 'calculation', JSON.stringify(answer));
    assert.ok(answer.evidence.includes(`currency:result:${expected}`), JSON.stringify(answer));
    assert.equal(answer.evidence.some(value => value.startsWith('currency:source:')), false);
  }
});

test('a missing or invalid publisher quote does not yield a conversion', {timeout: 15000}, async () => {
  const prompt = `Calculate 1 EUR in USD on ${quoteDate}`;
  const missing = await worker.solve(prompt, [], {offline: true}, {}, [], {});
  assert.equal(missing.intent, 'calculation_error', JSON.stringify(missing));
  const invalidBodies = [
    {quote: 'JPY'}, {base: 'GBP'}, {date: '2026-10-10'}, {date: '2026-11-18'},
    {date: '2026-02-30'}, {rate: -0.51}, {rate: '1.23'},
  ];
  const networkFetch = url => {
    const day = Number(new URL(url).searchParams.get('date')?.slice(-2));
    return new Response(JSON.stringify({
      date: '2026-11-11', base: 'EUR', quote: 'USD', rate: 1.1006, ...invalidBodies[day - 11],
    }));
  };
  const invalidWorker = createWorkerContext({}, {networkFetch});
  await evaluate(invalidWorker, 'loadSeed()');
  for (let day = 11; day <= 17; day += 1) {
    const quote = await evaluate(invalidWorker, `readCurrencyQuote('EUR', 'USD', '2026-11-${day}')`);
    assert.equal(quote, null, `invalid quote ${day - 11}`);
  }
  const priorBusinessDay = await evaluate(invalidWorker, "readCurrencyQuote('EUR', 'USD', '2026-11-18')");
  assert.equal(priorBusinessDay.date, '2026-11-11');
});
