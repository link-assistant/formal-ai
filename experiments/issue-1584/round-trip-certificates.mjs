// Usage: node experiments/issue-1584/round-trip-certificates.mjs
// Prints every (language, profile, density) projection of the declared
// "omniscient" meaning that the JS reference engine certifies as compliant.
// It reproduces the loop of the Rust test
// declared_multilingual_meanings_round_trip_without_certifying_unchecked_standards.
import {installHost} from '../../js/agentic/host.mjs';
import {parseLino, readRepoFile} from '../../js/server/lino.mjs';
import {cnlPolicy, formalizeOutput, outputLinksNotation, projectOutput, validateOutput}
  from '../../js/agentic/crate/cnl_output.mjs';
installHost({parseLino, readText: readRepoFile, realm: {}});
const policy = cnlPolicy();
const dense = {en: 'I am omniscient.', ru: 'Я всеведущ.', hi: 'मैं सर्वज्ञ हूँ।', zh: '我无所不知。', es: 'Soy omnisciente.'};
let total = 0;
for (const [language, text] of Object.entries(dense)) {
  const document = formalizeOutput(text, language, policy);
  const graph = outputLinksNotation(document, policy);
  for (const profile of policy.profiles) {
    for (const density of ['standard', 'primes', 'top-37', 'max-precision']) {
      const configuration = {output_standard: profile.id, lexical_density: density};
      const projected = projectOutput(document, configuration, policy);
      const same = outputLinksNotation(formalizeOutput(projected, language, policy), policy) === graph;
      const report = validateOutput(projected, language, configuration, policy);
      total += 1;
      if (report.compliant || !same) console.log(JSON.stringify({language, profile: profile.id, density, projected, compliant: report.compliant, same}));
    }
  }
}
console.log(`checked ${total}`);
