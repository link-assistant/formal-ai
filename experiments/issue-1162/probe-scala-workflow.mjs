// Probe for the R1159-4 JavaScript pin: the Scala workflow's compiler pin and
// the scalac publisher record name one release feed.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { baselineVersionSet } from '../../js/agentic/crate/version_resolution.mjs';
import { renderWith } from '../../js/agentic/ci_workflow.mjs';
import { programContractAnswer } from '../../js/agentic/crate/coding_program_contract.mjs';
import { readText } from '../../js/agentic/host.mjs';
import { parseRoot, findChildValue } from '../../js/agentic/crate/seed_parser.mjs';

await installNodeHost(new WorkerHost());
const versions = baselineVersionSet();
console.log(versions.scala);
const publisher = parseRoot(readText('data/seed/setup-publishers.lino')).children.find(node => findChildValue(node, 'program') === 'scalac');
console.log(findChildValue(publisher, 'documentation'));
const recipe = programContractAnswer('Write a Scala program that prints "Hello, World!" to stdout.').execution_recipe;
console.log(recipe.commands);
console.log(renderWith(recipe, versions));
