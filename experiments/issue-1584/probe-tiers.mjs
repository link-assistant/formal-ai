// Print the controlled answer and its compliance report under several configurations (issue #1584).
import {controlledDialogue} from '../../js/agentic/crate/cnl_output.mjs';
import {policy} from '../../rust/tests/web/support/cnl-matrix.mjs';
for (const [p, cfg] of [['Is a cat an animal?', {lexical_density: 'top-1'}], ['Is a cat an animal?', {lexical_density: 'top-50'}], ['Is a cat an animal?', {lexical_density: 'primes'}], ['Is a cat an animal?', {lexical_density: 'max-precision'}], ['Is a cat an animal?', {output_standard: 'e-prime'}], ['Is a cat an animal?', {output_standard: 'ace'}], ['Is a cat an animal?', {output_standard: 'gellish'}], ['Is a cat an animal?', {output_standard: 'sbvr'}],['猫是动物吗?', {}],['क्या बिल्ली एक जानवर है?', {}]]) {
  const t = controlledDialogue(p, [], cfg, policy);
  console.log(JSON.stringify(cfg), t?.intent, JSON.stringify(t?.content), JSON.stringify({c: t?.outputProjection?.report?.compliant, v: t?.outputProjection?.report?.violations, u: t?.outputProjection?.report?.unchecked, n: t?.outputProjection?.notice}));
}
