// Issue #1487: what the browser worker answers for affirmed, negated and
// unread-scope proof requests about Euclid's theorem.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
const host = new WorkerHost(); await installNodeHost(host); host.context.fetch = () => { throw new Error('offline'); };
for (const prompt of ['Prove that there are infinitely many primes.', 'Prove there are not infinitely many primes.',
  'Prove that it is not true that there are not infinitely many primes.', 'Prove there are infinitely many primes that are not even.',
  'Докажи что простых чисел бесконечно много.', 'Докажи что простых чисел не бесконечно много.', '证明素数无穷。', '证明不存在无穷多素数。',
  'साबित करो कि अभाज्य संख्याएँ अनंत हैं।', 'Prove that 2 + 2 = 4']) {
  const response = await host.solve(prompt);
  console.log(prompt, '=>', response.intent, (response.evidence || []).filter((item) => /proof_(outcome|binding|method)/.test(item)).join(' '));
  if (/not infinitely/.test(prompt) && !/that it is/.test(prompt)) console.log(response.content.split('\n').slice(0, 4).join('\n'));
}
