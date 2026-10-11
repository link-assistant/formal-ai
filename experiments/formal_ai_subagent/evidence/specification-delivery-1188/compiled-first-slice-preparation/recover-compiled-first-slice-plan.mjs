import {readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const base='/private/tmp/spec-recurrence-1188';
const target=base+'/typed-reader-next/compiled-first-slice-production-plan.md';
const before=readFileSync(target);writeFileSync(base+'/T2944-actual-malformed-plan.md',before);
const bytes=readFileSync(base+'/compiled-first-slice-plan.draft.md');writeFileSync(target,bytes);
if(!readFileSync(target).equals(bytes))throw new Error('exact reviewed plan mismatch');
console.log(JSON.stringify({reviewedPlanByteEqual:true,bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex'),originalLiteralT2944:'OPEN',malformedBeforeBytes:before.length,productionSourceMutations:0}));
