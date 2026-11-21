// Print the exact controlled answers the issue #1584 solver tests pin, so the
// Rust and JavaScript tests can show them as documentation.
// Usage: node experiments/issue-1584/rendered-answers.mjs
import {policy} from '../../rust/tests/web/support/cnl-matrix.mjs';
import {controlledDialogue, formalizeOutput, outputTurn, renderOutput} from '../../js/agentic/crate/cnl_output.mjs';

const question = 'Is a cat an animal?';
const first = controlledDialogue(question, [], {}, policy);
console.log('first', JSON.stringify(first.content));
const history = [{role: 'user', content: question}, {role: 'assistant', content: first.content}];
const turn = outputTurn('Use max-precision.', history, {}, policy);
const switched = renderOutput(formalizeOutput(turn.rewrite, 'en', policy), turn.config, policy);
console.log('switched', JSON.stringify(switched.content));
console.log('task', JSON.stringify(controlledDialogue('add 10 and 3', [], {}, policy)));
