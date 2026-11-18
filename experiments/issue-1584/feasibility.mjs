// Summarize which profile x density combinations have no compliant realization.
// Usage: node experiments/issue-1584/feasibility.mjs [--detail profile density]
import {installHost} from '../../js/agentic/host.mjs';
import {parseLino, readRepoFile} from '../../js/server/lino.mjs';
installHost({parseLino, readText: readRepoFile, realm: {}});
const {cnlPolicy, outputCandidates, validateOutput} = await import('../../js/agentic/crate/cnl_output.mjs');
const {graphKey} = await import('../../js/agentic/crate/cnl_grammar.mjs');
const policy = cnlPolicy();
const DENSITIES = ['standard', 'primes', 'top-100', 'top-200', 'top-500', 'top-1000', 'max-precision'];
const [flag, wantProfile, wantDensity] = process.argv.slice(2);
const graphs = [];
for (const fact of policy.shared.facts) {
  const frame = policy.shared.frames.find(item => item.id === fact.frame);
  for (const args of [fact.args, [...fact.args].reverse()]) for (const polarity of ['positive', 'negative'])
    for (const mood of ['statement', 'question'])
      graphs.push({frame: frame.id, slots: Object.fromEntries(frame.slots.map((slot, i) => [slot, args[i]])), polarity, mood});
}
graphs.push({frame: 'reply', slots: {}, polarity: 'positive', mood: 'statement'},
  {frame: 'clarify', slots: {count: '2'}, polarity: 'positive', mood: 'statement'},
  {frame: 'choose', slots: {}, polarity: 'positive', mood: 'request'});
for (const language of Object.keys(policy.grammars)) {
  for (const profile of policy.profiles) for (const density of DENSITIES) {
    if (flag === '--detail' && (profile.id !== wantProfile || density !== wantDensity)) continue;
    let bad = 0;
    const reasons = {};
    for (const graph of graphs) {
      const configuration = {output_standard: profile.id, lexical_density: density};
      const domain = outputCandidates(graph, language, configuration, policy);
      const reports = domain.map(text => [text, validateOutput(text, language, configuration, policy)]);
      if (reports.some(([, r]) => r.compliant)) continue;
      bad += 1;
      const best = reports.sort((a, b) => a[1].violations.length + a[1].unchecked.length - b[1].violations.length - b[1].unchecked.length)[0];
      const why = best ? [...best[1].violations, ...best[1].unchecked].join(',') : 'no-candidate';
      reasons[why] = (reasons[why] ?? 0) + 1;
      if (flag === '--detail') console.log(graphKey(graph), JSON.stringify(best?.[0]), why);
    }
    if (bad) console.log(language, profile.id, density, `${bad}/${graphs.length}`, JSON.stringify(reasons));
  }
}
