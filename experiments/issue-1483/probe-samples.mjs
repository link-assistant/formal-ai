// Probe: contract-derived samples for a repair preimage (issue #1483).
import { observedComparisonBoundaryOperation as op } from '../../js/server/source-boundary-operation.mjs';
const source = `// Keep this comment.
export function accepts(value,minimum,maximum) {
  return value >= minimum && value < maximum;
}
export function untouched(value) { return value * 3; }
`;
for (const inclusive of [true, false]) {
  try {
    const r = op({ source, owner: 'src/ceiling.mjs', declaration: null, bound: 'upper', inclusive, mode: 'regression', testOwner: 'src/__formal_ai_check.mjs' });
    console.log(inclusive, JSON.stringify(r.samples), JSON.stringify(r.verification));
  } catch (e) { console.log(inclusive, 'error', e.message); }
}
const repair = op({ source, owner: 'src/ceiling.mjs', declaration: null, bound: 'upper', inclusive: true, mode: 'repair' });
console.log('repair verification', JSON.stringify(repair.verification), JSON.stringify(repair.rule));
