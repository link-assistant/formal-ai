// Print the meta-executor trace of a request over a mock host that answers
// every atomic task and fails every composite one, so the composition wins.
// Usage: node experiments/issue-1550/trace.mjs "<request>" <language> [withheld-task-id]
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const { executeProcedure, decompositionShape } = await import('../../js/agentic/crate/meta_executor.mjs');
const [text, language = 'en', withheld] = process.argv.slice(2);
const host = {
  draft: async (task, node) => (node.children.length
    ? { intent: 'unknown', content: '', confidence: 0 }
    : { intent: 'answered', content: `answer(${task})`, confidence: 1 }),
};
const run = await executeProcedure({ text, language }, host, { withheldTests: withheld ? [withheld] : [] });
console.log(decompositionShape(run.root));
for (const event of run.events) console.log(event.kind, event.payload);
console.log('ANSWER', JSON.stringify(run.outcome.candidate?.content ?? null));
