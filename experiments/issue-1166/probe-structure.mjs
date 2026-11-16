// Probe: what the #1487 surface structure reads from coding-condition clauses.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { surfaceStructure } from '../../js/agentic/crate/proposition_structure.mjs';
await installNodeHost(new WorkerHost());
for (const t of process.argv.slice(2).length ? process.argv.slice(2) : [
  'Print "yes" when a is less than b, otherwise print "no".',
  'Print "yes" when b is less than a, otherwise print "no".',
  'Print "yes" when a is greater than b.',
  'Print "yes" when a is less than or equal to b.',
  'Print "yes" when a < b.',
  'Print "yes" when a is not less than b.',
  'Print "yes" when a is less than b and b is less than c.',
  'Print "yes" when a is less than b or b is less than c.',
  'Print "yes" when a is less than b and c.',
  'Print "Hello" twice.',
  'Print "Hello" three times.',
  'Print "yes" unless a is less than b.',
  'Выведи "да", если a меньше b.',
]) console.log(JSON.stringify(t), '=>', JSON.stringify(surfaceStructure(t)));
