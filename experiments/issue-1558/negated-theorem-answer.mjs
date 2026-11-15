// Prints the JS worker's answer to the negated theorem request so it can be
// compared with the exact Rust answer asserted in
// rust/tests/unit/proof_request.rs::negated_theorem_request_is_refuted_never_proven.
// Usage: node experiments/issue-1558/negated-theorem-answer.mjs ["prompt"]
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';

const host = new WorkerHost();
await installNodeHost(host);
host.context.fetch = () => { throw new Error('offline: no network'); };
const response = await host.solve(process.argv[2] ?? 'Prove there are not infinitely many primes.');
console.log(JSON.stringify({ intent: response.intent, answer: response.content, evidence: response.evidence }, null, 2));
process.exit(0);
