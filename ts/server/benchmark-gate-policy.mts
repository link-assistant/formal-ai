import { childValue, childrenNamed, parseLino, readRepoFile } from './lino.mjs';

export function benchmarkGatePolicies() {
  const parsed = parseLino(readRepoFile('data/meta/benchmark-promotion-gates.lino'));
  const policy = parsed.name === 'benchmark_promotion_gates'
    ? parsed
    : childrenNamed(parsed, 'benchmark_promotion_gates')[0];
  return childrenNamed(policy, 'gate').map((node) => ({
    id: node.id,
    runner: childValue(node, 'runner'),
    minimumPassCount: Number.parseInt(childValue(node, 'minimum_pass_count'), 10),
  }));
}
