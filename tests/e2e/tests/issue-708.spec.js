// @ts-check
const { test, expect } = require('@playwright/test');

const PREF_KEY = 'formal-ai.preferences.v1';

async function boot(page) {
  await page.addInitScript(({ key }) => {
    window.localStorage.setItem(
      key,
      [
        'demo_preferences',
        '  demoMode "off"',
        '  greetingVariations "off"',
        '  diagnosticsMode "on"',
        '  uiLanguage "en"',
      ].join('\n'),
    );
  }, { key: PREF_KEY });
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled();
  await page.evaluate(() => window.FormalAiMemory.clearEvents());
}

async function appendEvent(page, event) {
  await page.evaluate(
    (value) => window.FormalAiMemory.appendEvent(value),
    { sentAt: new Date().toISOString(), ...event },
  );
}

async function sendPrompt(page, prompt) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  const answers = page.locator('[data-testid="chat-message"].assistant');
  const before = await answers.count();
  await input.fill(prompt);
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect.poll(() => answers.count(), { timeout: 20_000 }).toBeGreaterThan(before);
  await expect(input).toBeEnabled({ timeout: 20_000 });
  return answers.last();
}

async function lastCompiledProgramId(page) {
  return page.evaluate(async () => {
    const events = await window.FormalAiMemory.listEvents();
    const answers = events
      .filter((event) => event && String(event.intent || '').startsWith('memory_program'))
      .reverse();
    for (const answer of answers) {
      for (const link of answer.evidence || []) {
        const id = /memory_program_[0-9a-f]{16}/.exec(String(link));
        if (id) return id[0];
      }
    }
    return null;
  });
}

test('seeded program renames selected browser facts and compiles identically across languages', async ({
  page,
}) => {
  await boot(page);
  await appendEvent(page, {
    kind: 'fact', role: 'user', content: 'X powers the engine', intent: 'engines',
  });
  await appendEvent(page, {
    kind: 'fact', role: 'assistant', content: 'X is only a draft', intent: 'engines',
  });

  const prompts = [
    {
      language: 'en',
      prompt: 'List every fact I contributed about X and rename X to Y in all of them.',
    },
    {
      language: 'ru',
      prompt: 'Перечисли все факты, которые я добавил о X, и переименуй X в Y во всех них.',
    },
    {
      language: 'hi',
      prompt: 'X के बारे में मेरे जोड़े हर तथ्य को सूचीबद्ध करो और उन सभी में X का नाम Y कर दो।',
    },
    {
      language: 'zh',
      prompt: '列出我贡献的关于 X 的每个事实，并在所有事实中将 X 重命名为 Y。',
    },
    {
      language: 'es',
      prompt: 'Enumera todos los hechos que aporté sobre X y cambia X por Y en todos ellos.',
    },
  ];
  const programIds = [];
  for (const { prompt } of prompts) {
    const answer = await sendPrompt(page, prompt);
    await expect(answer).toContainText(/memory_program_[0-9a-f]{16}/);
    programIds.push(await lastCompiledProgramId(page));
  }

  expect(new Set(programIds).size).toBe(1);
  expect(programIds[0]).toMatch(/^memory_program_[0-9a-f]{16}$/);
  const facts = await page.evaluate(async () =>
    (await window.FormalAiMemory.listEvents())
      .filter((event) => event.kind === 'fact')
      .map((event) => ({ role: event.role, content: event.content })),
  );
  expect(facts).toEqual([
    { role: 'user', content: 'Y powers the engine' },
    { role: 'assistant', content: 'X is only a draft' },
  ]);
});

test('browser programs store summaries, refuse destructive effects, and name gaps', async ({
  page,
}) => {
  await boot(page);
  await appendEvent(page, {
    kind: 'event', role: 'user', content: 'first', intent: 'engines',
  });
  await appendEvent(page, {
    kind: 'event', role: 'user', content: 'second', intent: 'engines',
  });
  await appendEvent(page, {
    kind: 'fact', role: 'user', content: 'engines need fuel', intent: 'engines',
  });

  await sendPrompt(page, 'Count events per topic this week and store the summary.');
  await expect.poll(async () => page.evaluate(async () => {
    const events = await window.FormalAiMemory.listEvents();
    return events.some(
      (event) => event.kind === 'topic_summary' && event.content === 'engines=2',
    );
  })).toBe(true);

  const refused = await sendPrompt(
    page,
    'Delete every fact I contributed about engines.',
  );
  await expect(refused).toContainText('require explicit human confirmation');
  expect(await page.evaluate(async () =>
    (await window.FormalAiMemory.listEvents())
      .filter((event) => event.kind === 'memory_retraction').length,
  )).toBe(0);

  const gap = await sendPrompt(page, 'Transpose every fact matrix in memory.');
  await expect(gap).toContainText('program_gap');
});
