// Issue #661 (R384 of issue #538): the browser worker warns when a requirement
// contradicts one an earlier user turn stated, before a contextual handler can
// act on it. JavaScript twin of rust/tests/unit/issue_661.rs over
// js/worker/formal_ai_worker_requirement_contradiction.js; both runtimes read
// the required and forbidden surfaces from data/seed/statement-audit-registry.lino.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { before, describe, it } from 'node:test';

import { WorkerHost, evaluate } from '../../../js/server/worker-host.mjs';

const read = (path) => readFileSync(new URL(`../../../${path}`, import.meta.url), 'utf8');

let host;
let context;
before(async () => {
  host = new WorkerHost();
  context = await host.boot();
});

/** The worker's answer to `second` after the user stated `first`. */
async function afterDirective(first, second) {
  const answer = await host.solve(first);
  return host.solve(second, [{ role: 'user', content: first }, { role: 'assistant', content: answer.content }]);
}

describe('opposing directives are flagged with both statements and a resolution', () => {
  it('English: the warning quotes both requirements, their weights, and the retraction protocol, in the established language', async () => {
    // The first turn established Russian (issue #724), so the warning is the
    // Russian template, as issue_661.rs pins natively through `language:ru`.
    const answer = await afterDirective('always answer in Russian', 'never answer in Russian');
    assert.equal(answer.intent, 'requirement_contradiction');
    assert.equal(answer.content, [
      'Предупреждение: два ваших требования противоречат друг другу.',
      '- Утверждение 1 (вес 0.500000): «always answer in Russian»',
      '- Утверждение 2 (вес 0.500000): «never answer in Russian»',
      'Они предъявляют противоположные требования к одному предмету (answer in russian), поэтому оба не могут выполняться одновременно.',
      'Предлагаемое решение: оставьте одно требование и отзовите другое (retract) замещающим требованием. Сеть требований работает только на добавление, поэтому отзыв записывается новым событием без удаления истории. Либо разделите смыслы или ограничьте требования разными контекстами.',
    ].join('\n'));
    assert.ok(answer.evidence.some((link) => link.startsWith('requirement_contradiction:subject=answer in russian ')));
    assert.ok(answer.evidence.includes('policy:add_only_history'));
    assert.ok(answer.evidence.includes('trace:language:ru'));
  });

  it('an English pair with no established language is warned in English', async () => {
    const answer = await afterDirective('always use tabs', 'never use tabs');
    assert.equal(answer.intent, 'requirement_contradiction');
    assert.equal(answer.content.split('\n').slice(0, 3).join('\n'), [
      'Warning: two of your requirements contradict each other.',
      '- Statement 1 (weight 0.500000): “always use tabs”',
      '- Statement 2 (weight 0.500000): “never use tabs”',
    ].join('\n'));
  });

  it('Russian directives produce the Russian warning', async () => {
    const answer = await afterDirective('всегда отвечай на русском', 'никогда не отвечай на русском');
    assert.equal(answer.intent, 'requirement_contradiction');
    assert.equal(answer.content.split('\n')[0], 'Предупреждение: два ваших требования противоречат друг другу.');
    assert.equal(answer.content.split('\n')[1], '- Утверждение 1 (вес 0.500000): «всегда отвечай на русском»');
    assert.equal(answer.content.split('\n')[2], '- Утверждение 2 (вес 0.500000): «никогда не отвечай на русском»');
  });

  it('Hindi and Chinese directives produce seed-backed warnings with both weights', async () => {
    for (const [first, second, heading] of [
      ['हमेशा हिंदी में उत्तर दें', 'कभी नहीं हिंदी में उत्तर दें', 'चेतावनी: आपकी दो आवश्यकताएँ एक-दूसरे का खंडन करती हैं।'],
      ['始终用中文回答', '永远不要用中文回答', '警告：您的两条要求相互矛盾。'],
    ]) {
      const answer = await afterDirective(first, second);
      assert.equal(answer.intent, 'requirement_contradiction', second);
      assert.equal(answer.content.split('\n')[0], heading);
      assert.ok(answer.content.includes(first) && answer.content.includes(second));
      assert.equal(answer.content.split('0.500000').length - 1, 2);
    }
  });

  it('a repeated directive, or one with no earlier opposite, is not a contradiction', async () => {
    assert.notEqual((await afterDirective('always answer in Russian', 'always answer in Russian')).intent, 'requirement_contradiction');
    assert.notEqual((await afterDirective('Hello.', 'never answer in Russian')).intent, 'requirement_contradiction');
  });
});

describe('the claim reader (statement_audit/extract.rs)', () => {
  const claim = (text) => JSON.parse(evaluate(context, `JSON.stringify(contradictionClaim(${JSON.stringify(text)}, contradictionRegistry()))`));

  it('forbidden surfaces win over the required surface they extend, and trailing punctuation is trimmed', () => {
    assert.deepEqual(claim('must not log secrets.'), { subject: 'log secrets', value: 'forbidden' });
    assert.deepEqual(claim('Must log every request;'), { subject: 'log every request', value: 'required' });
    assert.deepEqual(claim("Don't answer in English"), { subject: 'answer in english', value: 'forbidden' });
  });

  it('an ASCII surface only matches as a whole word, and a bare surface is no claim', () => {
    assert.equal(claim('mustard is yellow'), null);
    assert.equal(claim('always'), null);
    assert.deepEqual(claim('永远不要用中文回答'), { subject: '用中文回答', value: 'forbidden' });
  });

  it('three statements on one subject share the mass; temperature zero gives it all to the last', () => {
    const softmax = (expression) => JSON.parse(evaluate(context, `JSON.stringify(${expression})`));
    assert.deepEqual(softmax('contradictionSoftmax([0.6, 0.6, 0.6], 0.7).map((weight) => weight.toFixed(6))'), ['0.333333', '0.333333', '0.333333']);
    assert.deepEqual(softmax('contradictionSoftmax([0.6, 0.6], 1.1920929e-7)'), [0, 1]);
  });

  it('the Rust audit reads the same registry file the worker fetches', () => {
    assert.ok(read('rust/src/statement_audit/extract.rs').includes('include_str!("../../embedded/data/seed/statement-audit-registry.lino")'));
    assert.equal(read('rust/embedded/data/seed/statement-audit-registry.lino'), read('data/seed/statement-audit-registry.lino'));
  });
});

describe('a forbidden language is neither demonstrated nor established (issue #724, issue_724_response_language_binding.rs)', () => {
  const established = (history) => JSON.parse(evaluate(context, `JSON.stringify(establishedResponseLanguage(${JSON.stringify(history)}))`));
  const forbidden = (text) => evaluate(context, `responseLanguageForbidden(${JSON.stringify(text)})`);

  it('"Never answer in Russian." alone is not answered in Russian', async () => {
    const answer = await host.solve('Never answer in Russian.');
    assert.notEqual(answer.intent, 'response_language_demonstration');
    assert.ok(!answer.evidence.includes('language_to:ru'), answer.evidence.join(' '));
    assert.ok(answer.evidence.includes('trace:language:en'));
  });

  it('only the sentence that names the language decides', () => {
    assert.equal(forbidden('never answer in Russian'), true);
    assert.equal(forbidden('Никогда не отвечай на русском'), true);
    assert.equal(forbidden('Never use slang. Answer in Russian.'), false);
    assert.equal(forbidden('answer in Russian'), false);
  });

  it('the latest user turn naming a language establishes it, unless it forbids it', async () => {
    const say = { role: 'user', content: 'Say something to me in Russian.' };
    const reply = { role: 'assistant', content: 'Здравствуйте! Чем могу помочь?' };
    const never = { role: 'user', content: 'Never answer in Russian.' };
    assert.equal(established([say, reply]), 'ru');
    assert.equal(established([say, reply, never, { role: 'assistant', content: 'Understood.' }]), null);
    assert.equal(established([{ role: 'assistant', content: 'answer in Russian' }]), null);
    const bound = await host.solve('What is an isogram?', [say, reply]);
    assert.ok(bound.evidence.includes('trace:language:ru'), bound.evidence.join(' '));
    const unbound = await host.solve('What is an isogram?', [say, reply, never, { role: 'assistant', content: 'Understood.' }]);
    assert.ok(unbound.evidence.includes('trace:language:en'), unbound.evidence.join(' '));
  });

  it('a conversation bound to Russian greets in Russian', async () => {
    const answer = await host.solve('hello', [{ role: 'user', content: 'always answer in Russian' }, { role: 'assistant', content: 'Хорошо.' }]);
    assert.equal(answer.intent, 'greeting');
    assert.equal(answer.content, 'Здравствуйте! Чем могу помочь?');
  });
});
