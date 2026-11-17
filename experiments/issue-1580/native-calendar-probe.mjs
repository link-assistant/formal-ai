import {createWorkerContext, evaluate} from '../../rust/tests/web/support/browser-runtime.mjs';
const worker = createWorkerContext();
await evaluate(worker, 'loadSeed()');
const prompts = [
  'Назначь мне встречу с Александром на 20:00 по Грузии',
  'Запланируй встречу с Марией на 21:45 по Тбилиси',
  'Schedule a meeting with Alexander at 20:00 Georgia time',
  'Назначь мне встречу с Александром завтра в 20:00 по Грузии',
];
for (const prompt of prompts) {
  const lower = prompt.toLowerCase();
  const normalized = evaluate(worker, 'normalizePrompt(' + JSON.stringify(lower) + ')');
  const span = evaluate(worker, 'calendarCreateRequestSpan(' + JSON.stringify(lower) + ')');
  const clock = evaluate(worker, 'extractClockTime(' + JSON.stringify(normalized) + ')');
  const solved = await worker.solve(prompt, [], {}, {}, [], {});
  console.log(JSON.stringify({prompt, normalized, span, clock, intent:solved.intent,
    content:solved.content, evidence:solved.evidence.filter(row => /calendar:|capability/.test(row))}));
}
