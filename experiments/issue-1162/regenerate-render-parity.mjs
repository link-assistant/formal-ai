// Issue #1162: regenerate rust/tests/fixtures/issue-1168/workflow-render-parity.json
// after provenance notes became per-language. Rust asserts the same text, so the
// Rust unit test `issue_1168_workflow_render_parity` cross-checks this output.
import { writeFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { readText } from '../../js/agentic/host.mjs';
import { programContractAnswer } from '../../js/agentic/crate/coding_program_contract.mjs';
import { baselineVersionSet } from '../../js/agentic/crate/version_resolution.mjs';
import { renderWith } from '../../js/agentic/ci_workflow.mjs';

await installNodeHost(new WorkerHost());
const path = 'rust/tests/fixtures/issue-1168/workflow-render-parity.json';
const fixture = JSON.parse(readText(path));
const versions = baselineVersionSet();
for (const entry of fixture.cases) {
  entry.workflow = renderWith(programContractAnswer(entry.prompt).execution_recipe, versions);
  console.log(`== ${entry.id}\n${entry.workflow.split('\n').filter(line => line.startsWith('# ') && line.includes('version resolved')).join('\n')}`);
}
if (process.argv.includes('--write')) writeFileSync(path, `${JSON.stringify(fixture, null, 2)}\n`);
