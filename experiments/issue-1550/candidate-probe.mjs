// Issue #1550: print the candidate, validation and simplification events a
// worker turn carries, to check that their payloads are derived from the turn.
// node experiments/issue-1550/candidate-probe.mjs "Suggest a name for my project"
import { WorkerHost } from '../../js/server/worker-host.mjs';

const host = new WorkerHost();
const prompts = process.argv.slice(2);
for (const prompt of prompts.length ? prompts : ['Suggest a name for my project', 'Pick a prime number between 14 and 18', 'Hi']) {
  const answer = await host.solve(prompt);
  const events = (answer.rawSolverEvents || answer.solverEvents || [])
    .filter((event) => ['candidate', 'validation', 'trace:simplification'].includes(event.kind));
  console.log(JSON.stringify({ prompt, intent: answer.intent, content: String(answer.content).slice(0, 160),
    events: events.map((event) => `${event.kind} ${event.payload}`) }));
}
process.exit(0);
