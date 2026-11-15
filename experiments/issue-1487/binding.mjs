// Issue #1487: how a proof claim binds to a library theorem (affirmed / negated / unsupported).
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { bindTheoremClaim, libraryTheorems, theoremMatch } from '../../js/agentic/crate/theorem_claim_binding.mjs';
await installNodeHost(new WorkerHost());
console.log(libraryTheorems().map((t) => `${t.id}:${t.keywords.length}:${t.refutesNegation}`).join(' '));
for (const claim of ['there are infinitely many primes', 'there are not infinitely many primes', 'there are infinitely many primes that are not even',
  'it is not true that there are not infinitely many primes', 'не бесконечно много простых чисел', '不存在无穷多素数', '素数有无穷多个',
  'the universe is not deterministic', 'there are no infinitely many primes', 'pythagorean theorem']) {
  const match = theoremMatch(claim);
  console.log(JSON.stringify(claim), match?.theorem.id ?? null, match ? bindTheoremClaim(claim, match.keyword) : null);
}
process.exit(0);
