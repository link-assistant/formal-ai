// Issue #1571 A6: how Progress judges the driver's failed read_file versus the Agent CLI "File not found" read.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const { Progress } = await import('../../js/agentic/progress.mjs');
const path = 'data/benchmarks/industry-suite.lino';
const args = JSON.stringify({ path, filePath: path, file_path: path });
const variants = {
  driver: { content: 'No such file or directory (os error 2)', is_error: true, source_read: { path, success: false, complete: false, format: 'raw', error_code: 'ENOENT' } },
  agentCli: { content: `Error: File not found: /tmp/x/${path}` },
};
for (const [name, extra] of Object.entries(variants)) {
  const messages = [{ role: 'user', content: 'x' }, { role: 'assistant', content: '', tool_calls: [{ id: 'r', type: 'function', function: { name: 'read_file', arguments: args } }] }, { role: 'tool', tool_call_id: 'r', name: 'read_file', ...extra }];
  const progress = Progress.scan(messages);
  const failure = progress.latestFailure?.() ?? progress.latest_failure?.();
  console.log(name, 'latestFailure:', failure ? failure.detail : null, 'sourceRead:', JSON.stringify(progress.attempts.at(-1).source_read));
}
process.exit(0);
