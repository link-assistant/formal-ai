// Finite observed responses for the artifact-count/operand regression; no program is executed.
import {WorkerHost} from '../../js/server/worker-host.mjs';
const host = new WorkerHost();
await host.boot();
for (const prompt of [
  'Python में एक फ़ंक्शन लिखो जो n का फ़ैक्टोरियल लौटाए',
  'Write one function in Python that returns the factorial of n',
  'Python में एक फ़ंक्शन लिखो जो 5 का फ़ैक्टोरियल लौटाए',
  'Write one function in Python for factorial of five',
  'Write a Python function for factorial of one',
  'Write 2 functions in Python to calculate factorial of 5',
]) {
  const result = await host.solve(prompt);
  console.log(JSON.stringify({prompt, intent:result.intent, content:result.content}));
}
