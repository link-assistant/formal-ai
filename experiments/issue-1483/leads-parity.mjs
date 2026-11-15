// Prints what the JS leads module returns for the inputs the Rust unit tests assert.
import { workItemLeads, workItemSymbols, symbolMatches } from '../../js/agentic/work_item_leads.mjs';
console.log(workItemLeads('See ![shot](https://github.com/user-attachments/assets/abc) and the log https://example.com/run.log. Related: https://github.com/o/r/issues/2 and https://github.com/o/r/blob/main/src/a.rs', 'https://github.com/o/r/issues/1'));
console.log(workItemSymbols('The `planned_not_executed` reason in `Progress::scan()` and `word` and `camelCase`, `x.y`.'));
console.log(symbolMatches('./docs/notes.md\n/w/src/deep/a.rs:3: x\nsrc/b.mjs\nnode_modules/x/y.js\n', ['/w']));
