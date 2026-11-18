// Print the controlled-dialogue turn for each prompt argument (issue #1584).
// Usage: node experiments/issue-1584/dialogue-probe.mjs "<prompt>" [...]
import {controlledDialogue} from '../../js/agentic/crate/cnl_output.mjs';
import {policy} from '../../rust/tests/web/support/cnl-matrix.mjs';
let history = [];
for (const prompt of process.argv.slice(2)) {
  const turn = controlledDialogue(prompt, history, {}, policy);
  console.log(JSON.stringify(prompt), '->', turn && {intent: turn.intent, language: turn.language, content: turn.content ?? turn.prompt});
  history = [...history, {role: 'user', content: prompt}, {role: 'assistant', content: turn?.content ?? ''}];
}
