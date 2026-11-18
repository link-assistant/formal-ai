// Supplied investigation for issue #1580; no independent authorship credit.
import { WorkerHost } from '../../js/server/worker-host.mjs';
const host = new WorkerHost();
const prompts = [
  'Hi, what is Redis?',
  '你好, Redis是什么？',
  'I have a clarinet, a violin, and a flute. How many musical instruments do I have?',
  "Marisol's hens lay 24 eggs per day. She eats five for breakfast every morning and bakes cakes for her neighbors every day with seven. She sells the remainder at the market daily for $3 per fresh egg. How much in dollars does she make every day at the market?",
];
for (const prompt of prompts) {
  const response = await host.solve(prompt);
  process.stdout.write(JSON.stringify({ prompt, response }) + '\n');
}
