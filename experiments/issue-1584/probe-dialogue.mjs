// Probe controlled dialogue and graph projections directly against the crate module.
// Usage: node experiments/issue-1584/probe-dialogue.mjs ["prompt" ...]
import {installHost} from '../../js/agentic/host.mjs';
import {parseLino, readRepoFile} from '../../js/server/lino.mjs';
import {cnlPolicy, controlledDialogue, formalizeOutput, outputLinksNotation, projectOutput, validateOutput}
  from '../../js/agentic/crate/cnl_output.mjs';

installHost({parseLino, readText: readRepoFile, realm: {}});
const policy = cnlPolicy();
const prompts = process.argv.slice(2).length ? process.argv.slice(2) : [
  'Is a cat an animal?', 'is a cat a animal', 'um so like is a cat an anmal?', 'cat ... animal?',
  'Is a dog a plant?', 'Is a mouse an animal?', 'Is a mouse a part of a computer?', 'leaf ... tree?',
  'Does an account need a password?', 'Whats 2 plus 3?', 'add 2 and 3', 'cat animal', 'I know everything.',
];
const configs = [{}, {lexical_density: 'max-precision'}, {lexical_density: 'primes'}, {output_standard: 'basic-english'},
  {output_standard: 'gellish'}, {output_standard: 'sbvr'}, {output_standard: 'e-prime'}, {lexical_density: 'top-100'}];
for (const prompt of prompts) {
  for (const config of configs) {
    const t0 = performance.now();
    const turn = controlledDialogue(prompt, [], config, policy);
    const ms = (performance.now() - t0).toFixed(1);
    console.log(JSON.stringify({prompt, config, ms, intent: turn?.intent, prompt2: turn?.prompt,
      content: turn?.content, report: turn?.outputProjection?.report && {
        compliant: turn.outputProjection.report.compliant, v: turn.outputProjection.report.violations,
        u: turn.outputProjection.report.unchecked}}));
  }
}
const doc = formalizeOutput('Yes. Every cat is an animal.', 'en', policy);
console.log(outputLinksNotation(doc, policy));
for (const config of configs) console.log(JSON.stringify(config), JSON.stringify(projectOutput(doc, config, policy)),
  JSON.stringify(validateOutput(projectOutput(doc, config, policy), 'en', config, policy)));
