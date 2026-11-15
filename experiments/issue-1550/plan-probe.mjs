// Issue #1550 probe: the task tree (leaves and shape) the procedure plans for
// a prompt, i.e. the sub-impulses record_decomposition records.
//   node experiments/issue-1550/plan-probe.mjs <language> "prompt" ...
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';

await installNodeHost(new WorkerHost());
const { decompositionBounds, decompositionShape, planTasks, taskLeaves } = await import('../../js/agentic/crate/meta_executor.mjs');
const [language, ...prompts] = process.argv.slice(2);
for (const prompt of prompts) {
  const root = planTasks(prompt, language, decompositionBounds());
  console.log(`${decompositionShape(root)}\t${JSON.stringify(taskLeaves(root))}\t<- ${prompt}`);
}
