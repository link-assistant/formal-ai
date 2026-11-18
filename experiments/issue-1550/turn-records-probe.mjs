// Issue #1550: print answerValidation / selectionRecord values and the
// evidence links of the probe prompts, to pin the shared test expectations.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
const host = new WorkerHost();
await installNodeHost(host);
const { answerValidation, selectionRecord } = await import('../../js/agentic/crate/meta_executor.mjs');
console.log(answerValidation('greeting', 'Hi'), answerValidation('unknown', ''), answerValidation('calculation', '4'));
console.log(selectionRecord(['number_property_prime(14)=false', 'number_property_prime(15)=false', 'number_property_prime(16)=false', 'number_property_prime(17)=true'], 'unknown'));
console.log(selectionRecord(['number_property_prime(14)=false'], 'unknown'), selectionRecord([], 'unknown'));
for (const prompt of ['Pick a prime number between 14 and 18', 'Hi', 'Suggest a name for my project']) {
  const result = await host.solve(prompt);
  console.log(prompt, JSON.stringify(result.evidenceLinks ?? result.evidence_links ?? result.evidence));
}
