// Issue #1562: which program roles each fixture's whole text and its prose evidence.
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';

await ensureNodeHost({ worker: new WorkerHost() }, { sourceUsageRustGrammar: false });
import { mentionsRole } from '../../js/agentic/crate/seed_meanings.mjs';
import { proseOf } from '../../js/agentic/crate/language.mjs';
import { carriesAuthoringTask } from '../../js/agentic/shell_command.mjs';

const roles = ['program_request', 'program_kind', 'hello_world_reference', 'program_language'];
for (const file of process.argv.slice(2)) {
  const text = readFileSync(file, 'utf8');
  const prose = proseOf(text);
  const of = body => Object.fromEntries(roles.map(role => [role, mentionsRole(role, body)]));
  console.log(file, { whole: of(text), prose: of(prose), authoringWhole: carriesAuthoringTask(text), authoringProse: carriesAuthoringTask(prose) });
}
process.exit(0);
