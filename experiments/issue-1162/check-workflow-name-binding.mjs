// Issue #1162: print the parsed workflow-name binding patterns and the
// binding of sample lines. Usage: node experiments/issue-1162/check-workflow-name-binding.mjs
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { fill } from '../../js/agentic/work_item_steps.mjs';
import { requestTargetBinding, validWorkflowTarget } from '../../js/agentic/git_commit.mjs';

await installNodeHost(new WorkerHost());
console.log(JSON.parse(fill('workflow-name-binding-patterns', [])));
const valid = value => !value.includes('/') && validWorkflowTarget(value);
for (const line of [
  '- The workflow should have a meaningful name like `test-hello-world.yml`',
  'The workflow should have a meaningful name like test-hello-world.yml.',
  'Name the workflow something like hello.yml',
  '- The workflow should have a meaningful name like `ci/x.yml`',
  'Рабочий процесс должен иметь осмысленное имя, например `privet.yml`',
  '> The workflow should have a meaningful name like `quoted.yml`',
]) console.log(JSON.stringify(line), requestTargetBinding(line, 'workflow-name-binding-patterns', valid));
